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
    board_flags (id) {
        id -> Unsigned<Integer>,
        board_id -> Unsigned<Integer>,
        flag -> Unsigned<Tinyint>,
    }
}

diesel::table! {
    boards (id) {
        id -> Unsigned<Integer>,
        #[max_length = 8]
        handle -> Varchar,
        title -> Tinytext,
        description -> Nullable<Text>,
        created_at -> Datetime,
        created_by -> Unsigned<Integer>,
    }
}

diesel::table! {
    invites (id) {
        id -> Unsigned<Integer>,
        inviter_id -> Unsigned<Integer>,
        application_id -> Unsigned<Integer>,
        code -> Nullable<Text>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        mode -> Unsigned<Tinyint>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        #[max_length = 512]
        user_agent -> Nullable<Varchar>,
        created_at -> Datetime,
        ended_at -> Nullable<Datetime>,
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
diesel::joinable!(board_flags -> boards (board_id));
diesel::joinable!(boards -> users (created_by));
diesel::joinable!(invites -> applications (application_id));
diesel::joinable!(invites -> users (inviter_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    application_reviews,
    applications,
    board_flags,
    boards,
    invites,
    sessions,
    users,
);
