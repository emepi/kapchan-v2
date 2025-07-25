use actix_identity::Identity;
use actix_web::{http::StatusCode, HttpMessage, HttpRequest};
use chrono::Utc;
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use password_hash::{Output, PasswordHash, PasswordVerifier, Salt, SaltString};
use pbkdf2::{pbkdf2_hmac, Algorithm, Params, Pbkdf2};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use crate::models::{bans::Ban, users::{User, UserData}};

use super::users::create_anonymous_user;


pub async fn resolve_user(
    user: Option<Identity>,
    request: HttpRequest,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Result<UserData, Error> {
    let user = match user {
        Some(user) => {
            let id = match user.id().unwrap().parse::<u64>() {
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

    //TODO: configure for proxy
    let ip_addr = request.peer_addr().unwrap().ip().to_string();
    let user_agent = request.headers().get("User-Agent")
    .map(|agent| agent.to_str())
    .map(|val| val.unwrap_or("").to_string())
    .unwrap_or(String::default());

    let mut ban = match Ban::get_last_ban(&conn_pool, user.id, ip_addr.clone()).await {
        Ok(ban) => ban,
        Err(e) => return Err(e),
    };

    if let Some(ref ban_history) = ban {
        if Utc::now().timestamp() > ban_history.expires_at.and_utc().timestamp() {
            ban = None;
        }
    }

    Ok(UserData {
        id: user.id,
        access_level: user.access_level,
        ip_addr,
        user_agent,
        banned: ban,
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

    match validate_password_pbkdf2(&hash, password) {
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

    match validate_password_pbkdf2(&hash, password) {
        true => {
            Identity::login(&request.extensions(), user.id.to_string()).unwrap();
            Ok(())
        },
        false => Err(StatusCode::FORBIDDEN),
    }
}

pub fn hash_password_pbkdf2(password: &str) -> String {
    let mut salt_bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    OsRng.fill_bytes(&mut salt_bytes);

    let iterations = 5000;
    let password = password.as_bytes();

    let output = Output::init_with(Sha256::output_size(), |out| {
        pbkdf2_hmac::<Sha256>(password, &salt_bytes, iterations, out);
        Ok(())
    })
    .unwrap();

    let params = Params {
        rounds: iterations,
        output_length: Sha256::output_size(),
    }
    .try_into()
    .unwrap();

    let salt_b64 = SaltString::encode_b64(&salt_bytes).unwrap();

    let hash = PasswordHash {
        algorithm: Algorithm::PBKDF2_SHA256_IDENT,
        version: None,
        params,
        salt: Some(salt_b64.as_salt()),
        hash: Some(output),
    };

    hash.to_string()
}

pub fn validate_password_pbkdf2(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();

    Pbkdf2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}