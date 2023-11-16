use std::{time::Instant, collections::HashMap};

use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::info;
use serde::{Deserialize, Serialize};

use crate::user_service::user::UserSession;

use super::{
    WebsocketServer,
    Disconnect, 
    Connect, 
    ConnectionResponse::*, 
    ServiceRequest, service::{WebsocketService, ServiceFrame}
};



/// Websocket session for client - server communication.
pub struct WebsocketSession {

    /// User session of the user connected to this socket.
    pub user: UserSession,

    /// Parent server connection.
    pub server: Addr<WebsocketServer>,

    // Service feed handlers.
    //pub service_feeds: HashMap<u32, Addr<dyn WebsocketService>>,

    // Timestamp of the latest message from client socket.
    pub last_activity: Instant,
}

impl WebsocketSession {

    /// Getter for the session id
    pub fn id(&self) -> u32 {
        self.user.id
    }

    pub fn access(&self) -> u8 {
        self.user.access_level
    }

    fn request_service(
        &self, 
        msg: MessageFrame,
    ) {
        let _ = self.server
        .try_send(ServiceRequest {
            service_id: msg.s,
            user_id: self.id(),
            user_access_level: self.access(),
            msg: msg.r,
        });
    }

    fn add_feed(
        &mut self,
        srvc_id: u32,
        //srvc: Addr<dyn WebsocketService>,
    ) {
        //TODO: feed limiter
        //self.service_feeds.insert(srvc_id, srvc);
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

       // TODO: join to user service by default
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
        let _ = serde_json::to_string(
            &MessageFrame {
                s: msg.service_id,
                r: msg.response_message,
            }
        )
        .map(|resp| ctx.text(resp));
    }
}

impl Handler<ServiceFeedResponse> for WebsocketSession {
    type Result = ();

    fn handle(
        &mut self, 
        msg: ServiceFeedResponse, 
        ctx: &mut Self::Context
    ) -> Self::Result {
        //self.add_feed(msg.service_id, msg.service_handler);

        msg.response_message
        .and_then(|message| {
            serde_json::to_string(
                &MessageFrame {
                    s: msg.service_id,
                    r: message,
                }
            ).ok()
        })
        .map(|resp| ctx.text(resp));
    }
}

#[derive(Serialize, Deserialize)]
pub struct MessageFrame {
    s: u32,
    r: ServiceFrame,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServiceFeedResponse {
    pub service_id: u32,
    //pub service_handler: Addr<dyn WebsocketService>,
    pub response_message: Option<ServiceFrame>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServiceResponse {
    pub service_id: u32,
    pub response_message: ServiceFrame,
}