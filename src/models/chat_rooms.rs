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
use serde::Serialize;

use crate::schema::chat_rooms;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Clone)]
#[diesel(table_name = chat_rooms)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ChatRoom {
    pub id: u32,
    pub name: String,
    pub access_level: u8,
}

impl ChatRoom {
    pub async fn list_all(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Vec<ChatRoom>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let chat_rooms: Vec<ChatRoom> = chat_rooms::table
                    .select(ChatRoom::as_select())
                    .load(conn)
                    .await?;
            
                    Ok(chat_rooms)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn delete_chat_room(
        conn_pool: &Pool<AsyncMysqlConnection>,
        id: u32,
    ) -> Result<(), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    diesel::delete(
                        chat_rooms::table.find(id)
                    )
                    .execute(conn)
                    .await?;

                    Ok(())
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = chat_rooms)]
pub struct ChatRoomModel<'a> {
    pub name: &'a str,
    pub access_level: u8,
}

impl ChatRoomModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<ChatRoom, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(chat_rooms::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let chat_room = chat_rooms::table
                    .find(last_insert_id())
                    .first::<ChatRoom>(conn)
                    .await?;
            
                    Ok(chat_room)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);