// @generated automatically by Diesel CLI.

diesel::table! {
    bombs (id) {
        id -> Int8,
        belongs_to -> Int8,
        position -> Int8,
    }
}

diesel::table! {
    clicks (id) {
        id -> Int8,
        belongs_to -> Int8,
        position -> Int8,
        earned -> Money,
    }
}

diesel::table! {
    minesweeper (id) {
        id -> Int8,
        belongs_to -> Int8,
        #[max_length = 30]
        state -> Varchar,
        #[max_length = 30]
        result -> Varchar,
        stake -> Money,
        pool -> Money,
        created -> Timestamptz,
    }
}

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

diesel::joinable!(bombs -> minesweeper (belongs_to));
diesel::joinable!(clicks -> minesweeper (belongs_to));
diesel::joinable!(minesweeper -> users (belongs_to));

diesel::allow_tables_to_appear_in_same_query!(bombs, clicks, minesweeper, users,);
