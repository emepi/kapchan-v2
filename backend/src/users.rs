/// User access level table.
#[derive(Copy, Clone)]
pub enum AccessLevel {
    Anonymous = 10,
    Registered = 20,
    PendingMember = 30,
    Member = 50,
    Moderator = 90,
    Admin = 100,
    Owner = 200,
    Root = 255,
}


pub mod routes {

}

pub mod database {
    use std::env;

    use actix_web::HttpResponse;
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };
    use log::{error, info};

    use crate::{
        schema::users, 
        utils::{authentication::{hash_password_pbkdf2, validate_password_pbkdf2}, models::ErrorOutput}
    };

    use super::{models::{User, UserModel}, AccessLevel};


    pub async fn create_anonymous_user(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, HttpResponse> {
        UserModel {
            access_level: AccessLevel::Anonymous as u8,
            username: None,
            password_hash: None,
        }
        .insert(conn_pool)
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())
    }

    pub async fn create_root_user(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) {
        let root_pwd = env::var("ROOT_PASSWORD");

        match root_pwd {
            Ok(pwd) => match User::by_username("root", conn_pool).await {
                Ok(root_user) => {
                    // Update root user
                    let res = UserModel {
                        access_level: AccessLevel::Root as u8,
                        username: Some("root"),
                        password_hash: Some(&hash_password_pbkdf2(&pwd)),
                    }
                    .update_by_id(root_user.id, conn_pool)
                    .await;

                    match res {
                        Ok(_) => info!("Root user updated."),
                        Err(db_err) => error!(
                            "Root user was not set or updated due to an database error: {:?}.", 
                            db_err
                        ),
                    }
                },
                Err(db_err) => match db_err {
                    Error::NotFound => {
                        let res = UserModel {
                            access_level: AccessLevel::Root as u8,
                            username: Some("root"),
                            password_hash: Some(&hash_password_pbkdf2(&pwd)),
                        }
                        .insert(conn_pool)
                        .await;

                        match res {
                            Ok(_) => info!("Root user created."),
                            Err(db_err) => error!(
                                "Root user was not set or updated due to an database error: {:?}.", 
                                db_err
                            ),
                        }
                    },
                    _ => error!(
                        "Root user was not set or updated due to an database error: {:?}.", 
                        db_err
                    ),
                },
            },
            Err(_) => info!("Root user was not set or updated."),
        }
    }

    pub async fn login_user(
        username: &str,
        password: &str,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<User, HttpResponse> {
        let user = match User::by_username(username, conn_pool).await {
            Ok(user) => user,
            Err(err) => match err {
                Error::NotFound => return Err(HttpResponse::NotFound().json(ErrorOutput {
                    err: "User doesn't exist.",
                })),
                _ => return Err(HttpResponse::InternalServerError().finish()),
            },
        };

        let pwd_hash = match &user.password_hash {
            Some(hash) => hash,
            None => return Err(HttpResponse::InternalServerError().finish()), // Illegal state
        };

        match validate_password_pbkdf2(&pwd_hash, password) {
            true => Ok(user),
            false => Err(HttpResponse::Unauthorized().json(ErrorOutput {
                err: "Invalid password.",
            })),
        }
    }


    impl User {
        pub async fn by_username(
            username: &str,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<User, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let user = users::table
                        .filter(users::username.eq(username))
                        .first::<User>(conn)
                        .await?;
            
                        Ok(user)
                    }.scope_boxed())
                    .await
                },

                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }


    impl UserModel<'_> {
        /// Inserts `UserModel` into the database and returns the resulting `User`.
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<User, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(users::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let user = users::table
                        .find(last_insert_id())
                        .first::<User>(conn)
                        .await?;
                
                        Ok(user)
                    }.scope_boxed())
                    .await
                },
    
                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }

        pub async fn update_by_id(
            &self,
            user_id: u32,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<(), Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::update(users::table.find(user_id))
                        .set(self)
                        .execute(conn)
                        .await;
                
                        Ok(())
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
    use diesel::prelude::*;
    use chrono::NaiveDateTime;
    use serde::Serialize;
    use crate::schema::users;

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = users)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct User {
        pub id: u32,
        pub access_level: u8,
        pub username: Option<String>,
        pub password_hash: Option<String>,
        pub created_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = users)]
    pub struct UserModel<'a> {
        pub access_level: u8,
        pub username: Option<&'a str>,
        pub password_hash: Option<&'a str>,
    }
}