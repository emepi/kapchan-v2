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
    pub async fn insert_post_by_thread_id(
        thread_id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
        input: PostInput,
    ) -> Result<Post, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let _ = diesel::insert_into(posts::table)
                    .values(PostModel {
                        user_id: input.user_id,
                        thread_id,
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
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);