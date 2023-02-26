use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: add_weight [WEIGHT] [TIMESTAMP]");
        return Ok(());
    }

    let weight = args[1].parse::<f64>()?;

    let date_as_text = args[2].clone();
    let mut conn = rs_weight_tracker::establish_connection();

    let count = rs_weight_tracker::upsert_weight_for_date(&mut conn, weight, date_as_text)
        .map_err(|err| err.to_string())?;
    println!("Added {} new weight(s)", count);
    Ok(())
}
