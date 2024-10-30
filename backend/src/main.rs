pub mod boards;
pub mod files;
pub mod users;
pub mod posts;
pub mod schema;
pub mod threads;


use std::env;

use actix_web::{web, App, HttpServer};
use diesel_async::pooled_connection::{
    deadpool::Pool, 
    AsyncDieselConnectionManager
};
use dotenvy::dotenv;
use users::database::create_root_user;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();
    env_logger::init();

    // Init mysql connection pool.
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

    // Create or update the root user
    create_root_user(&mysql_connection_pool).await;

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(mysql_connection_pool.clone()))
        .configure(routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn routes(app: &mut web::ServiceConfig) {
    app
    .service(web::scope("/api")
        .service(web::resource("/boards")
         //   .route(web::get().to(boards::routes::boards))
         //   .route(web::post().to(boards::routes::create_board))
        )
        .service(web::resource("/register")
            .route(web::post().to(users::routes::register_user))
        )
        .service(web::resource("/sessions")
            .route(web::post().to(users::routes::create_session))
        )
    );
}