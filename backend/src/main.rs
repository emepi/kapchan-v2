mod ws_server;


use std::env;

use actix_files::Files;
use actix_web::{HttpServer, App, web, HttpResponse, Error, HttpRequest};
use actix_web_actors::ws;
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use ws_server::WsSession;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    HttpServer::new(|| {
        App::new()
        .app_data(web::Data::new(mysql_connection_pool()))
        .service(
            Files::new("/", "../frontend/dist")
                .show_files_listing()
                .index_file("index.html")
                .use_last_modified(true),
        )
        .service(web::resource("/ws").to(websocket_connect))
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn websocket_connect(
    req: HttpRequest, 
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(WsSession {}, &req, stream)
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