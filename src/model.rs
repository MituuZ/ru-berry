use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorData {
    pub(crate) temperature: f32,
    pub(crate) humidity: i32,
    pub(crate) linkquality: i32,
    pub(crate) device_id: String,
    pub(crate) received_at: String,
}
