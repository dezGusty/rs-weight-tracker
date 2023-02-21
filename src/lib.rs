mod models;
mod schema;

use chrono::NaiveDateTime;
use models::{NewWeight};

use diesel::{prelude::*, SqliteConnection};

pub fn add_weight(
    conn: &mut SqliteConnection,
    weight: &f64,
    measurement_date: &NaiveDateTime,
) -> Result<usize, diesel::result::Error> {
    let new_weight = NewWeight {
        weight,
        measurement_date,
    };
    use schema::weights;
    diesel::insert_into(weights::table)
        .values(&new_weight)
        .execute(conn)
}
