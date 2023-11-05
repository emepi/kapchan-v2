use std::env;

use actix_web::{HttpServer, App, web};
use diesel_async::pooled_connection::{
    deadpool::Pool, 
    AsyncDieselConnectionManager,
};
use dotenvy::dotenv;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables.
    dotenv().ok();

    let mysql_url = env::var("DATABASE_URL").expect(r#"
        Env variable `DATABASE_URL` must be set in `backend/.env` file.
        See the .env.example.
    "#);

    let mysql_connection_pool = Pool::builder(
        AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>
        ::new(mysql_url)
    )
    .build()
    .expect("Mysql connection pool was unable to start");

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(mysql_connection_pool.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}