pub mod user;


use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::server::service::{Service, ServiceDataFeed};

use self::user::{UserModel, UserSession};

const USER_SERVICE_ID: u32 = 1;

pub struct UserService {
    
}

impl UserService {
    pub fn new() -> Box<UserService> {

        Box::new(UserService {
            
        })
    }
}

impl Service for UserService {
    fn user_request(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<ServiceDataFeed> {
        None
    }

    fn id(&self) -> u32 { USER_SERVICE_ID }
}

// TODO: Make this prettier and handle errors.
pub async fn create_anonymous_session(
    ip: Option<&str>,
    user_agent: Option<&str>,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Option<UserSession> {
    // TODO: make this an default trait for an user.
    let anon_usr_model = UserModel {
        access_level: 1,
        username: None,
        email: None,
        password_hash: None,
    };

    let user = anon_usr_model.register_user(conn_pool).await?;

    user.create_session(None, None, ip, user_agent, &conn_pool).await
}