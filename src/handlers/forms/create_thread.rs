use actix_identity::Identity;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::boards::Board, services::{authentication::resolve_user, captchas::verify_captcha}};


#[derive(Debug, MultipartForm)]
pub struct ThreadForm {
    pub topic: Text<String>,
    pub message: Text<String>,
    pub captcha: Option<Text<String>>,
    pub captcha_id: Option<Text<u64>>,
    #[multipart(limit = "20MB")]
    pub attachment: TempFile,
}

pub async fn handle_thread_creation(
    path: web::Path<String>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    MultipartForm(input): MultipartForm<ThreadForm>,
    req: HttpRequest,
) -> impl Responder {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let handle = path.into_inner();

    let current_board = match Board::by_handle(&conn_pool, &handle).await {
        Ok(board) => board,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if current_board.access_level > user_data.access_level {
        return HttpResponse::Forbidden().finish()
    }

    if current_board.captcha {
        if input.captcha.is_some() && input.captcha_id.is_some() {
            match verify_captcha(
                &conn_pool, 
                *input.captcha_id.unwrap(), 
                input.captcha.unwrap().to_string()
            ).await {
                Ok(_) => {
                    println!("success");
                },
                Err(err) => {
                    println!("{}", err);
                    return HttpResponse::InternalServerError().finish()
                },
            }
        } else {
            return HttpResponse::Forbidden().finish()
        }
    }

    return HttpResponse::InternalServerError().finish()
}