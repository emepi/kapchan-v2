use chrono::NaiveDateTime;
use diesel::{result::Error, sql_function, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::{posts::{Post, PostModel}, threads::{Thread, ThreadInput, ThreadModel}}, schema::{posts, threads}};


impl Thread {
    pub async fn insert_thread(
        conn_pool: &Pool<AsyncMysqlConnection>,
        input: ThreadInput,
    ) -> Result<(), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(threads::table)
                    .values(ThreadModel {
                        board_id: input.board_id,
                        title: &input.title,
                        pinned: input.pinned,
                        archived: input.archived,
                    })
                    .execute(conn)
                    .await?;
                
                    let thread = threads::table
                    .find(last_insert_id())
                    .first::<Thread>(conn)
                    .await?;

                    let _ = diesel::insert_into(posts::table)
                    .values(PostModel {
                        user_id: input.post.user_id,
                        thread_id: thread.id,
                        show_username: input.post.show_username,
                        message: &input.post.message,
                        message_hash: &input.post.message_hash,
                        ip_address: &input.post.ip_address,
                        user_agent: &input.post.user_agent,
                        country_code: input.post.country_code.as_deref(),
                        hidden: input.post.hidden,
                    })
                    .execute(conn)
                    .await?;

                    let post = posts::table
                    .find(last_insert_id())
                    .first::<Post>(conn)
                    .await?;
            
                    Ok(())
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);