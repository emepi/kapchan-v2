pub mod schema;
pub mod user;
mod server;
mod user_service;


use std::{env, time::Instant};

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{
    HttpServer, 
    App, 
    web, 
    HttpResponse, 
    Error, 
    HttpRequest, 
    error::InternalError, http::{StatusCode, header},
};
use actix_web_actors::ws;
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use user::UserModel;
use server::{WebsocketServer, session::WebsocketSession};
use user_service::UserService;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    let server = WebsocketServer::new()
    .database(mysql_connection_pool())
    .service(UserService::new())
    .start();

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(mysql_connection_pool()))
        .app_data(web::Data::new(server.clone()))
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
    server: web::Data<Addr<WebsocketServer>>,
    db_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> Result<HttpResponse, Error> {

    // TODO: User from cookies & check how concurrent sessions user is allowed.

    // Create an anonymous user.
    let anon = UserModel {
        access_level: 1,
        username: None,
        email: None,
        password_hash: None,
    };

    let user = anon.register_user(&db_pool).await
    .ok_or(InternalError::new("User err", StatusCode::INTERNAL_SERVER_ERROR))?;

    let ip = req.peer_addr()
    .map(|addr| addr.ip().to_string());
    let agent = req.headers().get(header::USER_AGENT)
    .and_then(|header| header.to_str().ok());

    let user_session = user
    .create_session(None, None, ip.as_deref(), agent, &db_pool).await
    .ok_or(InternalError::new("Sess err", StatusCode::INTERNAL_SERVER_ERROR))?;

    ws::start(
        WebsocketSession {
            user: user_session,
            server: server.get_ref().clone(),
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