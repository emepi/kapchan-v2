pub mod authentication;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use log::info;
use serde::Deserialize;

use crate::server::service::{
    WebsocketService, 
    ServiceFrame, 
    WebsocketServiceManager, ServiceResponse
};

use self::{user::User, authentication::hashes_to_password};

pub const USER_SERVICE_ID: u32 = 1;
pub const LOGIN_REQUEST: u32 = 1;


pub struct UserService {
    pub srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
    pub conn_pool: Pool<AsyncMysqlConnection>, 
}

#[async_trait]
impl WebsocketService for UserService {
    fn new(
        srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
        conn_pool: Pool<AsyncMysqlConnection>, 
    ) -> Self where Self: Sized {
        
        UserService { 
            srvc_mgr, 
            conn_pool, 
        }
    }

    async fn user_request(
        &self,
        usr_id: u32, 
        usr_access: u8,
        req: ServiceFrame, 
    ) -> ServiceResponse {
        
        ServiceResponse {
            data_feed: true,
            resp: ServiceFrame::default(),
        }
    }

    fn id(&self) -> u32 {
        USER_SERVICE_ID
    }
}


async fn handle_login(
    credentials: LoginCredentials, 
    connection_pool: &Pool<AsyncMysqlConnection>
) -> ServiceFrame {
    let auth = User::by_username(&credentials.username, connection_pool)
    .await
    .and_then(|user_data| {
        user_data.password_hash
        .map(|hash| hashes_to_password(&hash, &credentials.password))
    })
    .unwrap_or(false);


    ServiceFrame {
        t: LOGIN_REQUEST,
        b: match auth {
            true => {
                String::from("")
            },
            false => {
                String::from("")
            },
        },
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String
}