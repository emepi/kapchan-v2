use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;

use crate::models::{boards::Board, threads::ThreadData};
use crate::services::time::fi_datetime;
use crate::services::files::display_filesize;


#[derive(TemplateOnce)]
#[template(path = "thread.stpl")]
pub struct ThreadTemplate {
    pub access_level: u8,
    pub user_id: u64,
    pub boards: Vec<Board>,
    pub current_board: Board,
    pub thread: ThreadData,
}

pub async fn render(
    template: ThreadTemplate,
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}