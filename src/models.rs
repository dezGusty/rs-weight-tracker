use super::schema::weights;
use chrono::NaiveDate;
use diesel::prelude::*;

#[derive(Identifiable, Insertable, Queryable, Debug)]
pub struct Weight {
    pub id: i32,
    pub weight_value: f64,
    pub measurement_date: NaiveDate,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = weights)]
pub struct NewWeight {
    pub weight_value: f64,
    pub measurement_date: NaiveDate,
}

impl Weight {
    pub fn all() -> weights::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::schema::weights::dsl::*;
        weights.order(measurement_date.desc()).into_boxed()
    }
}

impl NewWeight {
    pub fn new(weight_value: f64, measurement_date: NaiveDate) -> Self {
        NewWeight {
            weight_value,
            measurement_date,
        }
    }
}

impl Weight {
    pub fn upsert(&self, conn: &mut SqliteConnection) -> QueryResult<usize> {
        use crate::schema::weights::dsl::*;
        use diesel::{insert_into, prelude::*, update};

        let existing_weight = weights
            .filter(measurement_date.eq(self.measurement_date))
            .first::<Weight>(conn)
            .optional()?;

        if let Some(_) = existing_weight {
            update(weights)
                .set(weight_value.eq(self.weight_value))
                .execute(conn)
        } else {
            insert_into(weights).values(self).execute(conn)
        }
    }
}
