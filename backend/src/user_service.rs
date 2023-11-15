pub mod authentication;
pub mod user;


use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use log::info;

use crate::server::service::{Service, FeedControl};

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
    fn connection_request(
        &self,
        msg: String,
        session_id: u32,
        session_access: u8,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> FeedControl {
        info!("Message received in user services: {}", msg);
        
        FeedControl {
            op_allowed: true,
            response: Some(String::from("Roundtrip completed!")),
        }
    }

    fn id(&self) -> u32 { USER_SERVICE_ID }
}