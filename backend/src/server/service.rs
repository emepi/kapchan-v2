use actix::{Handler, Actor, Context, Addr, Message};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use super::session::WebsocketSession;

pub struct ServiceDataFeed {

}

pub struct WebsocketService {
    pub id: u32,
    pub service: Box<dyn Service>,
    pub conn_pool: Pool<AsyncMysqlConnection>,
}

impl WebsocketService {
    pub fn new(
        id: u32, 
        service: Box<dyn Service>, 
        conn_pool: Pool<AsyncMysqlConnection>,
    ) -> Self {

        WebsocketService {
            id,
            service,
            conn_pool,
        }
    }
}

impl Actor for WebsocketService {
    type Context = Context<Self>;
}

impl Handler<ConnectService> for WebsocketService {
    type Result = Option<ServiceDataFeed>;

    fn handle(
        &mut self, 
        msg: ConnectService, 
        ctx: &mut Self::Context
    ) -> Self::Result {

        self.service.user_request(msg.msg, &self.conn_pool)
    }
}

pub trait Service {

    fn user_request(
        &self,
        msg: String,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<ServiceDataFeed>;

    fn id(&self) -> u32;
}

#[derive(Message)]
#[rtype(result = "Option<ServiceDataFeed>")]
pub struct ConnectService {
    pub session: Addr<WebsocketSession>,
    pub user_access_level: u8,
    pub msg: String,
}