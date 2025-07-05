use std::path::PathBuf;

use actix_files::NamedFile;
use actix_identity::Identity;
use actix_web::{web, HttpRequest};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{models::posts::Attachment, services::authentication::resolve_user};


pub async fn serve_files(
    user: Option<Identity>,
    file: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<NamedFile> {
    let file_id = file.into_inner().0;

    let file_info = match Attachment::with_post_by_id(file_id, &conn_pool).await {
        Ok(info) => info,
        Err(err) => match err {
            diesel::result::Error::NotFound => return Err(actix_web::error::ErrorNotFound("Tiedostoa ei ole olemassa!")),
            _ => return Err(actix_web::error::ErrorInternalServerError("Palvelin ongelma!")),
        },
    };

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Palvelin ongelma!")),
    };

    if file_info.1.access_level > user_data.access_level {
        return Err(actix_web::error::ErrorForbidden("Ei käyttöoikeutta!"));
    }

    if user_data.banned.is_some() {
        return Err(actix_web::error::ErrorForbidden("Käyttäjätili on bannattu!"));
    }

    let file_path = format!("{}/{}", &file_info.0.file_location, &file_info.0.file_name);

    let path: PathBuf = match file_path.parse() {
        Ok(path) => path,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    Ok(NamedFile::open(path)?)
}

pub async fn serve_thumbnails(
    user: Option<Identity>,
    file: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<NamedFile> {
    let file_id = file.into_inner().0;

    let file_info = match Attachment::with_post_by_id(file_id, &conn_pool).await {
        Ok(info) => info,
        Err(err) => match err {
            diesel::result::Error::NotFound => return Err(actix_web::error::ErrorNotFound("Tiedostoa ei ole olemassa!")),
            _ => return Err(actix_web::error::ErrorInternalServerError("Palvelin ongelma!")),
        },
    };

    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Palvelin ongelma!")),
    };

    if file_info.1.access_level > user_data.access_level {
        return Err(actix_web::error::ErrorForbidden("Ei käyttöoikeutta!"));
    }

    if user_data.banned.is_some() {
        return Err(actix_web::error::ErrorForbidden("Käyttäjätili on bannattu!"));
    }

    let file_path = format!("{}/{}", &file_info.0.thumbnail_location, &file_info.0.file_name);

    let path: PathBuf = match file_path.parse() {
        Ok(path) => path,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    Ok(NamedFile::open(path)?)
}