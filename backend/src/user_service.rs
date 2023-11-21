pub mod application;
pub mod authentication;
pub mod session;
pub mod user;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;
use serde_json::json;

use crate::{server::{service::{
    WebsocketService,
    WebsocketServiceManager, ServiceRequestFrame, ServiceResponseFrame, 
}, session::UpgradeSession}, user_service::user::UserModel};

use self::{
    user::{User, AccessLevel}, 
    session::UserSession, 
    authentication::{validate_password_a2id, create_authentication_token}, 
    application::{ApplicationModel, Application, list_applications}
};


pub const USER_SERVICE_ID: u32 = 1;

// Service types (t) for input ServiceFrame
pub const LOGIN_REQUEST: u32 = 1;
pub const LOGOUT_REQUEST: u32 = 2;
pub const APPLICATION_REQUEST: u32 = 3;
pub const APPLICATION_FETCH_REQUEST: u32 = 4;

// Service response types (c)
pub const SUCCESS: u32 = 1;
pub const FAILURE: u32 = 2;
pub const NOT_FOUND: u32 = 3;
pub const NOT_AVAILABLE: u32 = 4;
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
            },

            APPLICATION_REQUEST => {
                application_handler(req, sess, &self.conn_pool, &self.srvc_mgr)
                .await
            },

            APPLICATION_FETCH_REQUEST => {
                application_fetch_handler(req, sess, &self.conn_pool)
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
                    c: NOT_AVAILABLE,
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

async fn application_handler(
    req: ServiceRequestFrame,
    curr_sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
    srvc_mgr: &Arc<Mutex<WebsocketServiceManager>>,
) -> ServiceResponseFrame {
    if curr_sess.access_level != AccessLevel::Anonymous as u8 {
        return ServiceResponseFrame {
            t: APPLICATION_REQUEST,
            c: NOT_ALLOWED,
            b: String::default(),
        };
    }

    let input = match serde_json::from_str::<ApplicationInput>(&req.b) {
        Ok(input) => input,
        Err(_) => return ServiceResponseFrame {
            t: APPLICATION_REQUEST,
            c: MALFORMATTED,
            b: String::default(),
        },
    };

    let user = match User::by_id(curr_sess.user_id, conn_pool).await {
        Some(mut user) => {
            user = user.modify(
                UserModel {
                    access_level: AccessLevel::PendingMember as u8,
                    username: Some(&input.username),
                    email: Some(&input.password),
                    password_hash: Some(&input.email),
                }, 
                conn_pool,
            )
            .await;

            user
        },
        None => return ServiceResponseFrame {
            t: APPLICATION_REQUEST,
            c: MALFORMATTED,
            b: String::default(),
        },
    };

    let session = user.create_session(
        curr_sess.ip_address.as_deref(), 
        curr_sess.user_agent.as_deref(), 
        conn_pool
    )
    .await;

    let token = match session {
        Some(session) => {

            let token = create_authentication_token(
                session.id, 
                session.access_level
            )
            .unwrap_or_default();

            srvc_mgr.lock().unwrap().get_client(curr_sess.id)
            .map(|cli| {
                let _ = cli.try_send(UpgradeSession {
                    sess: Arc::new(session),
                });
            });

            curr_sess.end_session(conn_pool).await;

            token
        },
        None => return ServiceResponseFrame {
            t: APPLICATION_REQUEST,
            c: FAILURE,
            b: String::default(),
        },
    };

    let application = ApplicationModel {
        user_id: curr_sess.user_id,
        accepted: false,
        background: &input.background,
        motivation: &input.motivation,
        other: input.referrer.as_deref(),
        closed_at: None,
    }
    .insert(conn_pool)
    .await;

    match application {
        Some(application) => {
            // TODO: return application data?
            
            ServiceResponseFrame {
                t: APPLICATION_REQUEST,
                c: SUCCESS,
                b: token,
            }
        },
        None => ServiceResponseFrame { 
            t: APPLICATION_REQUEST, 
            c: FAILURE, 
            b: String::default(), 
        },
    }
}

#[derive(Deserialize)]
pub struct ApplicationInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub background: String,
    pub motivation: String,
    pub referrer: Option<String>,
}

async fn application_fetch_handler(
    req: ServiceRequestFrame,
    curr_sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceResponseFrame {
    if curr_sess.access_level < AccessLevel::Admin as u8 {
        return ServiceResponseFrame {
            t: APPLICATION_FETCH_REQUEST,
            c: NOT_ALLOWED,
            b: String::default(),
        };
    }

    let input = match serde_json::from_str::<ApplicationFetchInput>(&req.b) {
        Ok(input) => input,
        Err(_) => return ServiceResponseFrame {
            t: APPLICATION_FETCH_REQUEST,
            c: MALFORMATTED,
            b: String::default(),
        },
    };

    let applications = list_applications(
        input.accepted, 
        input.handled, 
        input.limit, 
        conn_pool
    ).await;

    ServiceResponseFrame {
        t: APPLICATION_FETCH_REQUEST,
        c: SUCCESS,
        b: json!(applications).to_string(),
    }
}

#[derive(Deserialize)]
pub struct ApplicationFetchInput {
    pub accepted: bool,
    pub handled: bool,
    pub limit: Option<i64>,
}