// @generated automatically by Diesel CLI.

diesel::table! {
    application_reviews (id) {
        id -> Unsigned<Integer>,
        reviewer_id -> Unsigned<Integer>,
        application_id -> Unsigned<Integer>,
    }
}

diesel::table! {
    applications (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        accepted -> Bool,
        background -> Text,
        motivation -> Text,
        other -> Nullable<Text>,
        created_at -> Datetime,
        closed_at -> Nullable<Datetime>,
    }
}

diesel::table! {
    boards (id) {
        id -> Unsigned<Integer>,
        #[max_length = 8]
        handle -> Varchar,
        title -> Tinytext,
        access_level -> Unsigned<Tinyint>,
        bump_limit -> Unsigned<Integer>,
        nsfw -> Bool,
    }
}

diesel::table! {
    files (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 45]
        ip_address -> Varchar,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 32]
        hash -> Varchar,
        #[sql_name = "type"]
        type_ -> Unsigned<Tinyint>,
        size -> Unsigned<Integer>,
        #[max_length = 512]
        location -> Varchar,
        #[max_length = 512]
        thumbnail_location -> Varchar,
        created_at -> Datetime,
    }
}

diesel::table! {
    posts (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        thread -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 45]
        ip_address -> Varchar,
        attachment -> Nullable<Unsigned<Integer>>,
        body -> Nullable<Text>,
        sage -> Bool,
        note -> Nullable<Tinytext>,
        created_at -> Datetime,
    }
}

diesel::table! {
    sessions (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        created_at -> Datetime,
        ended_at -> Datetime,
    }
}

diesel::table! {
    thread_ids (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        thread_id -> Unsigned<Integer>,
    }
}

diesel::table! {
    threads (id) {
        id -> Unsigned<Integer>,
        board -> Unsigned<Integer>,
        title -> Nullable<Text>,
        views -> Unsigned<Integer>,
        sticky -> Bool,
        archived -> Bool,
        bump_time -> Datetime,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 16]
        username -> Nullable<Varchar>,
        #[max_length = 128]
        email -> Nullable<Varchar>,
        #[max_length = 128]
        password_hash -> Nullable<Varchar>,
        created_at -> Datetime,
    }
}

diesel::joinable!(application_reviews -> applications (application_id));
diesel::joinable!(application_reviews -> users (reviewer_id));
diesel::joinable!(applications -> users (user_id));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(posts -> files (attachment));
diesel::joinable!(posts -> threads (thread));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(thread_ids -> threads (thread_id));
diesel::joinable!(thread_ids -> users (user_id));
diesel::joinable!(threads -> boards (board));

diesel::allow_tables_to_appear_in_same_query!(
    application_reviews,
    applications,
    boards,
    files,
    posts,
    sessions,
    thread_ids,
    threads,
    users,
);
