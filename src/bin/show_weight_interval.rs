use chrono::NaiveDate;

use std::{env, error::Error};

#[derive(Debug, Clone)]
pub struct LocalParseError {
    pub message: String,
}

impl From<std::num::ParseIntError> for LocalParseError {
    fn from(_: std::num::ParseIntError) -> LocalParseError {
        LocalParseError {
            message: "Invalid data type".to_string(),
        }
    }
}

fn parse_date(date_string: &str) -> Result<NaiveDate, LocalParseError> {
    let mut parts = date_string.split('-');
    let year_str = parts.next().ok_or(LocalParseError {
        message: String::from("year problem"),
    })?;

    let year = year_str.parse::<i32>()?;
    let month_str = parts.next().ok_or(LocalParseError {
        message: String::from("month problem"),
    })?;
    let month = month_str.parse::<u32>()?;
    let day_str = parts.next().ok_or(LocalParseError {
        message: String::from("day problem"),
    })?;
    let day = day_str.parse::<u32>()?;
    Ok(NaiveDate::from_ymd(year, month, day))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: show_weight_interval [START_DATE] [END_DATE]");
        std::process::exit(1);
    }

    let start_date = parse_date(&args[1]);

    let start_date = match start_date {
        Ok(start_date) => start_date,
        Err(error) => panic!("Problem with parsing: {:?}", error),
    };

    let end_date = parse_date(&args[2]);
    let end_date = match end_date {
        Ok(end_date) => end_date,
        Err(error) => panic!("Problem with parsing: {:?}", error),
    };

    let mut conn = rs_weight_tracker::establish_connection();
    // let weights = rs_weight_tracker::weights_between_dates(&mut conn, start_date, end_date)?;
    let weights = rs_weight_tracker::weights_between_dates_with_interpolation(&mut conn, start_date, end_date)?;

    println!("Displaying {} weight(s)", weights.len());
    for weight in weights {
        println!(
            "{}: {:.1} kg {}",
            weight.0.measurement_date.format("%Y-%m-%d"),
            weight.0.weight_value,
            weight.1
        );
    }

    let averages = rs_weight_tracker::rolling_average_between_dates(
        &mut conn, 
        start_date, 
        end_date, 
        7)?;
    println!("Displaying {} average(s)", averages.len());
    for weight in averages {
        println!(
            "{}: {:.1} kg",
            weight.0.format("%Y-%m-%d"),
            weight.1
        );
    }

    Ok(())
}
