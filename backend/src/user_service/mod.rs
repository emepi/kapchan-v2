pub mod application;
pub mod authentication;
pub mod user;
pub mod session;


use actix_web::{web, HttpResponse, Responder, HttpRequest, http::header};
use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl, 
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, scoped_futures::ScopedFutureExt
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use self::{
    authentication::{
        authenticate_user, create_access_token, validate_password_pbkdf2, AccessLevel
    }, session::{Session, SessionModel, SessionModificationModel}, user::{User, UserModel}
};


/// API endpoints exposed by the user service.
pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/sessions")
        .route(web::post().to(create_session))
    )
    .service(
        web::resource("/sessions/{id}")
        .route(web::put().to(update_session))
    );
}


/// JSON body accepted by `POST /sessions` method.
/// 
/// Either `username` or `email` shall be provided for user identification. If 
/// both are present, `username` takes precedence.
#[derive(Debug, Deserialize)]
struct CreateSessionInput {
    pub email: Option<String>,
    /// Expiration time as UTC timestamp.
    pub exp: Option<i64>,
    pub password: String,
    pub username: Option<String>,
}

/// JSON response body from `POST /sessions` and `PUT /sessions` methods.
#[derive(Debug, Serialize)]
struct CreateSessionOutput {
    pub access_token: String,
}

/// Handler for `POST /sessions` request.
/// 
/// Client may request access token with `CreateSessionInput` serving as a login
/// method. Otherwise, token is created for new anonymous user.
async fn create_session(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: Option<web::Json<CreateSessionInput>>,
) -> impl Responder {
    // Resolve the user.
    let user = match &input {
        Some(login) => {
            let user_op;

            if let Some(username) = &login.username {
                user_op = User::by_username(username, &conn_pool).await
            } else if let Some(email) = &login.email {
                user_op = User::by_email(email, &conn_pool).await
            } else {
                // Not identifiable.
                return HttpResponse::BadRequest().finish();
            }

            user_op
        },
        None => UserModel::anon_user().insert(&conn_pool).await,
    };

    let user = match user {
        Ok(user) => user,
        Err(db_err) => match db_err {
            Error::NotFound => return HttpResponse::NotFound().finish(),
            _ => return HttpResponse::InternalServerError().finish(),
        },
    };

    // Check password protection.
    // NOTE: Identifiable users without password are accessible for everyone by
    //       default.
    if let Some(password_hash) = &user.password_hash {
        let password = &input.as_ref().unwrap().password;

        if !validate_password_pbkdf2(&password_hash, &password) {
            return HttpResponse::Unauthorized().finish();
        }
    }

    // Create a session.
    let role = AccessLevel::try_from((&user).access_level).unwrap();

    let exp = match &input {
        Some(input) => match input.exp {
            Some(exp) => match NaiveDateTime::from_timestamp_opt(exp, 0) {
                Some(time) => time,
                None => return HttpResponse::BadRequest().finish(),
            },
            None => role.default_exp().naive_utc(),
        },
        None => role.default_exp().naive_utc(),
    };

    let session = SessionModel {
        user_id: user.id,
        ended_at: &exp,
    }
    .insert(&conn_pool)
    .await;

    let session = match session {
        Ok(session) => session,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Create an access token.
    let token = create_access_token(role, exp.timestamp(), session.id);

    HttpResponse::Created().json(CreateSessionOutput {
        access_token: token,
    })
}

/// JSON body accepted by `PUT /sessions` method.
#[derive(Debug, Deserialize)]
struct UpdateSessionInput {
    /// Expiration time as UTC timestamp (seconds).
    pub exp: i64,
}

/// Handler for `PUT /sessions` request.
async fn update_session(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<UpdateSessionInput>,
    path: web::Path<u32>,
    req: HttpRequest,
) -> impl Responder {
    // Check user permissions.
    let conn_info = match authenticate_user(&conn_pool, req).await {
        Ok(conn_info) => conn_info,
        Err(mut err_res) => return err_res.finish(),
    };

    let target_session = match Session::by_id(path.into_inner(), &conn_pool).await {
        Ok(session) => session,
        Err(db_err) => match db_err {
            Error::NotFound => return HttpResponse::NotFound().finish(),
            _ => return HttpResponse::InternalServerError().finish(),
        },
    };

    let self_modification = conn_info.user_id == target_session.user_id;

    // Users below admin privileges may only modify their own session.
    if (!self_modification) && (conn_info.access_level < AccessLevel::Admin as u8) {
        return HttpResponse::Forbidden().finish();
    }

    // Modify session with the inputs.
    let exp = match NaiveDateTime::from_timestamp_opt(input.exp, 0) {
        Some(time) => time,
        None => return HttpResponse::BadRequest().finish(),
    };

    let modified_session = SessionModificationModel {
        user_id: None,
        ended_at: Some(&exp),
    }
    .modify(target_session.id, &conn_pool)
    .await;

    match modified_session {
        Ok(modified_session) => HttpResponse::Created().json(modified_session),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);