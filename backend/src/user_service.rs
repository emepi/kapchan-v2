pub mod application;
pub mod authentication;
pub mod session;
pub mod user;


use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl, 
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, scoped_futures::ScopedFutureExt
};

use crate::schema::users;


pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/users")
            .route(web::get().to(users))
    );
}


async fn users(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> impl Responder {

    let users = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let users = users::table
                .select((
                    users::access_level, 
                    users::created_at, 
                    users::email, 
                    users::id, 
                    users::username
                ))
                .limit(50)  // page size
                .offset(0)  // page * page size
                .load::<(u8, NaiveDateTime, Option<String>, u32, Option<String>)>(conn)
                .await?;

                Ok(users)
            }.scope_boxed())
            .await
            .ok()
        },

        Err(_) => None,
    };

    match users {
        Some(users) => {
            HttpResponse::Ok().json(serde_json::json!(users))
        },

        None => {
            HttpResponse::InternalServerError().finish()
        },
    }
}