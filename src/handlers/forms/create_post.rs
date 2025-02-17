use actix_identity::Identity;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::{boards::Board, error::UserError, threads::Thread}, services::{authentication::resolve_user, captchas::verify_captcha, posts::create_post_by_thread_id}};


#[derive(Debug, MultipartForm)]
pub struct PostForm {
    pub message: Text<String>,
    pub captcha: Option<Text<String>>,
    pub captcha_id: Option<Text<u64>>,
    #[multipart(limit = "5MB")]
    pub attachment: TempFile,
}

pub async fn handle_post_creation(
    path: web::Path<(String, u32)>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    MultipartForm(input): MultipartForm<PostForm>,
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

    let (handle, thread_id) = path.into_inner();

    let current_board = match Board::by_handle(&conn_pool, &handle).await {
        Ok(board) => board,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if current_board.access_level > user_data.access_level {
        return HttpResponse::Forbidden().finish()
    }

    let thread_replies = match Thread::count_replies(thread_id, &conn_pool).await {
        Ok(count) => count,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if thread_replies > current_board.thread_size_limit.into() {
        return HttpResponse::Forbidden().json(UserError {
            error: "Lanka on täynnä!".to_owned(),
        });
    }

    let current_thread = match Thread::thread_by_id(thread_id, &conn_pool).await {
        Ok(thread) => thread,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if current_thread.archived {
        return HttpResponse::Forbidden().json(UserError {
            error: "Lanka on arkistoitu, eikä siihen voi enää vastata!".to_owned(),
        });
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

    match create_post_by_thread_id(
        &conn_pool,
        user_data.id, 
        thread_id, 
        input.message.to_string(), 
        input.attachment,
        current_board.access_level
    ).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}