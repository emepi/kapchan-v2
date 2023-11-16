use std::{collections::HashMap, sync::{Arc, Mutex}};

use actix::{
    Handler, 
    Actor, 
    Context, 
    Addr, 
    Message, 
    ResponseFuture
};
use async_trait::async_trait;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use log::info;
use serde::{Serialize, Deserialize};

use super::session::{WebsocketSession, ServiceFeedResponse};


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
    ) -> ServiceResponse;

    fn id(&self) -> u32;
}

pub struct WebsocketServiceManager {
    pub subs: HashMap<u32, Addr<WebsocketSession>>,
    pub max_subs: usize,
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

        Box::pin(async move {
            let resp = srvc.user_request(
                msg.session_id, 
                msg.user_access_level, 
                msg.service_request
            )
            .await;

            let _ = msg.session.try_send(ServiceFeedResponse {
                service_id: srvc.id(),
                response_message: Some(resp.resp),
            });
        
            Ok(())
        })
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ServiceFrame {
    pub t: u32,
    pub b: String,
}

pub struct ServiceResponse {
    pub data_feed: bool,
    pub resp: ServiceFrame,
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct ConnectService {
    pub session: Addr<WebsocketSession>,
    pub session_id: u32,
    pub user_access_level: u8,
    pub service_request: ServiceFrame,
}