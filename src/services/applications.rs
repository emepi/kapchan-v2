use chrono::{DateTime, Utc};
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::models::{applications::{Application, ApplicationModel, ApplicationPreview}, users::{AccessLevel, User}};


pub async fn submit_application(
    conn_pool: &Pool<AsyncMysqlConnection>,
    user_id: u64,
    background: &str,
    motivation: &str,
    other: &str,
) -> Result<(), Error> {
    let application = ApplicationModel {
        user_id,
        accepted: false,
        background,
        motivation,
        other,
        closed_at: None,
    }
    .insert(conn_pool)
    .await;

    match application {
        Ok(_) => {
            match User::update_access_level(user_id, AccessLevel::PendingMember as u8, conn_pool)
            .await {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        },
        Err(err) => Err(err),
    }
}

pub async fn review_application (
    conn_pool: &Pool<AsyncMysqlConnection>,
    application_id: u32,
    reviewer_id: u64,
    accept: bool,
) -> Result<Application, Error> {
    let timestamp = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().naive_utc();

    Application::review(conn_pool, application_id, reviewer_id, accept, timestamp).await
}

pub async fn is_reviewed(
    conn_pool: &Pool<AsyncMysqlConnection>,
    application_id: u32,
) -> Result<bool, Error> {
    Application::closed_at(&conn_pool, application_id).await
    .map(|closed| closed.is_some())
}

pub async fn load_application_previews(
    conn_pool: &Pool<AsyncMysqlConnection>,
    page: i64,
    page_size: i64,
) -> Result<Vec<ApplicationPreview>, Error> {
    let offset = (page - 1) * page_size;

    Application::load_previews(conn_pool, page_size, offset).await
}

pub async fn count_preview_pages(
    conn_pool: &Pool<AsyncMysqlConnection>,
    page_size: u64,
) -> Result<u64, Error> {
    Application::count_previews(conn_pool).await
    .and_then(|count| {
        let count = u64::try_from(count).unwrap();
        Ok(count.div_ceil(page_size))
    })
}