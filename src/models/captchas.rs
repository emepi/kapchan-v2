use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

use crate::schema::captchas;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = captchas)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Captcha {
    pub id: u64,
    pub answer: String,
    pub expires: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = captchas)]
pub struct CaptchaModel<'a> {
    pub answer: &'a str,
    pub expires: &'a NaiveDateTime,
}