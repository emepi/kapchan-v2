mod board;


use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::Deserialize;

use crate::{
    server::service::{
        WebsocketService, 
        WebsocketServiceManager, 
        ServiceRequestFrame, 
        ServiceResponseFrame, 
        INVALID_SERVICE_TYPE, 
        NOT_ALLOWED, 
        MALFORMATTED, 
        SUCCESS
    }, 
    user_service::{session::UserSession, user::AccessLevel}, 
    board_service::board::BoardFlagModel, 
    BOARD_SERVICE_ID
};

use self::board::{BoardModel, query_boards};


// Service types (t) for input ServiceFrame
pub const CREATE_BOARD_REQUEST: u32 = 1;
pub const FETCH_BOARDS_REQUEST: u32 = 2;


pub struct BoardService {
    srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
    conn_pool: Pool<AsyncMysqlConnection>,
}

#[async_trait]
impl WebsocketService for BoardService {
    fn new(
        srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
        conn_pool: Pool<AsyncMysqlConnection>,
    ) -> Self where Self: Sized {

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
                create_board(sess, req, &self.conn_pool).await
            },

            FETCH_BOARDS_REQUEST => {
                fetch_boards(sess, req, &self.conn_pool).await
            }

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
    req: ServiceRequestFrame,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceResponseFrame {

    if sess.access_level < AccessLevel::Owner as u8 {
        return ServiceResponseFrame {
            t: CREATE_BOARD_REQUEST,
            c: NOT_ALLOWED,
            b: String::default(),
        }
    }

    let input = match serde_json::from_str::<BoardCreationInput>(&req.b) {
        Ok(input) => input,
        Err(_) => return ServiceResponseFrame {
            t: CREATE_BOARD_REQUEST,
            c: MALFORMATTED,
            b: String::default(),
        },
    };

    let board = BoardModel {
        handle: &input.handle,
        title: &input.title,
        description: &input.description,
        created_by: sess.user_id,
    };

    let board = board.insert(conn_pool).await;

    match board {
        Some(board) => {
            for flag in input.flags.iter() {
                BoardFlagModel {
                    board_id: board.id,
                    flag: *flag,
                }
                .insert(conn_pool)
                .await;
            }
        },

        None => (),
    }

    ServiceResponseFrame {
        t: CREATE_BOARD_REQUEST,
        c: SUCCESS,
        b: String::default(),
    }
}

#[derive(Deserialize)]
pub struct BoardCreationInput {
    pub handle: String,
    pub title: String,
    pub description: String,
    pub flags: Vec<u8>,
}

pub async fn fetch_boards (
    sess: &Arc<UserSession>,
    req: ServiceRequestFrame,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> ServiceResponseFrame {
    // TODO: implement user filters & display by rank.

    let boards = query_boards(conn_pool).await;

    ServiceResponseFrame {
        t: FETCH_BOARDS_REQUEST,
        c: SUCCESS,
        b: serde_json::json!(boards).to_string(),
    }
}