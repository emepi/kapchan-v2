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
    use actix_web::HttpResponse;
    use diesel::{result::Error, sql_function, QueryDsl};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::users;

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
    
                // Failed to get a connection from the pool.
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