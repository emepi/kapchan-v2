use std::env;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::{time::Duration, Key}, web, App, HttpServer};
use base64::{prelude::BASE64_STANDARD, Engine};
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use dotenvy::dotenv;
use handlers::{admin::admin_view, application_review::application_review_view, applications::applications_view, apply::application_view, board::board_view, captcha::generate_captcha, files::{serve_files, serve_thumbnails}, forms::{accept_application::handle_application_accept, apply::handle_application, create_board::handle_board_creation, create_post::handle_post_creation, create_thread::handle_thread_creation, deny_application::handle_application_deny, login::handle_login, logout::handle_logout, register::handle_register}, index::index_view, login::login_view, not_found::not_found_view, register::register_view, thread::thread_view};
use services::users::update_root_user;


mod database {
    pub mod applications;
    pub mod boards;
    pub mod captchas;
    pub mod users;
    pub mod threads;
    pub mod files;
    pub mod posts;
}

mod handlers {
    pub mod forms {
        pub mod accept_application;
        pub mod deny_application;
        pub mod apply;
        pub mod login;
        pub mod logout;
        pub mod register;
        pub mod create_board;
        pub mod create_thread;
        pub mod create_post;
    }
    pub mod admin;
    pub mod applications;
    pub mod application_review;
    pub mod board;
    pub mod captcha;
    pub mod apply;
    pub mod files;
    pub mod index;
    pub mod login;
    pub mod register;
    pub mod thread;
    pub mod not_found;
}

mod models {
    pub mod applications;
    pub mod boards;
    pub mod files;
    pub mod users;
    pub mod threads;
    pub mod posts;
    pub mod captchas;
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mysql_connection_pool.clone()))
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), private_key.clone())
                .cookie_name("kapchan-session".to_owned())
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(365)))
                .build(),
            )
            .service(
                web::resource("/")
                    .route(web::get().to(index_view))
            )
            .service(
                web::resource("/login")
                    .route(web::get().to(login_view))
                    .route(web::post().to(handle_login))
            )
            .service(
                web::resource("/logout")
                    .route(web::post().to(handle_logout))
            )
            .service(
                web::resource("/register")
                    .route(web::get().to(register_view))
                    .route(web::post().to(handle_register))
            )
            .service(
                web::resource("/apply")
                    .route(web::get().to(application_view))
                    .route(web::post().to(handle_application))
            )
            .service(
                web::resource("/admin")
                    .route(web::get().to(admin_view))
            )
            .service(
                web::resource("/applications/{page}")
                    .route(web::get().to(applications_view))
            )
            .service(
                web::resource("/application-review/{application_id}")
                    .route(web::get().to(application_review_view))
            )
            .service(
                web::resource("/accept-application/{application_id}")
                    .route(web::post().to(handle_application_accept))
            )
            .service(
                web::resource("/deny-application/{application_id}")
                    .route(web::post().to(handle_application_deny))
            )
            .service(
                web::resource("/boards")
                    .route(web::post().to(handle_board_creation))
            )
            .service(
                web::resource("/captcha")
                    .route(web::get().to(generate_captcha))
            )
            .service(
                web::resource("/{handle}")
                    .route(web::get().to(board_view))
                    .route(web::post().to(handle_thread_creation))
            )
            .service(
                web::resource("/{handle}/thread/{id}")
                    .route(web::get().to(thread_view))
                    .route(web::post().to(handle_post_creation))
            )
            .service(
                web::resource("/files/{id}")
                .route(web::get().to(serve_files))
            )
            .service(
                web::resource("/thumbnails/{id}")
                .route(web::get().to(serve_thumbnails))
            )
            .service(
                Files::new("/static", "./static")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .default_service(web::to(not_found_view))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}