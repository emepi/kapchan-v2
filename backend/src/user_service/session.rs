use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection,
    RunQueryDsl,
};
use serde::Serialize;

use crate::schema::sessions;


/// Database model for a session.
#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Session {
    pub id: u32,
    pub user_id: u32,
    pub created_at: NaiveDateTime,
    pub ended_at: NaiveDateTime,
}

impl Session {
    /// Fetches `Session` by id from the database.
    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Session, diesel::result::Error> {
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

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

/// Database insertion model for a session.
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = sessions)]
pub struct SessionModel<'a> {
    pub user_id: u32,
    pub ended_at: &'a NaiveDateTime,
}

impl SessionModel<'_> {
    /// Inserts `SessionModel` into the database and returns the resulting 
    /// `Session`.
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Session, diesel::result::Error> {
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

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

/// Database update model for a session.
#[derive(AsChangeset)]
#[diesel(table_name = sessions)]
pub struct SessionModificationModel<'a> {
    pub user_id: Option<u32>,
    pub ended_at: Option<&'a NaiveDateTime>,
}

impl SessionModificationModel<'_> {
    /// Updates existing session by id and returns the resulting `Session`.
    pub async fn modify(
        &self,
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Session, diesel::result::Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::update(sessions::table.find(id))
                    .set(self)
                    .execute(conn)
                    .await?;

                    let updated_session = sessions::table
                    .find(id)
                    .first::<Session>(conn)
                    .await?;
            
                    Ok(updated_session)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);