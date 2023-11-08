use actix::{Actor, StreamHandler};
use actix_web_actors::ws;


pub struct WsServer {

}

pub struct WsSession {

}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(
        &mut self, 
        item: Result<ws::Message, ws::ProtocolError>, 
        ctx: &mut Self::Context,
    ) {
        todo!()
    }
}