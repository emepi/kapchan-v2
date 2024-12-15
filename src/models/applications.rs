use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

use crate::schema::{application_reviews, applications};


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = applications)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Application {
    pub id: u32,
    pub user_id: u32,
    pub accepted: bool,
    pub background: String,
    pub motivation: String,
    pub other: String,
    pub created_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = applications)]
pub struct ApplicationModel<'a> {
    pub user_id: u32,
    pub accepted: bool,
    pub background: &'a str,
    pub motivation: &'a str,
    pub other: &'a str,
    pub closed_at: Option<NaiveDateTime>
}

#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = application_reviews)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ApplicationReview {
    pub id: u32,
    pub reviewer_id: u32,
    pub application_id: u32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = application_reviews)]
pub struct ApplicationReviewModel {
    pub reviewer_id: u32,
    pub application_id: u32,
}