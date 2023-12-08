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
    user_service::session::UserSession
};


pub const BOARD_SERVICE_ID: u32 = 2;

// Service types (t) for input ServiceFrame
pub const CREATION_REQUEST: u32 = 1;

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

            CREATION_REQUEST => {
                todo!()
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