/// Route handlers for user services.
/// 
/// 
pub mod routes {
    use actix_web::{web, HttpRequest, HttpResponse, Responder};
    use chrono::{DateTime, Duration, Utc};
    use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
    use validator::Validate;

    use super::{
        authentication::{authenticate_user, create_access_token, hash_password_pbkdf2}, 
        database::{create_anonymous_user, login_user}, 
        models::{AccessLevel, CreateSessionInput, CreateSessionOutput, RegisterUserInput, SessionModel, UserModel},
    };
    

    /// Handler for `POST /register` request.
    /// 
    /// This is used to register anonymous user with an username, password, and email.
    pub async fn register_user(
        conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
        input: web::Json<RegisterUserInput>,
        req: HttpRequest,
    ) -> impl Responder {
        // Validate user input
        match input.validate() {
            Ok(_) => (),
            Err(e) => return HttpResponse::UnprocessableEntity().json(&e.to_string()),
        }

        // Check user
        let conn_info = match authenticate_user(&conn_pool, req).await {
            Ok(conn_info) => conn_info,
            Err(err_res) => return err_res,
        };

        // Only anonymous users can complete registration
        if conn_info.access_level != AccessLevel::Anonymous as u8 {
            return HttpResponse::Forbidden().finish();
        }

        // Attempt to register user info
        let res = UserModel {
            access_level: AccessLevel::Registered as u8,
            username: Some(&input.username),
            email: Some(&input.email),
            password_hash: Some(&hash_password_pbkdf2(&input.password)),
        }
        .update_by_id(conn_info.user_id, &conn_pool)
        .await;

        match res {
            Ok(_) => (),
            Err(e) => match e {
                diesel::result::Error::DatabaseError(database_error_kind, _) => {
                    match database_error_kind {
                        diesel::result::DatabaseErrorKind::UniqueViolation => {
                            return HttpResponse::UnprocessableEntity().json(
                                String::from("Username is already taken.")
                            )
                        },
                        _ => return HttpResponse::InternalServerError().json(
                            String::from("Failed to register user.")
                        ),
                    }
                },
                _ => return HttpResponse::InternalServerError().json(
                    String::from("Failed to register user.")
                ),
            },
        }

        // End current session as anonymous user
        let _ = SessionModel {
            user_id: conn_info.user_id,
            ended_at: &Utc::now().naive_utc(),
        }
        .update_by_id(conn_info.session_id, &conn_pool)
        .await;

        // Create a new session
        let session = SessionModel {
            user_id: conn_info.user_id,
            ended_at: &(Utc::now() + Duration::days(365)).naive_utc(),
        }
        .insert(&conn_pool)
        .await;

        let session = match session {
            Ok(session) => session,
            Err(_) => return HttpResponse::InternalServerError().json(
                String::from("Failed to create a session.")
            ),
        };

        // Create a json web token
        let token = match create_access_token(
            AccessLevel::Registered as u8, 
            session.ended_at.and_utc().timestamp(), 
            session.id
        ) {
            Ok(jwt) => jwt,
            Err(_) => return HttpResponse::InternalServerError().json(
                String::from("Failed to create a session.")
            ),
        };

        HttpResponse::Created().json(CreateSessionOutput {
            access_token: token,
        })
    }

