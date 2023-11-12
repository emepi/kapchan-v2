use actix::{Message, Addr, Handler, Actor, Context};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use super::session::WebsocketSession;


#[derive(Message)]
#[rtype(result = "Option<ServiceDataFeed>")]
pub struct ServiceRequest {
    pub session: Addr<WebsocketSession>,
}

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

impl Handler<ServiceRequest> for WebsocketService {
    type Result = Option<ServiceDataFeed>;

    fn handle(
        &mut self, 
        msg: ServiceRequest, 
        ctx: &mut Self::Context
    ) -> Self::Result {

        self.service.data_feed(&self.conn_pool)
    }
}

pub trait Service {

    fn data_feed(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<ServiceDataFeed>;

    fn id(&self) -> u32;
}