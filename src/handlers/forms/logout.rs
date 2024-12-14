use actix_identity::Identity;
use actix_web::{http::StatusCode, web::Redirect, Responder};


pub async fn handle_logout(
    user: Identity
) -> impl Responder {
    user.logout();
    Redirect::to("/").using_status_code(StatusCode::FOUND)
}