// @generated automatically by Diesel CLI.

diesel::table! {
    archives (id) {
        id -> Integer,
        file -> Integer,
        data -> Longblob,
    }
}

diesel::table! {
    files (id) {
        id -> Integer,
        #[sql_name = "type"]
        type_ -> Integer,
        file_size -> Bigint,
        owner -> Integer,
        #[max_length = 512]
        location -> Nullable<Varchar>,
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
diesel::joinable!(files -> sessions (owner));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    archives,
    files,
    sessions,
    users,
);
