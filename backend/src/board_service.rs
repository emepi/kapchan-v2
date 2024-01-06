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