use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorData {
    pub(crate) temperature: f32,
    pub(crate) humidity: f32,
    pub(crate) linkquality: i32,
    pub(crate) device_id: String,
    pub(crate) received_at: String,
}

impl SensorData {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let received_at: String = row.get(5)?;
        let received_at_naive = NaiveDateTime::parse_from_str(&received_at, "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse received_at");
        let received_at_utc = Utc.from_utc_datetime(&received_at_naive);
        let received_at_with_tz = received_at_utc.with_timezone(&Local);

        Ok(SensorData {
            temperature: row.get::<_, f64>(1)? as f32,
            humidity: row.get(2)?,
            linkquality: row.get(3)?,
            device_id: row.get(4)?,
            received_at: received_at_with_tz.to_string(),
        })
    }
}
