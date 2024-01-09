pub mod application;
pub mod authentication;
pub mod session;
pub mod user;


use actix_web::{web, HttpResponse, Responder, HttpRequest, http::header};
use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl, 
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, scoped_futures::ScopedFutureExt
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::schema::{users, sessions};

use self::{
    authentication::{
        validate_claims, 
        validate_password_a2id, 
        create_authentication_token
    }, 
    user::{AccessLevel, User}, 
    session::{UserSessionModel, UserSession, is_active_session}
};


pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/sessions")
        .route(web::post().to(create_session))
    )
    .service(
        web::resource("/sessions/{id}")
        .route(web::put().to(update_session))
    )
    .service(
        web::resource("/users")
        .route(web::get().to(users))
    );
}


#[derive(Debug, Deserialize)]
struct LoginInfo {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    pub access_token: String,
}

async fn create_session(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    login: Option<web::Json<LoginInfo>>,
    req: HttpRequest,
) -> impl Responder {

    let user = match login {
        Some(login_info) => {
            let password = login_info.password.clone();

            // Fetch user by username or email.
            let user = match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {

                        let mut user: Option<User> = None;
                        let db_query = users::table.into_boxed();

                        if let Some(username) = &login_info.username {
                            user = db_query.filter(users::username.eq(username))
                            .first::<User>(conn)
                            .await
                            .ok();
                        } else if let Some(email) = &login_info.email {
                            user = db_query.filter(users::email.eq(email))
                            .first::<User>(conn)
                            .await
                            .ok();
                        }

                        Ok(user)
                    }.scope_boxed())
                    .await
                },
        
                Err(err) => {
                    error!("Connection pool reported an error.\n {:?}", err);
        
                    return HttpResponse::InternalServerError().finish();
                },
            };

            match user {
                Ok(user) => {
                    match user {
                        Some(user) => {
                            match validate_password_a2id(&user.password_hash, &password) {
                                true => Some(user),
                                false => return HttpResponse::Unauthorized()
                                .finish(),
                            }
                        },

                        None => return HttpResponse::NotFound().finish(),
                    }
                },

                Err(err) => {
                    error!("Error with a database query for user.\n {:?}", err);
        
                    return HttpResponse::InternalServerError().finish();
                },
            }
        },

        None => None,
    };

    let mut access_level = AccessLevel::Anonymous as u8;

    let mut user_id: Option<u32> = None;

    if let Some(user) = user {
        access_level = user.access_level;
        user_id = Some(user.id);
    }

    let session = UserSessionModel {
        user_id,
        access_level,
        mode: 1,
        ended_at: None,
    };

    let session = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let _ = diesel::insert_into(sessions::table)
                .values(session)
                .execute(conn)
                .await?;
            
                let user_session = sessions::table
                .find(last_insert_id())
                .first::<UserSession>(conn)
                .await?;

                Ok(user_session)
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match session {
        Ok(session) => {
            if let Some(token) = create_authentication_token(session.id, session.access_level) {
                return HttpResponse::Created().json(SessionResponse {
                    access_token: token,
                });
            } else {
                // Token creation failed.
                return HttpResponse::InternalServerError().finish();
            }
        },

        Err(err) => {
            error!("Error with a database insert for session.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    }
}

#[derive(Debug, Deserialize)]
struct SessionOptions {
    pub access_level: Option<u8>,
    pub continue_session: bool,
    pub mode: Option<u8>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = sessions)]
pub struct SessionUpdate {
    pub access_level: Option<u8>,
    pub mode: Option<u8>,
    pub ended_at: Option<NaiveDateTime>,
}

async fn update_session(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    path: web::Path<u32>,
    req: HttpRequest,
    sess_opt: web::Json<SessionOptions>,
) -> impl Responder {

    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::NotFound().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let curr_sess_id = match claims.sub.parse::<u32>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::NotAcceptable().finish(),
    };

    if !is_active_session(curr_sess_id, &conn_pool).await {
        return HttpResponse::Unauthorized().finish();
    }

    // Allow only self modification unless user is root.
    if (path.into_inner() != curr_sess_id) && (claims.role != AccessLevel::Root as u8) {
        return HttpResponse::Forbidden().finish();
    }

    // Allow access level modifications only for below access current level.
    if sess_opt.access_level.is_some() && sess_opt.access_level.unwrap() > claims.role {
        return HttpResponse::Forbidden().finish();
    }

    let sess_changes = SessionUpdate {
        access_level: sess_opt.access_level,
        mode: sess_opt.mode,
        ended_at: match sess_opt.continue_session {
            true => None,
            false => Some(Utc::now().naive_utc()),
        },
    };

    let res = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let res: usize = diesel::update(sessions::table.find(curr_sess_id))
                .set(sess_changes)
                .execute(conn)
                .await?;

                Ok(res)
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match res {
        Ok(res) => {
            HttpResponse::Ok()
            .insert_header(("X-Total-Count", res))
            .finish()
        },

        Err(err) => {
            error!("Error with a database update for session.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
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
    req: HttpRequest,
    query: web::Query<UsersQuery>
) -> impl Responder {

    if let Some(auth_token) = req.headers().get(header::AUTHORIZATION) {
        let auth_token = match auth_token.to_str() {
            Ok(token) => token,
            Err(_) => return HttpResponse::NotAcceptable().finish(),
        };

        let claims = match validate_claims(auth_token) {
            Some(claims) => claims,

            // Expired or invalid token.
            None => return HttpResponse::Unauthorized().finish(),
        };

        let curr_sess_id = match claims.sub.parse::<u32>() {
            Ok(id) => id,
            Err(_) => return HttpResponse::NotAcceptable().finish(),
        };

        if !is_active_session(curr_sess_id, &conn_pool).await {
            return HttpResponse::Unauthorized().finish();
        }

        if claims.role < AccessLevel::Admin as u8 {
            return HttpResponse::Forbidden().finish();
        }

    } else {
        return HttpResponse::Unauthorized().finish();
    }

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
                .load::<(u32, u8, String, Option<String>, NaiveDateTime)>(conn)
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

sql_function!(fn last_insert_id() -> Unsigned<Integer>);