use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest};
use diesel::result::Error;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::database::users::{create_anonymous_user, user_by_id};


pub struct UserData {
    pub id: u32,
    pub access_level: u8,
}

pub async fn resolve_user(
    user: Option<Identity>,
    request: HttpRequest,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Result<UserData, Error> {
    let user = match user {
        Some(user) => {
            //TODO:check if number
            let id = user.id().unwrap().parse::<u32>().unwrap();

            user_by_id(id, conn_pool).await?
        },
        None => {
            let user = create_anonymous_user(conn_pool).await?;

            Identity::login(&request.extensions(), user.id.to_string()).unwrap();

            user
        },
    };

    Ok(UserData {
        id: user.id,
        access_level: user.access_level,
    })
}