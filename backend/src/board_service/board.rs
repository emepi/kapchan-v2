use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, 
    scoped_futures::ScopedFutureExt
};
use serde::Serialize;

use crate::schema::{boards, board_flags};


#[derive(Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = boards)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Board {
    pub id: u32,
    pub handle: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub created_by: u32,
}


#[derive(Insertable)]
#[diesel(table_name = boards)]
pub struct BoardModel<'a> {
    pub handle: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub created_by: u32,
}

impl BoardModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<Board> {

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
                .ok()
            },

            Err(_) => None,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = board_flags)]
pub struct BoardFlagModel {
    pub board_id: u32,
    pub flag: u8,
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);