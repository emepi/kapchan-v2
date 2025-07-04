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

use crate::schema::captchas;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = captchas)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Captcha {
    pub id: u64,
    pub answer: String,
    pub expires: NaiveDateTime,
}

impl Captcha {
    pub async fn by_id(
        id: u64,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Captcha, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let captcha = captchas::table
                    .find(id)
                    .first::<Captcha>(conn)
                    .await?;
        
                    Ok(captcha)
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn delete_by_id(
        id: u64,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<(), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    diesel::delete(
                        captchas::table
                        .find(id)
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
#[diesel(table_name = captchas)]
pub struct CaptchaModel<'a> {
    pub answer: &'a str,
    pub expires: &'a NaiveDateTime,
}

impl CaptchaModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Captcha, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(captchas::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let captcha = captchas::table
                    .find(last_insert_id())
                    .first::<Captcha>(conn)
                    .await?;
            
                    Ok(captcha)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<BigInt>);