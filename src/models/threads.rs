use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{
    prelude::*, 
    result::Error, 
    sql_function, 
    ExpressionMethods, 
    QueryDsl, 
    SelectableHelper
};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};
use itertools::izip;
use serde::{Deserialize, Serialize};

use crate::schema::{attachments, posts, replies, threads};

use super::posts::{Attachment, Post, PostData, PostInput, PostModel, PostOutput, Reply, ReplyModel};


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = threads)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Thread {
    pub id: u32,
    pub user_id: u64,
    pub board_id: u32,
    pub title: String,
    pub pinned: bool,
    pub archived: bool,
    pub bump_time: NaiveDateTime,
}

impl Thread {
    pub async fn count_replies(
        thread_id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<i64, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let count = posts::table
                    .filter(posts::thread_id.eq(thread_id))
                    .count()
                    .get_result(conn)
                    .await?;

                    Ok(count)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn by_id(
        thread_id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<ThreadData, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let thread = threads::table
                    .find(thread_id)
                    .first::<Thread>(conn)
                    .await?;

                    let thread_posts: Vec<(Post, Option<Attachment>)> = Post::belonging_to(&thread)
                    .left_join(attachments::table)
                    .order_by(posts::id)
                    .select((
                        Post::as_select(),
                        Option::<Attachment>::as_select()
                    ))
                    .load::<(Post, Option<Attachment>)>(conn)
                    .await?;

                    let (posts, attachments): (Vec<_>, Vec<_>) = thread_posts.into_iter().unzip();

                    let replies = Reply::belonging_to(&posts)
                    .select(Reply::as_select())
                    .load::<Reply>(conn)
                    .await?;

                    let replies_per_post = replies.grouped_by(&posts);

                    let mut post_data: Vec<PostData> = vec![];

                    for (post, attachment, replies) in izip!(posts, attachments, replies_per_post) {
                        post_data.push(PostData {
                            post,
                            attachment,
                            replies: replies.into_iter().map(|reply| reply.reply_id).collect(),
                        });
                    }

                    Ok(ThreadData {
                        thread,
                        posts: post_data,
                    })
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn thread_by_id(
        thread_id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Thread, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let thread = threads::table
                    .find(thread_id)
                    .first::<Thread>(conn)
                    .await?;

                    Ok(thread)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn insert_thread(
        conn_pool: &Pool<AsyncMysqlConnection>,
        input: ThreadInput,
        active_threads_limit: u32,
    ) -> Result<(Thread, Post), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(threads::table)
                    .values(ThreadModel {
                        user_id: input.post.user_id,
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
                        country_code: input.post.country_code.as_deref(),
                        access_level: input.post.access_level,
                        sage: input.post.sage,
                        mod_note: input.post.mod_note.as_deref(),
                    })
                    .execute(conn)
                    .await?;

                    let post = posts::table
                    .find(last_insert_id())
                    .first::<Post>(conn)
                    .await?;

                    let replies: Vec<ReplyModel> = input.post.reply_ids
                    .iter()
                    .map(|reply_id| ReplyModel {
                        post_id: *reply_id,
                        reply_id: post.id,
                    })
                    .collect();

                    let _ = diesel::insert_into(replies::table)
                    .values(replies)
                    .execute(conn)
                    .await?;

                    // archive threads over active threads limits
                    let inactive_thread = threads::table
                    .filter(threads::board_id.eq(input.board_id))
                    .filter(threads::archived.eq(false))
                    .order((threads::pinned.eq(true), threads::bump_time.desc()))
                    .limit(1)
                    .offset(active_threads_limit.into())
                    .load::<Thread>(conn)
                    .await?;

                    match inactive_thread.get(0) {
                        Some(thread) => diesel::update(
                            threads::table.find(thread.id)
                        )
                        .set(threads::archived.eq(true))
                        .execute(conn).await?,
                        None => 0,
                    };
            
                    Ok((thread, post))
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
                        // TODO: add error handling
                        let op_post = posts.get(0).expect("Encountered thread without op post!");

                        ThreadCatalogOutput {
                            id: thread.id,
                            title: thread.title,
                            pinned: thread.pinned,
                            op_post: PostOutput {
                                id: op_post.0.id,
                                show_username: op_post.0.show_username,
                                message: op_post.0.message.clone(),
                                country_code: op_post.0.country_code.clone(),
                                attachment: op_post.1.clone(),
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

    pub async fn bump_thread(
        conn_pool: &Pool<AsyncMysqlConnection>,
        thread_id: u32,
    ) -> Result<(), Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let current_time = Utc::now().naive_utc();

                    diesel::update(
                        threads::table.find(thread_id)
                    )
                    .set(threads::bump_time.eq(current_time))
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
#[diesel(table_name = threads)]
pub struct ThreadModel<'a> {
    pub user_id: u64,
    pub board_id: u32,
    pub title: &'a str,
    pub pinned: bool,
    pub archived: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadInput {
    pub board_id: u32,
    pub title: String,
    pub pinned: bool,
    pub archived: bool,
    pub post: PostInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadCatalogOutput {
    pub id: u32,
    pub title: String,
    pub pinned: bool,
    pub op_post: PostOutput,
    pub replies: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadDbOutput {
    pub thread: Thread,
    pub post: Post,
    pub attachment: Option<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadData {
    pub thread: Thread,
    pub posts: Vec<PostData>,
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);