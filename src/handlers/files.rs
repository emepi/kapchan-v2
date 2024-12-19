use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::web;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::models::posts::Attachment;

//TODO: check permissions

pub async fn serve_files(
    file: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<NamedFile> {
    let file_id = file.into_inner().0;

    let file_info = match Attachment::by_id(file_id, &conn_pool).await {
        Ok(info) => info,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    let file_path = format!("{}/{}", &file_info.file_location, &file_info.file_name);

    let path: PathBuf = match file_path.parse() {
        Ok(path) => path,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    Ok(NamedFile::open(path)?)
}

pub async fn serve_thumbnails(
    file: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<NamedFile> {
    let file_id = file.into_inner().0;

    let file_info = match Attachment::by_id(file_id, &conn_pool).await {
        Ok(info) => info,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    let file_path = format!("{}/{}", &file_info.thumbnail_location, &file_info.file_name);

    let path: PathBuf = match file_path.parse() {
        Ok(path) => path,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    Ok(NamedFile::open(path)?)
}