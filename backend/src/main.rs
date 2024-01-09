pub mod schema;
mod server;
mod user_service;
mod board_service;


use std::env;

use actix_files::Files;
use actix_web::{HttpServer, App, web};
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use log::info;
use user_service::{
    user::{UserModel, AccessLevel, User}, 
    authentication::hash_password_a2id,
};


// Websocket service identifiers for message routing.
pub const USER_SERVICE_ID: u32  = 1;
pub const BOARD_SERVICE_ID: u32 = 2;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    let conn_pool = mysql_connection_pool();

    setup_root_user(&conn_pool).await;

    //let server = WebsocketServer::new(
    //    ServerSettings {
    //        max_sessions: 100,
    //        database: conn_pool.clone(),
    //    }
    //)
    //.service::<UserService>(USER_SERVICE_ID)
    //.service::<BoardService>(BOARD_SERVICE_ID)
    //.start();

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(conn_pool.clone()))
        .configure(user_service::endpoints)
        //.app_data(web::Data::new(server.clone()))
        //.route("/ws", web::get().to(websocket_connect))
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

async fn setup_root_user(conn_pool: &Pool<AsyncMysqlConnection>) {

    let root_pwd = env::var("ROOT_PASSWORD");

    match root_pwd {
        
        Ok(root_pwd) => {

            let mut root_mdl = UserModel {
                access_level: AccessLevel::Root as u8,
                username: "root",
                email: None,
                password_hash: &root_pwd,
            };

            let root = User::by_username("root", &conn_pool).await;

            match root {
                
                Some(root) => {
                    let pwd_hash = match hash_password_a2id(&root_pwd) {
                        Some(hash) => hash,
                        None => return,
                    };

                    root_mdl.password_hash = &pwd_hash;
                    
                    User::modify_by_id(root.id, root_mdl, conn_pool).await;
                    info!("Root user updated.");
                },

                None => {
                    let pwd_hash = match hash_password_a2id(&root_pwd) {
                        Some(hash) => hash,
                        None => return,
                    };

                    root_mdl.password_hash = &pwd_hash;

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