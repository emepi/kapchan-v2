use diesel::{prelude::*, result::Error};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection,
    RunQueryDsl,
};
use serde::Serialize;

use crate::schema::boards;


#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = boards)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Board {
    pub id: u32,
    pub handle: String,
    pub title: String,
    pub access_level: u8,
    pub bump_limit: u32,
    pub nsfw: bool,
}

impl Board {
    /// Returns all the boards from the database.
    pub async fn fetch_boards(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Vec<Board>, diesel::result::Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let boards = boards::table
                    .select(Board::as_select())
                    .load(conn)
                    .await?;
            
                    Ok(boards)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

/// Model for inserting a new board into the database.
#[derive(Insertable)]
#[diesel(table_name = boards)]
pub struct BoardModel<'a> {
    pub handle: &'a str,
    pub title: &'a str,
    pub access_level: u8,
    pub bump_limit: u32,
    pub nsfw: bool,
}

impl BoardModel<'_> {
    /// Inserts `BoardModel` into the database and returns the resulting 
    /// `Board`.
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Board, diesel::result::Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(boards::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let board = boards::table
                    .find(last_insert_id())
                    .first::<Board>(conn)
                    .await?;
            
                    Ok(board)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);