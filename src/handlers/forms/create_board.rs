use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{handlers::admin::{template, AdminTemplate}, models::{boards::{Board, BoardModel}, users::AccessLevel}, services::authentication::resolve_user};


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateBoardForm {
    #[validate(
        length(
            min = "1",
            max = "8",
            message = "Handle must be 1-8 characters long."
        ),
        regex(
            path = Regex::new(r"[a-zA-Z]+").unwrap(),
            message = "Handle must contain only alphabets."
        )
    )]
    pub handle: String,
    #[validate(
        length(
            min = "1",
            max = "255",
            message = "Title must be 1-255 characters long."
        )
    )]
    pub title: String,
    pub description: String,
    pub access_level: u8,
    pub cooldown: u32,
    pub threads_limit: u32,
    pub thread_size: u32,
    pub captcha: Option<String>,
    pub nsfw: Option<String>,
    pub unique_posts: Option<String>,
}

pub async fn handle_board_creation(
    user: Option<Identity>,
    form: web::Form<CreateBoardForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Admin as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    match form.validate() {
        Ok(_) => (),
        Err(e) => {
            let errors = e.field_errors()
            .iter()
            .map(|err| err.1.iter().map(|k| k.to_string()).collect::<Vec<String>>())
            .flat_map(|errors| errors)
            .collect();

            let boards = match Board::list_all(&conn_pool).await {
                Ok(boards) => boards,
                Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
            };

            let t = AdminTemplate {
                errors,
                access_level: user_data.access_level,
                boards,
            };

            let body = template(t).unwrap();

            return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body))
        },
    };

    let board = BoardModel {
        handle: &form.handle,
        title: &form.title,
        access_level: form.access_level,
        post_cooldown_time_sec: form.cooldown,
        active_threads_limit: form.threads_limit,
        thread_size_limit: form.thread_size,
        captcha: form.captcha.is_some(),
        nsfw: form.nsfw.is_some(),
        description: &form.description,
        unique_posts: form.unique_posts.is_some(),
    }
    .insert(&conn_pool)
    .await;

    match board {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/admin")).finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}