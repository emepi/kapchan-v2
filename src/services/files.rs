use std::{fs::remove_file, io::BufReader, thread};

use actix_multipart::form::tempfile::TempFile;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use image::ImageReader;

use crate::models::posts::{Attachment, AttachmentModel};


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
        //mime::VIDEO => mime.type_().to_string(),
        _ => return None, // unsupported file type
    };

    match mime.subtype().as_str() {
        "gif" => (),
        "jpg" => (),
        "jpeg" => (),
        "png" => (),
        "webp" => (),
        "avif" => (),
        _ => return None,
    };

    let file_name = match &attachment.file_name {
        Some(file_name) => file_name,
        None => return None,
    };

    let file_path = format!("files/{}", post_id);
    let file_location = format!("{}/{}", &file_path, &file_name);
    let file_location_clone = file_location.clone();

    let thumbnail_path = format!("thumbnails/{}", post_id);
    let thumbnail_location = format!("{}/{}", &thumbnail_path, &file_name);
    let thumbnail_location_clone = thumbnail_location.clone();

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

    if mime.type_() == mime::IMAGE {
        let img = match ImageReader::new(BufReader::new(attachment.file.as_file())).with_guessed_format() {
            Ok(i) => match i.decode() {
                Ok(decoded) => decoded,
                Err(_) => return None,
            },
            Err(_) => return None,
        };
    
        let width = img.width();
        let height = img.height();
    
        let handle = thread::spawn(move || {
            match attachment.file.persist(&file_location) {
                Ok(_) => (),
                Err(_) => return None,
            };
            
            let thumbnail = img.thumbnail(300, 300);
            let _ = thumbnail.save(&thumbnail_location);
            
            Some(())
        });
  
        match (AttachmentModel {
            id: post_id,
            width,
            height,
            file_size_bytes: metadata.len(),
            file_name: &file_name,
            file_type: &file_type,
            file_location: &file_path,
            thumbnail_location: &thumbnail_path,
        })
        .insert(conn_pool)
        .await {
            Ok(attachment) => return Some(attachment),
            Err(_) => {
                let _ = handle.join();

                match remove_file(file_location_clone) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error while removing file: {:?}", e);
                    },
                };
        
                match remove_file(thumbnail_location_clone) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error while removing file: {:?}", e);
                    },
                };
            },
        }
    }

    None
}

pub fn display_filesize(
    bytes: u64,
) -> String {
    if bytes as f64 / 1_000_000.0 > 1.0 {
        return format!("{:.02} MB", bytes as f64 / 1_000_000.0);
    } else if bytes as f64 / 1000.0 > 1.0 {
        return format!("{:.02} kB", bytes as f64 / 1000.0);
    } else {
        return format!("{} B", bytes);
    }
}