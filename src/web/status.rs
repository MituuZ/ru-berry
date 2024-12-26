use crate::conn::{get_conn, SqlitePool};
use crate::model::SensorData;
use crate::web::ru_berry_web::MyError;

pub async fn get_sensor_data_status(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = get_conn(&pool);
    println!("Getting sensor data");

    let mut stmt = match conn.prepare(
        "
            SELECT *, 1 as data_type, 'Lowest Temperature in the Last 3 Days' as title FROM (
                SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY temperature ASC LIMIT 1
            )
            UNION
            SELECT *, 2 as data_type, 'Highest Temperature in the Last 3 Days' as title FROM (
                SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY temperature DESC LIMIT 1
            )
            UNION
            SELECT *, 3 as data_type, 'Lowest Humidity in the Last 3 Days' as title FROM (
                SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY humidity ASC LIMIT 1
            )
            UNION
            SELECT *, 4 as data_type, 'Highest Humidity in the Last 3 Days' as title FROM (
                SELECT * FROM sensor_data WHERE received_at >= datetime('now', '-3 days') ORDER BY humidity DESC LIMIT 1
            )
            UNION
            SELECT *, 5 as data_type, 'Latest Reading' as title FROM (
                SELECT * FROM sensor_data ORDER BY received_at DESC LIMIT 1
            )
            ORDER BY data_type;
            "
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Err(warp::reject::custom(MyError::QueryPreparationError)),
    };

    let query_data = match stmt.query_map([], |row| {
        let sensor_data = SensorData::from_row(row)?;
        let title: String = row.get("title")?;
        Ok((sensor_data, title))
    }) {
        Ok(iter) => iter,
        Err(_) => return Err(warp::reject::custom(MyError::QueryExecutionError)),
    };

    let mut sensor_data_with_titles = Vec::new();
    for result in query_data {
        match result {
            Ok((data, title)) => {
                sensor_data_with_titles.push((data, title));
            }
            Err(_) => return Err(warp::reject::custom(MyError::DataMappingError)),
        }
    }

    if sensor_data_with_titles.len() == 0 {
        return Ok(warp::reply::html("No data found"));
    }

    let html = format!(
        "<html>
        <head>
            <title>Sensor Data Status</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                }}
                table {{
                    border-collapse: collapse;
                }}
                th, td {{
                    padding: 8px;
                }}
            </style>
        </head>
        <body>
            <h1>Sensor Data Status</h1>
            {}
            ",
        sensor_data_with_titles
            .iter()
            .map(|(data, title)| create_table(data, title))
            .collect::<String>()
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
