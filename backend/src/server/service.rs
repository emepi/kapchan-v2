use std::collections::HashMap;

use actix::{
    Handler, 
    Actor, 
    Context, 
    Addr, 
    Message, 
    AsyncContext,
};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};

use super::session::{WebsocketSession, ServiceFeedResponse, ServiceResponse};

pub struct FeedControl {
    pub op_allowed: bool,
    pub response: Option<String>,
}

pub struct WebsocketService {
    pub id: u32,
    pub service: Box<dyn Service>,
    pub subcribers: HashMap<u32, Addr<WebsocketSession>>,
    pub conn_pool: Pool<AsyncMysqlConnection>,
}

impl WebsocketService {

}

impl Actor for WebsocketService {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        
    }
}

impl Handler<ConnectService> for WebsocketService {
    type Result = ();

    fn handle(
        &mut self, 
        msg: ConnectService, 
        ctx: &mut Self::Context
    ) -> Self::Result {
        let feed_ctrl = self.service.connection_request(
            msg.msg,
            msg.session_id,
            msg.user_access_level, 
            &self.conn_pool
        );

        match feed_ctrl.op_allowed {
            true => {
                let _ = &msg.session.try_send(ServiceFeedResponse {
                    service_id: self.service.id(),
                    service_handler: ctx.address(),
                    response_message: feed_ctrl.response,
                });

                // TODO: check and handle existing
                self.subcribers.insert(msg.session_id, msg.session);
            },

            false => {
                let _ = &msg.session.try_send(ServiceResponse {
                    service_id: self.service.id(),
                    response_message: feed_ctrl.response.unwrap_or_default(),
                });
            },
        };
    }
}

pub trait Service {

    fn connection_request(
        &self,
        msg: String,
        session_id: u32,
        session_access: u8,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> FeedControl;

    fn id(&self) -> u32;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectService {
    pub session: Addr<WebsocketSession>,
    pub session_id: u32,
    pub user_access_level: u8,
    pub msg: String,
}