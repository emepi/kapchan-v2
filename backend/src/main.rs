pub mod user;
mod ws_server;


use std::{env, time::Instant};

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{HttpServer, App, web, HttpResponse, Error, HttpRequest};
use actix_web_actors::ws;
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use ws_server::{WsServer, session::WebsocketSession};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    let ws_server = WsServer::new().start();

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(mysql_connection_pool()))
        .app_data(web::Data::new(ws_server.clone()))
        .route("/ws", web::get().to(websocket_connect))
        .service(
            Files::new("/", "../frontend/dist")
                .show_files_listing()
                .index_file("index.html")
                .use_last_modified(true),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn websocket_connect(
    req: HttpRequest, 
    stream: web::Payload,
    ws_server: web::Data<Addr<WsServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        // TODO: create user session
        WebsocketSession {
            user: todo!(),
            server: ws_server.get_ref().clone(),
            last_activity: Instant::now(),
        }, 
        &req, 
        stream,
    )
}

fn mysql_connection_pool() -> Pool<AsyncMysqlConnection> {

    let mysql_url = env::var("DATABASE_URL").expect(r#"
        env variable `DATABASE_URL` must be set in `backend/.env`
        see: .env.example
    "#);

    let mysql_connection_pool = Pool::builder(
        AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>
        ::new(mysql_url)
    )
    .build()
    .expect("failed to establish connection pooling");

    mysql_connection_pool
}