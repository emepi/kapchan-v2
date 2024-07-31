pub mod routes {
    use actix_web::{web, HttpResponse, Responder};
    use chrono::{DateTime, Duration, Utc};
    use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::{
        users::database::{create_anonymous_user, login_user}, 
        utils::{authentication::create_access_token, models::ErrorOutput}
    };

    use super::models::SessionModel;
    

    /// JSON body accepted by `POST /sessions` method.
    #[derive(Debug, Validate, Deserialize)]
    pub struct CreateSessionInput {
        /// Expiration time as UTC timestamp.
        pub exp: Option<i64>,
        #[validate(length(
            min = "5",
            max = "128",
            message = "fails validation - must be 5-128 characters long"
        ))]
        pub password: String,
        #[validate(
            length(
                min = "1",
                max = "16",
                message = "fails validation - must be 1-16 characters long"
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9.-]+$").unwrap(),
                message = "fails validation - is not only alphanumeric and -"
            )
        )]
        pub username: String,
    }

    /// JSON response body from `POST /sessions` and `PUT /sessions` methods.
    #[derive(Debug, Serialize)]
    pub struct CreateSessionOutput {
        pub access_token: String,
    }

    /// Handler for `POST /sessions` request.
    /// 
    /// Client may request access token with `CreateSessionInput` serving as a login
    /// method. Otherwise, token is created for new anonymous user.
    pub async fn create_session(
        conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
        input: Option<web::Json<CreateSessionInput>>,
    ) -> impl Responder {
        // Validate user input
        let mut exp: Option<i64> = None;

        if let Some(input) = &input {
            match input.validate() {
                Ok(_) => (),
                Err(e) => return HttpResponse::UnprocessableEntity().json(ErrorOutput {
                    err: &e.to_string(),
                }),
            }

            exp = input.exp;
        }

        // Attempt to fetch or create an user
        let user = match &input {
            Some(login) => login_user(&login.username, &login.password, &conn_pool).await,
            None => create_anonymous_user(&conn_pool).await,
        };

        let user = match user {
            Ok(user) => user,
            Err(err_res) => return err_res,
        };

        // Resolve session expiry time (default to 1 year).
        let exp = match exp {
            Some(expiry) => match DateTime::from_timestamp(expiry, 0) {
                Some(time) => time.naive_utc(),
                None => return HttpResponse::UnprocessableEntity().json(ErrorOutput {
                    err: "Timestamp out of range."
                }),
            },
            None => (Utc::now() + Duration::days(365)).naive_utc(),
        };

        // Create the session
        let session = SessionModel {
            user_id: user.id,
            ended_at: &exp,
        }
        .insert(&conn_pool)
        .await;

        let session = match session {
            Ok(session) => session,
            Err(_) => return HttpResponse::InternalServerError().json(ErrorOutput {
                err: "Failed to create a session.",
            }),
        };

        // Create a json web token
        let token = match create_access_token(user.access_level, exp.and_utc().timestamp(), session.id) {
            Ok(jwt) => jwt,
            Err(_) => return HttpResponse::InternalServerError().json(ErrorOutput {
                err: "Failed to create a session.",
            }),
        };

        HttpResponse::Created().json(CreateSessionOutput {
            access_token: token,
        })
    }

}

pub mod database {
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::sessions;

    use super::models::{SessionModel, Session};


    impl Session {
        pub async fn by_id(
            id: u32,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Session, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let session = sessions::table
                        .find(id)
                        .first::<Session>(conn)
                        .await?;
                
                        Ok(session)
                    }.scope_boxed())
                    .await
                },

                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }


    impl SessionModel<'_> {
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Session, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(sessions::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let session = sessions::table
                        .find(last_insert_id())
                        .first::<Session>(conn)
                        .await?;
                
                        Ok(session)
                    }.scope_boxed())
                    .await
                },

                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }

    sql_function!(fn last_insert_id() -> Unsigned<Integer>);
}

pub mod models {
    use chrono::NaiveDateTime;
    use diesel::prelude::*;
    use serde::Serialize;

    use crate::schema::sessions;
    
    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = sessions)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Session {
        pub id: u32,
        pub user_id: u32,
        pub created_at: NaiveDateTime,
        pub ended_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = sessions)]
    pub struct SessionModel<'a> {
        pub user_id: u32,
        pub ended_at: &'a NaiveDateTime,
    }
}