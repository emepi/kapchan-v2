pub mod database {
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::posts;

    use super::models::{Post, PostModel};


    impl PostModel<'_> {
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Post, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(posts::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let post = posts::table
                        .find(last_insert_id())
                        .first::<Post>(conn)
                        .await?;
                
                        Ok(post)
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

    use crate::schema::posts;


    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = posts)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Post {
        pub id: u32,
        pub user_id: u32,
        pub thread_id: u32,
        pub access_level: u8,
        pub username: bool,
        pub message: String,
        pub created_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = posts)]
    pub struct PostModel<'a> {
        pub user_id: u32,
        pub thread_id: u32,
        pub access_level: u8,
        pub username: bool,
        pub message: &'a str,
    }
}