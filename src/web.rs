use std::net::SocketAddr;
use crate::conn::{get_conn, SqlitePool};
use rusqlite::Result;
use serde::Serialize;
use warp::Filter;
use crate::config::Config;

#[derive(Serialize)]
struct SensorData {
    temperature: f32,
    humidity: i32,
    linkquality: i32,
    device_id: String,
    received_at: String,
}

async fn get_sensor_data(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = get_conn(&pool);
    println!("Getting sensor data");

    let mut stmt = match conn.prepare("SELECT * FROM sensor_data") {
        Ok(stmt) => stmt,
        Err(_) => return Err(warp::reject::custom(MyError::QueryPreparationError)),
    };

    let sensor_data_iter = match stmt.query_map([], |row| {
        Ok(SensorData {
            temperature: row.get::<_, f64>(1)? as f32,
            humidity: row.get(2)?,
            linkquality: row.get(3)?,
            device_id: row.get(4)?,
            received_at: row.get(5)?,
        })
    }) {
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
enum MyError {
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

    let addr: SocketAddr = format!("{}:{}", ip, port).parse().expect("Invalid IP address or port");
    warp::serve(sensor_data_route)
        .run(addr).await;
}

fn with_db(pool: SqlitePool) -> impl Filter<Extract = (SqlitePool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
