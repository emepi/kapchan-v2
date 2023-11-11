use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    AsyncConnection,
    scoped_futures::ScopedFutureExt,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    RunQueryDsl,
};
use crate::schema::{users, sessions};


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
    pub async fn create_session(
        &self,
        _access_level: Option<u8>,
        _mode: Option<u8>,
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