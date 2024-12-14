use actix_identity::Identity;
use actix_web::{http::StatusCode, HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString}, 
    Argon2, 
    PasswordHash, 
    PasswordHasher, 
    PasswordVerifier
};
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::models::users::{User, UserData};

use super::users::create_anonymous_user;


pub async fn resolve_user(
    user: Option<Identity>,
    request: HttpRequest,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Result<UserData, Error> {
    let user = match user {
        Some(user) => {
            let id = match user.id().unwrap().parse::<u32>() {
                Ok(id) => id,
                Err(_) => return Err(Error::NotFound),
            };

            User::by_id(id, conn_pool).await?
        },
        None => {
            let user = create_anonymous_user(conn_pool).await?;

            Identity::login(&request.extensions(), user.id.to_string()).unwrap();
            user
        },
    };

    Ok(UserData {
        id: user.id,
        access_level: user.access_level,
    })
}

pub async fn login_by_username(
    username: &str,
    password: &str,
    conn_pool: &Pool<AsyncMysqlConnection>,
    request: HttpRequest,
) -> Result<(), StatusCode> {
    let user = User::by_username(username, conn_pool)
    .await
    .map_err(|err| match err {
            Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    )?;

    let hash = match user.password_hash {
        Some(pwd_hash) => pwd_hash,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match validate_password_argon2id(&hash, password) {
        true => {
            Identity::login(&request.extensions(), user.id.to_string()).unwrap();
            Ok(())
        },
        false => Err(StatusCode::FORBIDDEN),
    }
}

pub async fn login_by_email(
    email: &str,
    password: &str,
    conn_pool: &Pool<AsyncMysqlConnection>,
    request: HttpRequest,
) -> Result<(), StatusCode> {
    let user = User::by_email(email, conn_pool)
    .await
    .map_err(|err| match err {
            Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    )?;

    let hash = match user.password_hash {
        Some(pwd_hash) => pwd_hash,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match validate_password_argon2id(&hash, password) {
        true => {
            Identity::login(&request.extensions(), user.id.to_string()).unwrap();
            Ok(())
        },
        false => Err(StatusCode::FORBIDDEN),
    }
}

pub fn hash_password_argon2id(
    password: &str
) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string()
}

pub fn validate_password_argon2id(
    hash: &str, 
    password: &str
) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();

    Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}