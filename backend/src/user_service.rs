pub mod authentication;
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