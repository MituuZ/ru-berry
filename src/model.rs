use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorData {
    pub(crate) temperature: f32,
    pub(crate) humidity: i32,
    pub(crate) linkquality: i32,
    pub(crate) device_id: String,
    pub(crate) received_at: String,
}

impl SensorData {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(SensorData {
            temperature: row.get::<_, f64>(1)? as f32,
            humidity: row.get(2)?,
            linkquality: row.get(3)?,
            device_id: row.get(4)?,
            received_at: row.get(5)?,
        })
    }
}
