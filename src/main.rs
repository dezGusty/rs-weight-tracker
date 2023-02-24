#[macro_use] extern crate rocket;

use chrono::NaiveDate;

#[get("/rolling_average?<start_date>&<end_date>&<days>")]
fn rolling_average(start_date: String, end_date: String, days: u32) -> String {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages = rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, days).unwrap();

    let result = averages.into_iter()
        .map(|(date, avg)| {
            format!("{}: {:.1} kg", date.format("%Y-%m-%d"), avg)
        })
        .collect::<Vec<String>>()
        .join("\n");

    result
}


#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![rolling_average])
}
