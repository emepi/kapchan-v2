pub mod database {
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::threads;

    use super::models::{Thread, ThreadModel};


    impl ThreadModel<'_> {
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Thread, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(threads::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let thread = threads::table
                        .find(last_insert_id())
                        .first::<Thread>(conn)
                        .await?;
                
                        Ok(thread)
                    }.scope_boxed())
                    .await
                },

                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }


    sql_function!(fn last_insert_id() -> Unsigned<Integer>);
}

pub mod models {
    use chrono::NaiveDateTime;
    use diesel::prelude::*;
    use serde::Serialize;

    use crate::schema::threads;


    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = threads)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Thread {
        pub id: u32,
        pub board_id: u32,
        pub title: String,
        pub pinned: bool,
        pub bump_time: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = threads)]
    pub struct ThreadModel<'a> {
        pub board_id: u32,
        pub title: &'a str,
        pub pinned: bool,
    }
}