use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;

use crate::models::{bans::Ban, boards::Board, users::User};

use crate::services::time::fi_datetime;


#[derive(TemplateOnce)]
#[template(path = "user.stpl")]
pub struct UserTemplate {
    pub access_level: u8,
    pub boards: Vec<Board>,
    pub user: User,
    pub bans: Vec<(Ban, User)>,
}

pub async fn render(
    template: UserTemplate,
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}