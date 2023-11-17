use std::{collections::HashMap, sync::{Arc, Mutex}};

use actix::{
    Handler, 
    Actor, 
    Context, 
    Addr, 
    Message, 
    ResponseFuture, AsyncContext
};
use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use serde::{Serialize, Deserialize};

use super::session::{WebsocketSession, ServiceResponse, ServiceConnection, ServiceClose};


#[async_trait]
pub trait WebsocketService {
    fn new(
        srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
        conn_pool: Pool<AsyncMysqlConnection>,
    ) -> Self where Self: Sized;

    async fn user_request(
        &self,
        usr_id: u32, 
        usr_access: u8,
        req: ServiceFrame,
    ) -> ServiceFrame;

    fn id(&self) -> u32;
}

pub struct WebsocketServiceManager {
    pub subs: HashMap<u32, Addr<WebsocketSession>>,
    pub max_subs: usize,
}

impl WebsocketServiceManager {
    pub fn add_client(
        &mut self, 
        sess_id: u32, 
        sess: Addr<WebsocketSession>,
        srvc_id: u32,
        srvc: Addr<WebsocketServiceActor>,
    ) {
        if self.subs.len() < self.max_subs {

            match self.subs.insert(sess_id, sess.clone()) {

                Some(prev) => {
                    let _ = prev.try_send(ServiceClose {
                        srvc_id,
                    });
                },

                None => {
                    let _ = sess.try_send(ServiceConnection {
                        srvc_id,
                        srvc_addr: srvc,
                    });
                },
            }
        }
    }


}

pub struct WebsocketServiceActor {
    pub srvc: Arc<dyn WebsocketService>,
    pub srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
}

impl Actor for WebsocketServiceActor {
    type Context = Context<Self>;
}

impl Handler<ConnectService> for WebsocketServiceActor {
    type Result = ResponseFuture<Result<(),()>>;

    fn handle(
        &mut self, 
        msg: ConnectService, 
        ctx: &mut Context<Self>,
    ) -> Self::Result {

        let srvc = self.srvc.clone();
        let srvc_mgr = self.srvc_mgr.clone();
        let srvc_addr = ctx.address().clone();

        Box::pin(async move {
            let resp = srvc.user_request(
                msg.session_id, 
                msg.user_access_level, 
                msg.service_request
            )
            .await;
            
            let _ = &msg.session.try_send(ServiceResponse {
                srvc_id: srvc.id(),
                srvc_frame: resp,
            });

            match  srvc_mgr.lock() {
                Ok(mut mgr) => {
                    mgr.add_client(
                        msg.session_id, 
                        msg.session,
                        srvc.id(),
                        srvc_addr
                    );
                },

                Err(_) => {
                    // TODO
                },
            };
        
            Ok(())
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Serialize, Deserialize, Default)]
pub struct ServiceFrame {
    pub t: u32,
    pub b: String,
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct ConnectService {
    pub session: Addr<WebsocketSession>,
    pub session_id: u32,
    pub user_access_level: u8,
    pub service_request: ServiceFrame,
}