use actix_web::{error::InternalError, http::StatusCode, HttpResponse};
use sailfish::TemplateOnce;


#[derive(TemplateOnce)]
#[template(path = "register.stpl")]
pub struct RegisterTemplate {
    pub errors: Vec<String>,
}

pub async fn render(
    template: RegisterTemplate,
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}