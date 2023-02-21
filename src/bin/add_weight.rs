use chrono::NaiveDateTime;
use std::{env, error::Error};
use diesel::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: add_weight [WEIGHT] [TIMESTAMP]");
        return Ok(());
    }

    let weight = args[1].parse::<f64>()?;
    let timestamp = args[2].parse::<i64>()?;
    let measurement_date = NaiveDateTime::from_timestamp(timestamp, 0);

    let mut conn = rs_weight_tracker::establish_connection();

    let count = rs_weight_tracker::add_weight(&mut conn, &weight, &measurement_date)?;

    println!("Added {} new weight(s)", count);

    Ok(())
}
