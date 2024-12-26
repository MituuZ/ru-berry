use crate::conn::{get_conn, SqlitePool};
use crate::model::SensorData;
use crate::web::ru_berry_web::MyError;

pub async fn get_sensor_data_status(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = get_conn(&pool);
    println!("Getting sensor data");

    let mut stmt = match conn.prepare(
        "SELECT * FROM (
            SELECT *, 1 as data_type FROM (
                SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY temperature ASC LIMIT 1
            )
            )
            UNION
            SELECT * FROM (
                SELECT *, 2 as data_type FROM (
                    SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY humidity ASC LIMIT 1
                )
            )
            UNION
            SELECT * FROM (
                SELECT *, 3 as data_type FROM (
                    SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY humidity DESC LIMIT 1
                )
            )
            ORDER BY data_type;"
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Err(warp::reject::custom(MyError::QueryPreparationError)),
    };

    let query_data = match stmt.query_map([], |row| SensorData::from_row(row)) {
        Ok(iter) => iter,
        Err(_) => return Err(warp::reject::custom(MyError::QueryExecutionError)),
    };

    let sensor_data = match query_data.collect::<Result<Vec<SensorData>, rusqlite::Error>>() {
        Ok(data) => data,
        Err(_) => return Err(warp::reject::custom(MyError::DataMappingError)),
    };

    if sensor_data.len() == 0 {
        return Ok(warp::reply::html("No data found"));
    }

    let html = format!(
        "<html>
        <head><title>Sensor Data Status</title></head>
        <body>
            <h1>Sensor Data Status</h1>
            {}
            {}
            {}",
        create_table(&sensor_data[0], "Lowest Temperature"),
        create_table(&sensor_data[1], "Lowest Humidity"),
        create_table(&sensor_data[2], "Highest Humidity")
    );

    let html: &'static str = Box::leak(html.into_boxed_str());
    Ok(warp::reply::html(html))
}

fn create_table(sensor_data: &SensorData, title: &str) -> String {
    format!(
        "<h2>{}</h2>\
        <table border = \"1\">\
        <tr>\
            <th>Temperature</th>\
            <th>Humidity</th>\
            <th>Link Quality</th>\
            <th>Device ID</th>\
            <th>Received At</th>\
        </tr>\
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>
        </table>",
        title,
        sensor_data.temperature,
        sensor_data.humidity,
        sensor_data.linkquality,
        sensor_data.device_id,
        sensor_data.received_at
    )
}
