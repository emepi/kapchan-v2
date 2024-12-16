use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, Error, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::{TemplateOnce};

use crate::{models::{boards::Board, users::AccessLevel}, services::authentication::resolve_user};


#[derive(TemplateOnce)]
#[template(path = "pages/admin.stpl")]
pub struct AdminTemplate {
    pub access_level: u8,
    pub errors: Vec<String>,
    pub boards: Vec<Board>,
}

pub fn template(
    t: AdminTemplate
) -> Result<String, Error> {
    Ok(t
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?)
}

pub async fn admin_view(
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let boards = match Board::list_all(&conn_pool).await {
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let body = template(AdminTemplate {
        access_level: user_data.access_level,
        errors: vec![],
        boards,
    })?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}