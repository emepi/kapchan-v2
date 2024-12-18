use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{models::threads::Thread, schema::{attachments, posts}};


#[derive(Debug, Queryable, Identifiable, Selectable, Associations, Serialize, PartialEq)]
#[diesel(belongs_to(Thread))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(primary_key(id))]
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

#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Clone)]
#[diesel(table_name = attachments)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Attachment {
    pub id: u32,
    pub file_name: String,
    pub height: u32,
    pub width: u32,
    pub file_hash: String,
    pub file_location: String,
    pub thumbnail_location: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = attachments)]
pub struct AttachmentModel<'a> {
    pub file_name: &'a str,
    pub height: u32,
    pub width: u32,
    pub file_hash: &'a str,
    pub file_location: &'a str,
    pub thumbnail_location: &'a str,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostOutput {
    pub id: u32,
    pub show_username: bool,
    pub message: String,
    pub country_code: Option<String>,
    pub hidden: bool,
}