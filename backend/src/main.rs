pub mod schema;
mod server;
mod user_service;


use std::{env, time::Instant, collections::HashMap};

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{
    HttpServer, 
    App, 
    web, 
    HttpResponse, 
    Error, 
    HttpRequest, 
    error::InternalError, 
    http::{StatusCode, header},
};
use actix_web_actors::ws;
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use server::{WebsocketServer, session::WebsocketSession, ServerSettings};
use user_service::{UserService, user::UserModel, authentication::authenticate};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    let conn_pool = mysql_connection_pool();

    let server = WebsocketServer::new(
        ServerSettings {
            max_sessions: 100,
            database: conn_pool.clone(),
        }
    )
    .service(UserService::new())
    .start();

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(conn_pool.clone()))
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

// NOTE: User & user session could be handled in WebsocketSession in the future.
async fn websocket_connect(
    req: HttpRequest, 
    stream: web::Payload,
    server: web::Data<Addr<WebsocketServer>>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> Result<HttpResponse, Error> {

    let existing_user = match req.cookie("access_token") {
        Some(access_token) => authenticate(
            access_token.value(), 
            &conn_pool
        ).await,
        
        None => None,
    };
    
    let user = match existing_user {
        Some(existing_user) => existing_user,

        // Create an anonymous user for client without a valid access token.
        None => {
            UserModel::default()
            .register_user(&conn_pool)
            .await
            .ok_or(InternalError::new(
                "Error placeholder for failed user creation.", 
                StatusCode::INTERNAL_SERVER_ERROR)
            )?
        },
    };

    // TODO: simplify
    let ip = req.peer_addr().map(|addr| addr.ip().to_string());

    let user_agent = req.headers().get(header::USER_AGENT)
    .and_then(|header| header.to_str().ok());

    let user_session = user.create_session(
        ip.as_deref(),
        user_agent, 
        &conn_pool
    )
    .await
    .ok_or(InternalError::new(
        "Error placeholder for failed user session.", 
        StatusCode::INTERNAL_SERVER_ERROR)
    )?;

    ws::start(
        WebsocketSession {
            user: user_session,
            server: server.get_ref().clone(),
            last_activity: Instant::now(),
            service_feeds: HashMap::new(),
        }, 
        &req, 
        stream,
    )
    .and_then(|mut http_response| {

        if req.cookie("access_token").is_none() {
            let auth_token = &user.create_authentication_token()
            .ok_or(InternalError::new(
                "Error placeholder for failing to issue an access token", 
                StatusCode::INTERNAL_SERVER_ERROR)
            )?;

            http_response.add_cookie(auth_token)?;
        }

        Ok(http_response)
    })
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