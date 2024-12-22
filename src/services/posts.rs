use actix_multipart::form::tempfile::TempFile;
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use sha2::{Digest, Sha256};
use itertools::Itertools;

use crate::models::posts::{Post, PostInput};

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
    let reply_ids = parse_backlinks(&message);

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
        reply_ids,
    }).await {
        Ok(post) => post,
        Err(e) => return Err(e),
    };

    if post.attachment.is_some() {
        let _ = store_attachment(attachment, post.attachment.unwrap()).await;
    }

    Ok(())
}

pub fn parse_backlinks(
    message: &str,
) -> Vec<u32> {
    let re = Regex::new(r">>(\d+)").unwrap();

    let matches: Vec<u32> = re
    .find_iter(message)
    .map(|m| (&m.as_str()[2..]).parse::<u32>().unwrap_or(0))
    .unique()
    .filter(|x| *x > 0)
    .collect();

    return matches
}