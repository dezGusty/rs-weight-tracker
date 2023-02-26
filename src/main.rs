#[macro_use]
extern crate rocket;

use chrono::NaiveDate;
use rocket::fs::{relative, FileServer};
use rocket::serde::json::{self, Json};
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AddWeightPayload {
    pub weight_value: f64,
    pub measurement_date: String,
}

#[get("/rolling_average?<start_date>&<end_date>&<days>")]
fn rolling_average(
    start_date: String,
    end_date: String,
    days: u32,
) -> Json<Vec<serde_json::Value>> {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages =
        rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, days)
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

    Json(result)
}

#[post("/add_weight", format = "json", data = "<payload>")]
fn add_weight(payload: Json<AddWeightPayload>) -> Value {
    let mut conn = rs_weight_tracker::establish_connection();
    let Json(payload) = payload;

    // let mut post_response: AddWeightResponse = AddWeightResponse{ rows_affected: 0 };
    let mut post_response = 0;
    if let Ok(upsert_result) = rs_weight_tracker::upsert_weight_for_date(
        &mut conn,
        payload.weight_value,
        payload.measurement_date,
    ) {
        // post_response.rows_affected = upsert_result;
        // Flash::success(Redirect::to("/"), "Todo successfully added.")
        json!({ "status": "ok", "rows": post_response })
    } else {
        json!({ "status": "ok", "rows": 0 })
    }
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("static")))
        .mount("/api", routes![rolling_average, add_weight])
}
