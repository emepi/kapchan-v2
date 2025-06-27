use actix_identity::Identity;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::{boards::Board, error::UserError, threads::Thread}, services::{authentication::resolve_user, captchas::verify_captcha, threads::create_thread}, views::thread_view::{self, ThreadTemplate}};


#[derive(Debug, MultipartForm)]
pub struct ThreadForm {
    pub topic: Text<String>,
    pub message: Text<String>,
    pub captcha: Option<Text<String>>,
    pub captcha_id: Option<Text<u64>>,
    #[multipart(limit = "5MB")]
    pub attachment: TempFile,
}

pub async fn handle_thread_creation(
    path: web::Path<String>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    MultipartForm(input): MultipartForm<ThreadForm>,
    req: HttpRequest,
) -> impl Responder {
    if input.message.is_empty() {
        return HttpResponse::Forbidden().json(UserError {
            error: "Viesti ei voi olla tyhjä!".to_owned(),
        });
    }
    
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
                Ok(_) => (),
                Err(err) => {
                    return HttpResponse::Forbidden().json(UserError {
                        error: err,
                    });
                },
            }
        } else {
            return HttpResponse::Forbidden().json(UserError {
                error: "Captcha epäonnistui!".to_owned(),
            })
        }
    }

    let result = create_thread(
        &conn_pool, 
        user_data.id, 
        current_board.id, 
        input.topic.to_string(), 
        input.message.to_string(), 
        input.attachment, 
        current_board.access_level,
        current_board.active_threads_limit,
    ).await;

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn thread(
    path: web::Path<(String, u32)>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let boards = match Board::list_all(&conn_pool).await {
        Ok(board) => board,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let (handle, thread_id) = path.into_inner();

    let current_board = match Board::by_handle(&conn_pool, &handle).await {
        Ok(board) => board,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let thread = match Thread::by_id(thread_id, &conn_pool).await {
        Ok(thread) => thread,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    thread_view::render(ThreadTemplate {
        access_level: user_data.access_level,
        boards,
        current_board,
        thread,
    }).await
}