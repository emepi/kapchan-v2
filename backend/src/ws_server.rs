pub mod session;


use std::collections::HashMap;

use actix::prelude::*;


pub struct WsServer {
    sessions: HashMap<u32, Recipient<WsTask>>,
}

impl WsServer {
    pub fn new() -> Self {
        WsServer { 
            sessions: HashMap::new(),
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
    type Result = ();

    fn handle(
        &mut self,
        msg: Connect,
        ctx: &mut Self::Context
    ) -> Self::Result {

        //self.sessions.insert(id, msg.server);

        //id
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub session: Recipient<WsTask>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsTask(pub String);