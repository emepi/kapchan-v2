use std::{time::Instant, collections::HashMap, sync::{Arc, Mutex}};

use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::info;
use serde::{Deserialize, Serialize};

use crate::user_service::session::UserSession;

use super::{
    WebsocketServer,
    Disconnect, 
    Connect, 
    ConnectionResponse::*, 
    ServiceRequest, service::{ServiceFrame, WebsocketServiceActor}, Reconnect
};


/// Websocket session for client - server communication.
pub struct WebsocketSession {

    /// User session of the user connected to this socket.
    pub user: Arc<UserSession>,

    /// Parent server connection.
    pub server: Addr<WebsocketServer>,

    // Service feed handlers.
    pub service_feeds: HashMap<u32, Addr<WebsocketServiceActor>>,

    // Timestamp of the latest message from client socket.
    pub last_activity: Instant,
}

impl WebsocketSession {

    fn request_service(
        &self, 
        msg: MessageFrame,
    ) {
        info!("Sending a srvc req");
        let _ = self.server
        .try_send(ServiceRequest {
            service_id: msg.s,
            sess: self.user.clone(),
            msg: msg.r,
        });
    }

    fn add_feed(
        &mut self,
        srvc_id: u32,
        srvc: Addr<WebsocketServiceActor>,
    ) {
        //TODO: feed limiter
        self.service_feeds.insert(srvc_id, srvc);
    }

    fn drop_feed(
        &mut self,
        srvc_id: u32,
    ) {
        self.service_feeds.remove(&srvc_id);
    }

    fn upgrade_session(&mut self, sess: Arc<UserSession>) {

        let _ = self.server.try_send(Reconnect {
            from_session_id: self.user.id,
            to_session_id: sess.id,
        });

        self.service_feeds.values().for_each(|srvc| {
            srvc.try_send(Reconnect {
                from_session_id: self.user.id,
                to_session_id: sess.id,
            });
        });

        self.user = sess;
    }
}

impl Actor for WebsocketSession {
    type Context = WebsocketContext<Self>;

    // websocket connection is opened
    fn started(&mut self, context: &mut Self::Context) {

        // Register session to server.
        self.server
        .send(Connect {
            session_id: self.user.id,
            session_address: context.address(), 
        })
        .into_actor(self)
        .then(|conn_res, act, ctx| {

            // TODO: look into actor mailbox errors
            let mut connection_response = conn_res.ok();

            match connection_response.get_or_insert(Blocked) {
                Connected => {

                },

                Reconnected => {

                },

                ServerFull => {

                    ctx.stop();
                },

                Blocked => {

                    ctx.stop();
                },
            }

            fut::ready(())
        })
        .wait(context);

       // TODO: join to user service by default
    }

    // connection is closed
    fn stopping(&mut self, _: &mut Self::Context) -> Running {

        // Disconnect session from the server and service feeds
        self.server.do_send(Disconnect { id: self.user.id });

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

                    let _ = serde_json::from_str(&text)
                    .map(|msg| self.request_service(msg));
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

impl Handler<ServiceResponse> for WebsocketSession {
    type Result = ();

    fn handle(
        &mut self, 
        msg: ServiceResponse, 
        ctx: &mut Self::Context
    ) -> Self::Result {

        match serde_json::to_string(&MessageFrame {
            s: msg.srvc_id,
            r: msg.srvc_frame,
        }) {
            Ok(msg_frame) => ctx.text(msg_frame),
            Err(_) => (),
        };
    }
}

impl Handler<ServiceConnection> for WebsocketSession {
    type Result = ();

    fn handle(
        &mut self, 
        msg: ServiceConnection, 
        ctx: &mut Self::Context
    ) -> Self::Result {
        self.add_feed(msg.srvc_id, msg.srvc_addr);
    }
}

impl Handler<ServiceClose> for WebsocketSession {
    type Result = ();

    fn handle(
        &mut self, 
        msg: ServiceClose, 
        ctx: &mut Self::Context
    ) -> Self::Result {
        self.drop_feed(msg.srvc_id);
    }
}

impl Handler<UpgradeSession> for WebsocketSession {
    type Result = ();

    fn handle(
        &mut self, 
        msg: UpgradeSession, 
        ctx: &mut Self::Context
    ) -> Self::Result {
        self.upgrade_session(msg.sess);
    }
}

#[derive(Serialize, Deserialize)]
pub struct MessageFrame {
    s: u32,
    r: ServiceFrame,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServiceResponse {
    pub srvc_id: u32,
    pub srvc_frame: ServiceFrame,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServiceConnection {
    pub srvc_id: u32,
    pub srvc_addr: Addr<WebsocketServiceActor>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServiceClose {
    pub srvc_id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpgradeSession {
    pub sess: Arc<UserSession>,
}