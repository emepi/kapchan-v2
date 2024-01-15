use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, 
    scoped_futures::ScopedFutureExt
};
use serde::Serialize;

use crate::schema::{boards, board_groups};


#[derive(Queryable, Identifiable, Associations, Selectable, Serialize)]
#[diesel(belongs_to(BoardGroup))]
#[diesel(table_name = boards)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Board {
    pub id: u32,
    pub board_group_id: u32,
    pub handle: String,
    pub title: String,
    pub description: Option<String>,
}


#[derive(Insertable)]
#[diesel(table_name = boards)]
pub struct BoardModel<'a> {
    pub board_group_id: u32,
    pub handle: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
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

#[derive(Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = board_groups)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct BoardGroup {
    pub id: u32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = board_groups)]
pub struct BoardGroupModel<'a> {
    pub name: &'a str,
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);