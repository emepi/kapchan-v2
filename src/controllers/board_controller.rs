use std::fs::remove_file;

use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::{boards::Board, threads::Thread, users::AccessLevel}, services::authentication::resolve_user, views::{board_view::{self, BoardTemplate}, forbidden_view::{self, ForbiddenTemplate}, not_found_view}};


pub async fn board(
    path: web::Path<String>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let handle = path.into_inner();

    let current_board = match Board::by_handle(&conn_pool, &handle).await {
        Ok(board) => board,
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                return not_found_view::render().await;
            },
            _ => return Ok(HttpResponse::InternalServerError().finish()),
        },
    };

    if current_board.access_level > user_data.access_level {
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: current_board.access_level,
        })
        .await;
    }

    let boards = match Board::list_all(&conn_pool).await {
        Ok(board) => board,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let threads = match Thread::list_threads_by_board_catalog(&conn_pool, current_board.id).await {
        Ok(t) => t,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    board_view::render(BoardTemplate {
        access_level: user_data.access_level,
        user_id: user_data.id,
        handle,
        boards,
        current_board,
        threads,
    }).await
}

pub async fn delete_board(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let board_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.access_level < AccessLevel::Admin as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let catalog = match Board::list_all_threads_and_posts(&conn_pool, board_id).await {
        Ok(catalog) => catalog,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Delete files
    catalog.iter().for_each(|thread| {
        thread.posts.iter().for_each(|post| {
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
    });

    match Board::delete_board(&conn_pool, board_id).await {
        Ok(_) => HttpResponse::Found().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}