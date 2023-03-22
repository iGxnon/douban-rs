// @generated automatically by Diesel CLI.

diesel::table! {
    t_celebrities (id) {
        id -> Int8,
        name -> Varchar,
        name_en -> Nullable<Varchar>,
        pic_url -> Nullable<Varchar>,
        gender -> Varchar,
        imdb -> Varchar,
        info -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_movies (id) {
        id -> Int8,
        title -> Varchar,
        pic_url -> Nullable<Varchar>,
        name -> Varchar,
        alias_name -> Nullable<Varchar>,
        language -> Varchar,
        time_length -> Int4,
        released_date -> Date,
        imdb -> Varchar,
        plot -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_movies_actors (id) {
        id -> Int8,
        mid -> Int8,
        cid -> Int8,
    }
}

diesel::table! {
    t_movies_categories (id) {
        id -> Int8,
        mid -> Int8,
        category -> Varchar,
    }
}

diesel::table! {
    t_movies_country (id) {
        id -> Int8,
        mid -> Int8,
        country -> Varchar,
    }
}

diesel::table! {
    t_movies_directors (id) {
        id -> Int8,
        mid -> Int8,
        cid -> Int8,
    }
}

diesel::table! {
    t_movies_scores (id) {
        id -> Int8,
        mid -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        score_avg -> Float8,
        cnt_1 -> Int8,
        cnt_2 -> Int8,
        cnt_3 -> Int8,
        cnt_4 -> Int8,
        cnt_5 -> Int8,
    }
}

diesel::table! {
    t_movies_writers (id) {
        id -> Int8,
        mid -> Int8,
        cid -> Int8,
    }
}

diesel::table! {
    t_oauth (id) {
        id -> Int8,
        github -> Nullable<Int8>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        role_group -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        last_login -> Nullable<Timestamp>,
    }
}

diesel::joinable!(t_movies_actors -> t_celebrities (cid));
diesel::joinable!(t_movies_actors -> t_movies (mid));
diesel::joinable!(t_movies_categories -> t_movies (mid));
diesel::joinable!(t_movies_country -> t_movies (mid));
diesel::joinable!(t_movies_directors -> t_celebrities (cid));
diesel::joinable!(t_movies_directors -> t_movies (mid));
diesel::joinable!(t_movies_scores -> t_movies (mid));
diesel::joinable!(t_movies_writers -> t_celebrities (cid));
diesel::joinable!(t_movies_writers -> t_movies (mid));
diesel::joinable!(t_users -> t_oauth (oauth_id));

diesel::allow_tables_to_appear_in_same_query!(
    t_celebrities,
    t_movies,
    t_movies_actors,
    t_movies_categories,
    t_movies_country,
    t_movies_directors,
    t_movies_scores,
    t_movies_writers,
    t_oauth,
    t_users,
);
