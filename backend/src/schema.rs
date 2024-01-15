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
    board_groups (id) {
        id -> Unsigned<Integer>,
        name -> Tinytext,
    }
}

diesel::table! {
    boards (id) {
        id -> Unsigned<Integer>,
        board_group_id -> Unsigned<Integer>,
        #[max_length = 8]
        handle -> Varchar,
        title -> Tinytext,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    files (id) {
        id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 32]
        hash -> Varchar,
        #[sql_name = "type"]
        type_ -> Unsigned<Smallint>,
        size -> Unsigned<Integer>,
        #[max_length = 512]
        location -> Varchar,
        created_at -> Datetime,
        uploaded_by -> Unsigned<Integer>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Unsigned<Integer>,
        user_id -> Nullable<Unsigned<Integer>>,
        access_level -> Unsigned<Tinyint>,
        mode -> Unsigned<Tinyint>,
        created_at -> Datetime,
        ended_at -> Nullable<Datetime>,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Integer>,
        access_level -> Unsigned<Tinyint>,
        #[max_length = 16]
        username -> Varchar,
        #[max_length = 128]
        email -> Nullable<Varchar>,
        #[max_length = 128]
        password_hash -> Varchar,
        created_at -> Datetime,
    }
}

diesel::joinable!(application_reviews -> applications (application_id));
diesel::joinable!(application_reviews -> users (reviewer_id));
diesel::joinable!(applications -> users (user_id));
diesel::joinable!(boards -> board_groups (board_group_id));
diesel::joinable!(files -> users (uploaded_by));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    application_reviews,
    applications,
    board_groups,
    boards,
    files,
    sessions,
    users,
);
