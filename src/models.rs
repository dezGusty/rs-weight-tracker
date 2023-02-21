use chrono::NaiveDateTime;
use diesel::prelude::*;
use super::schema::weights;

#[derive(Queryable, Debug)]
pub struct Weight {
    pub id: i32,
    pub weight: f64,
    pub measurement_date: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "weights"]
pub struct NewWeight<'a> {
    pub weight: &'a f64,
    pub measurement_date: &'a NaiveDateTime,
}

impl Weight {
    pub fn new(id: i32, weight: f64, measurement_date: NaiveDateTime) -> Self {
        Self {
            id,
            weight,
            measurement_date,
        }
    }
}
