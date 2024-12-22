use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{models::threads::Thread, schema::{attachments, posts, replies}};

use super::files::FileInfo;


#[derive(Debug, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize, Clone, PartialEq)]
#[diesel(belongs_to(Thread))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(primary_key(id))]
pub struct Post {
    pub id: u32,
    pub user_id: u64,
    pub thread_id: u32,
    pub access_level: u8,
    pub show_username: bool,
    pub sage: bool,
    pub message: String,
    pub message_hash: String,
    pub country_code: Option<String>,
    pub mod_note: Option<String>,
    pub hidden: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = posts)]
pub struct PostModel<'a> {
    pub user_id: u64,
    pub thread_id: u32,
    pub access_level: u8,
    pub show_username: bool,
    pub sage: bool,
    pub message: &'a str,
    pub message_hash: &'a str,
    pub country_code: Option<&'a str>,
    pub mod_note: Option<&'a str>,
    pub hidden: bool,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = attachments)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Attachment {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub file_name: String,
    pub file_type: String,
    pub file_location: String,
    pub thumbnail_location: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = attachments)]
pub struct AttachmentModel<'a> {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub file_location: &'a str,
    pub thumbnail_location: &'a str,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize, Clone, PartialEq)]
#[diesel(belongs_to(Post))]
#[diesel(table_name = replies)]
#[diesel(primary_key(post_id, reply_id))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Reply {
    pub post_id: u32,
    pub reply_id: u32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = replies)]
pub struct ReplyModel {
    pub post_id: u32,
    pub reply_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostInput {
    pub access_level: u8,
    pub user_id: u64,
    pub show_username: bool,
    pub sage: bool,
    pub message: String,
    pub message_hash: String,
    pub country_code: Option<String>,
    pub mod_note: Option<String>,
    pub hidden: bool,
    pub reply_ids: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostOutput {
    pub id: u32,
    pub show_username: bool,
    pub message: String,
    pub country_code: Option<String>,
    pub hidden: bool,
    pub attachment: Option<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostData {
    pub post: Post,
    pub attachment: Option<Attachment>,
    pub replies: Vec<u32>,
}