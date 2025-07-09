use actix_identity::Identity;
use actix_web::{Error, web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use tokio::task::spawn_local;

use crate::{chat::{handler, server::ChatServerHandle}, models::{boards::Board, posts::Post, users::{AccessLevel, User}}, services::authentication::resolve_user, views::{banned_view::{self, BannedTemplate}, chat_view::{self, ChatTemplate}}};


pub async fn chat_ws(
    user: Option<Identity>,
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<ChatServerHandle>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let user = match User::by_id(user_data.id, &conn_pool).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    
    spawn_local(handler::chat_ws(
        user.username.unwrap_or(format!("anonyymi-{}", user_data.id)),
        (**chat_server).clone(),
        session,
        msg_stream,
    ));

    Ok(res)
}

pub async fn chat(
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
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    chat_view::render(ChatTemplate {
        access_level: user_data.access_level,
        boards,
    }).await
}