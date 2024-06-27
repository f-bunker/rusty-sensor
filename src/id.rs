use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use bson::Bson;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{sensor::TimeDoc, AppState, COL_NAME, DB_NAME};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct IdRes {
    pub sensor_id: Vec<String>,
}

#[utoipa::path(
        get,
        path = "/data/id",
        responses(
            (status = OK, description = "Array of data points", body = IdRes)
        )
)]
pub async fn get_ids(State(state): State<Arc<AppState>>) -> Result<Json<IdRes>, StatusCode> {
    let collection = state.db.database(DB_NAME).collection::<TimeDoc>(COL_NAME);

    let aggr = [
        doc! {
            "$group": doc! {
                "_id": Bson::Null,
                "sensor_id": doc! {
                    "$addToSet": "$sensor_id"
                }
            }
        },
        doc! {
            "$sort": doc! {
                "sensor_id": -1
            }
        },
    ];

    if let Ok(mut cursor) = collection.aggregate(aggr, None).await {
        if let Ok(e) = cursor.advance().await {
            if e {
                if let Ok(docs) = cursor.deserialize_current() {
                    if let Ok(doc) = bson::from_document::<IdRes>(docs) {
                        return Ok(Json(doc));
                    };
                }
            }
        }
    };
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
