use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;

use crate::services::time::fi_datetime;
use crate::models::{bans::Ban, posts::Post};


#[derive(TemplateOnce)]
#[template(path = "banned.stpl")]
pub struct BannedTemplate {
    pub ban: Ban,
    pub post: Option<Post>,
}

pub async fn render(
    template: BannedTemplate,
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}