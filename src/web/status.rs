use crate::conn::{get_conn, SqlitePool};
use crate::model::SensorData;
use std::collections::HashMap;

enum StatusType {
    Basic,
    Boolean,
    None,
}

pub async fn get_sensor_data_status(pool: SqlitePool) -> Result<impl warp::Reply, warp::Rejection> {
    let topics = fetch_topics(&pool);

    let topics: HashMap<String, StatusType> = match topics {
        None => return Ok(warp::reply::html("No topics configured")),
        Some(t) => t,
    };

    let mut html = html_start();

    for t in topics {
        match t.1 {
            StatusType::Basic => html.push_str(basic(t.0, &pool).as_str()),
            StatusType::Boolean => html.push_str(boolean(t.0, &pool).as_str()),
            StatusType::None => println!("No status type configured for topic: {}", t.0),
        }
    }

    html.push_str(html_end().as_str());
    let html: &'static str = Box::leak(html.into_boxed_str());
    Ok(warp::reply::html(html))
}

fn basic(device_id: String, pool: &SqlitePool) -> String {
    let conn = get_conn(&pool);
    println!("Getting basic sensor data for device: {}", device_id);

    let mut html = format!("<div class=\"device-data\"> \
    <h2>Basic Sensor Data for Device: {}</h2>", device_id);

    let mut stmt = match conn.prepare(
        "
        SELECT *, 1 as data_type, 'Lowest Temperature in the Last 3 Days' as title FROM (
            SELECT * FROM sensor_data WHERE device_id = ?1 AND received_at >= datetime('now', '-3 days') ORDER BY temperature ASC LIMIT 1
        )
        UNION
        SELECT *, 2 as data_type, 'Highest Temperature in the Last 3 Days' as title FROM (
            SELECT * FROM sensor_data WHERE device_id = ?1 AND received_at >= datetime('now', '-3 days') ORDER BY temperature DESC LIMIT 1
        )
        UNION
        SELECT *, 3 as data_type, 'Lowest Humidity in the Last 3 Days' as title FROM (
            SELECT * FROM sensor_data WHERE device_id = ?1 AND received_at >= datetime('now', '-3 days') ORDER BY humidity ASC LIMIT 1
        )
        UNION
        SELECT *, 4 as data_type, 'Highest Humidity in the Last 3 Days' as title FROM (
            SELECT * FROM sensor_data WHERE device_id = ?1 AND received_at >= datetime('now', '-3 days') ORDER BY humidity DESC LIMIT 1
        )
        UNION
        SELECT *, 5 as data_type, 'Latest Reading' as title FROM (
            SELECT * FROM sensor_data WHERE device_id = ?1 ORDER BY received_at DESC LIMIT 1
        )
        ORDER BY data_type;
    "
    ) {
        Ok(stmt) => stmt,
        Err(_) => return format!("Error preparing query for device: {}", device_id).to_string()
    };

    let query_data = match stmt.query_map([&device_id], |row| {
        let sensor_data = SensorData::from_row(row)?;
        let title: String = row.get("title")?;
        Ok((sensor_data, title))
    }) {
        Ok(iter) => iter,
        Err(_) => return format!("Error querying data for device: {}", device_id).to_string()
    };

    let mut sensor_data_with_titles = Vec::new();
    for result in query_data {
        match result {
            Ok((data, title)) => {
                sensor_data_with_titles.push((data, title));
            }
            Err(_) => return format!("Error fetching data for device: {}", device_id).to_string()
        }
    }

    if sensor_data_with_titles.len() == 0 {
        return format!("No data found for device: {}", device_id);
    }

    html.push_str(format!(
        "
            {}",
        sensor_data_with_titles
            .iter()
            .map(|(data, title)| create_table(data, title))
            .collect::<String>()
    ).as_str());

    html.push_str("</div>");
    html
}

fn boolean(topic: String, pool: &SqlitePool) -> String {
    "".to_string()
}

fn create_table(sensor_data: &SensorData, title: &str) -> String {
    format!(
        "<h3>{}</h3>\
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

fn fetch_topics(pool: &SqlitePool) -> Option<HashMap<String, StatusType>> {
    let conn = get_conn(pool);
    let mut stmt = conn
        .prepare("SELECT topic_name, status_type FROM topic_configuration;")
        .ok()?;

    let query_data = stmt
        .query_map([], |row| {
            let topic: String = row.get("topic_name")?;
            let status_type = match row.get::<_, String>("status_type")?.as_str() {
                "basic" => StatusType::Basic,
                "boolean" => StatusType::Boolean,
                _ => StatusType::None,
            };
            Ok((topic, status_type))
        })
        .ok()?;

    Some(query_data.filter_map(Result::ok).collect())
}

fn html_start() -> String {
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
                .device-data {
                    border: 1px solid #ddd;
                    margin-bottom: 20px;
                    padding: 10px;
                    border-radius: 5px;
                    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
                }
            </style>
        </head>
        <body>
            <h1>Sensor Data Status</h1>
    "
    .to_string()
}

fn html_end() -> String {
    "</body></html>".to_string()
}
