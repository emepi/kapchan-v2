use std::os::windows::thread;

use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};
use serde::Serialize;

use crate::schema::{files, posts, threads};


#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Post {
    pub id: u32,
    pub op_id: Option<u32>,
    pub body: String,
    pub access_level: u8,
    pub created_at: NaiveDateTime,
}

/// Model for inserting a new post into the database.
#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct PostModel {
    pub op_id: Option<u32>,
    pub body: String,
    pub access_level: u8,
}

impl PostModel {
    /// Inserts `PostModel` into the database and returns the resulting 
    /// `Post`.
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Post, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(posts::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let post = posts::table
                    .find(last_insert_id())
                    .first::<Post>(conn)
                    .await?;
            
                    Ok(post)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = threads)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Thread {
    pub id: u32,
    pub board: u32,
    pub title: String,
    pub pinned: bool,
    pub bump_date: NaiveDateTime,
}

/// Model for inserting a new thread into the database.
#[derive(Insertable)]
#[diesel(table_name = threads)]
pub struct ThreadModel {
    pub id: u32,
    pub board: u32,
    pub title: String,
    pub pinned: bool,
}

impl ThreadModel {
    /// Inserts `PostModel` into the database and returns the resulting 
    /// `Post`.
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Thread, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(threads::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let thread = threads::table
                    .find(last_insert_id())
                    .first::<Thread>(conn)
                    .await?;
            
                    Ok(thread)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct File {
    pub id: u32,
    pub file_name: String,
    pub thumbnail: String,
    pub file_path: String,
}

/// Model for inserting a new file into the database.
#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct FileModel {
    pub id: u32,
    pub file_name: String,
    pub thumbnail: String,
    pub file_path: String,
}

impl FileModel {
    /// Inserts `FileModel` into the database and returns the resulting 
    /// `File`.
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<File, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(files::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let file = files::table
                    .find(last_insert_id())
                    .first::<File>(conn)
                    .await?;
            
                    Ok(file)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);