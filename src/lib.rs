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
