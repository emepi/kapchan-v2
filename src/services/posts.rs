use actix_multipart::form::tempfile::TempFile;
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use sha2::{Digest, Sha256};

use crate::models::{posts::{Post, PostInput}, threads::{Thread, ThreadInput}};

use super::files::{read_file_info, store_attachment};


pub async fn create_post_by_post_id(
    conn_pool: &Pool<AsyncMysqlConnection>,
    user_id: u64,
    post_id: u32,
    message: String,
    attachment: TempFile,
    ip_addr: String,
    user_agent: String,
) -> Result<(), Error> {
    //TODO: handle replies

    let mut hasher = Sha256::new();
    hasher.update(message.clone());

    let message_hash = format!("{:X}", hasher.finalize());

    let post = match Post::insert_post_by_post_id(post_id, &conn_pool, PostInput {
        user_id,
        show_username: false,
        message,
        message_hash,
        ip_address: ip_addr,
        user_agent,
        country_code: None,
        hidden: false,
        attachment: read_file_info(&attachment),
    }).await {
        Ok(post) => post,
        Err(e) => return Err(e),
    };

    if post.attachment.is_some() {
        let _ = store_attachment(attachment, post.attachment.unwrap()).await;
    }

    Ok(())
}