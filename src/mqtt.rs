use std::time::Duration;
use crate::conn::{get_conn, SqlitePool};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::Deserialize;
use crate::config::Config;

pub async fn start_mqtt_client(config: &Config, pool: &SqlitePool) {
    println!("Starting MQTT client");

    let mut mqtt_options = MqttOptions::new("", &config.mqtt_ip, config.mqtt_port);
    // Initial last message time is 30 minutes ago so the first message is always processed
    let mut last_message_time = std::time::Instant::now() - Duration::from_secs(1800);
    mqtt_options.set_credentials(&config.username, &config.password);
    mqtt_options.set_keep_alive(Duration::from_secs(900)); // 15 minutes

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    client.subscribe(&config.mqtt_topic, QoS::AtMostOnce).await.unwrap();

    // Iterate to poll the eventloop for connection progress and print messages
    while let Ok(notification) = eventloop.poll().await {
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                let payload_str = String::from_utf8(publish.payload.to_vec()).unwrap();

                // Insert message into messages table
                let conn = get_conn(&pool);
                conn.execute(
                    "INSERT INTO messages (topic, payload) VALUES (?1, ?2)",
                    &[&publish.topic, &payload_str],
                ).expect("Failed to insert message into messages table");

                if last_message_time.elapsed() < Duration::from_secs(1800) {
                    println!("Last message was less than 30 minutes ago, skipping processing");
                    continue;
                }
                let now = std::time::Instant::now();
                last_message_time = now;

                let current_timestamp = chrono::Local::now().with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string();
                println!("{} - Received message: {:?}", current_timestamp, payload_str);

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
                    ).expect("Failed to insert data into sensor_data table");
                }
            },
            Event::Incoming(event) => println!("Received = {:?}", event),
            Event::Outgoing(_) => {},
        }
    }
}

#[derive(Deserialize)]
struct SensorData {
    temperature: f32,
    humidity: i32,
    linkquality: i32,
}
