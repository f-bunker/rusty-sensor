use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::{doc, DateTime};

use crate::{
    sensor::{TimeDoc, TimeRes},
    AppState, SensorArgs, COL_NAME, DB_NAME,
};

#[utoipa::path(
    get,
    path = "/data/hour",
    params(SensorArgs),
    responses(
        (status = OK, description = "Array of data points", body = Vec<TimeRes>)
    )
)]
pub async fn get_hour_data(
    State(state): State<Arc<AppState>>,
    Query(args): Query<SensorArgs>,
) -> Result<Json<Vec<TimeRes>>, StatusCode> {
    let collection = state.db.database(DB_NAME).collection::<TimeDoc>(COL_NAME);

    let time = DateTime::from_chrono(chrono::Utc::now() - chrono::Duration::hours(1));
    let filter =
        doc! { "timestamp": { "$gte": time , "$lt": DateTime::now() }, "sensor_id": args.sensor_id};
    let data = match collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut out = vec![];
            while cursor
                .advance()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            {
                let doc = cursor.deserialize_current().unwrap();
                out.push(doc.into());
            }
            out
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    Ok(Json(data))
}
