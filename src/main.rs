use std::net::SocketAddr;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;

// use serde_json::json;
use tower_http::services::{ServeDir};

#[derive(Debug, Deserialize)]
pub struct AddWeightPayload {
    pub weight_value: f64,
    pub measurement_date: String,
}

#[derive(Debug, Deserialize)]
struct Interval {
    start_date: String,
    end_date: String,
    days: u32,
}

async fn rolling_average(params: Query<Interval>) -> impl IntoResponse
//Result<serde_json::Value, tower::BoxError>
{
    let interval: Interval = params.0;

    let start_date = NaiveDate::parse_from_str(&interval.start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&interval.end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages = rs_weight_tracker::rolling_average_between_dates(
        &mut conn,
        start_date,
        end_date,
        interval.days,
    )
    .unwrap();

    let result = averages
        .into_iter()
        .map(|(date, avg)| {
            let mut map = serde_json::Map::new();
            map.insert(
                "date".to_string(),
                serde_json::Value::String(date.format("%Y-%m-%d").to_string()),
            );
            map.insert(
                "average".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(avg).unwrap()),
            );
            serde_json::Value::Object(map)
        })
        .collect::<Vec<serde_json::Value>>();

    (StatusCode::OK, Json(result))
    // Ok(json!(result))
}

async fn add_weight(
    payload: axum::extract::Json<AddWeightPayload>,
) -> impl IntoResponse {
    // Result<Value, tower::BoxError> {
    let mut conn = rs_weight_tracker::establish_connection();
    // let payload::AddWeightPayload = payload.into();

    if let Ok(changed_entries_count) = rs_weight_tracker::upsert_weight_for_date(
        &mut conn,
        payload.weight_value,
        payload.measurement_date.clone(),
    ) {
        (StatusCode::CREATED, Json(json!({ "status": "ok", "rows": changed_entries_count })))
        // Ok(json!({ "status": "ok", "rows": changed_entries_count }))
    } else {
        (StatusCode::CREATED, Json(json!({ "status": "ok", "rows": 0 })))
        // Ok(json!({ "status": "ok", "rows": 0 }))
    }
}

#[tokio::main]
async fn main() {
    let serve_dir_from_static = ServeDir::new("static");

    let app = Router::new()
        .route("/api/rolling_average", get(rolling_average))
        .route("/api/add_weight", post(add_weight))
        .nest_service("/", serve_dir_from_static);

    // let app = ServiceBuilder::new().buffer(100, 10).service(app);

    // let addr = SocketAddr::from(([0, 0, 0, 0], 12480));
    let addr = SocketAddr::from(([127, 0, 0, 1], 12480));
    println!("Server running on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
