pub mod schema;
mod server;
mod user_service;


use std::{env, time::Instant, collections::HashMap, sync::Arc};

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{
    HttpServer, 
    App, 
    HttpResponse, 
    Error, 
    HttpRequest, 
    error::InternalError, 
    http::{StatusCode, header}, cookie::{Cookie, self, SameSite}, web,
};
use actix_web_actors::ws;
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use log::info;
use server::{WebsocketServer, session::WebsocketSession, ServerSettings};
use user_service::{
    UserService, 
    user::{UserModel, AccessLevel, User}, 
    authentication::{
        validate_session_id, 
        create_authentication_token, 
        hash_password_a2id
    }, 
    session::UserSession
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    let conn_pool = mysql_connection_pool();

    setup_root_user(&conn_pool)
    .await;

    let server = WebsocketServer::new(
        ServerSettings {
            max_sessions: 100,
            database: conn_pool.clone(),
        }
    )
    .service::<UserService>()
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

async fn websocket_connect(
    req: HttpRequest, 
    stream: web::Payload,
    server: web::Data<Addr<WebsocketServer>>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> Result<HttpResponse, Error> {
    
    let prev_sess = req.cookie("access_token")
    .and_then(|access_token| validate_session_id(access_token.value()));

    match prev_sess {
        Some(sess_id) => {
            let sess = UserSession::by_id(sess_id, &conn_pool)
            .await
            .ok_or(InternalError::new(
                "Error placeholder for failed user creation.", 
                StatusCode::INTERNAL_SERVER_ERROR
            ))?;

            ws::start(
                WebsocketSession {
                    user: Arc::new(sess),
                    server: server.get_ref().clone(),
                    last_activity: Instant::now(),
                    service_feeds: HashMap::new(),
                }, 
                &req, 
                stream,
            )
        },

        None => {
            // Create an anonymous user for client without a valid access token.
            let user = UserModel::default()
            .insert(&conn_pool)
            .await
            .ok_or(InternalError::new(
                "Error placeholder for failed user creation.", 
                StatusCode::INTERNAL_SERVER_ERROR
            ))?;

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

            let sess_id = user_session.id;
            let role = user_session.access_level;

            ws::start(
                WebsocketSession {
                    user: Arc::new(user_session),
                    server: server.get_ref().clone(),
                    last_activity: Instant::now(),
                    service_feeds: HashMap::new(),
                }, 
                &req, 
                stream,
            )
            .and_then(|mut http_response| {

                let jwt_expiration = env::var("JWT_EXPIRATION")
                .expect(".env variable `JWT_EXPIRATION` must be set")
                .parse::<i64>()
                .expect("`JWT_EXPIRATION` must be a valid number");
        
                let access_token = create_authentication_token(sess_id, role)
                .map(|access_token| {
                    Cookie::build("access_token", access_token)
                    .max_age(cookie::time::Duration::new(jwt_expiration * 60, 0))
                    .same_site(SameSite::Strict)
                    .finish()
                })
                .ok_or(InternalError::new(
                    "Error placeholder for failing to issue an access token", 
                    StatusCode::INTERNAL_SERVER_ERROR)
                )?;
        
                http_response.add_cookie(&access_token)?;
        
                Ok(http_response)
            })
        },
    }
}

async fn setup_root_user(conn_pool: &Pool<AsyncMysqlConnection>) {

    let root_pwd = env::var("ROOT_PASSWORD");

    match root_pwd {
        
        Ok(root_pwd) => {

            let mut root_mdl = UserModel {
                access_level: AccessLevel::Root as u8,
                username: Some("root"),
                email: None,
                password_hash: Some(&root_pwd),
            };

            let root = User::by_username("root", &conn_pool).await;

            match root {
                
                Some(root) => {
                    let pwd_hash = match hash_password_a2id(&root_pwd) {
                        Some(hash) => hash,
                        None => return,
                    };

                    root_mdl.password_hash = Some(&pwd_hash);
                    
                    User::modify_by_id(root.id, root_mdl, conn_pool).await;
                    info!("Root user updated.");
                },

                None => {
                    let pwd_hash = match hash_password_a2id(&root_pwd) {
                        Some(hash) => hash,
                        None => return,
                    };

                    root_mdl.password_hash = Some(&pwd_hash);

                    let res = root_mdl.insert(conn_pool).await;

                    if res.is_some() {
                        info!("Root successfully created.");
                    }
                },
            };
        },

        Err(_) => info!("Root user was not set."),
    }
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