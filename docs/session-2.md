# 2023.02.21 ChatGPT session 2

## Input 1

Let's change the application a bit.
When storing the measurement_date, we only need the date; we don't need the time as well. And the weight measurements should be unique for each distinct day. Any attempt to add an entry for an existing day should simply replace the existing entry with the new one.

## Output 1

Sure, here are the changes you can make to the Weight model and the add_weight function to store only the date part of the measurement_date and to ensure that the weight measurements are unique for each distinct day:

1. Update the Weight model in models.rs to use the Date type from the chrono crate for the measurement_date field:

```rust
use chrono::NaiveDate;
use diesel::prelude::*;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "weights"]
pub struct Weight {
    pub id: Option<i32>,
    pub weight: f64,
    pub measurement_date: NaiveDate,
}

impl Weight {
    pub fn all() -> weights::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::schema::weights::dsl::*;
        weights.order(measurement_date.desc()).into_boxed()
    }
}

#[derive(Insertable, Debug)]
#[table_name = "weights"]
pub struct NewWeight {
    pub weight: f64,
    pub measurement_date: NaiveDate,
}

impl NewWeight {
    pub fn new(weight: f64, measurement_date: NaiveDate) -> Self {
        NewWeight {
            weight,
            measurement_date,
        }
    }
}

impl Weight {
    pub fn upsert(&self, conn: &SqliteConnection) -> QueryResult<usize> {
        use diesel::{insert_into, prelude::*, update};
        use schema::weights::dsl::*;

        let existing_weight = weights
            .filter(measurement_date.eq(self.measurement_date))
            .first::<Weight>(conn)
            .optional()?;

        if let Some(existing_weight) = existing_weight {
            update(existing_weight)
                .set(weight.eq(self.weight))
                .execute(conn)
        } else {
            insert_into(weights).values(self).execute(conn)
        }
    }
}
```

Here, we've updated the measurement_date field to use the NaiveDate type from chrono instead of NaiveDateTime. We've also added an upsert method to the Weight model that checks if there is an existing weight for the given date, and updates it if it exists, or inserts a new weight if it doesn't.

1. Update the add_weight function in lib.rs to use the new upsert method instead of insert:

```rust
pub fn add_weight(
    conn: &SqliteConnection,
    weight: &f64,
    measurement_date: &NaiveDate,
) -> QueryResult<usize> {
    let new_weight = NewWeight::new(*weight, *measurement_date);
    let weight = Weight {
        id: None,
        weight: new_weight.weight,
        measurement_date: new_weight.measurement_date,
    };
    weight.upsert(conn)
}
```

Here, we've updated the add_weight function to use the NewWeight::new method to create a NewWeight struct with the given weight and date, and the Weight::upsert method to add the weight to the database.

With these changes, the measurement_date field will only store the date part of the measurement date, and adding a weight for an existing day will simply replace the existing weight with the new weight.

## Analysis 1

There are recurring issues with the weight field

## Input 2

I want to rename the DB column for the `weights` table from `weight` to `weight_value`. Create a migration for this and provide me with the `up.sql` and `down.sql` content.

## Output 2

Sure, here are the migration files to rename the weight column to weight_value in the weights table:

up.sql:

```sql

ALTER TABLE weights RENAME TO tmp_weights;
CREATE TABLE weights (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    weight_value DOUBLE NOT NULL,
    measurement_date DATE NOT NULL UNIQUE
);
INSERT INTO weights (id, weight_value, measurement_date)
SELECT id, weight, measurement_date FROM tmp_weights;
DROP TABLE tmp_weights;
```

down.sql:

```sql

ALTER TABLE weights RENAME TO tmp_weights;
CREATE TABLE weights (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    weight DOUBLE NOT NULL,
    measurement_date DATE NOT NULL UNIQUE
);
INSERT INTO weights (id, weight, measurement_date)
SELECT id, weight_value, measurement_date FROM tmp_weights;
DROP TABLE tmp_weights;
```

Note that we first create a temporary table (tmp_weights) with the new schema, insert the data from the old table into the new table, and then drop the old table and rename the temporary table to the original name.

After running the migration, the Weight model in models.rs needs to be updated to reflect the new column name:

