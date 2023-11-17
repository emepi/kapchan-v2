pub mod authentication;
pub mod session;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use log::info;
use serde::Deserialize;

use crate::server::{service::{
    WebsocketService, 
    ServiceFrame, 
    WebsocketServiceManager, 
}, session::UpgradeSession};

use self::{user::User, session::UserSession, authentication::{hashes_to_password, create_authentication_token}};

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
        sess: &Arc<UserSession>,
        req: ServiceFrame, 
    ) -> ServiceFrame {

        match req.t {
            LOGIN_REQUEST => {
                login_handler(req, sess, &self.conn_pool, &self.srvc_mgr)
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
    curr_sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
    srvc_mgr: &Arc<Mutex<WebsocketServiceManager>>,
) -> ServiceFrame {

    info!("starting login..");

    let creds: LoginCredentials = match serde_json::from_str(&req.b) {
        Ok(creds) => creds,
        Err(_) => {
            return ServiceFrame::default()
        },
    };

    info!("creds ok");

    let auth;

    let user = match User::by_username(&creds.username, conn_pool)
    .await {
        Some(user) => {
            auth = user.password_hash.clone()
            .map(|hash| hashes_to_password(&hash, &creds.password))
            .unwrap_or(false); // <- password not set

            user
        },

        None => {
            return ServiceFrame {
                t: 2,
                b: String::from(""),
            };
        },
    };

    info!("user & auth ok");

    match auth {
        true => {
            match user.create_session(
                curr_sess.ip_address.as_deref(), 
                curr_sess.user_agent.as_deref(), 
                &conn_pool
            ).await {
                Some(n_sess) => {
                    let token = create_authentication_token(n_sess.id);

                    // send session upgrade
                    {
                        srvc_mgr.lock().unwrap().get_client(curr_sess.id)
                        .map(|cli| {
                            cli.try_send(UpgradeSession {
                                sess: Arc::new(n_sess),
                            })
                        });
                    }

                    // TODO: end current session in db

                    return ServiceFrame {
                        t: req.t,
                        b: token.unwrap_or_default(),
                    };
                },

                None => {
                    return ServiceFrame {
                        t: 2,
                        b: String::from(""),
                    };
                }
            };
        },

        false => {
            return ServiceFrame {
                t: 3,
                b: String::from(""),
            };
        },
    }
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