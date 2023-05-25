// @generated automatically by Diesel CLI.

diesel::table! {
    price_changes (name, addr, updated) {
        name -> Text,
        addr -> Text,
        updated -> Timestamp,
        price -> Integer,
    }
}
