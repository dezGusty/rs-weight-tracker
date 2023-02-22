use chrono::{Datelike, NaiveDate, NaiveDateTime};
use rs_weight_tracker::schema::weights::measurement_date;
use serde::Deserialize;
use std::{env, error::Error, fs::File};

#[derive(Deserialize)]
struct WeightData {
    weights: Vec<JsonWeight>,
}

#[derive(Deserialize)]
struct JsonWeight {
    date: i64,
    weight: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: import_weights [filename.json]");
        return Ok(());
    }

    let filename = &args[1];
    let file = File::open(filename)?;
    let data: WeightData = serde_json::from_reader(file)?;

    let mut conn = rs_weight_tracker::establish_connection();

    let mut total_count = 0;
    for json_weight in data.weights {
        let measurement_datetime: NaiveDateTime =
            NaiveDateTime::from_timestamp(json_weight.date / 1000, 0);

        if let Some(date_of_measurement) = NaiveDate::from_ymd_opt(
            measurement_datetime.year(),
            measurement_datetime.month(),
            measurement_datetime.day(),
        ) {
            let count =
                rs_weight_tracker::upsert_weight(&mut conn, json_weight.weight, date_of_measurement)?;
            total_count += count;
            println!("Added {} new weight(s)", count);
        } else {
            eprintln!("Failed to obtain date from date time");
            return Ok(());
        }
    }

    println!("Added a total of {} new weight(s)", total_count);

    Ok(())
}
