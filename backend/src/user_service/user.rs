use std::env;

use actix_web::cookie::{Cookie, SameSite, self};
use chrono::{NaiveDateTime, Utc, Duration};
use diesel::{prelude::*, result::Error};
use diesel_async::{
    AsyncConnection,
    scoped_futures::ScopedFutureExt,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    RunQueryDsl,
};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::schema::{users, sessions};

use super::authentication::Claims;


#[derive(Queryable, Selectable)]
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

impl User {
    pub fn create_authentication_token(&self) -> Option<Cookie> {
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
            sub: self.id.to_string(),
        };

        encode(
            &Header::default(), 
            &user_claims, 
            &EncodingKey::from_secret(jwt_secret.as_ref())
        )
        .map(|access_token| {
            Cookie::build("access_token", access_token)
            .max_age(cookie::time::Duration::new(jwt_expiration * 60, 0))
            .same_site(SameSite::Strict)
            .http_only(true)
            .finish()
        }).ok()
    }

    pub async fn create_session(
        &self,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<UserSession> {

        let session_model = UserSessionModel {
            user_id: self.id,
            access_level: self.access_level,
            mode: 1,
            ip_address: ip_address,
            user_agent: user_agent,
            ended_at: None,
        };

        let mut connection = db.get().await
        .ok()?;
        
        let result = connection.transaction::<_, Error, _>(|conn| async move {
            let _ = diesel::insert_into(sessions::table)
            .values(session_model)
            .execute(conn)
            .await?;

            let user_session = sessions::table
            .find(last_insert_id())
            .first::<UserSession>(conn)
            .await?;
            
            Ok(user_session)
        }.scope_boxed())
        .await
        .ok();

        result
    }
}

/// Model for inserting a new user into the database.
#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct UserModel<'a> {
    pub access_level: u8,
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
}

impl Default for UserModel<'_> {
    fn default() -> Self {
        Self {
            access_level: 1, 
            username: None,
            email: None,
            password_hash: None,
        }
    }
}

impl UserModel<'_> {
    pub async fn register_user(
        &self, 
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<User> {
        // TODO: inspect connection pooling errors
        let mut connection = db.get().await
        .ok()?;

        let result = connection.transaction::<_, Error, _>(|conn| async move {
            
            // TODO: inspect race conditions & error types
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
        .ok();

        result
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct UserSession {
    pub id: u32,
    pub user_id: u32,
    pub access_level: u8,
    pub mode: u8,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: NaiveDateTime,
    pub ended_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
pub struct UserSessionModel<'a> {
    pub user_id: u32,
    pub access_level: u8,
    pub mode: u8,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub ended_at: Option<&'a NaiveDateTime>,
}

// TODO: move to a different module
sql_function!(fn last_insert_id() -> Unsigned<Integer>);