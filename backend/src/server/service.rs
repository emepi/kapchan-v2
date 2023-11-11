use actix::{Message, Addr, Handler, Actor, Context};

use super::session::WebsocketSession;


#[derive(Message)]
#[rtype(result = "Option<ServiceDataFeed>")]
pub struct ServiceRequest {
    pub subscriber: Addr<WebsocketSession>,
}

pub struct ServiceDataFeed {

}

pub struct WebsocketService {
    pub id: u32,
    pub service: Box<dyn Service>,
}

impl WebsocketService {
    pub fn new(id: u32, service: Box<dyn Service>) -> Self {
        WebsocketService {
            id,
            service,
        }
    }
}

impl Actor for WebsocketService {
    type Context = Context<Self>;
}

impl Handler<ServiceRequest> for WebsocketService {
    type Result = Option<ServiceDataFeed>;

    fn handle(
        &mut self, 
        msg: ServiceRequest, 
        ctx: &mut Self::Context
    ) -> Self::Result {

        self.service.data_feed()
    }
}

pub trait Service {

    fn data_feed(&self) -> Option<ServiceDataFeed>;

    fn id(&self) -> u32;
}