use std::env;

use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::{time::Duration, Key}, web, App, HttpServer};
use base64::{prelude::BASE64_STANDARD, Engine};
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use dotenvy::dotenv;
use handlers::{forms::{login::handle_login, logout::handle_logout}, index::index_view, login::login_view};
use services::users::update_root_user;


mod database {
    pub mod users;
}

mod handlers {
    pub mod forms {
        pub mod login;
        pub mod logout;
    }
    pub mod index;
    pub mod login;
}

mod models {
    pub mod users;
}

mod services {
    pub mod authentication;
    pub mod users;
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}