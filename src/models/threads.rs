use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

use crate::schema::threads;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
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