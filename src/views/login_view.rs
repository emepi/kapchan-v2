use actix_web::{error::InternalError, http::{Error, StatusCode}, HttpResponse};
use sailfish::TemplateOnce;


#[derive(TemplateOnce)]
#[template(path = "login.stpl")]
pub struct LoginTemplate {
    pub errors: Vec<String>,
}

pub async fn render(
    template: LoginTemplate
) -> actix_web::Result<HttpResponse> {
    let body = template
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}