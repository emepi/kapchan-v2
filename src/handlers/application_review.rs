use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::{TemplateOnce};

use crate::{models::{applications::{Application, ApplicationView}, users::AccessLevel}, services::authentication::resolve_user};


#[derive(TemplateOnce)]
#[template(path = "pages/application_review.stpl")]
struct ApplicationReviewTemplate {
    pub access_level: u8,
    pub application: ApplicationView,
}

pub async fn application_review_view(
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

    let application_id = path.into_inner();

    let application = match Application::by_id(&conn_pool, application_id).await {
        Ok(app) => app,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let body = ApplicationReviewTemplate {
        access_level: user_data.access_level,
        application,
    }
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}