    /// Handler for `POST /sessions` request.
    /// 
    /// Client may request access token with `CreateSessionInput` serving as a login
    /// method. Otherwise, a token is created for new anonymous user.
    pub async fn create_session(
        conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
        input: Option<web::Json<CreateSessionInput>>,
    ) -> impl Responder {
        // Validate user input
        let mut exp: Option<i64> = None;

        if let Some(input) = &input {
            match input.validate() {
                Ok(_) => (),
                Err(e) => return HttpResponse::UnprocessableEntity().json(&e.to_string()),
            }

            exp = input.exp;
        }

        // Attempt to fetch or create an user
        let user = match &input {
            Some(login) => login_user(&login.username, &login.password, &conn_pool).await,
            None => create_anonymous_user(&conn_pool).await,
        };

        let user = match user {
            Ok(user) => user,
            Err(err_res) => return err_res,
        };

        // TODO: check if user is banned.

        // Resolve session expiry time (default to 1 year).
        let exp = match exp {
            Some(expiry) => match DateTime::from_timestamp(expiry, 0) {
                Some(time) => time.naive_utc(),
                None => return HttpResponse::UnprocessableEntity().json(
                    String::from("Timestamp out of range.")
                ),
            },
            None => (Utc::now() + Duration::days(365)).naive_utc(),
        };

        // Create the session
        let session = SessionModel {
            user_id: user.id,
            ended_at: &exp,
        }
        .insert(&conn_pool)
        .await;

        let session = match session {
            Ok(session) => session,
            Err(_) => return HttpResponse::InternalServerError().json(
                String::from("Failed to create a session.")
            ),
        };

        // Create a json web token
        let token = match create_access_token(user.access_level, exp.and_utc().timestamp(), session.id) {
            Ok(jwt) => jwt,
            Err(_) => return HttpResponse::InternalServerError().json(
                String::from("Failed to create a session.")
            ),
        };

        HttpResponse::Created().json(CreateSessionOutput {
            access_token: token,
        })
    }
}

/// Database interactions for user services.
/// 
/// 
pub mod database {
    use std::env;

    use actix_web::HttpResponse;
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };
    use log::{error, info};

    use crate::schema::{sessions, users};

    use super::{
        authentication::{hash_password_pbkdf2, validate_password_pbkdf2}, 
        models::{AccessLevel, Session, SessionModel, User, UserModel}
    };


    /// Creates an anonymous user to the database.
    pub async fn create_anonymous_user(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, HttpResponse> {
        UserModel {
            access_level: AccessLevel::Anonymous as u8,
            username: None,
            password_hash: None,
            email: None,
        }
        .insert(conn_pool)
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())
    }

    /// Creates or updates the root user from env file.
    pub async fn create_root_user(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) {
        let root_pwd = env::var("ROOT_PASSWORD");

        match root_pwd {
            Ok(pwd) => match User::by_username("root", conn_pool).await {
                Ok(root_user) => {
                    // Update root user
                    let res = UserModel {
                        access_level: AccessLevel::Root as u8,
                        username: Some("root"),
                        password_hash: Some(&hash_password_pbkdf2(&pwd)),
                        email: None,
                    }
                    .update_by_id(root_user.id, conn_pool)
                    .await;

                    match res {
                        Ok(_) => info!("Root user updated."),
                        Err(db_err) => error!(
                            "Root user was not set or updated due to an database error: {:?}.", 
                            db_err
                        ),
                    }
                },
                Err(db_err) => match db_err {
                    Error::NotFound => {
                        let res = UserModel {
                            access_level: AccessLevel::Root as u8,
                            username: Some("root"),
                            password_hash: Some(&hash_password_pbkdf2(&pwd)),
                            email: None,
                        }
                        .insert(conn_pool)
                        .await;

                        match res {
                            Ok(_) => info!("Root user created."),
                            Err(db_err) => error!(
                                "Root user was not set or updated due to an database error: {:?}.", 
                                db_err
                            ),
                        }
                    },
                    _ => error!(
                        "Root user was not set or updated due to an database error: {:?}.", 
                        db_err
                    ),
                },
            },
            Err(_) => info!("Root user was not set or updated."),
        }
    }

    /// Attempts to fetch an user by login details.
    pub async fn login_user(
        username: &str,
        password: &str,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, HttpResponse> {
        let user = match User::by_username(username, conn_pool).await {
            Ok(user) => user,
            Err(err) => match err {
                Error::NotFound => return Err(HttpResponse::NotFound().json(
                    String::from("User doesn't exist.")
                )),
                _ => return Err(HttpResponse::InternalServerError().finish()),
            },
        };

        let pwd_hash = match &user.password_hash {
            Some(hash) => hash,
            None => return Err(HttpResponse::InternalServerError().finish()), // Illegal state
        };

        match validate_password_pbkdf2(&pwd_hash, password) {
            true => Ok(user),
            false => Err(HttpResponse::Unauthorized().json(
                String::from("Invalid password.")
            )),
        }
    }

    /// Database methods of `User` struct.
    impl User {
        /// Get user by username from the database.
        pub async fn by_username(
            username: &str,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<User, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let user = users::table
                        .filter(users::username.eq(username))
                        .first::<User>(conn)
                        .await?;
            
                        Ok(user)
                    }.scope_boxed())
                    .await
                },

                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }

    /// Database methods of `UserModel` struct.
    impl UserModel<'_> {
        /// Inserts `UserModel` into the database and returns the resulting `User`.
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<User, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(users::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let user = users::table
                        .find(last_insert_id())
                        .first::<User>(conn)
                        .await?;
                
                        Ok(user)
                    }.scope_boxed())
                    .await
                },
    
                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }

        /// Update user by id with the contents of `UserModel`.
        pub async fn update_by_id(
            &self,
            user_id: u32,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<usize, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let res = diesel::update(users::table.find(user_id))
                        .set(self)
                        .execute(conn)
                        .await?;
                
                        Ok(res)
                    }.scope_boxed())
                    .await
                },
    
                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }

    /// Database methods of `Session` struct.
    impl Session {
        /// Get session by id from the database.
        pub async fn by_id(
            id: u32,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Session, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let session = sessions::table
                        .find(id)
                        .first::<Session>(conn)
                        .await?;
                
                        Ok(session)
                    }.scope_boxed())
                    .await
                },

                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }

    /// Database methods of `SessionModel` struct.
    impl SessionModel<'_> {
        /// Inserts `SessionModel` into the database and returns the resulting `Session`.
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Session, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(sessions::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let session = sessions::table
                        .find(last_insert_id())
                        .first::<Session>(conn)
                        .await?;
                
                        Ok(session)
                    }.scope_boxed())
                    .await
                },
    
                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }

        /// Update user session by id with the contents of `SessionModel`.
        pub async fn update_by_id(
            &self,
            session_id: u32,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<usize, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let res = diesel::update(sessions::table.find(session_id))
                        .set(self)
                        .execute(conn)
                        .await?;
                
                        Ok(res)
                    }.scope_boxed())
                    .await
                },
    
                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }

    sql_function!(fn last_insert_id() -> Unsigned<Integer>);
}

