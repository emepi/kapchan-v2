use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::posts;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Post {
    pub id: u32,
    pub user_id: u64,
    pub thread_id: u32,
    pub show_username: bool,
    pub message: String,
    pub message_hash: String,
    pub ip_address: String,
    pub user_agent: String,
    pub country_code: Option<String>,
    pub hidden: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = posts)]
pub struct PostModel<'a> {
    pub user_id: u64,
    pub thread_id: u32,
    pub show_username: bool,
    pub message: &'a str,
    pub message_hash: &'a str,
    pub ip_address: &'a str,
    pub user_agent: &'a str,
    pub country_code: Option<&'a str>,
    pub hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostInput {
    pub user_id: u64,
    pub show_username: bool,
    pub message: String,
    pub message_hash: String,
    pub ip_address: String,
    pub user_agent: String,
    pub country_code: Option<String>,
    pub hidden: bool,
}