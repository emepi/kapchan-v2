use diesel::{result::Error, sql_function, ExpressionMethods, QueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::users::{User, UserModel}, schema::users::{self, access_level}};


impl User {
    pub async fn by_id(
        id: u64,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let user = users::table
                    .find(id)
                    .first::<User>(conn)
                    .await?;
        
                    Ok(user)
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn by_username(
        username: &str,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let user = users::table
                    .filter(users::username.eq(username))
                    .first::<User>(conn)
                    .await?;
        
                    Ok(user)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn by_email(
        email: &str,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let user = users::table
                    .filter(users::email.eq(email))
                    .first::<User>(conn)
                    .await?;
        
                    Ok(user)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn update_access_level(
        user_id: u64,
        access_lvl: u8,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<usize, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let res = diesel::update(users::table.find(user_id))
                    .set(access_level.eq(access_lvl))
                    .execute(conn)
                    .await?;
            
                    Ok(res)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

impl UserModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(users::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let user = users::table
                    .find(last_insert_id())
                    .first::<User>(conn)
                    .await?;
            
                    Ok(user)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn update_by_id(
        &self,
        user_id: u64,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<usize, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let res = diesel::update(users::table.find(user_id))
                    .set(self)
                    .execute(conn)
                    .await?;
            
                    Ok(res)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<BigInt>);