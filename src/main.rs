mod web;
mod conn;

use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use tokio::time::Duration;
use serde::Deserialize;
use std::fs;
use crate::conn::{create_pool, get_conn, SqlitePool};

#[tokio::main]
async fn main() -> rusqlite::Result<()> {
    println!("Hello, world!");

    let config: Config = serde_json::from_str(&fs::read_to_string("config.json").expect("Unable to read config file"))
        .expect("Unable to parse config file");

    let pool = create_pool(&config.sqlite_database);
    setup_sqlite(&pool);

    let conn = get_conn(&pool);

    // Start the web server in a separate task
    tokio::spawn(async move {
        web::start_web_server(pool.clone()).await;
    });

    let mut mqttoptions = MqttOptions::new("", &config.mqtt_ip, config.mqtt_port);
    mqttoptions.set_credentials(&config.username, &config.password);
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe(&config.mqtt_topic, QoS::AtMostOnce).await.unwrap();

    // Iterate to poll the eventloop for connection progress and print messages
    while let Ok(notification) = eventloop.poll().await {
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                let payload_str = String::from_utf8(publish.payload.to_vec()).unwrap();
                println!("Received message: {:?}", payload_str);

                // Insert message into messages table
                conn.execute(
                    "INSERT INTO messages (topic, payload) VALUES (?1, ?2)",
                    &[&publish.topic, &payload_str],
                )?;

                // Assuming the payload is a JSON string with temperature, humidity, and linkquality fields
                if let Ok(sensor_data) = serde_json::from_str::<SensorData>(&payload_str) {
                    conn.execute(
                        "INSERT INTO sensor_data (temperature, humidity, linkquality, device_id) VALUES (?1, ?2, ?3, ?4)",
                        &[
                            &sensor_data.temperature as &dyn rusqlite::ToSql,
                            &sensor_data.humidity as &dyn rusqlite::ToSql,
                            &sensor_data.linkquality as &dyn rusqlite::ToSql,
                            &publish.topic as &dyn rusqlite::ToSql,
                        ],
                    )?;
                }
            },
            Event::Incoming(event) => println!("Received = {:?}", event),
            Event::Outgoing(_) => {},
        }
    }

    Ok(())
}

fn setup_sqlite(pool: &SqlitePool) {
    // Set up SQLite database
    let conn = get_conn(&pool);
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            topic TEXT NOT NULL,
            payload TEXT NOT NULL,
            received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).expect("Failed to create messages table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sensor_data (
            id INTEGER PRIMARY KEY,
            temperature DECIMAL(4,2) NOT NULL,
            humidity INTEGER NOT NULL,
            linkquality INTEGER NOT NULL,
            device_id TEXT NOT NULL,
            received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).expect("Failed to create sensor_data table");
}

#[derive(Deserialize)]
struct Config {
    username: String,
    password: String,

    mqtt_ip: String,
    mqtt_port: u16,
    mqtt_topic: String,

    sqlite_database: String,
}

#[derive(Deserialize)]
struct SensorData {
    temperature: f32,
    humidity: i32,
    linkquality: i32,
}
