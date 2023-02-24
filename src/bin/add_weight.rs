use chrono::{Datelike, NaiveDate, NaiveDateTime};
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: add_weight [WEIGHT] [TIMESTAMP]");
        return Ok(());
    }

    let weight = args[1].parse::<f64>()?;
    let timestamp = args[2].parse::<i64>()?;
    let measurement_datetime =
        NaiveDateTime::from_timestamp_opt(timestamp, 0).ok_or("Invalid timestamp")?;

    if let Some(measurement_date) = NaiveDate::from_ymd_opt(
        measurement_datetime.year(),
        measurement_datetime.month(),
        measurement_datetime.day(),
    ) {
        let mut conn = rs_weight_tracker::establish_connection();

        let count = rs_weight_tracker::add_weight(&mut conn, weight, measurement_date)?;

        println!("Added {} new weight(s)", count);

        Ok(())
    } else {
        eprintln!("Failed to obtain date from date time");
        return Ok(());
    }
}
