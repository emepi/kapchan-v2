use actix_web::{web, HttpRequest, HttpResponse, Responder};
use captcha::gen;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Serialize;


#[derive(Debug, Serialize)]
struct CaptchaOutput {
    pub captcha: String,
}

pub async fn generate_captcha(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let captcha = gen(captcha::Difficulty::Medium);
    let ans = captcha.chars_as_string();

    println!("captcha: {}", ans);

    HttpResponse::Ok().json(CaptchaOutput {
        captcha: captcha.as_base64().unwrap_or_default(),
    })
}