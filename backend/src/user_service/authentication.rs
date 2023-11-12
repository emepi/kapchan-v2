use std::env;

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,          // Expiration time (as UTC timestamp)
    pub iat: usize,          // Issued at (as UTC timestamp)
    pub sub: String,         // Subject (whom token refers to)
}

pub async fn authenticate(token: &str) {

    let jwt_secret = env::var("JWT_SECRET")
    .expect(".env variable `JWT_SECRET` must be set");

    let jwt_expiration = env::var("JWT_EXPIRATION")
    .expect(".env variable `JWT_EXPIRATION` must be set");

    let claims = match decode::<Claims>(
        token, 
        &DecodingKey::from_secret(jwt_secret.as_ref()), 
        &Validation::default(),
    ) {
        Ok(data) => { data.claims },

        Err(err) => {
            match err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidSignature => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_) => todo!(),
                jsonwebtoken::errors::ErrorKind::RsaFailedSigning => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidAlgorithmName => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidKeyFormat => todo!(),
                jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_) => todo!(),
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidAudience => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidSubject => todo!(),
                jsonwebtoken::errors::ErrorKind::ImmatureSignature => todo!(),
                jsonwebtoken::errors::ErrorKind::InvalidAlgorithm => todo!(),
                jsonwebtoken::errors::ErrorKind::MissingAlgorithm => todo!(),
                jsonwebtoken::errors::ErrorKind::Base64(_) => todo!(),
                jsonwebtoken::errors::ErrorKind::Json(_) => todo!(),
                jsonwebtoken::errors::ErrorKind::Utf8(_) => todo!(),
                jsonwebtoken::errors::ErrorKind::Crypto(_) => todo!(),
                _ => todo!(),
            }
            // Unauthorized token
            todo!()
        },
    };

    let user_id = claims.sub;
}