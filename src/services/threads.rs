use actix_multipart::form::tempfile::TempFile;
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sha2::{Digest, Sha256};

use crate::models::{posts::PostInput, threads::{Thread, ThreadInput}};

use super::{files::create_attachment, posts::parse_backlinks};


pub async fn create_thread(
    conn_pool: &Pool<AsyncMysqlConnection>,
    user_id: u64,
    board_id: u32,
    topic: String,
    message: String,
    attachment: TempFile,
    access_level: u8,
    active_threads_limit: u32,
) -> Result<(), Error> {
    let reply_ids = parse_backlinks(&message);

    let mut hasher = Sha256::new();
    hasher.update(message.clone());

    let message_hash = format!("{:X}", hasher.finalize());

    let thread_input = ThreadInput {
        board_id,
        title: topic,
        pinned: false,
        locked: false,
        archived: false,
        post: PostInput {
            user_id,
            show_username: false,
            message,
            message_hash,
            country_code: None,
            reply_ids,
            sage: false,
            mod_note: None,
            access_level,
        },
    };

    let thread_info = match Thread::insert_thread(&conn_pool, thread_input, active_threads_limit).await {
        Ok(thread_info) => thread_info,
        Err(e) => return Err(e),
    };

    let _ = create_attachment(&conn_pool, thread_info.1.id, attachment).await;

    Ok(())
}