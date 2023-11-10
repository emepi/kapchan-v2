use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    AsyncConnection,
    scoped_futures::ScopedFutureExt,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    RunQueryDsl,
};
use crate::schema::users;


#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u32,
    pub access_level: u8,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Model for inserting a new user into the database.
#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct UserModel<'a> {
    pub access_level: u8,
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
}

impl UserModel<'_> {
    pub async fn register_user(
        &self, 
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<User> {
        // TODO: inspect connection pooling errors
        let mut connection = db.get().await
        .ok()?;

        let result = connection.transaction::<_, Error, _>(|conn| async move {
            
            // TODO: inspect race conditions & error types
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
        .ok();

        result
    }
}

pub struct UserSession {
    pub id: u64,
}

// TODO: move to a different module
sql_function!(fn last_insert_id() -> Unsigned<Integer>);