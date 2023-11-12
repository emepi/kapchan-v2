pub mod session;
pub mod service;


use std::collections::HashMap;

use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::*;
use diesel_async::AsyncMysqlConnection;
use diesel_async::pooled_connection::deadpool::Pool;

use self::service::{ServiceRequest, WebsocketService, Service};
use self::session::WebsocketSession;


// TODO: profile mem use
pub struct WebsocketServer {
    pub sessions: HashMap<u32, Addr<WebsocketSession>>,

    pub sessions_limit: usize,

    pub services: HashMap<u32, Recipient<ServiceRequest>>,

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

    pub fn service(mut self, service: Box<dyn Service>) -> Self {

        let id = service.id();
        let service_addr = WebsocketService {
            id,
            service,
            conn_pool: self.database.clone(),
        }
        .start();
        
        self.services.insert(id, service_addr.recipient());
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
        ctx: &mut Self::Context
    ) -> Self::Result {

        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for WebsocketServer {
    type Result = ConnectionResponse;

    fn handle(
        &mut self,
        msg: Connect,
        ctx: &mut Self::Context
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
                ConnectionResponse::Connected
            },
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
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: u32,
}