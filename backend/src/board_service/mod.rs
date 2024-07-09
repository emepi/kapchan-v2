pub mod board;


use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel::result::{DatabaseErrorKind, Error::{self, DatabaseError, NotFound}};
use diesel_async::{pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection, AsyncMysqlConnection};
use log::info;
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
    pub title: Option<Text<String>>,
    pub body: Option<Text<String>>,
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


    

    HttpResponse::Created().finish()
}