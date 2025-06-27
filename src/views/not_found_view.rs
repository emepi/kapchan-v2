use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;


#[derive(TemplateOnce)]
#[template(path = "404.stpl")]
pub struct NotFoundTemplate {
}

pub async fn render() -> actix_web::Result<HttpResponse> {
    let body = NotFoundTemplate{}
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}