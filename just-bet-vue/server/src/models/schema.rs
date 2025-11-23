// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int8,
        #[max_length = 30]
        username -> Varchar,
        email -> Text,
        password_hash -> Text,
        balance -> Money,
        claim_expires_timestamp -> Timestamptz,
    }
}
