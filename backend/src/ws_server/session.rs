use std::time::Instant;

use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};

use crate::user::UserSession;

use super::{WsServer, Disconnect, WsTask, Connect};



/// Websocket session for client - server communication.
pub struct WebsocketSession {

    /// User session of the user connected to this socket.
    pub user: UserSession,

    /// Parent server connection.
    pub server: Addr<WsServer>,

    // TODO: service feeds
    // pub service_feeds: Vec<impl Service>,

    // Timestamp of the latest message from client socket.
    pub last_activity: Instant,
}

impl WebsocketSession {

    /// Getter for the session id.
    pub fn id(&self) -> u32 {
        self.user.id
    }
}

impl Actor for WebsocketSession {
    type Context = WebsocketContext<Self>;

    // connection is opened
    fn started(&mut self, context: &mut Self::Context) {

        // Register session to the server
        let session_actor = context.address();

        self.server
        .send(Connect { session: session_actor.recipient() })
        .into_actor(self)
        .then(|res, act, ctx| {
            
            //match res {
            //    Ok(res) => act.id = res,
            //
            //    _ => ctx.stop(),
            //}
            fut::ready(())
        })
        .wait(context);

    }

    // connection is closed
    fn stopping(&mut self, _: &mut Self::Context) -> Running {

        // Disconnect session from the server and service feeds
        self.server.do_send(Disconnect { id: self.id() });

        Running::Stop
    }
}

// Handle outgoing messages
impl Handler<WsTask> for WebsocketSession {
    type Result = ();

    fn handle(&mut self, msg: WsTask, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// Handle incoming messages from the client
impl StreamHandler<Result<Message, ProtocolError>> for WebsocketSession {
    
    fn handle(
        &mut self, 
        msg: Result<Message, ProtocolError>, 
        ctx: &mut Self::Context,
    ) {
        msg
        .map_err(|_err| {
            ctx.stop();
        })
        .map(|msg| {
            self.last_activity = Instant::now();

            match msg {
                Message::Text(_) => (),

                Message::Binary(_) => (),

                Message::Continuation(_) => {
                    ctx.stop();
                }

                Message::Ping(payload) => {
                    ctx.pong(&payload);
                },

                Message::Pong(_) => (),

                Message::Close(reason) => {
                    ctx.close(reason);
                    ctx.stop();
                },

                Message::Nop => (),
            }
        });
    }
}