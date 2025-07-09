use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;
use tokio::task::spawn_local;

use crate::{chat::{handler, server::ChatServerHandle}, models::{boards::Board, chat_rooms::{ChatRoom, ChatRoomModel}, posts::Post, users::{AccessLevel, User}}, services::authentication::resolve_user, views::{banned_view::{self, BannedTemplate}, chat_view::{self, ChatTemplate}}};


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

#[derive(Deserialize)]
pub struct ChatInput {
    pub name: String,
    pub access_level: u8,
}

pub async fn create_chat_room(
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<ChatInput>,
    req: HttpRequest,
) -> impl Responder {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }
    
    if user_data.access_level < AccessLevel::Admin as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let chat_room = ChatRoomModel {
        name: &input.name,
        access_level: input.access_level,
    }
    .insert(&conn_pool)
    .await;

    match chat_room {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_chat_room(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }
    
    if user_data.access_level < AccessLevel::Admin as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let id = path.into_inner();

    match ChatRoom::delete_chat_room(&conn_pool, id).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}