```rust

use chrono::NaiveDate;
use diesel::prelude::*;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "weights"]
pub struct Weight {
    pub id: Option<i32>,
    #[column_name = "weight_value"]
    pub weight: f64,
    pub measurement_date: NaiveDate,
}

impl Weight {
    pub fn all() -> weights::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::schema::weights::dsl::*;
        weights.order(measurement_date.desc()).into_boxed()
    }
}

#[derive(Insertable, Debug)]
#[table_name = "weights"]
pub struct NewWeight {
    #[column_name = "weight_value"]
    pub weight: f64,
    pub measurement_date: NaiveDate,
}

impl NewWeight {
    pub fn new(weight: f64, measurement_date: NaiveDate) -> Self {
        NewWeight {
            weight,
            measurement_date,
        }
    }
}

impl Weight {
    pub fn upsert(&self, conn: &SqliteConnection) -> QueryResult<usize> {
        use diesel::{insert_into, prelude::*, update};
        use schema::weights::dsl::*;

        let existing_weight = weights
            .filter(measurement_date.eq(self.measurement_date))
            .first::<Weight>(conn)
            .optional()?;

        if let Some(existing_weight) = existing_weight {
            update(existing_weight)
                .set(weight.eq(self.weight))
                .execute(conn)
        } else {
            insert_into(weights).values(self).execute(conn)
        }
    }
}
```

Here, we've added the #[column_name = "weight_value"] attribute to the weight field in the Weight model and the NewWeight model to reflect the new column name in the database

## Analysis 2

Executed

```cmd
diesel migration generate dateonly
```

Copy the content for up and down.sql

```cmd
diesel migration run
diesel migration redo
```

Instead of using the old field name, we'll update the field in the model as well

```rs
#[column_name = "weight_value"]
pub weight: f64,
```

becomes

```rs

pub struct Weight {
    pub id: i32,
    pub weight_value: f64,
    pub measurement_date: NaiveDate,
}
```

Some miscenllaneous changes - moved upsert functionality to lib.

## Input 3

Please add a function in `lib.rs` to return the entries between a start-date and an end-date. Use this function from a new script, stored in `src/bin/show_weight_interval.rs`.
Two mandatory arguments would specify the start and end dates, preferable as strings.

```cmd
cargo run --bin show_weight_interval [start] [end]
```

E.g.

```cmd
cargo run --bin show_weight_interval 2023-01-20 2023-02-22
```

Remember, When establishing the connection in `src/bin/show_weight_interval.rs`, please use a mutable variable, like this:

```rs
let mut conn = establish_connection();
```

## Output 3

Sure, here's the updated lib.rs with a new function to retrieve weights between a start and end date, and the show_weight_interval.rs script to display weights within the specified interval:

lib.rs:

```rust

use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rs_weight_tracker::models::Weight;
use rs_weight_tracker::schema::weights::dsl::*;

pub fn weights_between_dates(
    conn: &SqliteConnection,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> QueryResult<Vec<Weight>> {
    weights
        .filter(measurement_date.ge(start_date))
        .filter(measurement_date.le(end_date))
        .order(measurement_date.asc())
        .load::<Weight>(conn)
}

pub fn establish_connection() -> SqliteConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
```

show_weight_interval.rs:

```rust

use rs_weight_tracker::{establish_connection, models::Weight};
use std::{env, error::Error};
use chrono::{NaiveDate, ParseError, Datelike};
use diesel::prelude::*;

fn parse_date(date_string: &str) -> Result<NaiveDate, ParseError> {
    let mut parts = date_string.split('-');
    let year_str = parts.next().ok_or(ParseError::InvalidYear)?;
    let year = year_str.parse::<i32>()?;
    let month_str = parts.next().ok_or(ParseError::InvalidMonth)?;
    let month = month_str.parse::<u32>()?;
    let day_str = parts.next().ok_or(ParseError::InvalidDay)?;
    let day = day_str.parse::<u32>()?;
    Ok(NaiveDate::from_ymd(year, month, day))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: show_weight_interval [START_DATE] [END_DATE]");
        std::process::exit(1);
    }

    let start_date = parse_date(&args[1])?;
    let end_date = parse_date(&args[2])?;

    let conn = &mut establish_connection();
    let weights = weights_between_dates(conn, start_date, end_date)?;

    println!("Displaying {} weight(s)", weights.len());
    for weight in weights {
        println!(
            "{}: {:.1} kg",
            weight.measurement_date.format("%Y-%m-%d"),
            weight.weight
        );
    }

    Ok(())
}
```

Here, we've added a weights_between_dates function to lib.rs that takes a connection, a start date, and an end date, and retrieves all weights between those dates. We've also added a parse_date function to show_weight_interval.rs that parses a date string in the format "YYYY-MM-DD" into a NaiveDate. The main function in show_weight_interval.rs then retrieves the start and end dates from the command-line arguments, calls weights_between_dates to retrieve the weights in the specified interval, and displays the weights.

## Analysis 3

ChatGPT keeps using the non mutable definition DB access.
The correct definition would be this.
Also, just as a preferance, the use is moved into the function.

```rs
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
```

I don't know where it took ParseError::InvalidDay and the similar errors.
Ended up with an own version:

Realized another unnoticed error: the upsert_weight function updates the entire table. A filter needs to be added.

```rust
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
```
