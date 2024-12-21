use diesel::{result::Error, sql_function, BelongingToDsl, ExpressionMethods, GroupedBy, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::posts::{Attachment, AttachmentModel, Post, PostInput, PostModel, PostOutput, ReplyModel}, schema::{attachments, posts, replies}};


impl Post {
    pub async fn insert_post_by_post_id(
        post_id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
        input: PostInput,
    ) -> Result<PostOutput, Error> {
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
                        ip_address: &input.ip_address,
                        user_agent: &input.user_agent,
                        country_code: input.country_code.as_deref(),
                        hidden: input.hidden,
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

                    if input.attachment.is_some() {
                        let attachment = input.attachment.clone().unwrap();

                        // Unique file paths
                        let file_location = format!("files/{}", new_post.id);
                        let thumb_location = format!("thumbnails/{}", new_post.id);

                        let _ = diesel::insert_into(attachments::table)
                        .values(AttachmentModel {
                            id: new_post.id,
                            file_name: &attachment.file_name,
                            file_location: &file_location,
                            thumbnail_location: &thumb_location,
                            file_type: &attachment.file_type,
                        })
                        .execute(conn)
                        .await?;

                        let attachment_o = attachments::table
                        .find(new_post.id)
                        .first::<Attachment>(conn)
                        .await?;

                        return Ok(PostOutput {
                            id: new_post.id,
                            show_username: new_post.show_username,
                            message: new_post.message,
                            country_code: new_post.country_code,
                            hidden: new_post.hidden,
                            attachment: Some(attachment_o),
                        });
                    }
                
                    Ok(PostOutput {
                        id: new_post.id,
                        show_username: new_post.show_username,
                        message: new_post.message,
                        country_code: new_post.country_code,
                        hidden: new_post.hidden,
                        attachment: None,
                    })
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);