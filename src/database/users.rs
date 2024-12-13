use diesel::{result::Error, sql_function, QueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::users::{AccessLevel, User, UserModel}, schema::users};


pub async fn user_by_id(
    id: u32,
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

pub async fn create_anonymous_user(
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Result<User, Error> {
    let anon_user = UserModel {
        access_level: AccessLevel::Anonymous as u8,
        username: None,
        email: None,
        password_hash: None,
    };

    match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let _ = diesel::insert_into(users::table)
                .values(anon_user)
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

sql_function!(fn last_insert_id() -> Unsigned<Integer>);