use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, Error, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::{TemplateOnce};

use crate::{models::{boards::{Board, BoardSimple}, threads::{Thread, ThreadCatalogOutput}}, services::authentication::resolve_user};


#[derive(TemplateOnce)]
#[template(path = "board.stpl")]
pub struct BoardTemplate {
    pub access_level: u8,
    pub handle: String,
    pub boards: Vec<BoardSimple>,
    pub current_board: Board,
    pub threads: Vec<ThreadCatalogOutput>,
}

pub fn template(
    t: BoardTemplate
) -> Result<String, Error>{
    Ok(t
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?)
}

pub async fn board_view(
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
        return Ok(HttpResponse::Forbidden().finish())
    }

    let boards = match Board::list_all_simple(&conn_pool).await {
        Ok(board) => board,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let threads = match Thread::list_threads_by_board_catalog(&conn_pool, current_board.id).await {
        Ok(t) => t,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let body = template(BoardTemplate { 
        access_level: user_data.access_level,
        handle,
        boards,
        current_board,
        threads,
    })?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}