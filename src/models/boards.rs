use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::boards;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = boards)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Board {
    pub id: u32,
    pub handle: String,
    pub title: String,
    pub description: String,
    pub access_level: u8,
    pub active_threads_limit: u32,
    pub thread_size_limit: u32,
    pub captcha: bool,
    pub nsfw: bool,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = boards)]
pub struct BoardModel<'a> {
    pub handle: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub access_level: u8,
    pub active_threads_limit: u32,
    pub thread_size_limit: u32,
    pub captcha: bool,
    pub nsfw: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardSimple {
    pub handle: String,
    pub title: String,
    pub access_level: u8,
    pub nsfw: bool,
}