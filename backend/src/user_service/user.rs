use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    AsyncConnection,
    scoped_futures::ScopedFutureExt,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    RunQueryDsl,
};
use serde::Serialize;
use crate::schema::{users, sessions};

use super::session::{UserSession, UserSessionModel};


#[derive(Copy, Clone)]
pub enum AccessLevel {
    Anonymous = 10,
    Registered = 20,
    PendingMember = 30,
    Member = 50,
    Admin = 100,
    Owner = 200,
    Root = 255,
}


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u32,
    pub access_level: u8,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

impl User {

    pub async fn modify_by_id(
        id: u32,
        update_mdl: UserModel<'_>, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<()> {

        match conn_pool.get().await {

            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let _ = diesel::update(users::table.find(id))
                    .set(update_mdl)
                    .execute(conn)
                    .await;
            
                    Ok(())
                }.scope_boxed())
                .await
                .ok()
            },

            Err(_) => None,
        }
    }

    pub async fn create_session(
        &self,
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<UserSession> {

        let session_model = UserSessionModel {
            user_id: Some(self.id),
            access_level: self.access_level,
            mode: 1,
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

    pub async fn by_username(
        username: &str, 
        db: &Pool<AsyncMysqlConnection>
    ) -> Option<User> {
        let mut connection = db.get().await
        .ok()?;

        connection.transaction::<_, Error, _>(|conn| async move {
            let user = users::table
            .filter(users::username.eq(username))
            .first::<User>(conn)
            .await?;

            Ok(user)
        }.scope_boxed())
        .await
        .ok()
    }

    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<User> {
        let mut connection = conn_pool.get().await
        .ok()?;

        connection.transaction::<_, Error, _>(|conn| async move {
            let user = users::table
            .find(id)
            .first::<User>(conn)
            .await?;

            Ok(user)
        }.scope_boxed())
        .await
        .ok()
    }
}

/// Model for inserting a new user into the database.
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserModel<'a> {
    pub access_level: u8,
    pub username: &'a str,
    pub email: Option<&'a str>,
    pub password_hash: &'a str,
}

impl UserModel<'_> {
    pub async fn insert(
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


sql_function!(fn last_insert_id() -> Unsigned<Integer>);