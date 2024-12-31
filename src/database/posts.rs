use diesel::{result::Error, sql_function, BelongingToDsl, ExpressionMethods, GroupedBy, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::{boards::Board, posts::{Attachment, AttachmentModel, Post, PostInput, PostModel, PostOutput, PostPreview, ReplyModel}, threads::Thread}, schema::{attachments, boards, posts, replies, threads}};


impl Post {
    pub async fn insert_post_by_post_id(
        post_id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
        input: PostInput,
    ) -> Result<Post, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let post = posts::table
                    .find(post_id)
                    .first::<Post>(conn)
                    .await?;

                    let _ = diesel::insert_into(posts::table)
                    .values(PostModel {
                        user_id: input.user_id,
                        thread_id: post.thread_id,
                        show_username: input.show_username,
                        message: &input.message,
                        message_hash: &input.message_hash,
                        country_code: input.country_code.as_deref(),
                        access_level: input.access_level,
                        sage: input.sage,
                        mod_note: input.mod_note.as_deref(),
                    })
                    .execute(conn)
                    .await?;

                    let new_post = posts::table
                    .find(last_insert_id())
                    .first::<Post>(conn)
                    .await?;

                    let replies: Vec<ReplyModel> = input.reply_ids
                    .iter()
                    .map(|reply_id| ReplyModel {
                        post_id: *reply_id,
                        reply_id: new_post.id,
                    })
                    .collect();

                    let _ = diesel::insert_into(replies::table)
                    .values(replies)
                    .execute(conn)
                    .await?;
                
                    Ok(new_post)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn latest_posts_preview(
        conn_pool: &Pool<AsyncMysqlConnection>,
        access_level: u8,
        limit: i64,
    ) -> Result<Vec<PostPreview>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let posts: Vec<(Post, (Thread, Board))> = posts::table
                    .filter(posts::access_level.le(access_level))
                    .order(posts::created_at.desc())
                    .limit(limit)
                    .inner_join(
                        threads::table
                        .inner_join(boards::table)
                    )
                    .load::<(Post, (Thread, Board))>(conn)
                    .await?;

                    let posts: Vec<PostPreview> = posts.into_iter()
                    .map(|post| PostPreview {
                        post_id: post.0.id,
                        thread_id: post.1.0.id,
                        board_handle: post.1.1.handle,
                        board_name: post.1.1.title,
                        message: post.0.message,
                    })
                    .collect();

                    Ok(posts)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);