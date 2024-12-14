use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::models::users::{AccessLevel, User, UserModel};

use super::authentication::hash_password_argon2id;


pub async fn create_anonymous_user(
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Result<User, Error> {
    Ok(UserModel {
        access_level: AccessLevel::Anonymous as u8,
        username: None,
        email: None,
        password_hash: None,
    }
    .insert(conn_pool)
    .await?)
}

pub async fn update_root_user(
    conn_pool: &Pool<AsyncMysqlConnection>,
    password: &str,
) -> Result<(), Error> {
    let password_hash = hash_password_argon2id(password);

    let root_model = UserModel {
        access_level: AccessLevel::Root as u8,
        username: Some("root"),
        email: None,
        password_hash: Some(&password_hash),
    };

    match User::by_username("root", conn_pool).await {
        Ok(root_user) => Ok(root_model.update_by_id(root_user.id, conn_pool)
            .await
            .and_then(|_| Ok(()))?
        ),
        Err(_) => Ok(root_model.insert(conn_pool)
            .await
            .and_then(|_| Ok(()))?
        ),
    }
}