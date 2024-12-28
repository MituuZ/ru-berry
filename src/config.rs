use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub(crate) username: String,
    pub(crate) password: String,

    pub(crate) mqtt_ip: String,
    pub(crate) mqtt_port: u16,
    /// JSON array of strings
    pub(crate) mqtt_topics: Vec<String>,

    pub(crate) sqlite_database: String,

    pub(crate) web_server_ip: String,
    pub(crate) web_server_port: u16,
}

impl Config {
    pub(crate) fn clone(&self) -> Self {
        Config {
            username: self.username.clone(),
            password: self.password.clone(),
            mqtt_ip: self.mqtt_ip.clone(),
            mqtt_port: self.mqtt_port,
            mqtt_topics: self.mqtt_topics.clone(),
            sqlite_database: self.sqlite_database.clone(),
            web_server_ip: self.web_server_ip.clone(),
            web_server_port: self.web_server_port,
        }
    }
}
