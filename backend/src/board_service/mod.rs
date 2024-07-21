pub mod board;
pub mod post;


use std::{fs, str::FromStr};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel::result::{DatabaseErrorKind, Error::{self, DatabaseError, NotFound}};
use diesel_async::{pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection, AsyncMysqlConnection};
use log::info;
use post::{FileModel, PostModel, ThreadModel};
use serde::Deserialize;

use crate::user_service::authentication::{authenticate_user, AccessLevel};

use self::board::{Board, BoardModel};


/// API endpoints exposed by the board service.
pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/boards")
        .route(web::get().to(boards))
        .route(web::post().to(create_board))
    )
    .service(
        web::resource("/boards/{id}/threads")
    )
    .service(
        web::resource("/threads")
        .route(web::post().to(create_thread))
    );
}


/// Handler for `GET /boards` request.
async fn boards(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> impl Responder {
    let boards = Board::fetch_boards(&conn_pool).await;

    match boards {
        Ok(boards) => HttpResponse::Ok().json(boards),
        Err(err) => match err {
            NotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

/// JSON body accepted by `POST /boards` method.
#[derive(Debug, Deserialize)]
struct CreateBoardInput {
    pub title: String,
    pub handle: String,
    pub access_level: u8,
    pub bump_limit: u32,
    pub nsfw: bool,
}

/// Handler for `POST /boards` request.
async fn create_board(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<CreateBoardInput>,
    req: HttpRequest,
) -> impl Responder {
    // Check user permissions.
    let conn_info = match authenticate_user(&conn_pool, req).await {
        Ok(conn_info) => conn_info,
        Err(mut err_res) => return err_res.finish(),
    };

    if conn_info.access_level < AccessLevel::Owner as u8 {
        return HttpResponse::Forbidden().finish();
    }

    // Insert the board into database.
    let board = BoardModel {
        handle: &input.handle,
        title: &input.title,
        access_level: input.access_level,
        bump_limit: input.bump_limit,
        nsfw: input.nsfw,
    }
    .insert(&conn_pool)
    .await;

    match board {
        Ok(board) => HttpResponse::Created().json(board),
        Err(err) => match err {
            DatabaseError(db_err, _) => match db_err {
                DatabaseErrorKind::UniqueViolation => HttpResponse::BadRequest().finish(),
                _ => HttpResponse::InternalServerError().finish(),
            },
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

/// Multipart form accepted by `POST /threads` method.
#[derive(Debug, MultipartForm)]
struct CreateThreadInput {
    pub title: Text<String>,
    pub body: Text<String>,
    pub board: Text<String>,
    pub attachment: Option<TempFile>,
}

/// Handler for `POST /threads` request.
async fn create_thread(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    MultipartForm(input): MultipartForm<CreateThreadInput>,
    req: HttpRequest,
) -> impl Responder {
    // Fetch board info
    let handle = input.board.into_inner();
    let board = Board::by_handle(&handle, &conn_pool).await;

    let board = match board {
        Ok(board) => board,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };


    // Check permissions
    if board.access_level > AccessLevel::Anonymous as u8 {
        let conn_info = match authenticate_user(&conn_pool, req).await {
            Ok(conn_info) => conn_info,
            Err(mut err_res) => return err_res.finish(),
        };
    
        if conn_info.access_level < board.access_level {
            return HttpResponse::Forbidden().finish();
        }
    }

    // Create a thread
    let op_post = PostModel {
        op_id: None,
        body: input.body.into_inner(),
        access_level: board.access_level,
    }
    .insert(&conn_pool)
    .await;

    let op_post = match op_post {
        Ok(op_post) => op_post,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let thread = ThreadModel {
        id: op_post.id,
        board: board.id,
        title: input.title.into_inner(),
        pinned: false,
    };

    match thread.insert(&conn_pool).await {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }

    // Process attachment
    if let Some(attachment) = input.attachment {
        let dir_path = format!("target/files/{}", op_post.id);

        // TODO: tokio async v
        match fs::create_dir_all(&dir_path) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }

        let file = attachment.file;
        let file_name = match attachment.file_name {
            Some(name) => name,
            None => String::from("file"),
        };

        let mime = match attachment.content_type {
            Some(mime) => mime,
            None => return HttpResponse::UnprocessableEntity().finish(),
        };

        let file_path = format!("{}/file.{}", dir_path, mime.subtype().as_str()); // TODO

        match file.persist(&file_path) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }

        // TODO: create a thumbnail

        let file_model = FileModel {
            id: op_post.id,
            file_name,
            thumbnail: String::default(),
            file_path,
        };

        match file_model.insert(&conn_pool).await {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }
    

    HttpResponse::Created().finish()
}