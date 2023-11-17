pub mod authentication;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;

use crate::server::service::{
    WebsocketService, 
    ServiceFrame, 
    WebsocketServiceManager, 
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
    ) -> ServiceFrame {

        match req.t {
            LOGIN_REQUEST => {
                login_handler(req, &self.conn_pool)
                .await
            },

            _ => unspecified_handler(req),
        }
    }

    fn id(&self) -> u32 {
        USER_SERVICE_ID
    }
}


async fn login_handler(
    req: ServiceFrame, 
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceFrame {
    let creds: LoginCredentials = match serde_json::from_str(&req.b) {
        Ok(creds) => creds,
        Err(_) => {
            return ServiceFrame::default()
        },
    };

    User::by_username(&creds.username, conn_pool)
    .await
    .map(|user| {
        user.password_hash.clone()
        .is_some_and(|hash| hashes_to_password(&hash, &creds.password))
        .then(|| user.create_auth_token())
        .flatten()
        .unwrap_or(String::from("Authentication failed"))
    })
    .map_or(
        ServiceFrame {
            t: req.t,
            b: String::from("User not found"),
        }, 
        |token| ServiceFrame {
            t: req.t,
            b: token,
        })
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String
}

fn unspecified_handler(req: ServiceFrame) -> ServiceFrame {
    ServiceFrame {
        t: req.t,
        b: String::from("Unknown service type"),
    }
}