#[macro_use]
extern crate rocket;

use chrono::NaiveDate;
use rocket::serde::json::Json;

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

use rocket::fs::{relative, FileServer};

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("static")))
        .mount("/api", routes![rolling_average])
}
