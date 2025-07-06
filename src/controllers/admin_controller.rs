use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{models::{applications::Application, bans::{Ban, BanModel}, boards::{Board, BoardModel}, posts::Post, users::{AccessLevel, User}}, services::{applications::{count_preview_pages, is_reviewed, load_application_previews, review_application}, authentication::resolve_user}, views::{admin_view::{self, AdminTemplate}, application_list_view::{self, ApplicationListTemplate}, application_review_view::{self, ApplicationReviewTemplate}, banned_view::{self, BannedTemplate}, forbidden_view::{self, ForbiddenTemplate}, user_view::{self, UserTemplate}, users_view::{self, UsersTemplate}}};

use super::post_controller::BanUserInput;


pub async fn admin(
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

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: AccessLevel::Moderator as u8,
        })
        .await;
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

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return Ok(HttpResponse::Forbidden().finish());
    }

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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EditBoardForm {
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

pub async fn handle_board_edit(
    path: web::Path<u32>,
    user: Option<Identity>,
    input: web::Form<EditBoardForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let board_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return Ok(HttpResponse::Forbidden().finish());
    }

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

    match Board::update_board(&conn_pool, board_id, BoardModel {
        handle: &input.handle,
        title: &input.title,
        description: &input.description,
        access_level: input.access_level,
        active_threads_limit: input.threads_limit,
        thread_size_limit: input.thread_size,
        captcha: input.captcha.is_some(),
        nsfw: input.nsfw.is_some(),
    }).await {
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
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: AccessLevel::Admin as u8,
        })
        .await;
    }

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

    let page = path.into_inner();

    let previews = match load_application_previews(&conn_pool, page.into(), 20).await {
        Ok(previews) => previews,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let pages = match count_preview_pages(&conn_pool, 20).await {
        Ok(pages) => pages,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let boards = match Board::list_all(&conn_pool).await {
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    application_list_view::render(ApplicationListTemplate {
        access_level: user_data.access_level,
        boards,
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
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: AccessLevel::Admin as u8,
        })
        .await;
    }

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

    let application_id = path.into_inner();

    let application = match Application::by_id(&conn_pool, application_id).await {
        Ok(app) => app,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let boards = match Board::list_all(&conn_pool).await {
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    application_review_view::render(ApplicationReviewTemplate {
        access_level: user_data.access_level,
        boards,
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

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if user_data.access_level < AccessLevel::Admin as u8 {
        return Ok(HttpResponse::Forbidden().finish());
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

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return Ok(HttpResponse::Forbidden().finish());
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

#[derive(Debug, Deserialize)]
pub struct UsersRequest {
   min_access: Option<u8>,
   target_user: Option<String>,
}

pub async fn users_list(
    path: web::Path<u32>,
    info: web::Query<UsersRequest>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let page = path.into_inner();

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

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: AccessLevel::Moderator as u8,
        })
        .await;
    }

    let boards = match Board::list_all(&conn_pool).await {
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let pages = match User::count_users(
        &conn_pool, 
        info.target_user.clone(), 
        info.min_access.unwrap_or(AccessLevel::Anonymous as u8)
    ).await {
        Ok(count) => {
            let count = u64::try_from(count).unwrap();
            count.div_ceil(20)
        },
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let offset = (page - 1) * 20;

    let users = match User::load_users_data(
        &conn_pool, 
        info.target_user.clone(), 
        info.min_access.unwrap_or(AccessLevel::Anonymous as u8), 
        20, 
        offset.into()
    ).await {
        Ok(users) => users,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    return users_view::render(UsersTemplate {
        access_level: user_data.access_level,
        boards,
        pages,
        users,
    })
    .await;
}

pub async fn user(
    path: web::Path<u64>,
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let target_user_id = path.into_inner();

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

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return forbidden_view::render(ForbiddenTemplate {
            required_access_level: AccessLevel::Moderator as u8,
        })
        .await;
    }

    let boards = match Board::list_all(&conn_pool).await {
        Ok(boards) => boards,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let user = match User::by_id(target_user_id, &conn_pool).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let bans = match Ban::get_bans_by_user(&conn_pool, target_user_id).await {
        Ok(bans) => bans,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    return user_view::render(UserTemplate {
        access_level: user_data.access_level,
        boards,
        user,
        bans,
    })
    .await;
}

pub async fn handle_ban_deletion(
    path: web::Path<u32>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if user_data.access_level < AccessLevel::Moderator as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let ban_id = path.into_inner();

    match Ban::delete_ban(&conn_pool, ban_id).await {
        Ok(_) => Ok(HttpResponse::Found().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[derive(Deserialize)]
pub struct ModifyUserInput {
    pub access_level: u8,
    pub username: Option<String>,
    pub email: Option<String>,
}

pub async fn modify_user_by_id(
    path: web::Path<u64>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<ModifyUserInput>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = path.into_inner();

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

    match User::update_user(
        user_id, 
        input.access_level, 
        input.username.clone(), 
        input.email.clone(), 
        &conn_pool
    ).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn ban_user_by_id(
    path: web::Path<u64>,
    user: Option<Identity>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<BanUserInput>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = path.into_inner();

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.banned.is_some() && user_data.access_level != AccessLevel::Root as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let target_user = match User::by_id(user_id, &conn_pool).await {
        Ok(poster_user_data) => poster_user_data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if user_data.access_level < AccessLevel::Moderator as u8 || user_data.access_level <= target_user.access_level {
        return HttpResponse::Forbidden().finish();
    }

    let expires_at = (Utc::now() + Duration::days(input.ban_duration_days)).naive_utc();

    let ban_model = BanModel {
        moderator_id: user_data.id,
        user_id: Some(target_user.id),
        post_id: None,
        reason: Some(&input.reason),
        ip_address: "",
        expires_at,
    };

    match ban_model.insert(&conn_pool).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}