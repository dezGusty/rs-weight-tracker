# 2023.02.27  ChatGPT session 7

## Input 1

I think I want to also try out the `Axum` crate to serve as the web server instead of `Rocket`.

My `main.rs` file looks like this:

```rust
#[macro_use]
extern crate rocket;

use chrono::NaiveDate;
use rocket::fs::{relative, FileServer};
use rocket::serde::json::Json;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AddWeightPayload {
    pub weight_value: f64,
    pub measurement_date: String,
}

#[get("/rolling_average?<start_date>&<end_date>&<days>")]
fn rolling_average(
    start_date: String,
    end_date: String,
    days: u32,
) -> Json<Vec<serde_json::Value>> {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages =
        rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, days)
            .unwrap();

    let result = averages
        .into_iter()
        .map(|(date, avg)| {
            let mut map = serde_json::Map::new();
            map.insert(
                "date".to_string(),
                serde_json::Value::String(date.format("%Y-%m-%d").to_string()),
            );
            map.insert(
                "average".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(avg).unwrap()),
            );
            serde_json::Value::Object(map)
        })
        .collect::<Vec<serde_json::Value>>();

    Json(result)
}

#[post("/add_weight", format = "json", data = "<payload>")]
fn add_weight(payload: Json<AddWeightPayload>) -> Value {
    let mut conn = rs_weight_tracker::establish_connection();
    let Json(payload) = payload;

    if let Ok(changed_entries_count) = rs_weight_tracker::upsert_weight_for_date(
        &mut conn,
        payload.weight_value,
        payload.measurement_date,
    ) {
        json!({ "status": "ok", "rows": changed_entries_count })
    } else {
        json!({ "status": "ok", "rows": 0 })
    }
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("static")))
        .mount("/api", routes![rolling_average, add_weight])
}

```

How would the `main.rs` and `Cargo.toml` change ?

## Output 1

To switch from Rocket to Axum, you'll need to make a few changes to both Cargo.toml and main.rs.

In Cargo.toml, you need to remove the rocket dependency and add the axum and tower dependencies. You can do this by updating the [dependencies] section as follows:

```toml

[dependencies]
axum = "0.2"
tower = "0.4"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlite = "0.28"
```

In main.rs, you need to replace the rocket imports and macros with axum imports and macros. Here's an updated version of main.rs:

```rust

use axum::{handler::get, Router};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{json, Value};
use tower::ServiceBuilder;

#[derive(Debug, Deserialize)]
pub struct AddWeightPayload {
    pub weight_value: f64,
    pub measurement_date: String,
}

async fn rolling_average(
    start_date: String,
    end_date: String,
    days: u32,
) -> Result<serde_json::Value, tower::BoxError> {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages =
        rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, days)
            .unwrap();

    let result = averages
        .into_iter()
        .map(|(date, avg)| {
            let mut map = serde_json::Map::new();
            map.insert(
                "date".to_string(),
                serde_json::Value::String(date.format("%Y-%m-%d").to_string()),
            );
            map.insert(
                "average".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(avg).unwrap()),
            );
            serde_json::Value::Object(map)
        })
        .collect::<Vec<serde_json::Value>>();

    Ok(json!(result))
}

async fn add_weight(payload: axum::extract::Json<AddWeightPayload>) -> Result<Value, tower::BoxError> {
    let mut conn = rs_weight_tracker::establish_connection();
    let payload = payload.into_inner();

    if let Ok(changed_entries_count) = rs_weight_tracker::upsert_weight_for_date(
        &mut conn,
        payload.weight_value,
        payload.measurement_date,
    ) {
        Ok(json!({ "status": "ok", "rows": changed_entries_count }))
    } else {
        Ok(json!({ "status": "ok", "rows": 0 }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/rolling_average", get(rolling_average))
        .route("/api/add_weight", axum::handler::post(add_weight));

    let app = ServiceBuilder::new().buffer(100, 10).service(app);

    axum::Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

Note that the #[tokio::main] macro is used to set up

## Analysis 2

The explanation seems cut off

## Input 3

continue

## Output 3

Here is the updated main.rs file using Axum:

```rust

