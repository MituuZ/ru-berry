use crate::config::Config;
use crate::conn::{get_conn, SqlitePool};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use rusqlite::params;
use serde_json::{Map, Value};
use std::error::Error;
use std::time::Duration;

pub async fn start_mqtt_client(config: &Config, pool: &SqlitePool) {
    println!("Starting MQTT client");

    let mut mqtt_options = MqttOptions::new("", &config.mqtt_ip, config.mqtt_port);
    // Initial last message time is 30 minutes ago so the first message is always processed
    let mut last_message_time = std::time::Instant::now() - Duration::from_secs(1800);
    mqtt_options.set_credentials(&config.username, &config.password);
    mqtt_options.set_keep_alive(Duration::from_secs(900)); // 15 minutes

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    client
        .subscribe(&config.mqtt_topic, QoS::AtMostOnce)
        .await
        .unwrap();

    // Iterate to poll the eventloop for connection progress and print messages
    while let Ok(notification) = eventloop.poll().await {
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                let payload_str = String::from_utf8(publish.payload.to_vec()).unwrap();
                let local_timestamp = chrono::Local::now()
                    .with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                // Insert message into messages table
                audit_message(&pool, &publish.topic, &payload_str);

                if last_message_time.elapsed() < Duration::from_secs(1800) {
                    println!("{} - Last message was less than 30 minutes ago, skipping processing",
                             local_timestamp);
                    continue;
                }
                last_message_time = std::time::Instant::now();

                println!(
                    "{} - Handled message: {:?}",
                    local_timestamp, payload_str
                );

                let json_value = serde_json::from_str(&payload_str);
                match json_value {
                    Ok(value) => handle_message(&value, &pool, &publish.topic),
                    Err(e) => println!("Failed to parse message as JSON: {:?}", e),
                }
            }

            Event::Incoming(event) => println!("Received = {:?}", event),
            Event::Outgoing(_) => {}
        }
    }
}

fn audit_message(pool: &SqlitePool, topic: &str, payload_str: &str) {
    let conn = get_conn(&pool);
    match conn.execute(
        "INSERT INTO messages (topic, payload) VALUES (?1, ?2)",
        &[&topic, &payload_str],
    ) {
        Ok(_) => (),
        Err(e) => println!("Failed to insert message into messages table: {:?}", e),
    }
}

fn handle_message(payload: &Value, pool: &&SqlitePool, topic: &str) {
    if let Some(key_value_json) = payload.as_object() {
        if key_value_json.contains_key("temperature") && key_value_json.contains_key("humidity") {
            match temperature_and_humidity_sensor(key_value_json, pool, topic) {
                Ok(..) => (),
                Err(e) => println!(
                    "Failed to insert temperature and humidity sensor data: {:?}",
                    e
                ),
            }
        }
    } else {
        println!("Payload is not a JSON object");
    }
}

fn temperature_and_humidity_sensor(
    json_object: &Map<String, Value>,
    pool: &SqlitePool,
    topic: &str,
) -> Result<(), Box<dyn Error>> {
    let conn = get_conn(&pool);

    let temperature = json_object
        .get("temperature")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .ok_or("Temperature not found or not a valid f64")?;

    let humidity = json_object
        .get("humidity")
        .and_then(Value::as_i64)
        .ok_or("Humidity not found or not a valid i64")?;

    let linkquality = json_object
        .get("linkquality")
        .and_then(Value::as_i64)
        .ok_or("Linkquality not found or not a valid i64")?;

    let device_id = topic.split('/').last().ok_or("Invalid topic format")?;

    conn.execute(
        "INSERT INTO sensor_data (temperature, humidity, linkquality, device_id) VALUES (?1, ?2, ?3, ?4)",
        params![temperature, humidity, linkquality, device_id],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conn::get_test_pool;
    use serde_json::json;

    #[test]
    fn test_temperature_and_humidity_sensor_valid_data() {
        let pool = get_test_pool();
        let json_object = json!({
            "temperature": 22.5,
            "humidity": 60,
            "linkquality": 100
        })
        .as_object()
        .unwrap()
        .clone();
        let topic = "sensor/device123";

        let result = temperature_and_humidity_sensor(&json_object, &pool, topic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_temperature_and_humidity_sensor_missing_temperature() {
        let pool = get_test_pool();
        let json_object = json!({
            "humidity": 60,
            "linkquality": 100
        })
        .as_object()
        .unwrap()
        .clone();
        let topic = "sensor/device123";

        let result = temperature_and_humidity_sensor(&json_object, &pool, topic);
        assert!(result.is_err());
    }

    #[test]
    fn test_temperature_and_humidity_sensor_invalid_humidity() {
        let pool = get_test_pool();
        let json_object = json!({
            "temperature": 22.5,
            "humidity": "invalid",
            "linkquality": 100
        })
        .as_object()
        .unwrap()
        .clone();
        let topic = "sensor/device123";

        let result = temperature_and_humidity_sensor(&json_object, &pool, topic);
        assert!(result.is_err());
    }

    #[test]
    fn test_temperature_and_humidity_sensor_missing_linkquality() {
        let pool = get_test_pool();
        let json_object = json!({
            "temperature": 22.5,
            "humidity": 60
        })
        .as_object()
        .unwrap()
        .clone();
        let topic = "sensor/device123";

        let result = temperature_and_humidity_sensor(&json_object, &pool, topic);
        assert!(result.is_err());
    }

    #[test]
    fn test_audit_message() {
        let pool = get_test_pool();
        let topic = "sensor/device123";
        let payload_str = "{\"temperature\": 22.5, \"humidity\": 60, \"linkquality\": 100}";

        audit_message(&pool, topic, payload_str);

        let conn = get_conn(&pool);
        let mut stmt = conn
            .prepare("SELECT topic, payload FROM messages WHERE topic = ?1")
            .unwrap();
        let mut rows = stmt.query(params![topic]).unwrap();

        if let Some(row) = rows.next().unwrap() {
            let db_topic: String = row.get(0).unwrap();
            let db_payload: String = row.get(1).unwrap();
            assert_eq!(db_topic, topic);
            assert_eq!(db_payload, payload_str);
        } else {
            panic!("No message found in the database");
        }
    }
}
