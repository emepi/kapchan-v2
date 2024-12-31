use actix_identity::Identity;
use actix_web::{error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sailfish::{TemplateOnce};

use crate::{models::{boards::{Board, BoardSimple}, posts::{Post, PostPreview}}, services::authentication::resolve_user};


#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct IndexTemplate {
    access_level: u8,
    boards: Vec<BoardSimple>,
    latest_posts: Vec<PostPreview>,
}

pub async fn index_view(
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

    let latest_posts = match Post::latest_posts_preview(&conn_pool, user_data.access_level, 10).await {
        Ok(posts) => posts,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let body = IndexTemplate { 
        access_level: user_data.access_level,
        boards,
        latest_posts, 
    }
    .render_once()
    .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(body))
}