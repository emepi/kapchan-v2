use actix_web::{http::StatusCode, web::{self}, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{handlers::login::{template, LoginTemplate}, services::authentication::{login_by_email, login_by_username}};


#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    username: String,
    pwd: String,
}

pub async fn handle_login(
    form: web::Form<LoginForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    request: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    let is_email = email_re.is_match(&form.username);
    
    let result = match is_email {
        true => login_by_email(&form.username, &form.pwd, &conn_pool, request).await,
        false => login_by_username(&form.username, &form.pwd, &conn_pool, request).await,
    };

    match result {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(err) => {
            let t = LoginTemplate {
                errors: match err {
                    StatusCode::FORBIDDEN => vec!["Virheellinen salasana!".to_string()],
                    StatusCode::NOT_FOUND => vec!["Käyttäjää ei ole olemassa!".to_string()],
                    _ => vec!["Palvelin virhe!".to_string()],
                },
            };

            let body = template(t).unwrap();

            Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body))
        },
    }
}