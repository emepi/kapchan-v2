use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::threads;

use super::posts::{Attachment, Post, PostData, PostInput, PostOutput};


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = threads)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Thread {
    pub id: u32,
    pub board_id: u32,
    pub title: String,
    pub pinned: bool,
    pub archived: bool,
    pub bump_time: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = threads)]
pub struct ThreadModel<'a> {
    pub board_id: u32,
    pub title: &'a str,
    pub pinned: bool,
    pub archived: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadInput {
    pub board_id: u32,
    pub title: String,
    pub pinned: bool,
    pub archived: bool,
    pub post: PostInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadCatalogOutput {
    pub title: String,
    pub pinned: bool,
    pub op_post: PostOutput,
    pub replies: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadDbOutput {
    pub thread: Thread,
    pub post: Post,
    pub attachment: Option<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadData {
    pub thread: Thread,
    pub posts: Vec<PostData>,
}