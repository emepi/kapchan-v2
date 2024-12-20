// @generated automatically by Diesel CLI.

diesel::table! {
    application_reviews (id) {
        id -> Unsigned<Integer>,
        reviewer_id -> Unsigned<Bigint>,
        application_id -> Unsigned<Integer>,
    }
}

diesel::table! {
    applications (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Bigint>,
        accepted -> Bool,
        background -> Text,
        motivation -> Text,
        other -> Text,
        created_at -> Datetime,
        closed_at -> Nullable<Datetime>,
    }
}

diesel::table! {
    attachments (id) {
        id -> Unsigned<Integer>,
        file_name -> Tinytext,
        file_type -> Tinytext,
        #[max_length = 512]
        file_location -> Varchar,
        #[max_length = 512]
        thumbnail_location -> Varchar,
    }
}

diesel::table! {
    boards (id) {
        id -> Unsigned<Integer>,
        #[max_length = 8]
        handle -> Varchar,
        title -> Tinytext,
        description -> Text,
        access_level -> Unsigned<Tinyint>,
        post_cooldown_time_sec -> Unsigned<Integer>,
        active_threads_limit -> Unsigned<Integer>,
        thread_size_limit -> Unsigned<Integer>,
        unique_posts -> Bool,
        captcha -> Bool,
        nsfw -> Bool,
    }
}

diesel::table! {
    captchas (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 6]
        answer -> Varchar,
        expires -> Datetime,
    }
}

diesel::table! {
    posts (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Bigint>,
        thread_id -> Unsigned<Integer>,
        show_username -> Bool,
        message -> Text,
        #[max_length = 64]
        message_hash -> Varchar,
        #[max_length = 45]
        ip_address -> Varchar,
        #[max_length = 512]
        user_agent -> Varchar,
        #[max_length = 2]
        country_code -> Nullable<Varchar>,
        hidden -> Bool,
        created_at -> Datetime,
    }
}

diesel::table! {
    replies (post_id, reply_id) {
        post_id -> Unsigned<Integer>,
        reply_id -> Unsigned<Integer>,
    }
}

diesel::table! {
    threads (id) {
        id -> Unsigned<Integer>,
        board_id -> Unsigned<Integer>,
        title -> Tinytext,
        pinned -> Bool,
        archived -> Bool,
        bump_time -> Datetime,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
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
diesel::joinable!(attachments -> posts (id));
diesel::joinable!(posts -> threads (thread_id));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(threads -> boards (board_id));

diesel::allow_tables_to_appear_in_same_query!(
    application_reviews,
    applications,
    attachments,
    boards,
    captchas,
    posts,
    replies,
    threads,
    users,
);
