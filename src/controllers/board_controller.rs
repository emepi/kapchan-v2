use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::{boards::Board, threads::Thread}, services::authentication::resolve_user, views::{board_view::{self, BoardTemplate}, forbidden_view::{self, ForbiddenTemplate}}};


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
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
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
        handle,
        boards,
        current_board,
        threads,
    }).await
}