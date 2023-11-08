use std::{
    collections::HashMap, 
    time::{Instant, Duration}, 
};

use actix::prelude::*;
use actix_web_actors::ws;


const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

const TIMEOUT: Duration = Duration::from_secs(10);


pub struct WsServer {
    sessions: HashMap<usize, Recipient<WsTask>>,
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
    type Result = usize;

    fn handle(
        &mut self,
        msg: Connect,
        ctx: &mut Self::Context
    ) -> Self::Result {
        let id = self.sessions.len();

        self.sessions.insert(id, msg.server);

        id
    }
}

pub struct WsSession {
    pub id: usize,
    pub heartbeat: Instant,
    pub server: Addr<WsServer>,
}

impl WsSession {
    fn heartbeat(&self, context: &mut ws::WebsocketContext<Self>) {

        context.run_interval(HEARTBEAT_INTERVAL, |actor, context| {

            if Instant::now().duration_since(actor.heartbeat) > TIMEOUT {

                actor.server.do_send(Disconnect { id: actor.id });

                context.stop();
            }

            return;
        });

        context.ping(b"");
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;


    fn started(&mut self, context: &mut Self::Context) {

        self.heartbeat(context);

        // register to server

        let addr = context.address();

        self.server
            .send(Connect {
                server: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,

                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(context);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<WsTask> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsTask, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(
        &mut self, 
        msg: Result<ws::Message, ws::ProtocolError>, 
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Text(_) => (),
            ws::Message::Binary(_) => (),
            ws::Message::Continuation(_) => ctx.stop(),
            ws::Message::Ping(m) => {
                self.heartbeat = Instant::now();
                ctx.pong(&m);
            },
            ws::Message::Pong(_) => {
                self.heartbeat = Instant::now();
            },
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            },
            ws::Message::Nop => (),
        }
    }
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub server: Recipient<WsTask>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsTask(pub String);