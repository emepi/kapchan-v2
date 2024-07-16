pub mod database {
    use diesel::{result::Error, sql_function, QueryDsl};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::{sessions, users};

    use super::models::{NewSession, NewUser, Session, User};

    /// Inserts `NewUser` into the database and returns the resulting `User`.
    pub async fn create_user(
        user: NewUser<'_>, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(users::table)
                    .values(user)
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

            // Failed to get a connection from the pool.
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    /// Inserts `NewSession` into the database and returns the resulting 
    /// `Session`.
    pub async fn create_session(
        session: NewSession<'_>, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Session, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(sessions::table)
                    .values(session)
                    .execute(conn)
                    .await?;
                
                    let session = sessions::table
                    .find(last_insert_id())
                    .first::<Session>(conn)
                    .await?;
            
                    Ok(session)
                }.scope_boxed())
                .await
            },

            // Failed to get a connection from the pool.
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    sql_function!(fn last_insert_id() -> Unsigned<Integer>);
}

pub mod models {
    use diesel::prelude::*;
    use chrono::NaiveDateTime;
    use serde::Serialize;
    use crate::schema::{sessions, users};

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = users)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct User {
        pub id: u32,
        pub access_level: u8,
        pub username: Option<String>,
        pub password_hash: Option<String>,
        pub created_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = users)]
    pub struct NewUser<'a> {
        pub access_level: u8,
        pub username: Option<&'a str>,
        pub password_hash: Option<&'a str>,
    }

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = sessions)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Session {
        pub id: u32,
        pub user_id: u32,
        pub ip_address: String,
        pub user_agent: String,
        pub created_at: NaiveDateTime,
        pub ended_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = sessions)]
    pub struct NewSession<'a> {
        pub user_id: u32,
        pub ip_address: &'a str,
        pub user_agent: &'a str,
        pub ended_at: &'a NaiveDateTime,
    }
}