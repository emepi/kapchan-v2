use actix_web::{web, HttpRequest, HttpResponse, Responder};
use captcha::gen;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Serialize;

use crate::services::captchas::new_captcha;


#[derive(Debug, Serialize)]
pub struct CaptchaOutput {
    pub id: u64,
    pub captcha: String,
}

pub async fn captcha(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let captcha = gen(captcha::Difficulty::Medium);
    let ans = captcha.chars_as_string();

    let captcha_info = match new_captcha(&conn_pool, ans).await {
        Ok(captcha) => captcha,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(CaptchaOutput {
        id: captcha_info.id,
        captcha: captcha.as_base64().unwrap_or_default(),
    })
}