/// Models of user services.
/// 
/// 
pub mod models {
    use diesel::prelude::*;
    use chrono::NaiveDateTime;
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use validator::Validate;
    use crate::schema::{sessions, users};

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = users)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct User {
        pub id: u32,
        pub access_level: u8,
        pub username: Option<String>,
        pub email: Option<String>,
        pub password_hash: Option<String>,
        pub created_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = users)]
    pub struct UserModel<'a> {
        pub access_level: u8,
        pub username: Option<&'a str>,
        pub email: Option<&'a str>,
        pub password_hash: Option<&'a str>,
    }

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = sessions)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Session {
        pub id: u32,
        pub user_id: u32,
        pub created_at: NaiveDateTime,
        pub ended_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = sessions)]
    pub struct SessionModel<'a> {
        pub user_id: u32,
        pub ended_at: &'a NaiveDateTime,
    }

    /// User access level table.
    #[derive(Copy, Clone)]
    pub enum AccessLevel {
        Anonymous = 10,
        Registered = 20,
        PendingMember = 30,
        Member = 40,
        Moderator = 50,
        Admin = 100,
        Owner = 200,
        Root = 255,
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

    #[derive(Debug)]
    pub struct UserInfo {
        pub user_id: u32,
        pub session_id: u32,
        pub access_level: u8,
    }

    /// JSON body accepted by `POST /sessions` method.
    #[derive(Debug, Validate, Deserialize)]
    pub struct CreateSessionInput {
        /// Expiration time as UTC timestamp.
        pub exp: Option<i64>,
        #[validate(length(
            min = "5",
            max = "128",
            message = "Password must be 5-128 characters long."
        ))]
        pub password: String,
        #[validate(
            length(
                min = "1",
                max = "16",
                message = "Username must be 1-16 characters long."
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9.-]+$").unwrap(),
                message = "Username contains illegal characters."
            )
        )]
        pub username: String,
    }

    /// JSON response body from `POST /sessions` and `PUT /sessions` methods.
    #[derive(Debug, Serialize)]
    pub struct CreateSessionOutput {
        pub access_token: String,
    }

    /// JSON body accepted by `POST /sessions` method.
    #[derive(Debug, Validate, Deserialize)]
    pub struct RegisterUserInput {
        #[validate(length(
            min = "5",
            max = "128",
            message = "Password must be 5-128 characters long."
        ))]
        pub password: String,
        #[validate(
            length(
                min = "1",
                max = "16",
                message = "Username must be 1-16 characters long."
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9.-]+$").unwrap(),
                message = "Username contains illegal characters."
            )
        )]
        pub username: String,
        #[validate(
            length(
                min = "1",
                max = "128",
                message = "Email must be 1-128 characters long."
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap(),
                message = "Email address is invalid."
            )
        )]
        pub email: String,
    }
}

