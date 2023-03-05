use std::net::SocketAddr;
use std::path::PathBuf;

use axum::extract::Query;
use axum::http::{self, HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use chrono::NaiveDate;
use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::json;
use std::env;

use tower_http::{cors::CorsLayer, services::ServeDir};

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

async fn rolling_average(params: Query<Interval>) -> impl IntoResponse {
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
}

async fn add_weight(payload: axum::extract::Json<AddWeightPayload>) -> impl IntoResponse {
    let mut conn = rs_weight_tracker::establish_connection();

    if let Ok(changed_entries_count) = rs_weight_tracker::upsert_weight_for_date(
        &mut conn,
        payload.weight_value,
        payload.measurement_date.clone(),
    ) {
        (
            StatusCode::CREATED,
            Json(json!({ "status": "ok", "rows": changed_entries_count })),
        )
    } else {
        (
            StatusCode::CREATED,
            Json(json!({ "status": "ok", "rows": 0 })),
        )
    }
}

/// Load the local.env file if it exists, and then load the .env file.
pub fn load_dotenv() -> Option<PathBuf> {
    let local_file_result = dotenvy::from_filename("local.env");

    match &local_file_result {
        Ok(path) => {
            println!("Loaded local.env file from {:?}", path);
            return local_file_result.ok();
        }
        Err(e) => {
            println!(
                "No local.env file found, or error reading it. Skipping: {:?}",
                e
            );
        }
    }

    dotenv().ok()
}

#[tokio::main]
async fn main() {
    load_dotenv();

    let port_num_backend: String =
        env::var("SERVER_BACKEND_PORT_NUM").expect("SERVER_BACKEND_PORT_NUM must be set");
    let port_num_backend: u16 = port_num_backend
        .parse::<u16>()
        .expect("Invalid SERVER_BACKEND_PORT_NUM in environment");

    let port_num_frontend: String =
        env::var("SERVER_FRONTEND_PORT_NUM").expect("SERVER_FRONTEND_PORT_NUM must be set");
    let port_num_frontend: u16 = port_num_frontend
        .parse::<u16>()
        .expect("Invalid SERVER_FRONTEND_PORT_NUM in environment");

    let serve_dir_from_static = ServeDir::new("static");

    let frontend = async {
        let app = Router::new().nest_service("/", serve_dir_from_static);

        let addr = SocketAddr::from(([127, 0, 0, 1], port_num_frontend));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    };

    let backend = async {
        let local_address_for_cors: String =
            env::var("CORS_ALLOWED_ORIGINS").expect("CORS_ALLOWED_ORIGINS must be set");
        let split_cors_addresses: Vec<String> = local_address_for_cors
            .split(',')
            .map(String::from)
            .collect();

        for address in &split_cors_addresses {
            println!("Will add address {} in CORS allowed_origin list", address);
        }

        let allowed_origins: Vec<HeaderValue> = split_cors_addresses
            .iter()
            .map(|address| {
                address
                    .parse::<HeaderValue>()
                    .expect("Can enable CORS for this address.")
            })
            .collect();

        let app = Router::new()
            .route("/api/rolling_average", get(rolling_average))
            .route("/api/add_weight", post(add_weight))
            .layer(
                // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
                // for more details
                //
                // pay attention that for some request types like posting content-type: application/json
                // it is required to add ".allow_headers([http::header::CONTENT_TYPE])"
                // or see this issue https://github.com/tokio-rs/axum/issues/849
                CorsLayer::new()
                    .allow_origin(allowed_origins)
                    .allow_methods([Method::GET, Method::POST])
                    .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]),
            );
        let addr = SocketAddr::from(([127, 0, 0, 1], port_num_backend));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    };

    tokio::join!(frontend, backend);
}
