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
use log::info;
use serde::{Serialize, Deserialize};

use crate::user_service::session::UserSession;

use super::{session::{
    WebsocketSession, 
    ServiceResponse, 
    ServiceConnection, 
    ServiceClose
}, Reconnect, ConnectionResponse};


#[async_trait]
pub trait WebsocketService {
    fn new(
        srvc_mgr: Arc<Mutex<WebsocketServiceManager>>,
        conn_pool: Pool<AsyncMysqlConnection>,
    ) -> Self where Self: Sized;

    async fn user_request(
        &self,
        sess: &Arc<UserSession>,
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

    pub fn get_client(&self, sess_id: u32) -> Option<&Addr<WebsocketSession>> {
        self.subs.get(&sess_id)
    }

    pub fn reconnect(
        &mut self, 
        from_sess_id: u32, 
        to_sess_id: u32
    ) -> ConnectionResponse {

        let prev = self.subs.remove(&from_sess_id);

        match prev {
            Some(sess_addr) => {
                self.subs.insert(to_sess_id, sess_addr);

                ConnectionResponse::Reconnected
            },

            None => ConnectionResponse::Blocked,
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
        info!("Srvc request received.");

        let srvc = self.srvc.clone();
        let srvc_mgr = self.srvc_mgr.clone();
        let srvc_addr = ctx.address().clone();

        Box::pin(async move {

            match  srvc_mgr.lock() {
                Ok(mut mgr) => {
                    mgr.add_client(
                        msg.session.id, 
                        msg.session_actor.clone(),
                        srvc.id(),
                        srvc_addr
                    );
                },

                Err(_) => {
                    // TODO
                },
            };

            let resp = srvc.user_request(
                &msg.session, 
                msg.service_request
            )
            .await;
            
            let _ = &msg.session_actor.try_send(ServiceResponse {
                srvc_id: srvc.id(),
                srvc_frame: resp,
            });
        
            Ok(())
        })
    }
}

impl Handler<Reconnect> for WebsocketServiceActor {
    type Result = ConnectionResponse;

    fn handle(
        &mut self, 
        msg: Reconnect, 
        ctx: &mut Self::Context
    ) -> Self::Result {

        self.srvc_mgr.lock().unwrap()
        .reconnect(msg.from_session_id, msg.to_session_id)
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
    pub session: Arc<UserSession>,
    pub session_actor: Addr<WebsocketSession>,
    pub service_request: ServiceFrame,
}