/// User authentication.
/// 
/// 
pub mod authentication {
    use std::env;

    use actix_web::{http::header, HttpRequest, HttpResponse};
    use chrono::Utc;
    use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use password_hash::{Output, PasswordHash, PasswordVerifier, Salt, SaltString};
    use pbkdf2::{pbkdf2_hmac, Algorithm, Params, Pbkdf2};
    use rand_core::{OsRng, RngCore};
    use regex::Regex;
    use sha2::{Digest, Sha256};

    use super::models::{Claims, Session, UserInfo};


    /// Authenticates user and extracts `UserInfo` from the http-request.
    /// `HttpResponseBuilder` is returned for invalid user sessions. 
    pub async fn authenticate_user(
        conn_pool: &Pool<AsyncMysqlConnection>,
        req: HttpRequest,
    ) -> Result<UserInfo, HttpResponse> {
        // Read authorization header for an access token.
        let token = match req.headers().get(header::AUTHORIZATION) {
            Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return Err(HttpResponse::UnprocessableEntity().finish()),
            },
            None => return Err(HttpResponse::Unauthorized().finish()),
        };

        // Decode & validate access token.
        let claims = match validate_claims(token) {
            Some(claims) => claims,
            None => return Err(HttpResponse::Unauthorized().finish()),
        };

        // Check if session has ended.
        let session = match Session::by_id(claims.sub, &conn_pool).await {
            Ok(session) => session,
            Err(_) => return Err(HttpResponse::InternalServerError().finish()),
        };

        if session.ended_at.and_utc().timestamp() <= Utc::now().timestamp() {
            return Err(HttpResponse::Unauthorized().finish());
        }

        // TODO: check for bans

        Ok(UserInfo {
            user_id: session.user_id,
            session_id: session.id,
            access_level: claims.role,
        })
    }

    /// Creates JSON web tokens for user authentication.
    /// 
    /// # Panics
    /// 
    /// Panics if the private key is not found in the environment.
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

    /// JSON web token authentication.
    /// 
    /// Returns decoded `Claims` if provided token is valid.
    pub fn validate_claims(
        token: &str,
    ) -> Option<Claims> {
        let jwt_secret = env::var("JWT_SECRET")
        .expect(".env variable `JWT_SECRET` must be set");

        let bearer_scheme = Regex::new(r"Bearer (?<token>\w.+)").unwrap();

        bearer_scheme.captures(token)
        .and_then(|capture| {
            decode::<Claims>(
                &capture["token"], 
                &DecodingKey::from_secret(jwt_secret.as_ref()), 
                &Validation::default(),
            )
            .ok()
            .map(|data| data.claims)
        })
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