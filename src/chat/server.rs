use std::collections::HashMap;

use actix::{Actor, Context, Handler, Message, MessageResult, Recipient};
use serde_json::json;


const USER_JOINED: u8 = 1;
const USER_LEFT: u8 = 2;
const NEW_MESSAGE: u8 = 3;

#[derive(serde::Serialize)]
pub struct UsersChanged<'a> {
    pub event: u8,
    pub username: &'a str,
}

#[derive(serde::Serialize)]
pub struct MessagesChanged<'a> {
    pub event: u8,
    pub username: &'a str,
    pub message: &'a str,
    pub room: &'a str,
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: u64,
    pub message: String,
    pub room: String,
}

#[derive(Message)]
#[rtype(u64)]
pub struct Connect {
    pub addr: Recipient<ServerMessage>,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: u64,
}

#[derive(Message)]
#[rtype(result = "Vec<String>")]
pub struct ListRooms;

#[derive(Message)]
#[rtype(result = "Vec<String>")]
pub struct ListUsers;


#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<u64, Recipient<ServerMessage>>,
    users: HashMap<u64, String>,
    rooms: Vec<String>,
}

impl ChatServer {
    pub fn new(
        rooms: Vec<String>
    ) -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            users: HashMap::new(),
            rooms,
        }
    }

    fn send_message(
        &self,
        message: &str,
    ) {
        for (_, session) in self.sessions.iter() {
            session.do_send(ServerMessage(message.to_owned()));
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = u64;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {

        self.send_message(&json!(UsersChanged { 
            event: USER_JOINED, 
            username: &msg.username, 
        }).to_string());


        let id = self.sessions.keys().len();
        self.sessions.insert(id.try_into().unwrap(), msg.addr);
        self.users.insert(id.try_into().unwrap(), msg.username);
        
        id.try_into().unwrap()
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);

        if let Some(username) = self.users.remove(&msg.id) {
            self.send_message(&json!(UsersChanged { 
                event: USER_LEFT, 
                username: &username, 
            }).to_string());
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&json!(MessagesChanged { 
            event: NEW_MESSAGE, 
            username: match self.users.get(&msg.id) {
                Some(usr_name) => usr_name,
                None => "",
            }, 
            message: &msg.message, 
            room: &msg.room, 
        }).to_string());
    }
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        MessageResult(self.rooms.clone())
    }
}

impl Handler<ListUsers> for ChatServer {
    type Result = MessageResult<ListUsers>;

    fn handle(&mut self, _: ListUsers, _: &mut Context<Self>) -> Self::Result {
        let mut usernames = Vec::new();

        for (_, username) in self.users.iter() {
            usernames.push(username.to_owned());
        }

        MessageResult(usernames)
    }
}