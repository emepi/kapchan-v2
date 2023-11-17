pub mod authentication;
pub mod session;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;

use crate::{server::service::{
    WebsocketService, 
    ServiceFrame, 
    WebsocketServiceManager, 
}, schema::users::password_hash};

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
        sess: &Arc<Mutex<UserSession>>,
        req: ServiceFrame, 
    ) -> ServiceFrame {

        match req.t {
            LOGIN_REQUEST => {
                login_handler(req, sess, &self.conn_pool)
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
    curr_sess: &Arc<Mutex<UserSession>>,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceFrame {

    let creds: LoginCredentials = match serde_json::from_str(&req.b) {
        Ok(creds) => creds,
        Err(_) => {
            return ServiceFrame::default()
        },
    };

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

    match auth {
        true => {
            let ip_address;
            let user_agent;

            {
                let sess = curr_sess.lock().unwrap();
                ip_address = sess.ip_address.clone();
                user_agent = sess.user_agent.clone();
            }

            match user.create_session(
                ip_address.as_deref(), 
                user_agent.as_deref(), 
                &conn_pool
            ).await {
                Some(n_sess) => {
                    let token = create_authentication_token(n_sess.id);

                    let mut c_sess = curr_sess.lock().unwrap();
                    c_sess.access_level = n_sess.access_level;
                    c_sess.created_at = n_sess.created_at;
                    c_sess.id = n_sess.id;
                    c_sess.mode = n_sess.mode;
                    c_sess.user_id = n_sess.user_id;

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