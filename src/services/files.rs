use std::io::{BufReader, Cursor, Read};

use actix_multipart::form::tempfile::TempFile;
use image::ImageReader;

use crate::models::{files::FileInfo, posts::Attachment};


pub fn read_file_info(
    file_i: &TempFile,
) -> Option<FileInfo> {
    let mime = match &file_i.content_type {
        Some(mime) => mime,
        None => return None,
    };

    let file_type = match mime.type_() {
        mime::IMAGE => mime.type_().to_string(),
        mime::VIDEO => mime.type_().to_string(),
        _ => return None, // unsupported file type
    };

    //TODO: store subtype separately & check support
    let file_name = match &file_i.file_name {
        Some(file_name) => file_name,
        None => return None,
    };

    Some(FileInfo {
        file_name: file_name.to_string(),
        file_type,
    })
}

pub async fn store_attachment(
    file: TempFile,
    attachment_info: Attachment,
) -> Option<()> {
    match tokio::fs::create_dir_all(&attachment_info.file_location).await {
        Ok(_) => (),
        Err(_) => return None,
    };

    match tokio::fs::create_dir_all(&attachment_info.thumbnail_location).await {
        Ok(_) => (),
        Err(_) => return None,
    };

    let file_path = format!("{}/{}", &attachment_info.file_location, &attachment_info.file_name);
    let file = file.file;

    let img = match ImageReader::new(BufReader::new(&mut file.as_file())).with_guessed_format() {
        Ok(i) => match i.decode() {
            Ok(decoded) => decoded,
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    let thumbnail = img.thumbnail(300, 300);

    let thumbnail_path = format!("{}/{}", &attachment_info.thumbnail_location, &attachment_info.file_name);
    let _ = thumbnail.save(thumbnail_path);

    match file.persist(&file_path) {
        Ok(_) => (),
        Err(_) => return None,
    };

    Some(())
}