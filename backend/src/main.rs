pub mod schema;
mod user_service;
mod board_service;


use std::env;

use actix_files::Files;
use actix_multipart::form::MultipartFormConfig;
use actix_web::{HttpServer, App, web};
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}, 
};
use dotenvy::dotenv;
use log::info;
use user_service::{
    user::{UserModel, User}, 
    authentication::{AccessLevel},
};

use crate::user_service::authentication::hash_password_pbkdf2;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    let conn_pool = mysql_connection_pool();

    setup_root_user(&conn_pool).await;

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(conn_pool.clone()))
        .app_data(
            MultipartFormConfig::default()
            .total_limit(21_076_377)
        )
        .configure(user_service::endpoints)
        .configure(board_service::endpoints)
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
                username: Some("root"),
                email: None,
                password_hash: Some(&root_pwd),
            };

            let root = User::by_username("root", &conn_pool).await;

            match root {
                
                Ok(root) => {
                    let pwd_hash = hash_password_pbkdf2(&root_pwd);

                    root_mdl.password_hash = Some(&pwd_hash);
                    
                    let _ = User::modify_by_id(root.id, root_mdl, conn_pool)
                    .await;
                    info!("Root user updated.");
                },

                Err(_) => {
                    let pwd_hash = hash_password_pbkdf2(&root_pwd);

                    root_mdl.password_hash = Some(&pwd_hash);

                    let res = root_mdl.insert(conn_pool).await;

                    if res.is_ok() {
                        info!("Root successfully created.");
                    }
                },
            };
        },

        Err(_) => info!("Root user was not set."),
    }
}