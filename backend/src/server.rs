pub mod session;
pub mod service;


use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::*;
use diesel_async::AsyncMysqlConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use log::info;

use crate::user_service::session::UserSession;

use self::service::{
    WebsocketService, 
    ConnectService, 
    WebsocketServiceActor, 
    WebsocketServiceManager, ServiceRequestFrame
};
use self::session::WebsocketSession;


// TODO: profile mem use
pub struct WebsocketServer {
    pub sessions: HashMap<u32, Addr<WebsocketSession>>,

    pub sessions_limit: usize,

    pub services: HashMap<u32, Recipient<ConnectService>>,

    pub database: Pool<AsyncMysqlConnection>,
}

impl WebsocketServer {
    pub fn new(settings: ServerSettings) -> Self {
        WebsocketServer { 
            sessions: HashMap::with_capacity(settings.max_sessions),
            sessions_limit: settings.max_sessions,
            services: HashMap::new(),
            database: settings.database,
        }
    }

    pub fn service<S>(mut self, srvc_id: u32) -> Self 
    where
        S: WebsocketService + 'static,
    {
        let srvc_mgr = Arc::new(
            Mutex::new(WebsocketServiceManager {
                subs: HashMap::new(),
                max_subs: self.sessions_limit,
            })
        );

        let srvc = S::new(srvc_mgr.clone(), self.database.clone());


        let service_actor = WebsocketServiceActor {
            srvc: Arc::new(srvc),
            srvc_mgr: srvc_mgr.clone(),
        }
        .start();

        self.services.insert(srvc_id, service_actor.recipient());
        self
    }
}

impl Actor for WebsocketServer {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for WebsocketServer {
    type Result = ();

    fn handle(
        &mut self, 
        msg: Disconnect, 
        _ctx: &mut Self::Context
    ) -> Self::Result {

        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for WebsocketServer {
    type Result = ConnectionResponse;

    fn handle(
        &mut self,
        msg: Connect,
        _ctx: &mut Self::Context
    ) -> Self::Result {

        if self.sessions.len() >= self.sessions_limit {
            // TODO: check if other connections can be purged for this user.
            return ConnectionResponse::ServerFull;
        }

        // Accept session
        match self.sessions.insert(msg.session_id, msg.session_address) {
            Some(previous_session) => {
                ConnectionResponse::Reconnected
            },

            None => {
                info!("Session id: {} connected to server.", msg.session_id);
                ConnectionResponse::Connected
            },
        }
    }
}

impl Handler<ServiceRequest> for WebsocketServer {
    type Result = ConnectionResponse;

    fn handle(
        &mut self, 
        msg: ServiceRequest, 
        _ctx: &mut Self::Context
    ) -> Self::Result {

        let session_actor = match self.sessions.get(&msg.sess.id) {
            Some(session) => session.clone(),
            None => return ConnectionResponse::Blocked,
        };

        self.services
        .get(&msg.service_id)
        .and_then(|service| {
            service.try_send(ConnectService {
                service_request: msg.msg,
                session: msg.sess,
                session_actor, 
            })
            .ok()
        })
        .map(|_| ConnectionResponse::Connected)
        .unwrap_or(ConnectionResponse::Blocked)
        
    }
}

impl Handler<Reconnect> for WebsocketServer {
    type Result = ConnectionResponse;

    fn handle(
        &mut self, 
        msg: Reconnect, 
        _ctx: &mut Self::Context
    ) -> Self::Result {
        let prev = self.sessions.remove(&msg.from_session_id);

        match prev {
            Some(sess_addr) => {
                self.sessions.insert(msg.to_session_id, sess_addr);

                ConnectionResponse::Reconnected
            },

            None => ConnectionResponse::Blocked,
        }
    }
}

pub struct ServerSettings {
    pub max_sessions: usize,
    pub database: Pool<AsyncMysqlConnection>,
}

pub enum ConnectionResponse {
    Connected,
    Reconnected,
    ServerFull,
    Blocked,
}

impl<A, M> MessageResponse<A, M> for ConnectionResponse
where
    A: Actor,
    M: Message<Result = ConnectionResponse>,
{
    fn handle(
        self, 
        _ctx: &mut <A as Actor>::Context, 
        tx: Option<OneshotSender<<M as Message>::Result>>
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

#[derive(Message)]
#[rtype(result = "ConnectionResponse")]
pub struct Connect {
    pub session_id: u32,
    pub session_address: Addr<WebsocketSession>,
}

#[derive(Message)]
#[rtype(result = "ConnectionResponse")]
pub struct Reconnect {
    pub from_session_id: u32,
    pub to_session_id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: u32,
}

#[derive(Message)]
#[rtype(result = "ConnectionResponse")]
pub struct ServiceRequest {
    pub service_id: u32,
    pub sess: Arc<UserSession>,
    pub msg: ServiceRequestFrame,
}