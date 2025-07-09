use std::env;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::{time::Duration, Key}, web, App, HttpServer};
use base64::{prelude::BASE64_STANDARD, Engine};
use chat::server::ChatServer;
use controllers::{admin_controller, application_controller, board_controller, captcha_controller, chat_controller, file_controller, index_controller, post_controller, thread_controller, user_controller};
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use dotenvy::dotenv;
use services::users::update_root_user;
use tokio::{spawn, try_join};
use views::not_found_view;


mod chat {
    pub mod handler;
    pub mod server;
}

mod controllers {
    pub mod admin_controller;
    pub mod application_controller;
    pub mod board_controller;
    pub mod captcha_controller;
    pub mod chat_controller;
    pub mod file_controller;
    pub mod index_controller;
    pub mod post_controller;
    pub mod thread_controller;
    pub mod user_controller;
}

mod views {
    pub mod admin_view;
    pub mod application_list_view;
    pub mod application_review_view;
    pub mod application_view;
    pub mod banned_view;
    pub mod board_view;
    pub mod chat_view;
    pub mod forbidden_view;
    pub mod index_view;
    pub mod login_view;
    pub mod not_found_view;
    pub mod register_view;
    pub mod thread_view;
    pub mod user_view;
    pub mod users_view;
}

mod models {
    pub mod applications;
    pub mod bans;
    pub mod boards;
    pub mod chat_rooms;
    pub mod files;
    pub mod users;
    pub mod threads;
    pub mod posts;
    pub mod captchas;
    pub mod error;
}

mod services {
    pub mod authentication;
    pub mod applications;
    pub mod captchas;
    pub mod files;
    pub mod users;
    pub mod time;
    pub mod threads;
    pub mod posts;
}

mod schema;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    
    // Initialize database connection pool.
    let mysql_url = env::var("DATABASE_URL").expect(r#"
        env variable `DATABASE_URL` must be set in `.env`
        see: .env.example
    "#);

    let mysql_connection_pool = Pool::builder(
        AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>
        ::new(mysql_url)
    )
    .build()
    .expect("failed to establish connection pooling");

    // Read private key for cookie sessions.
    let private_key = env::var("COOKIE_SECRET").expect(r#"
        env variable `COOKIE_SECRET` must be set in `.env`
        see: .env.example
    "#);

    let private_key = Key::from(&BASE64_STANDARD.decode(private_key).unwrap());

    // Create or update root user.
    let root_pwd = env::var("ROOT_PASSWORD").expect(r#"
        env variable `ROOT_PASSWORD` must be set in `.env`
        see: .env.example
    "#);

    update_root_user(&mysql_connection_pool, &root_pwd).await.unwrap();

    // Create a chat server
    let (chat_server, server_tx) = ChatServer::new(vec![
        "päähuone".to_owned()
    ]);

    let chat_server = spawn(chat_server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_tx.clone()))
            .app_data(web::Data::new(mysql_connection_pool.clone()))
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), private_key.clone())
                .cookie_name("kapchan-session".to_owned())
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(365)))
                .build(),
            )
            .service(
                web::resource("/")
                    .route(web::get().to(index_controller::index))
            )
            .service(
                web::resource("/ws")
                    .route(web::get().to(chat_controller::chat_ws))
            )
            .service(
                web::resource("/chat")
                    .route(web::get().to(chat_controller::chat))
            )
            .service(
                web::resource("/login")
                    .route(web::get().to(user_controller::login))
                    .route(web::post().to(user_controller::handle_login))
            )
            .service(
                web::resource("/logout")
                    .route(web::post().to(user_controller::handle_logout))
            )
            .service(
                web::resource("/register")
                    .route(web::get().to(application_controller::register))
                    .route(web::post().to(application_controller::handle_registration))
            )
            .service(
                web::resource("/apply")
                    .route(web::get().to(application_controller::application))
                    .route(web::post().to(application_controller::handle_application))
            )
            .service(
                web::resource("/admin")
                    .route(web::get().to(admin_controller::admin))
            )
            .service(
                web::resource("/applications/{page}")
                    .route(web::get().to(admin_controller::applications_list))
            )
            .service(
                web::resource("/users/{page}")
                    .route(web::get().to(admin_controller::users_list))
            )
            .service(
                web::resource("/user/{user_id}")
                    .route(web::get().to(admin_controller::user))
            )
            .service(
                web::resource("/application-review/{application_id}")
                    .route(web::get().to(admin_controller::application_review))
            )
            .service(
                web::resource("/accept-application/{application_id}")
                    .route(web::post().to(admin_controller::handle_application_accept))
            )
            .service(
                web::resource("/deny-application/{application_id}")
                    .route(web::post().to(admin_controller::handle_application_deny))
            )
            .service(
                web::resource("/boards")
                    .route(web::post().to(admin_controller::handle_board_creation))
            )
            .service(
                web::resource("/edit-board/{id}")
                    .route(web::post().to(admin_controller::handle_board_edit))
            )
            .service(
                web::resource("/create-chat")
                    .route(web::post().to(chat_controller::create_chat_room))
            )
            .service(
                web::resource("/delete-chat/{id}")
                    .route(web::post().to(chat_controller::delete_chat_room))
            )
            .service(
                web::resource("/captcha")
                    .route(web::get().to(captcha_controller::captcha))
            )
            .service(
                web::resource("/{handle}")
                    .route(web::get().to(board_controller::board))
                    .route(web::post().to(thread_controller::handle_thread_creation))
            )
            .service(
                web::resource("/{handle}/thread/{id}")
                    .route(web::get().to(thread_controller::thread))
                    .route(web::post().to(post_controller::handle_post_creation))
            )
            .service(
                web::resource("/pin-thread/{id}")
                    .route(web::get().to(thread_controller::handle_thread_pin))
            )
            .service(
                web::resource("/unpin-thread/{id}")
                    .route(web::get().to(thread_controller::handle_thread_unpin))
            )
            .service(
                web::resource("/lock-thread/{id}")
                    .route(web::post().to(thread_controller::handle_thread_lock))
            )
            .service(
                web::resource("/modify-user/{id}")
                    .route(web::post().to(admin_controller::modify_user_by_id))
            )
            .service(
                web::resource("/delete-board/{id}")
                    .route(web::post().to(board_controller::delete_board))
            )
            .service(
                web::resource("/delete-thread/{id}")
                    .route(web::post().to(thread_controller::delete_thread))
            )
            .service(
                web::resource("/delete-ban/{id}")
                    .route(web::post().to(admin_controller::handle_ban_deletion))
            )
            .service(
                web::resource("/delete-post/{id}")
                    .route(web::post().to(post_controller::delete_post))
            )
            .service(
                web::resource("/ban-user-by-post/{id}")
                    .route(web::post().to(post_controller::ban_user_by_post_id))
            )
            .service(
                web::resource("/ban-user-by-id/{id}")
                    .route(web::post().to(admin_controller::ban_user_by_id))
            )
            .service(
                web::resource("/post-details/{id}")
                    .route(web::get().to(post_controller::handle_post_details))
            )
            .service(
                web::resource("/full-post/{id}")
                    .route(web::get().to(post_controller::get_post_by_id))
            )
            .service(
                web::resource("/files/{id}")
                .route(web::get().to(file_controller::serve_files))
            )
            .service(
                web::resource("/thumbnails/{id}")
                .route(web::get().to(file_controller::serve_thumbnails))
            )
            .service(
                Files::new("/static", "./static")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .default_service(web::to(not_found_view::render))
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    try_join!(http_server, async move { chat_server.await.unwrap() })?;

    Ok(())
}