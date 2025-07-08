use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;

use crate::models::boards::Board;


#[derive(TemplateOnce)]
#[template(path = "chat.stpl")]
pub struct ChatTemplate {
    pub access_level: u8,
    pub boards: Vec<Board>,
}

pub async fn render(
    template: ChatTemplate
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}