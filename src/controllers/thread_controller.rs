use std::fs::remove_file;

use actix_identity::Identity;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;

use crate::{models::{boards::Board, error::UserError, posts::Post, threads::Thread, users::AccessLevel}, services::{authentication::resolve_user, captchas::verify_captcha, threads::create_thread}, views::{banned_view::{self, BannedTemplate}, forbidden_view::{self, ForbiddenTemplate}, not_found_view, thread_view::{self, ThreadTemplate}}};


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

    if input.message.len() > 20_000 {
        return HttpResponse::Forbidden().json(UserError {
            error: "Viesti on liian pitkä (yli 20 000 merkkiä)".to_owned(),
        });
    }
    
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().json(UserError {
            error: "Käyttäjätilisi on bannattu!".to_owned(),
        });
    }

    let handle = path.into_inner();

    let current_board = match Board::by_handle(&conn_pool, &handle).await {
        Ok(board) => board,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if current_board.access_level > user_data.access_level {
        return HttpResponse::Forbidden().finish();
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
        user_data.ip_addr,
        input.attachment, 
        current_board.access_level,
        current_board.active_threads_limit,
    ).await;

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(err) => match err {
            diesel::result::Error::NotFound => HttpResponse::Forbidden().json(UserError {
                error: "Ongelma tiedoston käsittelyssä!".to_owned(),
            }),
            _ => HttpResponse::InternalServerError().finish(),
        },
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

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        let mut ban_post: Option<Post> = None;

        if let Some(post_id) = user_data.banned.clone().unwrap().post_id {
            match Post::by_id(post_id, &conn_pool).await {
                Ok(post) => ban_post = Some(post),
                Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
            };
        }

        return banned_view::render(BannedTemplate {
            ban: user_data.banned.unwrap(),
            post: ban_post,
        })
        .await;
    }

    let boards = match Board::list_all(&conn_pool).await {
        Ok(board) => board,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let (handle, thread_id) = path.into_inner();

    let current_board = match Board::by_handle(&conn_pool, &handle).await {
        Ok(board) => board,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < current_board.access_level {
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: current_board.access_level,
        })
        .await;
    }

    let thread = match Thread::by_id(thread_id, &conn_pool).await {
        Ok(thread) => thread,
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                return not_found_view::render().await;
            },
            _ => return Ok(HttpResponse::InternalServerError().finish()),
        },
    };

    thread_view::render(ThreadTemplate {
        access_level: user_data.access_level,
        user_id: user_data.id,
        boards,
        current_board,
        thread,
    }).await
}

pub async fn handle_thread_pin(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let thread_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return HttpResponse::Forbidden().finish();
    }

    match Thread::pin_thread(&conn_pool, thread_id, true).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn handle_thread_unpin(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let thread_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return HttpResponse::Forbidden().finish();
    }

    match Thread::pin_thread(&conn_pool, thread_id, false).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
pub struct ThreadLockInput {
    pub lock_status: bool,
}

pub async fn handle_thread_lock(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<ThreadLockInput>,
    req: HttpRequest,
) -> impl Responder {
    let thread_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return HttpResponse::Forbidden().finish();
    }

    match Thread::lock_thread(&conn_pool, thread_id, input.lock_status).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_thread(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let thread_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let thread_wrapper = match Thread::by_id(thread_id, &conn_pool).await {
        Ok(thread) => thread,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.access_level < AccessLevel::Moderator as u8 && user_data.id != thread_wrapper.thread.user_id {
        return HttpResponse::Forbidden().finish();
    }

    // Delete files
    thread_wrapper.posts.iter().for_each(|post| {
        if let Some(attachment) = &post.attachment {
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
    });

    match Thread::delete_thread(&conn_pool, thread_id).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}