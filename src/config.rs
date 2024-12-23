use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub(crate) username: String,
    pub(crate) password: String,

    pub(crate) mqtt_ip: String,
    pub(crate) mqtt_port: u16,
    pub(crate) mqtt_topic: String,

    pub(crate) sqlite_database: String,

    pub(crate) web_server_ip: String,
    pub(crate) web_server_port: u16,
}
