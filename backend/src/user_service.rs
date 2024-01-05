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
use log::error;
use serde::Deserialize;

use crate::schema::users;


pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/users")
            .route(web::get().to(users))
    );
}


#[derive(Debug, Deserialize)]
struct UsersQuery {
   pub page: Option<u32>,
   pub size: Option<u32>,
   pub username: Option<String>,
}

async fn users(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    query: web::Query<UsersQuery>
) -> impl Responder {

    let page = match query.page {
        Some(page) => page,
        None => 0,
    };

    let page_size = match query.size {
        Some(size) => {
            // Max page size.
            if size > 50 {
                return HttpResponse::BadRequest().finish();
            }

            size
        },
        None => 50,
    };

    let users = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let mut db_query = users::table.into_boxed();

                if let Some(username) = &query.username {
                    db_query = db_query.filter(
                        users::username.like(format!("{}%", username))
                    );
                }

                let users = db_query.select((
                    users::id,
                    users::access_level,
                    users::username,  
                    users::email,  
                    users::created_at,
                ))
                .limit(page_size.into())
                .offset((page * page_size).into())
                .load::<(u32, u8, Option<String>, Option<String>, NaiveDateTime)>(conn)
                .await?;

                Ok(users)
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match users {
        Ok(users) => {
            // Change response code to 404 on no matches.
            if page_size > 0 && users.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            HttpResponse::Ok().json(serde_json::json!(users))
        },

        Err(err) => {
            error!("Error with a database query for users.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}