use crate::config::Config;
use crate::conn::{get_conn, SqlitePool};
use crate::model::SensorData;
use rusqlite::Result;
use std::net::SocketAddr;
use warp::Filter;
use crate::web::status::get_sensor_data_status;

async fn get_sensor_data(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = get_conn(&pool);
    println!("Getting sensor data");

    let mut stmt = match conn.prepare(
        "
        SELECT * FROM sensor_data
        WHERE received_at >= datetime('now', '-3 days')
        order by device_id, received_at
        "
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Err(warp::reject::custom(MyError::QueryPreparationError)),
    };

    let sensor_data_iter = match stmt.query_map([], |row| SensorData::from_row(row)) {
        Ok(iter) => iter,
        Err(_) => return Err(warp::reject::custom(MyError::QueryExecutionError)),
    };

    let mut sensor_data_vec = Vec::new();
    for sensor_data in sensor_data_iter {
        match sensor_data {
            Ok(data) => sensor_data_vec.push(data),
            Err(_) => return Err(warp::reject::custom(MyError::DataMappingError)),
        }
    }

    Ok(warp::reply::json(&sensor_data_vec))
}

#[derive(Debug)]
pub enum MyError {
    QueryPreparationError,
    QueryExecutionError,
    DataMappingError,
}

impl warp::reject::Reject for MyError {}

pub async fn start_web_server(config: &Config, pool: &SqlitePool) {
    let ip = config.web_server_ip.clone();
    let port = config.web_server_port;
    println!("Starting web server on {}:{}", ip, port);

    let sensor_data_route = warp::path("sensor_data")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(get_sensor_data);

    let sensor_data_status_route = warp::path("sensor_data_status")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(get_sensor_data_status);

    let routes = sensor_data_route.or(sensor_data_status_route);

    let addr: SocketAddr = format!("{}:{}", ip, port)
        .parse()
        .expect("Invalid IP address or port");
    warp::serve(routes).run(addr).await;
}

fn with_db(
    pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
