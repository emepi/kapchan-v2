use chrono::NaiveDateTime;
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

use crate::schema::bans::{self, expires_at};


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Clone)]
#[diesel(table_name = bans)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Ban {
    pub id: u32,
    pub moderator_id: u64,
    pub user_id: Option<u64>,
    pub post_id: Option<u32>,
    pub reason: Option<String>,
    pub ip_address: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl Ban {
    pub async fn get_last_ban(
        conn_pool: &Pool<AsyncMysqlConnection>,
        user_id: u64,
        ip_address: String,
    ) -> Result<Option<Ban>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                
                    let mut ban = bans::table
                    .filter(bans::user_id.eq(user_id).or(bans::ip_address.eq(ip_address)))
                    .order(expires_at.desc())
                    .limit(1)
                    .load::<Ban>(conn)
                    .await?;

                    let ban = if ban.is_empty() {
                        None
                    } else {
                        Some(ban.remove(0))
                    };
            
                    Ok(ban)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = bans)]
pub struct BanModel<'a> {
    pub moderator_id: u64,
    pub user_id: Option<u64>,
    pub post_id: Option<u32>,
    pub reason: Option<&'a str>,
    pub ip_address: &'a str,
    pub expires_at: NaiveDateTime,
}

impl BanModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Ban, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(bans::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let ban = bans::table
                    .find(last_insert_id())
                    .first::<Ban>(conn)
                    .await?;
            
                    Ok(ban)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);