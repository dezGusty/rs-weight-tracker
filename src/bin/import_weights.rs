use chrono::NaiveDateTime;
use rs_weight_tracker::{add_weight, establish_connection};
use serde::Deserialize;
use std::{env, error::Error, fs::File};

#[derive(Deserialize)]
struct WeightData {
    weights: Vec<Weight>,
}

#[derive(Deserialize)]
struct Weight {
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

    let mut conn = establish_connection();

    for weight in data.weights {
        let measurement_date = NaiveDateTime::from_timestamp(weight.date / 1000, 0);
        let count = add_weight(&mut conn, &weight.weight, &measurement_date)?;
        println!("Added {} new weight(s)", count);
    }

    Ok(())
}
