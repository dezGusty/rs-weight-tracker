pub mod models;
pub mod schema;

use chrono::NaiveDate;
use models::{NewWeight, Weight};

use diesel::{prelude::*, SqliteConnection};
use dotenv::dotenv;
use std::env;

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

pub fn weights_between_dates_with_interpolation_vec(
    conn: &mut SqliteConnection,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> QueryResult<Vec<Weight>> {
    let actual_weights = weights_between_dates(conn, start_date, end_date)?;

    let mut interpolated_weights = Vec::new();
    let mut current_weight = None;
    let mut current_date = start_date;
    for weight in actual_weights {
        while current_date < weight.measurement_date {
            let interpolated_weight = current_weight.map(|weight_value| {
                let weight_ratio =
                    (current_date - start_date).num_days() as f64 / (weight.measurement_date - start_date).num_days() as f64;
                let interpolated_weight_value = weight_value + weight_ratio * (weight.weight_value - weight_value);
                Weight {
                    id: 0,
                    weight_value: interpolated_weight_value,
                    measurement_date: current_date,
                }
            });
            if let Some(interpolated_weight) = interpolated_weight {
                interpolated_weights.push(interpolated_weight);
            }
            current_date = current_date.succ();
        }
        current_weight = Some(weight.weight_value);
        current_date = weight.measurement_date.succ();
        interpolated_weights.push(weight);
    }
    while current_date <= end_date {
        let interpolated_weight = current_weight.map(|weight_value| {
            Weight {
                id: 0,
                weight_value,
                measurement_date: current_date,
            }
        });
        if let Some(interpolated_weight) = interpolated_weight {
            interpolated_weights.push(interpolated_weight);
        }
        current_date = current_date.succ();
    }

    Ok(interpolated_weights)
}

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
                let weight_ratio =
                    (current_date - start_date).num_days() as f64 / (weight.measurement_date - start_date).num_days() as f64;
                let interpolated_weight_value = weight_value + weight_ratio * (weight.weight_value - weight_value);
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
            current_date = current_date.succ();
        }
        current_weight = Some(weight.weight_value);
        current_date = weight.measurement_date.succ();
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
        current_date = current_date.succ();
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

    let weights = weights_between_dates_with_interpolation(conn, start_date, end_date)?;

    let mut date_weights = Vec::new();
    for (weight, is_interpolated) in weights {
        if is_interpolated {
            continue;
        }
        date_weights.push((weight.measurement_date, weight.weight_value));
    }

    let mut rolling_averages = Vec::new();
    let mut sum = 0.0;
    let mut count = 0;
    let mut window = date_weights.iter().rev().take(amount_of_days as usize);
    let mut date = start_date;
    while date <= end_date {
        while let Some(&(window_date, weight)) = window.last() {
            if window_date != date {
                break;
            }
            window.next();
            sum += weight;
            count += 1;
        }
        if count > 0 {
            let rolling_average = sum / count as f64;
            rolling_averages.push((date, rolling_average));
        }
        if let Some(&(window_date, _)) = window.next() {
            sum -= date_weights.iter().rev().skip(amount_of_days as usize).take_while(|&&(d, _)| d == window_date).map(|&(_, w)| w).sum::<f64>();
            count -= date_weights.iter().rev().skip(amount_of_days as usize).take_while(|&&(d, _)| d == window_date).count();
        }
        date = date.succ();
    }

    Ok(rolling_averages)
}
