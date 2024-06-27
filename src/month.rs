use std::{f64, sync::Arc};

use axum::{extract::{Query, State}, http::StatusCode, Json};
use chrono::{DateTime, Duration, Timelike, Utc};
use mongodb::bson::{doc, Bson, DateTime as DateTimeDB};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{sensor::TimeDoc, AppState, SensorArgs, COL_NAME, DB_NAME};

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthDoc {
    pub temp_median: f64,
    pub hum_median: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MonthRes {
    pub timestamp: DateTime<Utc>,
    pub temp_median: f64,
    pub hum_median: f64,
}

impl MonthDoc {
    pub fn to_monthres(&self, time: DateTimeDB) -> MonthRes {
        MonthRes {
            timestamp: time.into(),
            temp_median: self.temp_median,
            hum_median: self.hum_median,
        }
    }
}

#[utoipa::path(
    get,
    path = "/data/month",
    params (SensorArgs),    
    responses(
            (status = OK, description = "Array of data points", body = Vec<MonthRes>)
        )
)]
pub async fn get_month_data(
    State(state): State<Arc<AppState>>,
    Query(args): Query<SensorArgs>
) -> Result<Json<Vec<MonthRes>>, StatusCode> {
    let collection = state
        .db
        .database(DB_NAME)
        .collection::<TimeDoc>(COL_NAME);
    let mut today = get_start_of_today() + Duration::hours(24);
    let mut curr = 0;
    let mut data = vec![];
    while curr < 30 {
        let gte = DateTimeDB::from_chrono(today - Duration::hours(24));
        let aggr = vec![
            doc! {
                "$match": {
                    "timestamp": {
                        "$gte": gte,
                        "$lt": DateTimeDB::from_chrono(today)
                    },
                    "sensor_id": args.sensor_id.clone()
                }
            },
            doc! {
                "$group": {
                    "_id": Bson::Null,
                    "temp_median": {
                        "$median": {
                            "input": "$temp",
                            "method": "approximate"
                        }
                    },
                    "hum_median": {
                        "$median": {
                            "input": "$hum",
                            "method": "approximate"
                        }
                    }
                }
            },
        ];
        match collection.aggregate(aggr, None).await {
            Ok(mut cursor) => {
                if let Ok(e) = cursor.advance().await {
                    if e {
                        if let Ok(docs) = cursor.deserialize_current() {
                            if let Ok(doc) = bson::from_document::<MonthDoc>(docs) {
                                data.push(doc.to_monthres(gte));
                            };
                        }
                    }
                }
            }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
        today -= Duration::hours(24);
        curr += 1;
    }

    Ok(Json(data))
}

fn get_start_of_today() -> DateTime<Utc> {
    Utc::now()
        .with_nanosecond(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_hour(0)
        .unwrap()
}
