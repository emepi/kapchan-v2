use actix_web::{error::InternalError, http::StatusCode, HttpRequest, HttpResponse};
use sailfish::{TemplateOnce};


#[derive(TemplateOnce)]
#[template(path = "pages/index.stpl")]
struct IndexTemplate {
    boards: Vec<String>,
}

pub async fn index_view(
    req: HttpRequest
) -> actix_web::Result<HttpResponse> {
    let body = IndexTemplate { 
        boards: vec!["test".to_string(), "test-2".to_string(), "test-3".to_string()] 
    }
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}