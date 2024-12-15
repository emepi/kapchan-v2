use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use ammonia::is_html;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::{Deserialize, Serialize};

use crate::{handlers::apply::{template, ApplyTemplate}, models::users::AccessLevel, services::{applications::submit_application, authentication::resolve_user}};

#[derive(Serialize, Deserialize)]
pub struct ApplicationForm {
    background: String,
    motivation: String,
    other: String,
}

pub async fn handle_application(
    user: Option<Identity>,
    form: web::Form<ApplicationForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level != AccessLevel::Registered as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    if is_html(&form.background) || is_html(&form.motivation) || is_html(&form.other) {
        let t = ApplyTemplate {
            errors: vec!["HTML tags are not allowed.".to_string()]
        };

        let body = template(t).unwrap();

        return Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
    }

    let res = submit_application(
        &conn_pool, 
        user_data.id, 
        &form.background, 
        &form.motivation, 
        &form.other
    )
    .await;

    match res {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(_) => {
            let t = ApplyTemplate {
                errors: vec!["Server error.".to_string()]
            };
    
            let body = template(t).unwrap();
    
            return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body))
        },
    }
}