use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use crate::{
    server::service::{
        WebsocketService, 
        WebsocketServiceManager, 
        ServiceRequestFrame, 
        ServiceResponseFrame
    }, 
    user_service::{session::UserSession, user::AccessLevel}
};


pub const BOARD_SERVICE_ID: u32 = 2;

// Service types (t) for input ServiceFrame
pub const CREATE_BOARD_REQUEST: u32 = 1;

// Service response types (c)
pub const SUCCESS: u32 = 1;
pub const FAILURE: u32 = 2;
pub const NOT_FOUND: u32 = 3;
pub const NOT_AVAILABLE: u32 = 4;
pub const NOT_ALLOWED: u32 = 5;
pub const MALFORMATTED: u32 = 6;
pub const INVALID_SERVICE_TYPE: u32 = 7;


pub struct BoardService {
    srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
    conn_pool: Pool<AsyncMysqlConnection>,
}

#[async_trait]
impl WebsocketService for BoardService {
    fn new(
        srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
        conn_pool: Pool<AsyncMysqlConnection>,
    ) -> Self where Self:Sized {

        BoardService { 
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

            CREATE_BOARD_REQUEST => {
                create_board(sess, &self.conn_pool).await
            },

            unknown_type => ServiceResponseFrame {
                t: unknown_type,
                c: INVALID_SERVICE_TYPE,
                b: String::default(),
            },
        }
    }

    fn id(&self) -> u32 {
        BOARD_SERVICE_ID
    }
}


async fn create_board(
    sess: &Arc<UserSession>,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceResponseFrame {

    if sess.access_level < AccessLevel::Admin as u8 {
        return ServiceResponseFrame {
            t: CREATE_BOARD_REQUEST,
            c: NOT_ALLOWED,
            b: String::default(),
        }
    }

    

    todo!()
}