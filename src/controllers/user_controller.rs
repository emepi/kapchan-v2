use actix_identity::Identity;
use actix_web::{http::StatusCode, web::{self, Redirect}, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{services::authentication::{login_by_email, login_by_username}, views::login_view::{self, LoginTemplate}};


pub async fn login() -> actix_web::Result<HttpResponse> {
    login_view::render(LoginTemplate {
        errors: vec![],
    }).await
}

#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    username: String,
    pwd: String,
}

pub async fn handle_login(
    input: web::Form<LoginForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    let is_email = email_re.is_match(&input.username);
    
    let result = match is_email {
        true => login_by_email(&input.username, &input.pwd, &conn_pool, req).await,
        false => login_by_username(&input.username, &input.pwd, &conn_pool, req).await,
    };

    match result {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(err) => {
            let template = LoginTemplate {
                errors: match err {
                    StatusCode::FORBIDDEN => vec!["Virheellinen salasana!".to_string()],
                    StatusCode::NOT_FOUND => vec!["Käyttäjää ei ole olemassa!".to_string()],
                    _ => vec!["Palvelin virhe!".to_string()],
                },
            };

            login_view::render(template).await
        },
    }
}

pub async fn handle_logout(
    user: Identity
) -> impl Responder {
    user.logout();
    Redirect::to("/").using_status_code(StatusCode::FOUND)
}