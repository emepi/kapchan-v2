// @generated automatically by Diesel CLI.

diesel::table! {
    applications (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        reviewer_id -> Nullable<Unsigned<Integer>>,
        referer_id -> Nullable<Unsigned<Integer>>,
        accepted -> Bool,
        background -> Text,
        motivation -> Text,
        other -> Nullable<Text>,
        created_at -> Datetime,
        closed_at -> Nullable<Datetime>,
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

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    applications,
    sessions,
    users,
);
