use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, result::Error};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, 
    scoped_futures::ScopedFutureExt,
    RunQueryDsl,
};

use crate::schema::sessions;

use super::authentication::validate_session_id;

#[derive(Queryable, Selectable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct UserSession {
    pub id: u32,
    pub user_id: Option<u32>,
    pub access_level: u8,
    pub mode: u8,
    pub created_at: NaiveDateTime,
    pub ended_at: Option<NaiveDateTime>,
}

impl UserSession {

    pub async fn by_id(
        sess_id: u32, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<UserSession> {
        let mut conn = conn_pool.get().await.ok()?;
        
        conn.transaction::<_, Error, _>(|conn| async move {
            
            let sess = sessions::table
            .find(sess_id)
            .first::<UserSession>(conn)
            .await?;
        
            Ok(sess)
        }.scope_boxed())
        .await
        .ok()
    }

    pub async fn by_token(
        token: &str,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Option<UserSession> {
        let sess_id = validate_session_id(token)?;

        UserSession::by_id(sess_id, conn_pool)
        .await
    }

    pub async fn end_session(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>
    ) {
        let conn = conn_pool.get().await;

        match conn {

            Ok(mut conn) => {
                let _ = conn.transaction::<_, Error, _>(|conn| async move {
                    let time = Utc::now().naive_utc();

                    let _ = diesel::update(sessions::table.find(self.id))
                    .set(sessions::ended_at.eq(time))
                    .execute(conn)
                    .await;
                
                    Ok(())
                }.scope_boxed())
                .await;
            },

            Err(_) => (),
        }
    }

    pub fn valid(&self) -> bool {
        self.ended_at.is_none()
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = sessions)]
pub struct UserSessionModel<'a> {
    pub user_id: Option<u32>,
    pub access_level: u8,
    pub mode: u8,
    pub ended_at: Option<&'a NaiveDateTime>,
}


pub async fn active_sessions_by_user_id(
    user_id: u32,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Vec<UserSession> {
    match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let sess: Vec<UserSession> = sessions::table
                .filter(sessions::user_id.eq(user_id).and(sessions::ended_at.is_null()))
                .select(UserSession::as_select())
                .load::<UserSession>(conn)
                .await
                .unwrap_or(Vec::new());

                Ok(sess)
            }.scope_boxed())
            .await
            .unwrap_or(Vec::new())
        },

        Err(_) => Vec::new(),
    }
}

pub async fn is_active_session(
    sess_id: u32,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> bool {
    match UserSession::by_id(sess_id, conn_pool).await {
        Some(sess) => sess.valid(),
        None => false,
    }
}


sql_function!(fn last_insert_id() -> Unsigned<Integer>);