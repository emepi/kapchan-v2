use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::{TemplateOnce};

use crate::{models::{boards::{Board, BoardSimple}, threads::{Thread, ThreadData}}, services::authentication::resolve_user};
use crate::services::time::fi_datetime;
use crate::services::files::display_filesize;


#[derive(TemplateOnce)]
#[template(path = "thread.stpl")]
struct ThreadTemplate {
    access_level: u8,
    boards: Vec<BoardSimple>,
    current_board: Board,
    thread: ThreadData,
}

pub async fn thread_view(
    path: web::Path<(String, u32)>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let boards = match Board::list_all_simple(&conn_pool).await {
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

    let body = ThreadTemplate { 
        access_level: user_data.access_level,
        boards,
        current_board,
        thread, 
    }
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}