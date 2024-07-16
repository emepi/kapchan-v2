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
    attachments (id) {
        id -> Unsigned<Integer>,
        file_name -> Nullable<Tinytext>,
        #[max_length = 512]
        file_location -> Nullable<Varchar>,
        #[max_length = 512]
        thumbnail_location -> Nullable<Varchar>,
    }
}

diesel::table! {
    board_groups (id) {
        id -> Unsigned<Integer>,
        group_name -> Tinytext,
    }
}

diesel::table! {
    boards (id) {
        id -> Unsigned<Integer>,
        group_id -> Nullable<Unsigned<Integer>>,
        #[max_length = 8]
        handle -> Varchar,
        title -> Tinytext,
        access_level -> Unsigned<Tinyint>,
        nsfw -> Bool,
    }
}

diesel::table! {
    posts (id) {
        id -> Unsigned<Integer>,
        session_id -> Unsigned<Integer>,
        thread_id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 10]
        tripcode -> Varchar,
        message -> Nullable<Text>,
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
    sessions (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        #[max_length = 45]
        ip_address -> Varchar,
        #[max_length = 512]
        user_agent -> Varchar,
        created_at -> Datetime,
        ended_at -> Datetime,
    }
}

diesel::table! {
    threads (id) {
        id -> Unsigned<Integer>,
        board_id -> Nullable<Unsigned<Integer>>,
        title -> Tinytext,
        pinned -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 16]
        username -> Nullable<Varchar>,
        #[max_length = 128]
        password_hash -> Nullable<Varchar>,
        created_at -> Datetime,
    }
}

diesel::joinable!(application_reviews -> applications (application_id));
diesel::joinable!(application_reviews -> users (reviewer_id));
diesel::joinable!(applications -> users (user_id));
diesel::joinable!(attachments -> posts (id));
diesel::joinable!(boards -> board_groups (group_id));
diesel::joinable!(posts -> sessions (session_id));
diesel::joinable!(posts -> threads (thread_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(threads -> boards (board_id));

diesel::allow_tables_to_appear_in_same_query!(
    application_reviews,
    applications,
    attachments,
    board_groups,
    boards,
    posts,
    replies,
    sessions,
    threads,
    users,
);
