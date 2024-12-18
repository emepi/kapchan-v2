use actix_multipart::form::tempfile::TempFile;
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sha2::{Digest, Sha256};

use crate::models::{posts::PostInput, threads::{Thread, ThreadInput}};


pub async fn create_thread(
    conn_pool: &Pool<AsyncMysqlConnection>,
    user_id: u64,
    board_id: u32,
    topic: String,
    message: String,
    attachment: TempFile,
    ip_addr: String,
    user_agent: String,
) -> Result<(), Error> {
    //TODO: handle files
    //TODO: handle replies

    let mut hasher = Sha256::new();
    hasher.update(message.clone());

    let message_hash = format!("{:X}", hasher.finalize());



    let thread_input = ThreadInput {
        board_id,
        title: topic,
        pinned: false,
        archived: false,
        post: PostInput {
            user_id,
            show_username: false,
            message,
            message_hash,
            ip_address: ip_addr,
            user_agent,
            country_code: None,
            hidden: false,
        },
    };

    Thread::insert_thread(&conn_pool, thread_input).await
}