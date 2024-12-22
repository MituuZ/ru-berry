use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use serde::Deserialize;
use std::time::Duration;
use std::fs;
use rusqlite::Connection;

fn main() {
    println!("Hello, world!");

    let config: Config = serde_json::from_str(&fs::read_to_string("config.json").expect("Unable to read config file"))
        .expect("Unable to parse config file");

    let _ = setup_sqlite(&config);

    let mut mqttoptions = MqttOptions::new("", &config.mqtt_ip, config.mqtt_port);
    mqttoptions.set_credentials(&config.username, &config.password);
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, mut connection) = Client::new(mqttoptions, 10);
    client.subscribe(&config.mqtt_topic, QoS::AtMostOnce).unwrap();

    // Iterate to poll the eventloop for connection progress and print messages
    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                println!("Received message: {:?}", String::from_utf8(publish.payload.to_vec()).unwrap());
            },
            Ok(event) => println!("Received = {:?}", event),
            Err(e) => eprintln!("Error = {:?}", e),
        }
    }
}

fn setup_sqlite(config: &Config) -> rusqlite::Result<()> {
    // Set up SQLite database
    let conn = Connection::open(&config.sqlite_database)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            topic TEXT NOT NULL,
            payload TEXT NOT NULL,
            received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
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