use diesel::{
    prelude::*, 
    result::Error, 
    sql_function, 
    ExpressionMethods, 
    QueryDsl
};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{models::threads::Thread, schema::{attachments, boards, posts, replies, threads}};

use super::boards::Board;


#[derive(Debug, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize, Clone, PartialEq)]
#[diesel(belongs_to(Thread))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(primary_key(id))]
pub struct Post {
    pub id: u32,
    pub user_id: u64,
    pub thread_id: u32,
    pub access_level: u8,
    pub show_username: bool,
    pub sage: bool,
    pub message: String,
    pub message_hash: String,
    pub ip_address: String,
    pub country_code: Option<String>,
    pub mod_note: Option<String>,
    pub created_at: NaiveDateTime,
}

impl Post {
    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Post, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let post = posts::table
                    .find(id)
                    .first::<Post>(conn)
                    .await?;
        
                    Ok(post)
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn full_post_by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<PostData, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let post = posts::table
                    .find(id)
                    .left_join(attachments::table)
                    .first::<(Post, Option<Attachment>)>(conn)
                    .await?;

                    let replies = Reply::belonging_to(&post.0)
                    .select(Reply::as_select())
                    .load::<Reply>(conn)
                    .await?
                    .into_iter()
                    .map(|reply| reply.reply_id)
                    .collect();
        
                    Ok(PostData {
                        post: post.0,
                        attachment: post.1,
                        replies,
                    })
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

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
                        ip_address: &input.ip_address,
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

    pub async fn delete_post(
        conn_pool: &Pool<AsyncMysqlConnection>,
        post_id: u32,
    ) -> Result<(), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    diesel::delete(
                        posts::table.find(post_id)
                    )
                    .execute(conn)
                    .await?;

                    Ok(())
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = posts)]
pub struct PostModel<'a> {
    pub user_id: u64,
    pub thread_id: u32,
    pub access_level: u8,
    pub show_username: bool,
    pub sage: bool,
    pub message: &'a str,
    pub message_hash: &'a str,
    pub ip_address: &'a str,
    pub country_code: Option<&'a str>,
    pub mod_note: Option<&'a str>,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = attachments)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Attachment {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub file_name: String,
    pub file_type: String,
    pub file_location: String,
    pub thumbnail_location: String,
}

impl Attachment {
    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Attachment, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let attachment = attachments::table
                    .find(id)
                    .first::<Attachment>(conn)
                    .await?;
        
                    Ok(attachment)
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn with_post_by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<(Attachment,Post), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let attachment = attachments::table
                    .find(id)
                    .inner_join(posts::table)
                    .first::<(Attachment, Post)>(conn)
                    .await?;
        
                    Ok(attachment)
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = attachments)]
pub struct AttachmentModel<'a> {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub file_location: &'a str,
    pub thumbnail_location: &'a str,
}

impl AttachmentModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Attachment, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(attachments::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let attachment = attachments::table
                    .find(self.id)
                    .first::<Attachment>(conn)
                    .await?;
            
                    Ok(attachment)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize, Clone, PartialEq)]
#[diesel(belongs_to(Post))]
#[diesel(table_name = replies)]
#[diesel(primary_key(post_id, reply_id))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Reply {
    pub post_id: u32,
    pub reply_id: u32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = replies)]
pub struct ReplyModel {
    pub post_id: u32,
    pub reply_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostInput {
    pub access_level: u8,
    pub user_id: u64,
    pub show_username: bool,
    pub sage: bool,
    pub message: String,
    pub message_hash: String,
    pub ip_address: String,
    pub country_code: Option<String>,
    pub mod_note: Option<String>,
    pub reply_ids: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostOutput {
    pub id: u32,
    pub show_username: bool,
    pub message: String,
    pub country_code: Option<String>,
    pub attachment: Option<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostData {
    pub post: Post,
    pub attachment: Option<Attachment>,
    pub replies: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostPreview {
    pub post_id: u32,
    pub thread_id: u32,
    pub board_handle: String,
    pub board_name: String,
    pub message: String,
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);