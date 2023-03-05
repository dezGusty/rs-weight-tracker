pub mod models;
pub mod schema;

use chrono::NaiveDate;
pub use models::{NewWeight, Weight};

use diesel::{prelude::*, SqliteConnection};
use dotenvy::dotenv;
use std::{env, fmt};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn add_weight(
    conn: &mut SqliteConnection,
    weight_value: f64,
    measurement_date: NaiveDate,
) -> Result<usize, diesel::result::Error> {
    let new_weight = NewWeight {
        weight_value,
        measurement_date,
    };
    use schema::weights;
    diesel::insert_into(weights::table)
        .values(&new_weight)
        .execute(conn)
}

#[derive(Debug, Clone)]
pub struct LocalParseError {
    pub message: String,
}

impl fmt::Display for LocalParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LPErr {}", self.message)
    }
}

impl From<std::num::ParseIntError> for LocalParseError {
    fn from(_: std::num::ParseIntError) -> LocalParseError {
        LocalParseError {
            message: "Invalid data type".to_string(),
        }
    }
}

impl From<diesel::result::Error> for LocalParseError {
    fn from(err: diesel::result::Error) -> LocalParseError {
        LocalParseError {
            message: format!("Some db error {}", err.to_string()),
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
    if let Some(result) = NaiveDate::from_ymd_opt(year, month, day) {
        return Ok(result);
    }

    Err(LocalParseError {
        message: String::from("invalid date"),
    })
}

pub fn upsert_weight(
    conn: &mut SqliteConnection,
    in_weight_value: f64,
    in_measurement_date: NaiveDate,
) -> QueryResult<usize> {
    use crate::schema::weights::dsl::*;
    use diesel::{prelude::*, update};

    let existing_weight = weights
        .filter(measurement_date.eq(in_measurement_date))
        .first::<Weight>(conn)
        .optional()?;

    if let Some(existing_weight) = existing_weight {
        update(weights)
            .filter(id.eq(existing_weight.id))
            .set(weight_value.eq(in_weight_value))
            .execute(conn)
    } else {
        add_weight(conn, in_weight_value, in_measurement_date)
    }
}

pub fn upsert_weight_for_date(
    conn: &mut SqliteConnection,
    in_weight_value: f64,
    in_measurement_date: String,
) -> Result<usize, LocalParseError> {
    let measurement_date = parse_date(&in_measurement_date)?;

    let result = upsert_weight(conn, in_weight_value, measurement_date)?;
    Ok(result)
}

pub fn weights_between_dates(
    conn: &mut SqliteConnection,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> QueryResult<Vec<Weight>> {
    use crate::schema::weights::dsl::*;
    weights
        .filter(measurement_date.ge(start_date))
        .filter(measurement_date.le(end_date))
        .order(measurement_date.asc())
        .load::<Weight>(conn)
}

/// Returns a vector of weights between two given dates, with additional interpolated weights added
/// to fill gaps between the actual weights. The interpolated weights are calculated by linearly
/// interpolating between adjacent actual weights.
///
/// # Arguments
///
/// * `conn` - The connection to the SQLite database
/// * `start_date` - The starting date for the weight range (inclusive)
/// * `end_date` - The ending date for the weight range (inclusive)
///
/// # Returns
///
/// A `QueryResult` with a vector of tuples, where the first element of the tuple is a `Weight`
/// struct representing the actual or interpolated weight, and the second element is a boolean
/// indicating whether the weight is interpolated or not.
///
/// # Examples
///
/// ```rust
/// use rs_weight_tracker::Weight;
/// use chrono::{NaiveDate};
///
/// let mut conn = rs_weight_tracker::establish_connection();
/// let start_date = NaiveDate::from_ymd(2021, 1, 1);
/// let end_date = NaiveDate::from_ymd(2021, 1, 31);
///
/// let weights = rs_weight_tracker::weights_between_dates_with_interpolation(&mut conn, start_date, end_date);
/// ```
///
/// This will return a vector of weights between the 1st and 31st of January 2021, with additional
/// interpolated weights added to fill gaps between the actual weights.
pub fn weights_between_dates_with_interpolation(
    conn: &mut SqliteConnection,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> QueryResult<Vec<(Weight, bool)>> {
    let weights = weights_between_dates(conn, start_date, end_date)?;

    let mut interpolated_weights = Vec::new();
    let mut current_weight = None;
    let mut current_date = start_date;
    for weight in weights {
        while current_date < weight.measurement_date {
            let interpolated_weight = current_weight.map(|weight_value| {
                let weight_ratio = (current_date - start_date).num_days() as f64
                    / (weight.measurement_date - start_date).num_days() as f64;
                let interpolated_weight_value =
                    weight_value + weight_ratio * (weight.weight_value - weight_value);
                let interpolated_weight = Weight {
                    weight_value: interpolated_weight_value,
                    measurement_date: current_date,
                    id: 0,
                };
                (interpolated_weight, true)
            });
            if let Some((interpolated_weight, is_interpolated)) = interpolated_weight {
                interpolated_weights.push((interpolated_weight, is_interpolated));
            }
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }
        current_weight = Some(weight.weight_value);
        current_date = weight.measurement_date.succ_opt().unwrap_or(current_date);
        interpolated_weights.push((weight, false));
    }
    while current_date <= end_date {
        let interpolated_weight = current_weight.map(|weight_value| {
            let interpolated_weight = Weight {
                weight_value,
                measurement_date: current_date,
                id: 0,
            };
            (interpolated_weight, true)
        });
        if let Some((interpolated_weight, is_interpolated)) = interpolated_weight {
            interpolated_weights.push((interpolated_weight, is_interpolated));
        }
        current_date = current_date.succ_opt().unwrap_or(current_date);
    }

    Ok(interpolated_weights)
}

pub fn rolling_average_between_dates(
    conn: &mut SqliteConnection,
    start_date: NaiveDate,
    end_date: NaiveDate,
    amount_of_days: u32,
) -> QueryResult<Vec<(NaiveDate, f64)>> {
    assert!(amount_of_days > 0 && amount_of_days <= 7);

    let start_date_in_advance = start_date - chrono::Duration::days(amount_of_days as i64 - 1);

    let weights = weights_between_dates_with_interpolation(conn, start_date_in_advance, end_date)?;

    let mut rolling_window: Vec<f64> = vec![0.0; amount_of_days as usize];
    let mut rolling_sum = 0.0;
    let mut results = Vec::new();

    for (weight, _) in &weights {
        let value = weight.weight_value;
        rolling_sum += value;

        if rolling_window.len() == amount_of_days as usize {
            rolling_sum -= rolling_window[0];
            rolling_window.remove(0);
        }

        rolling_window.push(value);

        let days = (weight.measurement_date - start_date_in_advance).num_days() as u32;

        if days >= amount_of_days - 1 {
            let average = rolling_sum / rolling_window.len() as f64;
            results.push((weight.measurement_date, average));
        }
    }

    Ok(results)
}
