use std::env;

use argon2::{
    password_hash::{SaltString, rand_core::OsRng}, 
    Argon2, 
    PasswordHasher, 
    PasswordHash, 
    PasswordVerifier
};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, 
    DecodingKey, 
    Validation, 
    EncodingKey, 
    Header, 
    encode
};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,          // Expiration time (as UTC timestamp)
    pub iat: usize,          // Issued at (as UTC timestamp)
    pub sub: String,         // Subject (whom token refers to)
    pub role: u8,            // session access level
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

pub fn validate_claims(token: &str) -> Option<Claims> {
    let jwt_secret = env::var("JWT_SECRET")
    .expect(".env variable `JWT_SECRET` must be set");

    decode::<Claims>(
        token, 
        &DecodingKey::from_secret(jwt_secret.as_ref()), 
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

pub fn create_authentication_token(
    sess_id: u32, 
    access_level: u8
) -> Option<String> {
    
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
        role: access_level,
    };

    encode(
        &Header::default(), 
        &user_claims, 
        &EncodingKey::from_secret(jwt_secret.as_ref())
    )
    .ok()
}

pub fn hash_password_a2id(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .map(|hash| hash.to_string())
    .ok()
}

pub fn validate_password_a2id(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}