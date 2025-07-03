use std::fs::remove_file;

use actix_identity::Identity;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::{Deserialize, Serialize};

use crate::{models::{boards::Board, error::UserError, posts::Post, threads::Thread, users::AccessLevel}, services::{authentication::resolve_user, captchas::verify_captcha, posts::create_post_by_thread_id}};


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

    if input.message.len() > 40_000 {
        return HttpResponse::Forbidden().json(UserError {
            error: "Viesti on liian pitkä (yli 40 000 merkkiä)".to_owned(),
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

    if current_thread.locked {
        return HttpResponse::Forbidden().json(UserError {
            error: "Lanka on lukittu, eikä siihen voi enää vastata!".to_owned(),
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
        Err(e) => {
            match e {
                diesel::result::Error::DatabaseError(database_error_kind, _) => match database_error_kind {
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation => return HttpResponse::Forbidden().json(UserError {
                        error: "Et voi vastata viestiin, jota ei ole olemassa!".to_owned(),
                    }),
                    _ => return HttpResponse::InternalServerError().finish(),
                },
                _ => return HttpResponse::InternalServerError().finish(),
            }
        },
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostDetailsOutput {
    pub thread_id: u32,
    pub board_handle: String,
}

pub async fn handle_post_details(
    path: web::Path<u32>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> impl Responder {
    let post_id = path.into_inner();

    let post = match Post::by_id(post_id, &conn_pool).await {
        Ok(post) => post,
        Err(e) => match e {
            diesel::result::Error::NotFound => return HttpResponse::NotFound().finish(),
            _ => return HttpResponse::InternalServerError().finish(),
        },
    };

    let thread = match Thread::thread_by_id(post.thread_id, &conn_pool).await {
        Ok(thread) => thread,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let board = match Board::by_id(thread.board_id, &conn_pool).await {
        Ok(board) => board,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    
    HttpResponse::Ok().json(PostDetailsOutput {
        thread_id: thread.id,
        board_handle: board.handle,
    })
}

pub async fn delete_post(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let post_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let post_wrapper = match Post::full_post_by_id(post_id, &conn_pool).await {
        Ok(post) => post,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.access_level < AccessLevel::Moderator as u8 && user_data.id != post_wrapper.post.user_id {
        return HttpResponse::Forbidden().finish();
    }

    let thread_op_post = match Thread::get_op_post(&conn_pool, post_wrapper.post.thread_id).await {
        Ok(op_post) => op_post,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if thread_op_post.id == post_wrapper.post.id {
        return HttpResponse::Forbidden().finish();
    }

    if let Some(attachment) = &post_wrapper.attachment {
        let file_location = format!("{}/{}", &attachment.file_location, &attachment.file_name);
        let thumbnail_location = format!("{}/{}", &attachment.thumbnail_location, &attachment.file_name);

        match remove_file(file_location) {
            Ok(_) => (),
            Err(e) => {
                println!("Error while removing file: {:?}", e);
            },
        };

        match remove_file(thumbnail_location) {
            Ok(_) => (),
            Err(e) => {
                println!("Error while removing file: {:?}", e);
            },
        };
    }

    match Post::delete_post(&conn_pool, post_wrapper.post.id).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}