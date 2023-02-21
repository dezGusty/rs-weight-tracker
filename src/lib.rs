mod models;
mod schema;

use chrono::NaiveDateTime;
use models::{NewWeight};

use diesel::{prelude::*, SqliteConnection};
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> SqliteConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  SqliteConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

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
