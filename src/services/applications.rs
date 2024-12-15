use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::models::{applications::ApplicationModel, users::{AccessLevel, User}};


pub async fn submit_application(
    conn_pool: &Pool<AsyncMysqlConnection>,
    user_id: u32,
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