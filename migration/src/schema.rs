// @generated automatically by Diesel CLI.

diesel::table! {
    t_oauth (id) {
        id -> Int8,
        github -> Nullable<Int8>,
    }
}

diesel::table! {
    t_users (id) {
        id -> Int8,
        oauth_id -> Nullable<Int8>,
        username -> Varchar,
        phone -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        nickname -> Varchar,
        hashed_password -> Varchar,
        create_at -> Timestamp,
        update_at -> Timestamp,
        delete_at -> Nullable<Timestamp>,
        last_login -> Nullable<Timestamp>,
    }
}

diesel::joinable!(t_users -> t_oauth (oauth_id));

diesel::allow_tables_to_appear_in_same_query!(t_oauth, t_users,);
