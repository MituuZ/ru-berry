use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use crate::conn::{get_conn, SqlitePool};
use crate::model::SensorData;
use crate::web::ru_berry_web::MyError;

pub async fn get_sensor_data_status(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = get_conn(&pool);
    println!("Getting sensor data");

    let mut stmt = match conn.prepare(
        "select * from sensor_data where received_at \
    >= datetime('now', '-3 days') order by temperature asc limit 1;",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Err(warp::reject::custom(MyError::QueryPreparationError)),
    };

    let min_temp_data = match stmt.query_map([], |row| SensorData::from_row(row)) {
        Ok(iter) => iter,
        Err(_) => return Err(warp::reject::custom(MyError::QueryExecutionError)),
    };

    let sensor_data = match min_temp_data.collect::<Result<Vec<SensorData>, rusqlite::Error>>() {
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

