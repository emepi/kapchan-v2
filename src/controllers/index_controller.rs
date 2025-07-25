use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{
    models::{boards::Board, posts::Post, users::AccessLevel}, 
    services::authentication::resolve_user, 
    views::{banned_view::{self, BannedTemplate}, index_view::{self, IndexTemplate}}
};


pub async fn index(
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

    let latest_posts = match Post::latest_posts_preview(&conn_pool, user_data.access_level, 10).await {
        Ok(posts) => posts,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    index_view::render(IndexTemplate {
        access_level: user_data.access_level,
        boards,
        latest_posts,
    }).await
}