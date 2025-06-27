use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;

use crate::models::{boards::Board, threads::ThreadCatalogOutput};


#[derive(TemplateOnce)]
#[template(path = "board.stpl")]
pub struct BoardTemplate {
    pub access_level: u8,
    pub handle: String,
    pub boards: Vec<Board>,
    pub current_board: Board,
    pub threads: Vec<ThreadCatalogOutput>,
}

pub async fn render(
    template: BoardTemplate,
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}