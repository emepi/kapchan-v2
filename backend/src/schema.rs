// @generated automatically by Diesel CLI.

diesel::table! {
    archives (id) {
        id -> Integer,
        file -> Integer,
        data -> Longblob,
    }
}

diesel::table! {
    board_attachments (id) {
        id -> Integer,
        board -> Integer,
        allowed_type -> Integer,
        size_limit -> Bigint,
    }
}

diesel::table! {
    boards (id) {
        id -> Integer,
        #[max_length = 8]
        handle -> Varchar,
        #[max_length = 64]
        title -> Varchar,
        #[max_length = 4096]
        full_description -> Nullable<Varchar>,
        read_access -> Tinyint,
        post_access -> Tinyint,
        attachments_limit -> Tinyint,
        post_limit -> Unsigned<Smallint>,
        geo_locations -> Bool,
        visible -> Bool,
        nsfw -> Bool,
        created_at -> Datetime,
    }
}

diesel::table! {
    files (id) {
        id -> Integer,
        #[sql_name = "type"]
        type_ -> Integer,
        file_size -> Bigint,
        owner -> Integer,
        #[max_length = 32]
        md5_hash -> Char,
        #[max_length = 512]
        location -> Nullable<Varchar>,
        #[max_length = 32]
        file_name -> Nullable<Varchar>,
        created_at -> Datetime,
        modified_at -> Nullable<Datetime>,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        board -> Integer,
        thread -> Nullable<Integer>,
        owner -> Integer,
        title -> Nullable<Tinytext>,
        content -> Nullable<Text>,
        read_access -> Tinyint,
        post_access -> Tinyint,
        created_at -> Datetime,
        modified_at -> Nullable<Datetime>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Integer,
        user_id -> Integer,
        mode -> Tinyint,
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
        id -> Integer,
        access_level -> Tinyint,
        #[max_length = 16]
        username -> Nullable<Varchar>,
        #[max_length = 128]
        email -> Nullable<Varchar>,
        #[max_length = 128]
        password_hash -> Nullable<Varchar>,
        created_at -> Datetime,
    }
}

diesel::joinable!(archives -> files (file));
diesel::joinable!(board_attachments -> boards (board));
diesel::joinable!(files -> sessions (owner));
diesel::joinable!(posts -> boards (board));
diesel::joinable!(posts -> sessions (owner));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    archives,
    board_attachments,
    boards,
    files,
    posts,
    sessions,
    users,
);
