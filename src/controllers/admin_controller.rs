use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{models::{applications::Application, boards::{Board, BoardModel}, users::{AccessLevel, User}}, services::{applications::{count_preview_pages, is_reviewed, load_application_previews, review_application}, authentication::resolve_user}, views::{admin_view::{self, AdminTemplate}, application_list_view::{self, ApplicationListTemplate}, application_review_view::{self, ApplicationReviewTemplate}}};


pub async fn admin(
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let boards = match Board::list_all(&conn_pool).await {
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    admin_view::render(AdminTemplate {
        access_level: user_data.access_level,
        errors: vec![],
        boards,
    }).await
}

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
    pub threads_limit: u32,
    pub thread_size: u32,
    pub captcha: Option<String>,
    pub nsfw: Option<String>,
}

pub async fn handle_board_creation(
    user: Option<Identity>,
    input: web::Form<CreateBoardForm>,
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

    match input.validate() {
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

            let template = AdminTemplate {
                errors,
                access_level: user_data.access_level,
                boards,
            };

            return admin_view::render(template).await;
        },
    };

    let board = BoardModel {
        handle: &input.handle,
        title: &input.title,
        access_level: input.access_level,
        active_threads_limit: input.threads_limit,
        thread_size_limit: input.thread_size,
        captcha: input.captcha.is_some(),
        nsfw: input.nsfw.is_some(),
        description: &input.description,
    }
    .insert(&conn_pool)
    .await;

    match board {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/admin")).finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn applications_list(
    path: web::Path<u32>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Admin as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let page = path.into_inner();

    let previews = match load_application_previews(&conn_pool, page.into(), 20).await {
        Ok(previews) => previews,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let pages = match count_preview_pages(&conn_pool, 20).await {
        Ok(pages) => pages,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    application_list_view::render(ApplicationListTemplate {
        access_level: user_data.access_level,
        previews,
        pages,
    }).await
}

pub async fn application_review(
    path: web::Path<u32>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level < AccessLevel::Admin as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let application_id = path.into_inner();

    let application = match Application::by_id(&conn_pool, application_id).await {
        Ok(app) => app,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    application_review_view::render(ApplicationReviewTemplate {
        access_level: user_data.access_level,
        application,
    }).await
}

pub async fn handle_application_accept(
    path: web::Path<u32>,
    user: Option<Identity>,
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

    let application_id = path.into_inner();

    let reviewed = match is_reviewed(&conn_pool, application_id).await {
        Ok(status) => status,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if reviewed {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let application = match review_application(&conn_pool, application_id, user_data.id, true).await {
        Ok(app) => app,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    match User::update_access_level(application.user_id, AccessLevel::Member as u8, &conn_pool).await {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/applications/1")).finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn handle_application_deny(
    path: web::Path<u32>,
    user: Option<Identity>,
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

    let application_id = path.into_inner();

    let reviewed = match is_reviewed(&conn_pool, application_id).await {
        Ok(status) => status,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if reviewed {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let application = match review_application(&conn_pool, application_id, user_data.id, false).await {
        Ok(app) => app,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    match User::update_access_level(application.user_id, AccessLevel::Registered as u8, &conn_pool).await {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/applications/1")).finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}