
# 2023.02.24 ChatGPT session 4

## Input 1

I want to make this functionality available in a web application, so that I can call an endpoint in a browser and view json data.
Choose a Rust crate to provide the web support and refactor the code so that we provide a web-api to show the rolling average between two dates (reusing the `rolling_average_between_dates` from `lib.rs`)

## Output 1

There are a few web frameworks available in Rust that you could use to build a web application with an API, such as Rocket, Actix-web, and Warp.

For this purpose, I'll suggest using the Rocket framework as it is easy to use and has great documentation.

Here is how you can create a web API to show the rolling average between two dates using Rocket:

1. Add rocket and serde and serde_json dependencies to your Cargo.toml file:

```toml

[dependencies]
rocket = "0.5.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

1. Create a new file called api.rs and add the following code to it:

```rust

use crate::models::Weight;
use crate::lib::rolling_average_between_dates;
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct DatesQuery {
    start_date: String,
    end_date: String,
    amount_of_days: u8,
}

#[derive(Serialize)]
struct RollingAverageResponse {
    date: String,
    value: f64,
    is_interpolated: bool,
}

#[get("/rolling_average?<dates>")]
pub fn rolling_average(dates: DatesQuery) -> Option<rocket::response::Json<Vec<RollingAverageResponse>>> {
    let start_date = dates.start_date.parse().ok()?;
    let end_date = dates.end_date.parse().ok()?;
    let amount_of_days = dates.amount_of_days;

    let rolling_average = rolling_average_between_dates(start_date, end_date, amount_of_days).ok()?;
    let response: Vec<RollingAverageResponse> = rolling_average
        .iter()
        .map(|(weight, is_interpolated)| RollingAverageResponse {
            date: weight.measurement_date.to_string(),
            value: weight.weight_value,
            is_interpolated: *is_interpolated,
        })
        .collect();

    Some(rocket::response::Json(response))
}
```

1. In your main.rs file, add the following code to the rocket::build() function:

```rust

mod api;

fn main() {
    // ...
    rocket::build()
        .mount("/", routes![api::rolling_average])
        // ...
}
```

This adds the rolling_average endpoint to the application.

1. To test it, you can run cargo run and then navigate to <http://localhost:8000/rolling_average?start_date=2022-01-01&end_date=2022-02-01&amount_of_days=7> in your browser. You should see a JSON response with the rolling average values for the specified dates.

Please note that in a real-world application, you would need to add error handling, authentication, and other features to ensure the security and reliability of your API.

## Analysis 1

Additions to toml... need to be adjusted.
Also, the code does not compile yet.

## Input 2

This doesn't quite match what I was hoping to reuse. Allow me to clarify.
I have this code for the console app (`/src/bin/show_weight_interval.rs`) making use of the `rolling_average_between_dates` function.

```rust
// show_weight_interval.rs

use chrono::{NaiveDate};

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
    if let Some(result) = NaiveDate::from_ymd_opt(year, month, day) {
        return Ok(result);
    }

    Err(LocalParseError{ message: String::from("invalid date")})
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
    let averages =
        rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, 7)?;
    println!("Displaying {} average(s)", averages.len());
    for weight in averages {
        println!("{}: {:.1} kg", weight.0.format("%Y-%m-%d"), weight.1);
    }

    Ok(())
}

```

Please adapt the code so that it can be integrated into the rocket application.

## Output 2

Sure! Here is an updated main.rs for your Rocket application that includes the endpoint to get the rolling average between two dates:

```rust

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rs_weight_tracker::{models::{Weight}, rolling_average_between_dates, establish_connection};
use chrono::NaiveDate;

#[get("/rolling_average?<start_date>&<end_date>&<days>")]
fn rolling_average(start_date: String, end_date: String, days: i32) -> String {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = establish_connection();
    let averages = rolling_average_between_dates(&mut conn, start_date, end_date, days).unwrap();

    let result = averages.into_iter()
        .map(|(date, avg, is_interpolated)| {
            format!("{}: {:.1} kg ({})", date.format("%Y-%m-%d"), avg, if is_interpolated { "interpolated" } else { "original" })
        })
        .collect::<Vec<String>>()
        .join("\n");

    result
}

fn main() {
    rocket::ignite()
        .mount("/", routes![rolling_average])
        .launch();
}
```

This defines an endpoint at /rolling_average that takes three query parameters: start_date, end_date, and days. It parses the start and end dates, calculates the rolling averages for the specified interval using the rolling_average_between_dates function from lib.rs, and returns the result as a string.

For example, to get the rolling averages for the interval from 2023-01-01 to 2023-01-15 with a rolling window of 7 days, you would make a GET request to:

```bash
http://localhost:8000/rolling_average?start_date=2023-01-01&end_date=2023-01-15&days
```

## Analysis 2

Almost. It did not take Json into account. That may be due to the rephrasing.
It still tries to use the interpolation flag, which makes the code fail compilation.
The main function also needs to be specified differently.

## Input 3

Thanks. Your version is almost correct. I'm using another version of rocket, so there are changes in the required macros
The version from my `Cargo.toml`:

```toml
[dependencies]
rocket = { version = "0.5.0-rc.2", features = ["tls", "mtls", "json"] }
```

I corrected the main.rs file to contain the code:

```rust
#[macro_use] extern crate rocket;

use chrono::NaiveDate;

#[get("/rolling_average?<start_date>&<end_date>&<days>")]
fn rolling_average(start_date: String, end_date: String, days: u32) -> String {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages = rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, days).unwrap();

    let result = averages.into_iter()
        .map(|(date, avg)| {
            format!("{}: {:.1} kg", date.format("%Y-%m-%d"), avg)
        })
        .collect::<Vec<String>>()
        .join("\n");

    result
}


#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![rolling_average])
}

```

Starting from this correct version, help me refactor it so that it returns the data in Json format instead of plain text.