pub mod application;
pub mod authentication;
pub mod session;
pub mod user;


use actix_web::{web, HttpResponse, Responder};
use chrono::{NaiveDateTime, offset};
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
   pub offset: Option<u32>,
   pub limit: Option<u32>,
   pub role: Option<u8>,
   pub username: Option<String>,
}

async fn users(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    query: web::Query<UsersQuery>
) -> impl Responder {

    let offset = match query.offset {
        Some(offset) => offset,
        None => 0,
    };

    let limit = match query.limit {
        Some(limit) => {
            // Max page size.
            if limit > 50 {
                return HttpResponse::BadRequest().finish();
            }

            limit
        },
        None => 50,
    };

    let users = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let mut db_query = users::table.into_boxed();
                let mut count_query = users::table.into_boxed();

                if let Some(role) = query.role {
                    db_query = db_query.filter(
                        users::access_level.ge(role)
                    );
                    count_query = count_query.filter(
                        users::access_level.ge(role)
                    );
                }

                if let Some(username) = &query.username {
                    db_query = db_query.filter(
                        users::username.like(format!("{}%", username))
                    );
                    count_query = count_query.filter(
                        users::username.like(format!("{}%", username))
                    );
                }

                let total_count: i64 = count_query.count()
                .get_result(conn)
                .await?;

                let users = db_query.select((
                    users::id,
                    users::access_level,
                    users::username,  
                    users::email,  
                    users::created_at,
                ))
                .limit(limit.into())
                .offset(offset.into())
                .load::<(u32, u8, Option<String>, Option<String>, NaiveDateTime)>(conn)
                .await?;

                Ok((total_count, users))
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
            if limit > 0 && users.1.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            HttpResponse::Ok()
            .insert_header(("X-Total-Count", users.0))
            .json(serde_json::json!(users.1))
        },

        Err(err) => {
            error!("Error with a database query for users.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}