// @generated automatically by Diesel CLI.

diesel::table! {
    weights (id) {
        id -> Integer,
        weight_value -> Double,
        measurement_date -> Date,
    }
}
