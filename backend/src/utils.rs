pub mod authentication {
    use std::env;

    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use password_hash::{Output, PasswordHash, PasswordVerifier, Salt, SaltString};
    use pbkdf2::{pbkdf2_hmac, Algorithm, Params, Pbkdf2};
    use rand_core::{OsRng, RngCore};
    use sha2::{Digest, Sha256};

    use super::models::Claims;

    /// Creates JSON web tokens for user authentication.
    /// 
    /// # Panics
    /// 
    /// Panics if private key is not found in the environment.
    pub fn create_access_token(
        access_level: u8,
        exp: i64,
        session_id: u32, 
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let jwt_secret = env::var("JWT_SECRET")
        .expect(".env variable `JWT_SECRET` must be set");

        let user_claims = Claims {
            exp: exp as usize,
            iat: Utc::now().timestamp() as usize,
            sub: session_id,
            role: access_level,
        };

        encode(
            &Header::default(), 
            &user_claims, 
            &EncodingKey::from_secret(jwt_secret.as_ref())
        )
    }


    /// Hashes passwords with PBKDF2 key derivation function.
    /// 
    /// Result is returened as PHC string format containing the unique password
    /// salt, hash, & parameters used for hashing.
    /// 
    /// # Panics
    /// 
    /// Panics if an illegal state occurs in rust crypto conversions.
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

    /// Checks if password can produce the hash given in PHC string format.
    /// 
    /// # Panics
    /// 
    /// Panics if hash string is malformatted.
    pub fn validate_password_pbkdf2(hash: &str, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(hash).unwrap();

        Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    }
}

pub mod models {
    use serde::{Deserialize, Serialize};


    #[derive(Debug, Serialize)]
    pub struct ErrorOutput<'a> {
        pub err: &'a str,
    }

    /// Access token claim set.
    /// 
    /// Forms JSON web token payload fully readable by the client in Base64Url 
    /// encoding.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        /// Expiration time as UTC timestamp.
        pub exp: usize,
        /// Issued at as UTC timestamp.
        pub iat: usize,
        /// Subject (session id).
        pub sub: u32,
        /// Access level.
        pub role: u8,
    }
}