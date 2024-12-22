use std::time::Duration;
use crate::conn::{get_conn, SqlitePool};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::Deserialize;
use crate::config::Config;

pub async fn start_mqtt_client(config: &Config, pool: &SqlitePool) {
    println!("Starting MQTT client");

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
                let conn = get_conn(&pool);

                conn.execute(
                    "INSERT INTO messages (topic, payload) VALUES (?1, ?2)",
                    &[&publish.topic, &payload_str],
                ).expect("Failed to insert message into messages table");

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
