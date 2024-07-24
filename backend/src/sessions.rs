pub mod routes {
    
}

pub mod database {
    use diesel::{result::Error, sql_function, QueryDsl};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::sessions;

    use super::models::{SessionModel, Session};

    /// Inserts `NewSession` into the database and returns the resulting 
    /// `Session`.
    pub async fn insert_session(
        session: SessionModel<'_>, 
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
    use chrono::NaiveDateTime;
    use diesel::prelude::*;
    use serde::Serialize;

    use crate::schema::sessions;
    
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
    pub struct SessionModel<'a> {
        pub user_id: u32,
        pub ip_address: &'a str,
        pub user_agent: &'a str,
        pub ended_at: &'a NaiveDateTime,
    }
}