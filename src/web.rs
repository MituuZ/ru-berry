use crate::config::Config;
use crate::conn::{get_conn, SqlitePool};
use rusqlite::Result;
use serde::Serialize;
use std::net::SocketAddr;
use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use warp::Filter;

#[derive(Serialize, Debug)]
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

async fn get_sensor_data_status(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = get_conn(&pool);
    println!("Getting sensor data");

    let mut stmt = match conn.prepare(
        "select * from sensor_data where received_at \
    >= datetime('now', '-3 days') order by temperature asc limit 1;",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Err(warp::reject::custom(MyError::QueryPreparationError)),
    };

    let min_temp_data = match stmt.query_map([], |row| {
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

    let sensor_data = match min_temp_data.collect::<Result<Vec<SensorData>>>() {
        Ok(data) => data,
        Err(_) => return Err(warp::reject::custom(MyError::DataMappingError)),
    };

    // Inside the map function
    let received_at_naive = NaiveDateTime::parse_from_str(&sensor_data[0].received_at, "%Y-%m-%d %H:%M:%S")
        .expect("Failed to parse received_at");
    let received_at_utc = Utc.from_utc_datetime(&received_at_naive);
    let received_at_with_tz = received_at_utc.with_timezone(&Local);

    let html = format!(
        "<html>
        <head><title>Sensor Data Status</title></head>
        <body>
            <h1>Sensor Data Status</h1>
            <h2>Minimum Temperature in the Last 3 Days</h2>
            <table border=\"1\">
                <tr>
                    <th>Temperature</th>
                    <th>Humidity</th>
                    <th>Link Quality</th>
                    <th>Device ID</th>
                    <th>Received At</th>
                </tr>
                {}
            </table>
        </body>
    </html>",
        sensor_data
            .iter()
            .map(|data| format!(
                "<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>",
                data.temperature,
                data.humidity,
                data.linkquality,
                data.device_id,
                received_at_with_tz
            ))
            .collect::<String>()
    );

    Ok(warp::reply::html(html))
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
