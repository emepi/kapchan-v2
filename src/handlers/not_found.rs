use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, Error, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::TemplateOnce;

use crate::services::authentication::resolve_user;


#[derive(TemplateOnce)]
#[template(path = "404.stpl")]
pub struct NotFoundTemplate {
    pub access_level: u8,
}

pub fn template(
    t: NotFoundTemplate
) -> Result<String, Error>{
    Ok(t
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?)
}

pub async fn not_found_view(
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let body = template(NotFoundTemplate {
        access_level: user_data.access_level,
    })?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}