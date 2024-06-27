use std::{env, time::Duration};

use chrono::{DateTime, Utc};
use dht22_pi::read;
use mongodb::{bson::DateTime as DateTimeDB, Client};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use utoipa::ToSchema;

use crate::{COL_NAME, DB_NAME, SENSOR_PIN};

#[derive(Serialize, Deserialize)]
pub struct TimeDoc {
    pub timestamp: DateTimeDB,
    pub sensor_id: String,
    pub temp: f32,
    pub hum: f32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TimeRes {
    pub timestamp: DateTime<Utc>,
    pub sensor_id: String,
    pub temp: f32,
    pub hum: f32,
}

impl TimeDoc {
    pub fn new(data: (f32, f32), hw_id: &str) -> Self {
        let (temp, hum) = data;
        Self {
            timestamp: DateTimeDB::now(),
            sensor_id: hw_id.to_string(),
            temp,
            hum,
        }
    }
}

impl From<TimeDoc> for TimeRes {
    fn from(value: TimeDoc) -> Self {
        Self {
            timestamp: value.timestamp.into(),
            sensor_id: value.sensor_id,
            temp: value.temp,
            hum: value.hum,
        }
    }
}

pub async fn start_sensor_loop(hw_id: String) {
    let client = Client::with_uri_str(env::var("MONGO_URL").unwrap())
        .await
        .unwrap();
    let collection = client.database(DB_NAME).collection::<TimeDoc>(COL_NAME);
    loop {
        let _ = collection
            .insert_one(TimeDoc::new(read_sensor().await, &hw_id), None)
            .await;
    }
}

pub async fn read_sensor() -> (f32, f32) {
    let mut temp_sum = 0_f32;
    let mut hum_sum = 0_f32;
    let mut count = 0;

    while count < 3 {
        if let Ok(data) = read(SENSOR_PIN) {
            if data.humidity < 100_f32 {
                temp_sum += data.temperature;
                hum_sum += data.humidity;
                count += 1;
            }
        }
        sleep(Duration::from_millis(2200)).await;
    }

    (temp_sum / 3_f32, hum_sum / 3_f32)
}
