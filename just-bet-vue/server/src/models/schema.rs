// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> BigInt,
        #[max_length = 30]
        username -> Varchar,
        email -> Text,
        password_hash -> Text,
        balance -> BigInt,
        claim_expires_timestamp -> Integer,
    }
}
