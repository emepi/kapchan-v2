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

use crate::schema::{users, sessions, applications::{self, accepted}, application_reviews};

use self::{
    authentication::{
        validate_claims, 
        validate_password_a2id, 
        create_authentication_token, hash_password_a2id
    }, 
    user::{AccessLevel, User, UserModel}, 
    session::{UserSessionModel, UserSession, is_active_session}, 
    application::{ApplicationModel, Application, ApplicationReviewModel}
};


pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/applications")
        .route(web::get().to(applications))
        .route(web::post().to(create_application))
    )
    .service(
        web::resource("/applications/{id}")
        .route(web::put().to(update_application))
    )
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
        .route(web::post().to(create_user))
    );
}


#[derive(Debug, Deserialize)]
struct ApplicationsQuery {
   pub offset: Option<u32>,
   pub limit: Option<u32>,
   pub accepted: Option<bool>,
   pub username: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApplicationsResponse {
    pub id: u32,
    pub username: String,
    pub email: Option<String>,
    pub accepted: bool,
    pub background: String,
    pub motivation: String,
    pub other: Option<String>,
    pub created_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
}

async fn applications(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    query: web::Query<ApplicationsQuery>,
    req: HttpRequest,
) -> impl Responder {
    // Check access level.
    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::Unauthorized().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if claims.role < AccessLevel::Admin as u8 {
        return HttpResponse::Forbidden().finish();
    }

    // Check query bounds.
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

    let applications = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let mut db_query = applications::table.into_boxed()
                .inner_join(users::table);
                let mut count_query = applications::table.into_boxed()
                .inner_join(users::table);

                if let Some(acc) = query.accepted {
                    db_query = db_query.filter(
                        applications::accepted.eq(acc)
                    );
                    count_query = count_query.filter(
                        applications::accepted.eq(acc)
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

                let applications: Vec<ApplicationsResponse> = db_query.select((
                    applications::id,
                    users::username,  
                    users::email,  
                    applications::accepted,
                    applications::background,
                    applications::motivation,
                    applications::other,
                    applications::created_at,
                    applications::closed_at,
                ))
                .limit(limit.into())
                .offset(offset.into())
                .load::<(
                    u32, 
                    String, 
                    Option<String>, 
                    bool, 
                    String, 
                    String, 
                    Option<String>, 
                    NaiveDateTime, 
                    Option<NaiveDateTime>)>(conn)
                .await?
                .iter()
                .map(|data| {
                    ApplicationsResponse {
                        id: data.0,
                        username: data.1.clone(),
                        email: data.2.clone(),
                        accepted: data.3,
                        background: data.4.clone(),
                        motivation: data.5.clone(),
                        other: data.6.clone(),
                        created_at: data.7,
                        closed_at: data.8,
                    }
                })
                .collect();

                Ok((total_count, applications))
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match applications {
        Ok(applications) => {
            // Change response code to 404 on no matches.
            if limit > 0 && applications.1.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            HttpResponse::Ok()
            .insert_header(("X-Total-Count", applications.0))
            .json(serde_json::json!(applications.1))
        },

        Err(err) => {
            error!("Error with a database query for applications.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}


#[derive(Debug, Deserialize)]
struct CreateApplicationInput {
    pub background: String,
    pub motivation: String,
    pub referrer: Option<String>,
}

async fn create_application(
    application: web::Json<CreateApplicationInput>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {

    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::Unauthorized().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let curr_sess_id = match claims.sub.parse::<u32>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::NotAcceptable().finish(),
    };

    if claims.role < AccessLevel::Registered as u8 || 
    claims.role >= AccessLevel::PendingMember as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let res = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let session = sessions::table
                .find(curr_sess_id)
                .first::<UserSession>(conn)
                .await?;

                if let Some(user_id) = session.user_id {
                    let _ = diesel::insert_into(applications::table)
                    .values(ApplicationModel {
                        user_id,
                        accepted: false,
                        background: &application.background,
                        motivation: &application.motivation,
                        other: application.referrer.as_deref(),
                        closed_at: None,
                    })
                    .execute(conn)
                    .await;

                    let _ = diesel::update(users::table.find(user_id))
                    .set(users::access_level.eq(AccessLevel::PendingMember as u8))
                    .execute(conn)
                    .await;

                    let _ = diesel::update(sessions::table.find(curr_sess_id))
                    .set(sessions::ended_at.eq(Utc::now().naive_utc()))
                    .execute(conn)
                    .await;
                }

                Ok(())
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match res {
        Ok(_) => {
            return HttpResponse::Created().finish();
        },

        Err(err) => {
            error!("Error with a database insert for application.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    }
}


#[derive(Debug, Deserialize)]
struct UpdateApplicationInput {
    pub accepted: bool,
}

async fn update_application(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<UpdateApplicationInput>,
    req: HttpRequest,
    path: web::Path<u32>,
) -> impl Responder {
    // Check access level.
    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::Unauthorized().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if claims.role < AccessLevel::Admin as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let curr_sess_id = match claims.sub.parse::<u32>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::NotAcceptable().finish(),
    };

    let rank = match input.accepted {
        true => AccessLevel::Member as u8,
        false => AccessLevel::Registered as u8,
    };

    let app_id = path.into_inner();

    let res = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let _ = diesel::update(applications::table.find(app_id))
                .set((
                    applications::accepted.eq(input.accepted),
                    applications::closed_at.eq(Utc::now().naive_utc())
                ))
                .execute(conn)
                .await?;

                let application = applications::table
                .find(app_id)
                .first::<Application>(conn)
                .await?;

                let session = sessions::table
                .find(curr_sess_id)
                .first::<UserSession>(conn)
                .await?;

                let _ = diesel::insert_into(application_reviews::table)
                .values(ApplicationReviewModel {
                    reviewer_id: session.user_id.unwrap(),
                    application_id: application.id,
                })
                .execute(conn)
                .await?;

                let _ = diesel::update(users::table.find(application.user_id))
                .set(users::access_level.eq(rank))
                .execute(conn)
                .await?;

                // TODO: invalidate open sessions

                Ok(())
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match res {
        Ok(_) => {
            HttpResponse::Created().finish()
        },
        Err(err) => {
            error!("Error with a database update for application.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
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
struct CreateUserInput {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

async fn create_user(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<CreateUserInput>,
    req: HttpRequest,
) -> impl Responder {

    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::Unauthorized().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let curr_sess_id = match claims.sub.parse::<u32>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::NotAcceptable().finish(),
    };

    let session = match UserSession::by_id(curr_sess_id, &conn_pool).await {
        Some(session) => session,
        None => return HttpResponse::NotFound().finish(),
    };

    // Session already belongs to an user.
    if session.user_id.is_some() {
        return HttpResponse::Forbidden().finish();
    }

    let password = match hash_password_a2id(&input.password) {
        Some(password) => password,
        None => return HttpResponse::InternalServerError().finish(),
    };

    let user = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {

                let _ = diesel::insert_into(users::table)
                .values(UserModel {
                    access_level: AccessLevel::Registered as u8,
                    username: &input.username,
                    email: input.email.as_deref(),
                    password_hash: &password,
                })
                .execute(conn)
                .await?;
            
                let user = users::table
                .find(last_insert_id())
                .first::<User>(conn)
                .await?;

                let _ = diesel::update(sessions::table.find(curr_sess_id))
                .set((
                    sessions::user_id.eq(user.id),
                    sessions::ended_at.eq(Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await;
        
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
        Ok(_user) => {
            HttpResponse::Created().finish()
        },

        Err(err) => {
            error!("Error with a database update for session.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
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

    if sess_opt.access_level.is_some() && sess_opt.access_level.unwrap() > claims.role
    || claims.role < AccessLevel::Admin as u8 {
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
                let _ = diesel::update(sessions::table.find(curr_sess_id))
                .set(sess_changes)
                .execute(conn)
                .await?;

                let session = sessions::table
                .find(curr_sess_id)
                .first::<UserSession>(conn)
                .await?;

                Ok(session)
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match res {
        Ok(sess) => {
            match create_authentication_token(sess.id, sess.access_level) {
                Some(token) => HttpResponse::Created().json(SessionResponse {
                    access_token: token,
                }),
                None => HttpResponse::InternalServerError().finish(),
            }
        },

        Err(err) => {
            error!("Error with a database update for session.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}

#[derive(Debug, Serialize)]
struct UserDataResponse {
    pub id: u32,
    pub access_level: u8,
    pub username: String,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
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

                let users: Vec<UserDataResponse> = db_query.select((
                    users::id,
                    users::access_level,
                    users::username,  
                    users::email,  
                    users::created_at,
                ))
                .limit(limit.into())
                .offset(offset.into())
                .load::<(u32, u8, String, Option<String>, NaiveDateTime)>(conn)
                .await?
                .iter()
                .map(|data| {
                    UserDataResponse {
                        id: data.0,
                        access_level: data.1,
                        username: data.2.clone(),
                        email: data.3.clone(),
                        created_at: data.4,
                    }
                })
                .collect();

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