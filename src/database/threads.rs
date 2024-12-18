use std::os::windows::thread;

use chrono::NaiveDateTime;
use diesel::{result::Error, sql_function, BelongingToDsl, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::{posts::{Post, PostModel, PostOutput}, threads::{Thread, ThreadCatalogOutput, ThreadInput, ThreadModel}}, schema::{posts, threads}};


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

    pub async fn list_threads_by_board_catalog(
        conn_pool: &Pool<AsyncMysqlConnection>,
        board_id: u32,
    ) -> Result<Vec<ThreadCatalogOutput>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let threads = threads::table
                    .filter(threads::board_id.eq(board_id))
                    .filter(threads::archived.eq(false))
                    .order((threads::pinned.eq(true), threads::bump_time.desc()))
                    .load::<Thread>(conn)
                    .await?;

                    let thread_posts = Post::belonging_to(&threads)
                    .order_by(posts::id)
                    .select((
                        posts::thread_id,
                        posts::id,
                        posts::show_username,
                        posts::message,
                        posts::country_code,
                        posts::hidden
                    ))
                    .load::<(u32, u32, bool, String, Option<String>, bool)>(conn)
                    .await?;

                    //TODO: improve
                    let mut grouped_posts: Vec<Vec<(u32, u32, bool, String, Option<String>, bool)>> = Vec::with_capacity(threads.len());

                    for _ in 0..threads.len() {
                        grouped_posts.push(vec![]);
                    }

                    for (i, thread) in threads.iter().enumerate() {
                        for (j, post) in thread_posts.iter().enumerate() {
                            if post.0 == thread.id {
                                grouped_posts[i].push(thread_posts[j].clone());
                            }
                        } 
                    }

                    let catalog = threads.into_iter()
                    .zip(grouped_posts)
                    .map(|(thread, posts)| {
                        ThreadCatalogOutput {
                            title: thread.title,
                            pinned: thread.pinned,
                            op_post: PostOutput {
                                id: posts[0].1,
                                show_username: posts[0].2,
                                message: posts[0].3.clone(),
                                country_code: posts[0].4.clone(),
                                hidden: posts[0].5,
                            },
                            replies: posts.len() - 1,
                        }
                    })
                    .collect::<Vec<ThreadCatalogOutput>>();
                      
                    Ok(catalog)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);