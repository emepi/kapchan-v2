use actix_multipart::form::tempfile::TempFile;
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use sha2::{Digest, Sha256};
use itertools::Itertools;

use crate::models::posts::{Post, PostInput};

use super::files::create_attachment;


pub async fn create_post_by_post_id(
    conn_pool: &Pool<AsyncMysqlConnection>,
    user_id: u64,
    post_id: u32,
    message: String,
    attachment: TempFile,
    access_level: u8,
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
        country_code: None,
        reply_ids,
        sage: false,
        mod_note: None,
        access_level,
    }).await {
        Ok(post) => post,
        Err(e) => return Err(e),
    };

    let _ = create_attachment(&conn_pool, post.id, attachment).await;

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