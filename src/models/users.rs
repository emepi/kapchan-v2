use chrono::NaiveDateTime;
use diesel::{
    prelude::*, 
    result::Error, 
    sql_function, 
    ExpressionMethods, 
    QueryDsl
};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};
use serde::Serialize;

use crate::schema::{bans::{self, user_id}, users};

use super::bans::Ban;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u64,
    pub access_level: u8,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: NaiveDateTime,
}

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
        target_user_id: u64,
        access_lvl: u8,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<usize, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let res = diesel::update(users::table.find(target_user_id))
                    .set(users::access_level.eq(access_lvl))
                    .execute(conn)
                    .await?;
            
                    Ok(res)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn update_user(
        target_user_id: u64,
        access_lvl: u8,
        username: Option<String>,
        email: Option<String>,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<usize, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let res = diesel::update(users::table.find(target_user_id))
                    .set(users::access_level.eq(access_lvl))
                    .execute(conn)
                    .await?;

                    if let Some(username) = username {
                        let _ = diesel::update(users::table.find(target_user_id))
                        .set(users::username.eq(username))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(email) = email {
                        let _ = diesel::update(users::table.find(target_user_id))
                        .set(users::email.eq(email))
                        .execute(conn)
                        .await?;
                    }
            
                    Ok(res)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn count_users(
        conn_pool: &Pool<AsyncMysqlConnection>,
        target_username: Option<String>,
        min_access_level: u8,
    ) -> Result<i64, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let mut query = users::table.into_boxed()
                    .filter(users::access_level.ge(min_access_level));

                    if let Some(name) = target_username {
                        let pattern = format!("%{}%", name);
                        query = query.filter(users::username.like(pattern));
                    }

                    let count = query
                    .count()
                    .get_result(conn)
                    .await?;
            
                    Ok(count)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn load_users_data(
        conn_pool: &Pool<AsyncMysqlConnection>,
        target_username: Option<String>,
        min_access_level: u8,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<User>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let mut query = users::table.into_boxed()
                    .filter(users::access_level.ge(min_access_level));

                    if let Some(name) = target_username {
                        let pattern = format!("%{}%", name);
                        query = query.filter(users::username.like(pattern));
                    }

                    let users = query
                    .limit(page_size)
                    .offset(offset)
                    .select(User::as_select())
                    .load::<User>(conn)
                    .await?;

                    Ok(users)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserModel<'a> {
    pub access_level: u8,
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
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
        other_user_id: u64,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<usize, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let res = diesel::update(users::table.find(other_user_id))
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

#[derive(Debug, Copy, Clone)]
pub enum AccessLevel {
    Anonymous = 10,
    Registered = 20,
    PendingMember = 30,
    Member = 40,
    Moderator = 90,
    Admin = 100,
    Owner = 200,
    Root = 255,
}

#[derive(Debug)]
pub struct UserData {
    pub id: u64,
    pub access_level: u8,
    pub ip_addr: String,
    pub user_agent: String,
    pub banned: Option<Ban>,
}

sql_function!(fn last_insert_id() -> Unsigned<BigInt>);