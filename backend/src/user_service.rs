pub mod application;
pub mod authentication;
pub mod user;


use actix_web::{web, HttpResponse, Responder, HttpRequest, http::header};
use chrono::{NaiveDateTime, Utc};
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
        create_access_token, validate_claims, validate_password_pbkdf2, AccessLevel
    }, 
    user::{User, UserModel},
};


/// API endpoints exposed by the user service.
pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/sessions")
        .route(web::post().to(create_session))
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

/// JSON response body from `POST /sessions` method.
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
            }
            else if let Some(email) = &login.email {
                user_op = User::by_email(email, &conn_pool).await
            }
            else {
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
            db_err => {
                error!("Database error occured: {:?}", db_err);
                return HttpResponse::InternalServerError().finish();
            },
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

    // Create access token.
    let role = AccessLevel::try_from((&user).access_level).unwrap();

    let exp = match &input {
        Some(input) => match input.exp {
            Some(exp) => exp,
            None => role.default_exp(),
        },
        None => role.default_exp(),
    };

    let token = create_access_token(role, exp, user.id);

    HttpResponse::Created().json(CreateSessionOutput {
        access_token: token,
    })
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);