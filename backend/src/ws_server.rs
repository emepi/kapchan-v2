pub mod session;


use std::collections::HashMap;

use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::*;

use self::session::WebsocketSession;


// TODO: profile mem use
pub struct WsServer {
    pub sessions: HashMap<u32, Addr<WebsocketSession>>,

    pub sessions_limit: usize,
}

impl WsServer {
    pub fn new() -> Self {
        WsServer { 
            sessions: HashMap::with_capacity(100),
            sessions_limit: 100,
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(
        &mut self, 
        msg: Disconnect, 
        ctx: &mut Self::Context
    ) -> Self::Result {

        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for WsServer {
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsTask(pub String);