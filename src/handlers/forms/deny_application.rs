use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::users::{AccessLevel, User}, services::{applications::{is_reviewed, review_application}, authentication::resolve_user}};


pub async fn handle_application_deny(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Admin as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let application_id = path.into_inner();

    let reviewed = match is_reviewed(&conn_pool, application_id).await {
        Ok(status) => status,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if reviewed {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let application = match review_application(&conn_pool, application_id, user_data.id, false).await {
        Ok(app) => app,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    match User::update_access_level(application.user_id, AccessLevel::Registered as u8, &conn_pool).await {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/applications/1")).finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}