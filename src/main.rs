use std::env;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::{time::Duration, Key}, web, App, HttpServer};
use base64::{prelude::BASE64_STANDARD, Engine};
use controllers::{admin_controller, application_controller, board_controller, captcha_controller, file_controller, index_controller, post_controller, thread_controller, user_controller};
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use dotenvy::dotenv;
use services::users::update_root_user;
use views::not_found_view;


mod controllers {
    pub mod admin_controller;
    pub mod application_controller;
    pub mod board_controller;
    pub mod captcha_controller;
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
    pub mod board_view;
    pub mod index_view;
    pub mod login_view;
    pub mod not_found_view;
    pub mod register_view;
    pub mod thread_view;
}

mod models {
    pub mod applications;
    pub mod boards;
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
                    .route(web::get().to(index_controller::index))
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
                web::resource("/post-details/{id}")
                    .route(web::get().to(post_controller::handle_post_details))
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
    .run()
    .await
}