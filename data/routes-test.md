# Routes Test

These are some links for some quick testing using the VSCode plugin: Rest Client.

Use:

1. Launch your server
2. Highlight a command below
3. launch command (CTRL + SHIFT + P)
4. Rest client Send Request

```http
GET http://127.0.0.1:14280/api/rolling_average?start_date=2023-02-06&end_date=2023-02-18&days=3
```

```http
POST http://127.0.0.1:14280/api/add_weight HTTP/1.1
content-type: application/json

{
    "weight_value":19,
    "measurement_date":"2023-02-26"
}
```