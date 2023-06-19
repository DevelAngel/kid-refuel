// @generated automatically by Diesel CLI.

diesel::table! {
    price_changes (id) {
        id -> Nullable<Integer>,
        station_id -> Integer,
        updated -> Timestamp,
        price -> Integer,
    }
}

diesel::table! {
    stations (id) {
        id -> Nullable<Integer>,
        name -> Text,
        addr -> Text,
    }
}

diesel::joinable!(price_changes -> stations (station_id));

diesel::allow_tables_to_appear_in_same_query!(
    price_changes,
    stations,
);
