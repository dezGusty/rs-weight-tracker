// @generated automatically by Diesel CLI.

diesel::table! {
    weights (id) {
        id -> Integer,
        weight -> Double,
        measurement_date -> Timestamp,
    }
}
