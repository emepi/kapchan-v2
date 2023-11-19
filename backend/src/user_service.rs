pub mod authentication;
pub mod session;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;

use crate::server::{service::{
    WebsocketService,
    WebsocketServiceManager, ServiceRequestFrame, ServiceResponseFrame, 
}, session::UpgradeSession};

use self::{
    user::User, 
    session::UserSession, 
    authentication::{validate_password_a2id, create_authentication_token}
};


pub const USER_SERVICE_ID: u32 = 1;

// Service types (t) for input ServiceFrame
pub const LOGIN_REQUEST: u32 = 1;
pub const LOGOUT_REQUEST: u32 = 2;

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
        req: ServiceRequestFrame, 
    ) -> ServiceResponseFrame {

        match req.t {
            LOGIN_REQUEST => {
                login_handler(req, sess, &self.conn_pool, &self.srvc_mgr)
                .await
            },

            LOGOUT_REQUEST => {
                logout_handler(sess, &self.conn_pool)
                .await
            }

            unknown_type => ServiceResponseFrame {
                t: unknown_type,
                c: INVALID_SERVICE_TYPE,
                b: String::default(),
            },
        }
    }

    fn id(&self) -> u32 {
        USER_SERVICE_ID
    }
}


async fn login_handler(
    req: ServiceRequestFrame,
    curr_sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
    srvc_mgr: &Arc<Mutex<WebsocketServiceManager>>,
) -> ServiceResponseFrame {

    let creds: LoginCredentials = match serde_json::from_str(&req.b) {
        Ok(creds) => creds,
        
        Err(_) => {
            return ServiceResponseFrame {
                t: LOGIN_REQUEST,
                c: MALFORMATTED,
                b: String::default(),
            }
        },
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

        None => return ServiceResponseFrame {
            t: LOGIN_REQUEST,
            c: NOT_FOUND,
            b: String::default(),
        },
    };

    match auth {
        true => {
            match user.create_session(
                curr_sess.ip_address.as_deref(), 
                curr_sess.user_agent.as_deref(), 
                &conn_pool
            ).await {
                Some(n_sess) => {
                    let token = create_authentication_token(
                        n_sess.id, 
                        n_sess.access_level
                    )
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

                    ServiceResponseFrame {
                        t: LOGIN_REQUEST,
                        c: SUCCESS,
                        b: token,
                    }
                },

                None => ServiceResponseFrame {
                    t: LOGIN_REQUEST,
                    c: NOT_AVAILABE,
                    b: String::default(),
                },
            }
        },

        false => ServiceResponseFrame {
            t: LOGIN_REQUEST,
            c: FAILURE,
            b: String::default(),
        },
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String
}

async fn logout_handler(
    curr_sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceResponseFrame {
    curr_sess.end_session(conn_pool)
    .await;

    ServiceResponseFrame {
        t: LOGOUT_REQUEST,
        c: SUCCESS,
        b: String::default(),
    }
}