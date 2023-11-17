use std::env;

use actix_web::cookie::{Cookie, SameSite, self};
use argon2::{
    password_hash::{SaltString, rand_core::OsRng}, 
    Argon2, 
    PasswordHasher, 
    PasswordHash, 
    PasswordVerifier
};
use chrono::{Duration, Utc};
use diesel::{result::Error, QueryDsl, ExpressionMethods};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, 
    RunQueryDsl, 
    scoped_futures::ScopedFutureExt,
};
use jsonwebtoken::{decode, DecodingKey, Validation, EncodingKey, Header, encode};
use serde::{Serialize, Deserialize};

use crate::schema::users;

use super::user::User;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,          // Expiration time (as UTC timestamp)
    pub iat: usize,          // Issued at (as UTC timestamp)
    pub sub: String,         // Subject (whom token refers to)
}


pub fn validate_session_id(token: &str) -> Option<u32> {
    
    let jwt_secret = env::var("JWT_SECRET")
    .expect(".env variable `JWT_SECRET` must be set");

    let claims = match decode::<Claims>(
        token, 
        &DecodingKey::from_secret(jwt_secret.as_ref()), 
        &Validation::default(),
    ) {
        Ok(data) =>  Some(data.claims),

        Err(err) => {
            // TODO: match specific errors
            match err.kind() {
                _ => None,
            }
        },
    }?;

    claims.sub.parse::<u32>().ok()
}

pub fn create_authentication_token(sess_id: u32) -> Option<String> {
    
    let jwt_secret = env::var("JWT_SECRET")
    .expect(".env variable `JWT_SECRET` must be set");

    let jwt_expiration = env::var("JWT_EXPIRATION")
    .expect(".env variable `JWT_EXPIRATION` must be set")
    .parse::<i64>()
    .expect("`JWT_EXPIRATION` must be a valid number");

    let now = Utc::now();

    let user_claims = Claims {
        exp: (now + Duration::minutes(jwt_expiration)).timestamp() as usize,
        iat: now.timestamp() as usize,
        sub: sess_id.to_string(),
    };

    encode(
        &Header::default(), 
        &user_claims, 
        &EncodingKey::from_secret(jwt_secret.as_ref())
    )
    .ok()
}


// TODO: sanitize user inputs and refactor
pub async fn register_user(
    user_id: u32,
    username: &str,
    email: &str,
    password: &str,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Option<()> {
    let mut connection = conn_pool.get().await.ok()?;

    let password = encrypt_password(password)?;

    connection.transaction::<_, Error, _>(|conn| async move {

        let _ = diesel::update(users::table.find(user_id))
        .set((
            users::username.eq(username),
            users::email.eq(email),
            users::password_hash.eq(password),
        ))
        .execute(conn)
        .await;


        Ok(())
    }.scope_boxed())
    .await
    .ok()
}

// TODO: Test

pub fn encrypt_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);

    // TODO: minimize memory footprint
    Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .map(|hash| hash.to_string())
    .ok()
}

pub fn hashes_to_password(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}