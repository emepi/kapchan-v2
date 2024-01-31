pub mod board;


use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;

use crate::user_service::authentication::{authenticate_user, AccessLevel};

use self::board::BoardModel;


/// API endpoints exposed by the board service.
pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/boards")
        .route(web::post().to(create_board))
    );
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
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}