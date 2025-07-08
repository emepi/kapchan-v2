use std::{collections::HashMap, io};

use rand::Rng;
use serde_json::json;
use tokio::sync::{mpsc, oneshot};


const USER_JOINED: u8 = 1;
const USER_LEFT: u8 = 2;
pub const NEW_MESSAGE: u8 = 3;

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

pub type ConnId = u64;
pub type Room = String;
pub type User = String;
pub type Msg = String;

#[derive(Debug)]
enum Command {
    Connect {
        user: User,
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
    },

    Disconnect {
        conn: ConnId,
    },

    ListRoom {
        res_tx: oneshot::Sender<Vec<Room>>,
    },

    ListUser {
        res_tx: oneshot::Sender<Vec<User>>,
    },

    Message {
        msg: Msg,
        res_tx: oneshot::Sender<()>,
    },
}

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,
    users: HashMap<ConnId, User>,
    rooms: Vec<Room>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl ChatServer {
    pub fn new(rooms: Vec<Room>) -> (Self, ChatServerHandle) {

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                users: HashMap::new(),
                rooms,
                cmd_rx,
            },
            ChatServerHandle { cmd_tx },
        )
    }

    async fn send_message(&self, msg: impl Into<Msg>) {
        let msg = msg.into();

        for (_, session) in self.sessions.iter() {
            let _ = session.send(msg.clone());
        }
    }

    async fn connect(&mut self, user: User, tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        self.send_message(json!(UsersChanged { 
            event: USER_JOINED, 
            username: &user,
        }).to_string()).await;

        let id = rand::rng().random::<ConnId>();
        self.sessions.insert(id, tx);
        self.users.insert(id, user);

        id
    }

    async fn disconnect(&mut self, conn_id: ConnId) {
        self.sessions.remove(&conn_id);

        if let Some(username) = self.users.remove(&conn_id) {
            self.send_message(&json!(UsersChanged { 
                event: USER_LEFT, 
                username: &username, 
            }).to_string()).await;
        }
    }

    fn list_rooms(&self) -> Vec<Room> {
        self.rooms.clone()
    }

    fn list_users(&self) -> Vec<User> {
        let mut usernames = Vec::new();

        for (_, username) in self.users.iter() {
            usernames.push(username.to_owned());
        }

        usernames
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn_tx, res_tx, user } => {
                    let conn_id = self.connect(user, conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }

                Command::ListRoom { res_tx } => {
                    let _ = res_tx.send(self.list_rooms());
                }

                Command::ListUser { res_tx } => {
                    let _ = res_tx.send(self.list_users());
                }

                Command::Message { msg, res_tx } => {
                    self.send_message(msg).await;
                    let _ = res_tx.send(());
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChatServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ChatServerHandle {
    pub async fn connect(&self, user: User, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Connect { user, conn_tx, res_tx })
            .unwrap();

        res_rx.await.unwrap()
    }

    pub async fn list_rooms(&self) -> Vec<Room> {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx.send(Command::ListRoom { res_tx }).unwrap();

        res_rx.await.unwrap()
    }

    pub async fn list_users(&self) -> Vec<User> {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx.send(Command::ListUser { res_tx }).unwrap();

        res_rx.await.unwrap()
    }

    pub async fn send_message(&self, msg: impl Into<Msg>) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                res_tx,
            })
            .unwrap();

        res_rx.await.unwrap();
    }

    pub fn disconnect(&self, conn: ConnId) {
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}