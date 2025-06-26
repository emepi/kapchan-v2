use diesel::{
    prelude::*, 
    result::Error, 
    sql_function, 
    ExpressionMethods, 
    QueryDsl, 
    SelectableHelper
};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};
use serde::{Deserialize, Serialize};

use crate::schema::boards;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = boards)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Board {
    pub id: u32,
    pub handle: String,
    pub title: String,
    pub description: String,
    pub access_level: u8,
    pub active_threads_limit: u32,
    pub thread_size_limit: u32,
    pub captcha: bool,
    pub nsfw: bool,
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
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = boards)]
pub struct BoardModel<'a> {
    pub handle: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub access_level: u8,
    pub active_threads_limit: u32,
    pub thread_size_limit: u32,
    pub captcha: bool,
    pub nsfw: bool,
}

//WARNING: DEPRACATED
#[derive(Debug, Serialize, Deserialize)]
pub struct BoardSimple {
    pub handle: String,
    pub title: String,
    pub access_level: u8,
    pub nsfw: bool,
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);