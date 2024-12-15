use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::{TemplateOnce};

use crate::{models::{applications::ApplicationPreview, users::AccessLevel}, services::{applications::load_application_previews, authentication::resolve_user}};


#[derive(TemplateOnce)]
#[template(path = "pages/applications.stpl")]
struct ApplicationsTemplate {
    pub access_level: u8,
    pub previews: Vec<ApplicationPreview>,
}

pub async fn applications_view(
    path: web::Path<u32>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Admin as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let page = path.into_inner();

    let previews = match load_application_previews(&conn_pool, page.into(), 50).await {
        Ok(previews) => previews,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let body = ApplicationsTemplate {
        access_level: user_data.access_level,
        previews,
    }
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}