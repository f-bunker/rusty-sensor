use std::{env, sync::Arc};

use axum::{response::Redirect, routing::get, Router};
use hour::get_hour_data;
use id::{get_ids, IdRes};
use mongodb::Client;
use month::{get_month_data, MonthRes};
use sensor::{start_sensor_loop, TimeRes};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, spawn};
use tower_http::services::ServeDir;
use utoipa::{IntoParams, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

mod hour;
mod id;
mod month;
mod sensor;

const SENSOR_PIN: u8 = 4;
const PORT: u16 = 3000;
const DB_NAME: &str = "rusty-sensor";
const COL_NAME: &str = "sensor-data";

pub struct AppState {
    pub db: Client,
}

#[derive(Serialize, Deserialize, IntoParams)]
pub struct SensorArgs {
    pub sensor_id: String,
}

#[tokio::main]
async fn main() {
    spawn(async move { start_sensor_loop(hardware_id::get_id().unwrap().to_string()).await });

    #[derive(OpenApi)]
    #[openapi(
        paths(hour::get_hour_data, month::get_month_data, id::get_ids),
        components(schemas(TimeRes, MonthRes, IdRes))
    )]
    struct ApiDoc;

    let client = Client::with_uri_str(env::var("MONGO_URL").unwrap())
        .await
        .unwrap();
    let shared_state = Arc::new(AppState { db: client });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/data/hour", get(get_hour_data))
        .route("/data/month", get(get_month_data))
        .route("/data/id", get(get_ids))
        .route("/", get(|| async { Redirect::to("/ui") }))
        .nest_service(
            "/ui",
            ServeDir::new("/var/sensor").append_index_html_on_directories(true),
        )
        .with_state(shared_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{PORT}")).await.unwrap();

    println!("Listening on {PORT}");
    axum::serve(listener, app).await.unwrap();
}
