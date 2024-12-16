use diesel::{result::Error, sql_function, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::boards::{Board, BoardModel, BoardSimple}, schema::boards::{self, access_level, handle, nsfw, title}};


impl BoardModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Board, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(boards::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let application = boards::table
                    .find(last_insert_id())
                    .first::<Board>(conn)
                    .await?;
            
                    Ok(application)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

impl Board {
    pub async fn list_all(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Vec<Board>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let boards: Vec<Board> = boards::table
                    .select(Board::as_select())
                    .load(conn)
                    .await?;
            
                    Ok(boards)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn list_all_simple(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Vec<BoardSimple>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let boards: Vec<(String, String, u8, bool)> = boards::table
                    .select((handle, title, access_level, nsfw))
                    .load::<(String, String, u8, bool)>(conn)
                    .await?;

                    let boards = boards.into_iter()
                    .map(|board| BoardSimple {
                        handle: board.0,
                        title: board.1,
                        access_level: board.2,
                        nsfw: board.3,
                    })
                    .collect();
            
                    Ok(boards)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);