use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    AsyncConnection,
    scoped_futures::ScopedFutureExt,
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    RunQueryDsl,
};
use crate::schema::{users, sessions};

use super::{
    session::{UserSession, UserSessionModel}, 
    authentication::hash_password_a2id
};


#[derive(Copy, Clone)]
pub enum AccessLevel {
    Anonymous = 10,
    PendingMember = 15,
    Member = 20,
    Admin = 100,
    Root = 255,
}


#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u32,
    pub access_level: u8,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: NaiveDateTime,
}

impl User {

    pub async fn modify(
        &self, 
        update_mdl: UserModel<'_>, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) {
        let username = update_mdl.username
        .map(|username| {
            // TODO: sanitize
            username
        })
        .or(self.username.as_deref());

        let password = update_mdl.password_hash
        .and_then(|pwd| {
            // TODO: sanitize
            hash_password_a2id(pwd)
        })
        .or(self.password_hash.clone());

        let email = update_mdl.email
        .map(|email| {
            // TODO: sanitize
            email
        })
        .or(self.email.as_deref());

        match conn_pool.get().await {

            Ok(mut conn) => {
                let _ = conn.transaction::<_, Error, _>(|conn| async move {

                    let _ = diesel::update(users::table.find(self.id))
                    .set((
                        users::access_level.eq(update_mdl.access_level),
                        users::username.eq(username),
                        users::email.eq(email),
                        users::password_hash.eq(password),
                    ))
                    .execute(conn)
                    .await;
            
                    Ok(())
                }.scope_boxed())
                .await;
            },

            Err(_) => (),
        }
    }

    pub async fn modify_by_id(
        id: u32,
        update_mdl: UserModel<'_>, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<()> {
        let username = match update_mdl.username {
            Some(usrname) => {
                 // TODO: sanitize
                usrname
            },
            None => return None,
        };

        let password = match update_mdl.password_hash {
            Some(pwd) => {
                // TODO: sanitize
                hash_password_a2id(pwd)
            },
            None => return None,
        };

        let email = match update_mdl.email {
            Some(email) => {
                // TODO: sanitize
                email
            },
            None => return None,
        };

        match conn_pool.get().await {

            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let _ = diesel::update(users::table.find(id))
                    .set((
                        users::access_level.eq(update_mdl.access_level),
                        users::username.eq(username),
                        users::email.eq(email),
                        users::password_hash.eq(password),
                    ))
                    .execute(conn)
                    .await;
            
                    Ok(())
                }.scope_boxed())
                .await
                .ok()
            },

            Err(_) => None,
        }
    }

    pub async fn create_session(
        &self,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<UserSession> {

        let session_model = UserSessionModel {
            user_id: self.id,
            access_level: self.access_level,
            mode: 1,
            ip_address: ip_address,
            user_agent: user_agent,
            ended_at: None,
        };

        let mut connection = db.get().await
        .ok()?;
        
        let result = connection.transaction::<_, Error, _>(|conn| async move {
            let _ = diesel::insert_into(sessions::table)
            .values(session_model)
            .execute(conn)
            .await?;

            let user_session = sessions::table
            .find(last_insert_id())
            .first::<UserSession>(conn)
            .await?;
            
            Ok(user_session)
        }.scope_boxed())
        .await
        .ok();

        result
    }

    pub async fn by_username(
        username: &str, 
        db: &Pool<AsyncMysqlConnection>
    ) -> Option<User> {
        let mut connection = db.get().await
        .ok()?;

        connection.transaction::<_, Error, _>(|conn| async move {
            let user = users::table
            .filter(users::username.eq(username))
            .first::<User>(conn)
            .await?;

            Ok(user)
        }.scope_boxed())
        .await
        .ok()
    }

    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<User> {
        let mut connection = conn_pool.get().await
        .ok()?;

        connection.transaction::<_, Error, _>(|conn| async move {
            let user = users::table
            .find(id)
            .first::<User>(conn)
            .await?;

            Ok(user)
        }.scope_boxed())
        .await
        .ok()
    }
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

impl Default for UserModel<'_> {
    fn default() -> Self {
        Self {
            access_level: AccessLevel::Anonymous as u8, 
            username: None,
            email: None,
            password_hash: None,
        }
    }
}

impl UserModel<'_> {
    pub async fn insert(
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


sql_function!(fn last_insert_id() -> Unsigned<Integer>);