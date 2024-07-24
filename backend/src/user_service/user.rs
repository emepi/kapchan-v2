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
use crate::schema::users;

use super::authentication::AccessLevel;


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

impl User {
    /// Modifies existing user with the `UserModel` and returns the `User`.
    pub async fn modify_by_id(
        id: u32,
        model: UserModel<'_>, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, diesel::result::Error> {

        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::update(users::table.find(id))
                    .set(model)
                    .execute(conn)
                    .await;
        
                    let user = users::table
                    .find(id)
                    .first::<User>(conn)
                    .await?;
            
                    Ok(user)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }

    /// Fetches `User` by username from the database.
    pub async fn by_username(
        username: &str, 
        conn_pool: &Pool<AsyncMysqlConnection>
    ) -> Result<User, diesel::result::Error> {
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

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }

    /// Fetches `User` by email from the database.
    pub async fn by_email(
        email: &str, 
        conn_pool: &Pool<AsyncMysqlConnection>
    ) -> Result<User, diesel::result::Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let user = users::table
                    .filter(users::email.eq(email))
                    .first::<User>(conn)
                    .await?;
        
                    Ok(user)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }

    /// Fetches `User` by id from the database.
    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, diesel::result::Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let user = users::table
                    .find(id)
                    .first::<User>(conn)
                    .await?;
        
                    Ok(user)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

/// Model for inserting a new user into the database.
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserModel<'a> {
    pub access_level: u8,
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
}

impl UserModel<'_> {
    /// Anonymous user constructor.
    pub fn anon_user() -> Self {
        UserModel {
            access_level: AccessLevel::Anonymous as u8,
            username: None,
            email: None,
            password_hash: None,
        }
    }

    /// Inserts `UserModel` into the database and returns the resulting `User`.
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, diesel::result::Error> {
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

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}


sql_function!(fn last_insert_id() -> Unsigned<Integer>);