use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use captcha::gen;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Serialize;

use crate::{models::users::AccessLevel, services::{authentication::resolve_user, captchas::new_captcha}};


#[derive(Debug, Serialize)]
pub struct CaptchaOutput {
    pub id: u64,
    pub captcha: String,
}

pub async fn captcha(
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }

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