use std::time::{Duration, Instant};

use actix_ws::AggregatedMessage;
use chrono::Utc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{sync::mpsc, time::interval};

use crate::models::users::AccessLevel;

use super::server::{ChatServerHandle, ConnId, MessagesChanged, User, NEW_MESSAGE};


const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


pub async fn chat_ws(
    user: User,
    access_level: u8,
    chat_server: ChatServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    let conn_id = chat_server.connect(user.clone(), conn_tx).await;

    let mut msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let close_reason = loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        process_text_msg(conn_id, user.clone(), access_level, &chat_server, &mut session, &text)
                            .await;
                    }

                    AggregatedMessage::Binary(_bin) => {}

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            Some(chat_msg) = conn_rx.recv() => {
                session.text(chat_msg).await.unwrap();
            }

            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    break None;
                }
                let _ = session.ping(b"").await;
            }

            else => {
                break None;
            }
        }
    };

    chat_server.disconnect(conn_id);

    let _ = session.close(close_reason).await;
}


const UNKNOWN_COMMAND: u8 = 0;
const SEND_MESSAGE: u8 = 1;
const LIST_ROOMS: u8 = 2;
const LIST_USERS: u8 = 3;
const TOO_LONG_MESSAGE_ERROR: u8 = 7;

#[derive(Deserialize, Debug, Clone)]
pub struct InputCommand {
    event: u8,
    message: Option<String>,
    room: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct OutputCommand {
    event: u8,
    data: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct ErrorOutput {
    event: u8,
    message: String,
}

async fn process_text_msg(
    id: ConnId,
    user: User,
    access_level: u8,
    chat_server: &ChatServerHandle,
    session: &mut actix_ws::Session,
    text: &str,
) {
    let command = match serde_json::from_str::<InputCommand>(text) {
        Ok(command) => command,
        Err(_) => {
            InputCommand {
                event: UNKNOWN_COMMAND,
                message: None,
                room: None,
            }
        },
    };

    match command.event {
        LIST_USERS => {
            let users = chat_server.list_users().await;

            session
                .text(json!(OutputCommand { 
                    event: 4, 
                    data: users 
                }).to_string())
                .await
                .unwrap();
        },

        LIST_ROOMS => {
            let rooms = chat_server.list_rooms().await;

            session
                .text(json!(OutputCommand { 
                    event: 5, 
                    data: rooms 
                }).to_string())
                .await
                .unwrap();
        },

        SEND_MESSAGE => {
            let msg = command.clone().message.unwrap_or_default().trim().to_owned();

            if msg.len() > 2000 {
                session
                .text(json!(ErrorOutput { 
                    event: TOO_LONG_MESSAGE_ERROR, 
                    message: "Viestisi on liian pitkä (yli 2000 tavua)!".to_owned(), 
                }).to_string())
                .await
                .unwrap();

                return;
            }

            if msg.starts_with('/') {
                let mut cmd_args = msg.splitn(3, ' ');

                match cmd_args.next().unwrap() {
                    "/timeout" => match cmd_args.next() {
                        Some(user) => match cmd_args.next() {
                            Some(time) => {
                                let time = match time.parse::<i64>() {
                                    Ok(time) => {
                                        if access_level >= AccessLevel::Moderator as u8 {
                                            let timeout = (Utc::now() + chrono::Duration::minutes(time)).naive_utc();

                                            chat_server.timeout_user(user.to_owned(), timeout).await;
                                        }
                                    },
                                    Err(_) => (),
                                };
                            },
                            None => (),
                        },
                        None => (),
                    },

                    _ => ()
                }
            } else {
                chat_server.send_chat_message(
                    id,
                    user.clone(),
                    access_level,
                    command.room.clone().unwrap_or_default(),
                    json!(MessagesChanged { 
                        event: NEW_MESSAGE, 
                        username: &user, 
                        message: &command.message.unwrap_or_default(), 
                        room: &command.room.unwrap_or_default() 
                    }).to_string()
                ).await
            }
        },

        _ => {
            session
                .text("unknown command!")
                .await
                .unwrap();
        },
    }
}