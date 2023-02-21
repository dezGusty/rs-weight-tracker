use rs_weight_tracker::{models::Weight};
use std::{env, error::Error};
use diesel::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let limit = args.get(1).map_or(Ok(None), |s| s.parse::<i64>().map(Some))?;

    use rs_weight_tracker::schema::weights::dsl::*;

    let mut conn = rs_weight_tracker::establish_connection();
    
    let results = match limit {
        Some(limit) => weights.limit(limit).load::<Weight>(&mut conn)?,
        None => weights.load::<Weight>(&mut conn)?,
    };

    println!("Displaying {} weight(s)", results.len());
    for item in results {
        println!(
            "{:>3}. {:<10} {}",
            item.id,
            item.weight,
            item.measurement_date.format("%Y-%m-%d %H:%M:%S")
        );
    }

    Ok(())
}
