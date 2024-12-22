use chrono::{Duration, Utc};
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::models::captchas::{Captcha, CaptchaModel};


pub async fn new_captcha(
    conn_pool: &Pool<AsyncMysqlConnection>,
    answer: String,
) -> Result<Captcha, Error> {
    CaptchaModel {
        answer: &answer,
        expires: &(Utc::now() + Duration::minutes(5)).naive_utc(),
    }
    .insert(conn_pool)
    .await
}

pub async fn verify_captcha(
    conn_pool: &Pool<AsyncMysqlConnection>,
    captcha_id: u64,
    answer: String,
) -> Result<(), String> {
    let captcha_info = match Captcha::by_id(captcha_id, conn_pool).await {
        Ok(info) => info,
        Err(err) => match err {
            Error::NotFound => return Err("Captcha not found. recomplete.".to_string()),
            _ => return Err("Server error".to_owned()),
        },
    };

    if Utc::now().timestamp() > captcha_info.expires.and_utc().timestamp() {
        return Err("Captcha expired!".to_owned());
    }

    let _ = Captcha::delete_by_id(captcha_id, &conn_pool).await;

    if captcha_info.answer.eq(&answer) {
        Ok(())
    } else {
        Err("Captcha failed!".to_owned())
    }
}