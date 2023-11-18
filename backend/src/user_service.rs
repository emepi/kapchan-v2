pub mod authentication;
pub mod session;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::{Deserialize, Serialize};

use crate::server::{service::{
    WebsocketService, 
    ServiceFrame, 
    WebsocketServiceManager, 
}, session::UpgradeSession};

use self::{
    user::User, 
    session::UserSession, 
    authentication::{validate_password_a2id, create_authentication_token}
};


pub const USER_SERVICE_ID: u32 = 1;

// Service types (t) for input ServiceFrame
pub const LOGIN_REQUEST: u32 = 1;

// Service response types (c)
pub const SUCCESS: u32 = 1;
pub const FAILURE: u32 = 2;
pub const NOT_FOUND: u32 = 3;
pub const NOT_AVAILABE: u32 = 4;
pub const NOT_ALLOWED: u32 = 5;
pub const MALFORMATTED: u32 = 6;
pub const INVALID_SERVICE_TYPE: u32 = 7;


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
        sess: &Arc<UserSession>,
        req: ServiceFrame, 
    ) -> ServiceFrame {

        match req.t {
            LOGIN_REQUEST => {
                login_handler(req, sess, &self.conn_pool, &self.srvc_mgr)
                .await
            },

            _ => user_service_frame_type(INVALID_SERVICE_TYPE),
        }
    }

    fn id(&self) -> u32 {
        USER_SERVICE_ID
    }
}


async fn login_handler(
    req: ServiceFrame,
    curr_sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
    srvc_mgr: &Arc<Mutex<WebsocketServiceManager>>,
) -> ServiceFrame {

    let creds: LoginCredentials = match serde_json::from_str(&req.b) {
        Ok(creds) => creds,
        // TODO: add a reason
        Err(_) => return user_service_frame_type(MALFORMATTED),
    };

    let auth;

    let user = match User::by_username(&creds.username, conn_pool)
    .await {
        Some(user) => {
            auth = user.password_hash.clone()
            .map(|hash| validate_password_a2id(&hash, &creds.password))
            .unwrap_or(false); // <- password not set

            user
        },

        None => return user_service_frame_type(NOT_FOUND),
    };

    match auth {
        true => {
            match user.create_session(
                curr_sess.ip_address.as_deref(), 
                curr_sess.user_agent.as_deref(), 
                &conn_pool
            ).await {
                Some(n_sess) => {
                    let token = create_authentication_token(n_sess.id)
                    .unwrap_or_default();

                    // send session upgrade
                    {
                        srvc_mgr.lock().unwrap().get_client(curr_sess.id)
                        .map(|cli| {
                            cli.try_send(UpgradeSession {
                                sess: Arc::new(n_sess),
                            })
                        });
                    }

                    curr_sess.end_session(&conn_pool).await;

                    ServiceFrame {
                        t: USER_SERVICE_ID,
                        b: serde_json::to_string(&UserServiceResponseBody {
                            c: SUCCESS,
                            m: Some(token),
                        }).unwrap_or_default(),
                    }
                },

                None => user_service_frame_type(NOT_AVAILABE),
            }
        },

        false => user_service_frame_type(FAILURE),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserServiceResponseBody {
    pub c: u32,
    pub m: Option<String>, // optional message
}

pub fn user_service_frame_type(resp_type: u32) -> ServiceFrame {
    let body = serde_json::to_string(&UserServiceResponseBody {
        c: resp_type,
        m: None,
    });

    ServiceFrame { t: USER_SERVICE_ID, b: body.unwrap_or_default() }
}