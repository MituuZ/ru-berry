use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use warp::Filter;
use rusqlite::{Connection, Result};
use serde::Serialize;
use crate::conn::{get_conn, SqlitePool};

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
    let mut stmt = conn.prepare("SELECT * FROM sensor_data").unwrap();
    let sensor_data_iter = stmt.query_map([], |row| {
        Ok(SensorData {
            temperature: row.get(0)?,
            humidity: row.get(1)?,
            linkquality: row.get(2)?,
            device_id: row.get(3)?,
            received_at: row.get(4)?,
        })
    }).unwrap();

    let mut sensor_data_vec = Vec::new();
    for sensor_data in sensor_data_iter {
        sensor_data_vec.push(sensor_data.unwrap());
    }

    Ok(warp::reply::json(&sensor_data_vec))
}

pub async fn start_web_server(pool: SqlitePool) {
    let sensor_data_route = warp::path("sensor_data")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(get_sensor_data);

    warp::serve(sensor_data_route)
        .run(([127, 0, 0, 1], 3030)).await;
}

fn with_db(pool: SqlitePool) -> impl Filter<Extract = (SqlitePool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