use std::net::SocketAddr;
use axum::{
    handler::{get, post},
    http::StatusCode,
    Router,
    body::Body,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct AddWeightPayload {
    pub weight_value: f64,
    pub measurement_date: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_url = "sqlite:weight_tracker.db";
    let pool = SqlitePool::connect(db_url).await?;

    let app = Router::new()
        .route("/rolling_average", get(rolling_average))
        .route("/add_weight", post(add_weight))
        .route("/", get(|| async { "Hello, world!" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("Server running on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| format!("server error: {}", e))?;

    Ok(())
}

async fn rolling_average(
    start_date: String,
    end_date: String,
    days: u32,
) -> Result<Json<Vec<Value>>, StatusCode> {
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let mut conn = rs_weight_tracker::establish_connection();
    let averages =
        rs_weight_tracker::rolling_average_between_dates(&mut conn, start_date, end_date, days)
            .unwrap();

    let result = averages
        .into_iter()
        .map(|(date, avg)| {
            let mut map = serde_json::Map::new();
            map.insert(
                "date".to_string(),
                serde_json::Value::String(date.format("%Y-%m-%d").to_string()),
            );
            map.insert(
                "average".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(avg).unwrap()),
            );
            serde_json::Value::Object(map)
        })
        .collect::<Vec<serde_json::Value>>();

    Ok(Json(result))
}

async fn add_weight(
    payload: Json<AddWeightPayload>,
    pool: axum::extract::Extension<SqlitePool>,
) -> Result<Json<Value>, StatusCode> {
    let payload = payload.0;

    let changed_entries_count = rs_weight_tracker::upsert_weight_for_date(
        &mut pool.acquire().await.unwrap(),
        payload.weight_value,
        payload.measurement_date,
    ).unwrap_or(0);

    Ok(Json(json!({ "status": "ok", "rows": changed_entries_count })))
}
```

The Cargo.toml file will need to be updated to include the axum and tokio dependencies:

```toml

[dependencies]
sqlx = { version = "0.5.8", features = ["sqlite"] }
chrono = { version = "0.4.19", features = ["serde"] }
serde_json = "1.0"
axum = "0.2"
tokio = { version = "1.16", features = ["full"] }

[dependencies.rs_weight_tracker]
path = "../rs-weight-tracker"
```

Note that we need to

## Analysis 3

Keeps getting cut off. This required more work.

The dependencies are wrong for sqlx, but they need

```toml
axum = "0.6.9"
tower = { version = "0.4", features = ["util"] }
tokio = { version = "1.25.0", features = ["macros", "rt-multi-thread"] }
tower-http = { version = "0.4.0", features = ["fs", "trace"] }
```

## Input 4

I made some changes but I got it to work with axum; thanks

## Output 4

You're welcome! If you have any other questions or issues, don't hesitate to ask.

## Input 5

I do actually.
Do you remember the 2 html files we have?
I'd like to add some styling to them. Let's start with the main page (`index.html`)

```html
<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rolling Average</title>
  <link rel="shortcut icon" type="image/x-icon" href="./favicon.ico" />
</head>

<body>
  <a href="add-weight.html">Add Weight</a>

  <form>
    <label for="start-date">Start Date:</label>
    <input type="date" id="start-date" name="start-date"><br>

    <label for="end-date">End Date:</label>
    <input type="date" id="end-date" name="end-date"><br>

    <label for="days">Days:</label>
    <input type="number" id="days" name="days" min="1" max="7" value="7"><br>

    <button type="submit" id="submit">Submit</button>
  </form>

  <div>
    <canvas id="chart"></canvas>
  </div>

  <table id="result-table">
    <thead>
      <tr>
        <th>Date</th>
        <th>Weight</th>
      </tr>
    </thead>
    <tbody></tbody>
  </table>

  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <script>
    // when deploying on a server, the api path could be https://server.com/app-1-path/api
    // and we'd call it from https://server.com/app-1-path/ or https://server.com/app-1-path/index.html
    // or https://server.com/app-1-path/subpage/page1.html
    let subpaths = window.location.pathname.split('/');
    let subpath = '';
    if (subpaths && subpaths.length > 0) {
      subpath = subpath + subpaths[0];
      if (subpaths[0] == '' && subpaths.length > 1) {
        if (!subpaths[1].endsWith('.html')) {
          subpath = subpath + '/' + subpaths[1];
        }
      }
    }

    let baseUrl = window.location.origin + subpath;

    const form = document.querySelector('form');
    const tableBody = document.querySelector('table tbody');
    let chart;

    form.addEventListener('submit', (event) => {
      event.preventDefault();

      const startDate = document.getElementById('start-date').value;
      const endDate = document.getElementById('end-date').value;
      const days = document.getElementById('days').value;

      fetch(`${baseUrl}/api/rolling_average?start_date=${startDate}&end_date=${endDate}&days=${days}`)
        .then(response => response.json())
        .then(data => {
          tableBody.innerHTML = '';
          data.forEach(entry => {
            const row = document.createElement('tr');
            const dateCell = document.createElement('td');
            const weightCell = document.createElement('td');

            dateCell.textContent = entry.date;
            weightCell.textContent = `${entry.average.toFixed(1)} kg`;

            row.appendChild(dateCell);
            row.appendChild(weightCell);

            tableBody.appendChild(row);
          });

          if (chart) {
            chart.destroy();
          }

          const chartCanvas = document.getElementById('chart');
          chart = new Chart(chartCanvas, {
            type: 'line',
            data: {
              labels: data.map(entry => entry.date),
              datasets: [
                {
                  label: 'Rolling Average',
                  data: data.map(entry => entry.average.toFixed(1)),
                  fill: false,
                  borderColor: 'rgb(75, 192, 192)',
                  tension: 0.1,
                },
              ],
            },
            options: {
              scales: {
                y: {
                  beginAtZero: false,
                },
              },
            },
          });
        })
        .catch(error => {
          console.error(error);
        });
    });
  </script>
</body>

</html>
```

Can you add some CSS styling to it ?

## Output 5

Sure, here's an example of how you could add some CSS styling to the index.html file:

```html

<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rolling Average</title>
  <link rel="shortcut icon" type="image/x-icon" href="./favicon.ico" />

  <style>
    body {
      font-family: Arial, sans-serif;
    }

    h1 {
      text-align: center;
    }

    form {
      display: flex;
      flex-direction: column;
      align-items: center;
      margin-top: 20px;
    }

    form label {
      margin-top: 10px;
    }

    form input[type="date"],
    form input[type="number"],
    form button {
      margin: 5px;
      padding: 5px;
      border-radius: 5px;
      border: none;
      background-color: #eee;
      font-size: 16px;
    }

    form button {
      background-color: #2f80ed;
      color: #fff;
      cursor: pointer;
    }

    table {
      border-collapse: collapse;
      margin-top: 20px;
      margin-bottom: 20px;
    }

    th, td {
      border: 1px solid black;
      padding: 5px;
      text-align: center;
    }

    #chart {
      width: 80%;
      margin: auto;
      margin-top: 20px;
      margin-bottom: 20px;
    }
  </style>
</head>

<body>
  <h1>Rolling Average</h1>

  <a href="add-weight.html">Add Weight</a>

  <form>
    <label for="start-date">Start Date:</label>
    <input type="date" id="start-date" name="start-date">

    <label for="end-date">End Date:</label>
    <input type="date" id="end-date" name="end-date">

    <label for="days">Days:</label>
    <input type="number" id="days" name="days" min="1" max="7" value="7">

    <button type="submit" id="submit">Submit</button>
  </form>

  <table id="result-table">
    <thead>
      <tr>
        <th>Date</th>
        <th>Weight</th>
      </tr>
    </thead>
    <tbody></tbody>
  </table>

  <div>
    <canvas id="chart"></canvas>
  </div>

  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <script>
    // when deploying on a server, the api path could be https://server.com/app-1-path/api
    // and we'd call it from https://server.com/app-1-path/ or https://server.com/app-1-path/index.html
    // or https://server.com/app-1-path/subpage/page1.html
    let subpaths = window.location.pathname.split('/');
    let subpath = '';
    if (subpaths && subpaths.length > 0) {
      subpath = subpath + subpaths[0];
      if (subpaths[0] == '' && subpaths.length > 1) {
        if (!subpaths[1].endsWith('.html')) {
          subpath = subpath + '/' + subpaths[1];
        }
      }
    }

    let baseUrl = window.location.origin + subpath;

    const form =
```

## Input 6

your response got interrupted, please go on

## Output 6

Sure! Here's the CSS for the index.html file:

```css

body {
  font-family: sans-serif;
  margin: 0;
  padding: 0;
}

header {
  background-color: #343a40;
  color: white;
  padding: 0.5rem;
}

header h1 {
  margin: 0;
  padding: 0;
}

a {
  color: #007bff;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

main {
  margin: 1rem;
}

form {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

form label {
  display: block;
  margin-bottom: 0.5rem;
}

form input[type='date'],
form input[type='number'] {
  padding: 0.5rem;
  border-radius: 0.25rem;
  border: 1px solid #ced4da;
  background-color: #fff;
  color: #495057;
}

form input[type='date']:focus,
form input[type='number']:focus {
  border-color: #007bff;
  outline: none;
}

form button[type='submit'] {
  background-color: #007bff;
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.25rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

form button[type='submit']:hover {
  background-color: #0069d9;
}

table {
  border-collapse: collapse;
  width: 100%;
}

table th,
table td {
  border: 1px solid #dee2e6;
  padding: 0.5rem;
  text-align: left;
  vertical-align: top;
}

table th {
  background-color: #e9ecef;
}

canvas {
  margin-top: 2rem;
}
```

You can add this CSS to the static/css/style.css file and include it in the index.html file using the following line in the <head> section:

```html

<link rel="stylesheet" type="text/css" href="css/style.css">
```

This assumes that the style.css file is located in the static/css directory.

## Input 7

can you make the layout responsive, with a mobile-first approach ?

## Output 7

(skipped). Some CSS.

## Input 8

Thanks, can you show me another one ? I'd like to try out several versions

## Output 8

(skipped). Some CSS.

## Input 9

Can you give me another version? I'd like something with a touch of orange.
And I'd like you to only provide styling to the classes used in the html section:

```html
  <a href="add-weight.html">Add Weight</a>

  <form>
    <label for="start-date">Start Date:</label>
    <input type="date" id="start-date" name="start-date"><br>

    <label for="end-date">End Date:</label>
    <input type="date" id="end-date" name="end-date"><br>

    <label for="days">Days:</label>
    <input type="number" id="days" name="days" min="1" max="7" value="7"><br>

    <button type="submit" id="submit">Submit</button>
  </form>

  <div>
    <canvas id="chart"></canvas>
  </div>

  <table id="result-table">
    <thead>
      <tr>
        <th>Date</th>
        <th>Weight</th>
      </tr>
    </thead>
    <tbody></tbody>
  </table>
```

## Output 9

Sure! Here's another version with a touch of orange:

```css

/* Font styling */
body {
  font-family: sans-serif;
}

/* Header styling */
header {
  background-color: orange;
  color: white;
  padding: 20px;
  text-align: center;
}

header a {
  color: white;
  text-decoration: none;
}

/* Form styling */
form {
  display: flex;
  flex-direction: column;
  margin: 20px 0;
}

label {
  font-weight: bold;
}

input[type=date],
input[type=number] {
  border: 1px solid gray;
  border-radius: 3px;
  padding: 8px;
  margin-bottom: 10px;
}

button[type=submit] {
  background-color: orange;
  color: white;
  border: none;
  border-radius: 3px;
  padding: 8px;
  cursor: pointer;
}

button[type=submit]:hover {
  background-color: #ff8300;
}

/* Chart styling */
#chart {
  width: 100%;
  max-width: 800px;
  height: 400px;
  margin: 20px 0;
}

/* Table styling */
#result-table {
  border-collapse: collapse;
  margin: 20px 0;
  width: 100%;
  max-width: 800px;
}

#result-table th,
#result-table td {
  border: 1px solid gray;
  padding: 10px;
}

#result-table th {
  background-color: orange;
  color: white;
}
```

And here's the HTML with the classes added:

```html

<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rolling Average</title>
  <link rel="shortcut icon" type="image/x-icon" href="./favicon.ico" />
  <link rel="stylesheet" type="text/css" href="./style.css">
</head>

<body>
  <header>
    <h1>Rolling Average</h1>
    <a href="add-weight.html">Add Weight</a>
  </header>

  <form class="form">
    <label for="start-date">Start Date:</label>
    <input type="date" id="start-date" name="start-date">

    <label for="end-date">End Date:</label>
    <input type="date" id="end-date" name="end-date">

    <label for="days">Days:</label>
    <input type="number" id="days" name="days" min="1" max="7" value="7">

    <button type="submit" id="submit">Submit</button>
  </form>

  <div class="chart-wrapper">
    <canvas id="chart"></canvas>
  </div>

  <table class="result-table">
    <thead>
      <tr>
        <th>Date</th>
        <th>Weight</th>
      </tr>
    </thead>
    <tbody></tbody>
  </table>

  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <script src="./main.js"></script>
</body>

</html>
```

## Input 10

thanks; we have these fields in the html

```html
<form>
    <label for="start-date">Start Date:</label>
    <input type="date" id="start-date" name="start-date"><br>

    <label for="end-date">End Date:</label>
    <input type="date" id="end-date" name="end-date"><br>

    <label for="days">Days:</label>
    <input type="number" id="days" name="days" min="1" max="7" value="7"><br>

    <button type="submit" id="submit">Submit</button>
  </form>
```

can you help me initialize the `end-date` input with the current date and the `start-date` with  current-date minus 3 weeks ?

## Output 10

Sure, you can use JavaScript to set the value property of each input element with the desired date.

Here's an example code snippet that sets the value of the end-date input with the current date, and the value of the start-date input with the date three weeks ago:

```html

<form>
  <label for="start-date">Start Date:</label>
  <input type="date" id="start-date" name="start-date"><br>

  <label for="end-date">End Date:</label>
  <input type="date" id="end-date" name="end-date"><br>

  <label for="days">Days:</label>
  <input type="number" id="days" name="days" min="1" max="7" value="7"><br>

  <button type="submit" id="submit">Submit</button>
</form>

<script>
  // Get current date and subtract 3 weeks (21 days)
  const currentDate = new Date();
  const threeWeeksAgo = new Date(currentDate.getTime() - (21 * 24 * 60 * 60 * 1000));

  // Format dates as "yyyy-MM-dd"
  const currentDateString = currentDate.toISOString().slice(0, 10);
  const threeWeeksAgoString = threeWeeksAgo.toISOString().slice(0, 10);

  // Set input values
  const startDateInput = document.getElementById("start-date");
  const endDateInput = document.getElementById("end-date");
  startDateInput.value = threeWeeksAgoString;
  endDateInput.value = currentDateString;
</script>
```

You can add this script at the bottom of your HTML document, just before the closing </body> tag.
