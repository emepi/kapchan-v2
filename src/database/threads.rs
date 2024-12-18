use std::os::windows::thread;

use chrono::NaiveDateTime;
use diesel::{result::Error, sql_function, BelongingToDsl, ExpressionMethods, GroupedBy, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::{posts::{Attachment, Post, PostModel, PostOutput}, threads::{Thread, ThreadCatalogOutput, ThreadInput, ThreadModel}}, schema::{attachments, posts, threads}};


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

                    let thread_posts: Vec<(Post, Option<Attachment>)> = Post::belonging_to(&threads)
                    .left_join(attachments::table)
                    .order_by(posts::id)
                    .select((
                        Post::as_select(),
                        Option::<Attachment>::as_select()
                    ))
                    .load::<(Post, Option<Attachment>)>(conn)
                    .await?;

                    let catalog = thread_posts
                    .grouped_by(&threads)
                    .into_iter()
                    .zip(threads)
                    .map(|(posts, thread)| {
                        ThreadCatalogOutput {
                            title: thread.title,
                            pinned: thread.pinned,
                            op_post: PostOutput {
                                id: posts[0].0.id,
                                show_username: posts[0].0.show_username,
                                message: posts[0].0.message.clone(),
                                country_code: posts[0].0.country_code.clone(),
                                hidden: posts[0].0.hidden,
                            },
                            replies: posts.len() - 1,
                        }
                    })
                    .collect();
                      
                    Ok(catalog)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);