use std::time::Instant;

use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::info;

use crate::user_service::user::UserSession;

use super::{WebsocketServer, Disconnect, Connect, ConnectionResponse::*};



/// Websocket session for client - server communication.
pub struct WebsocketSession {

    /// User session of the user connected to this socket.
    pub user: UserSession,

    /// Parent server connection.
    pub server: Addr<WebsocketServer>,

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

    // websocket connection is opened
    fn started(&mut self, context: &mut Self::Context) {

        // Register session to server.
        self.server
        .send(Connect {
            session_id: self.id(),
            session_address: context.address(), 
        })
        .into_actor(self)
        .then(|conn_res, act, ctx| {

            // TODO: look into actor mailbox errors
            let mut connection_response = conn_res.ok();

            match connection_response.get_or_insert(Blocked) {
                Connected => {
                    info!("User session {} connected.", act.id());
                },

                Reconnected => {
                    info!("User session {} reconnected.", act.id());
                },

                ServerFull => {
                    info!(
                        "User session {} blocked. Not enough server capacity", 
                        act.id()
                    );

                    ctx.stop();
                },

                Blocked => {
                    info!("User session {} blocked.", act.id());

                    ctx.stop();
                },
            }

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

// Handle incoming messages from the client
impl StreamHandler<Result<Message, ProtocolError>> for WebsocketSession {
    
    fn handle(
        &mut self, 
        msg: Result<Message, ProtocolError>, 
        ctx: &mut Self::Context,
    ) {
        let _ = msg
        .map_err(|_err| {
            ctx.stop();
        })
        .map(|msg| {
            self.last_activity = Instant::now();

            match msg {
                Message::Text(text) => {
                    info!("Text received from the user: {}", text);
                },

                Message::Binary(bin) => {
                    info!("binary with size {} from the user.", bin.len());
                },

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