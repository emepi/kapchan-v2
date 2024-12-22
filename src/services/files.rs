use std::{fs::Metadata, io::{BufReader, Cursor, Read, Seek}};

use actix_multipart::form::tempfile::TempFile;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use image::ImageReader;

use crate::models::{files::FileInfo, posts::{Attachment, AttachmentModel}};


pub async fn create_attachment(
    conn_pool: &Pool<AsyncMysqlConnection>,
    post_id: u32,
    attachment: TempFile,
) -> Option<Attachment> {
    let mime = match &attachment.content_type {
        Some(mime) => mime,
        None => return None,
    };

    let file_type = match mime.type_() {
        mime::IMAGE => mime.type_().to_string(),
        mime::VIDEO => mime.type_().to_string(),
        _ => return None, // unsupported file type
    };

    let file_name = match &attachment.file_name {
        Some(file_name) => file_name,
        None => return None,
    };

    let file_path = format!("files/{}", post_id);
    let file_location = format!("{}/{}", &file_path, &file_name);

    let thumbnail_path = format!("thumbnails/{}", post_id);
    let thumbnail_location = format!("{}/{}", &thumbnail_path, &file_name);

    match tokio::fs::create_dir_all(&file_path).await {
        Ok(_) => (),
        Err(_) => return None,
    };

    match tokio::fs::create_dir_all(&thumbnail_path).await {
        Ok(_) => (),
        Err(_) => return None,
    };

    let metadata = match attachment.file.as_file().metadata() {
        Ok(metadata) => metadata,
        Err(_) => return None,
    };

    let img = match ImageReader::new(BufReader::new(attachment.file.as_file())).with_guessed_format() {
        Ok(i) => match i.decode() {
            Ok(decoded) => decoded,
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    let thumbnail = img.thumbnail(300, 300);
    let _ = thumbnail.save(&thumbnail_location);

    match attachment.file.persist(&file_location) {
        Ok(_) => (),
        Err(_) => return None,
    };

    AttachmentModel {
        id: post_id,
        width: img.width(),
        height: img.height(),
        file_size_bytes: metadata.len(),
        file_name: &file_name,
        file_type: &file_type,
        file_location: &file_path,
        thumbnail_location: &thumbnail_path,
    }
    .insert(conn_pool)
    .await
    .ok()
}