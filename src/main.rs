mod web;
mod conn;
mod mqtt;
mod config;
mod model;

use crate::conn::create_pool;
use config::Config;
use std::fs;
use crate::web::ru_berry_web;

#[tokio::main]
async fn main() -> rusqlite::Result<()> {
    let config: Config = serde_json::from_str(&fs::read_to_string("config.json").expect("Unable to read config file"))
        .expect("Unable to parse config file");

    let pool = create_pool(&config.sqlite_database).expect("Failed to create SQLite connection pool");
    println!("Connected to SQLite database");

    // Start the web server in a separate task
    let web_pool = pool.clone();
    let web_config = config.clone();
    tokio::spawn(async move {
        ru_berry_web::start_web_server(&web_config, &web_pool).await;
    });

    // Start the MQTT client in a separate task
    let mqtt_pool = pool.clone();
    let mqtt_config = config.clone();
    tokio::spawn(async move {
        mqtt::start_mqtt_client(&mqtt_config, &mqtt_pool).await;
    });

    // Keep the main function alive
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");

    Ok(())
}

