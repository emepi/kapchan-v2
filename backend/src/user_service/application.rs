use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl, 
    AsyncMysqlConnection, 
    pooled_connection::deadpool::Pool,
    AsyncConnection, 
    scoped_futures::ScopedFutureExt
};
use serde::Serialize;

use crate::schema::{applications::{self, accepted, closed_at}, users, application_reviews, invites};

use super::user::{User, AccessLevel, UserModel};


#[derive(Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = applications)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Application {
    pub id: u32,
    pub user_id: u32,
    pub accepted: bool,
    pub background: String,
    pub motivation: String,
    pub other: Option<String>,
    pub created_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
}

impl Application {
    pub async fn by_id(
        id: u32,
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<Application> {
        
        match db.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
        
                    let application = applications::table
                    .find(id)
                    .first::<Application>(conn)
                    .await?;
                    
                    Ok(application)
                }.scope_boxed())
                .await
                .ok()
            },

            Err(_) => None,
        }
    }

    pub async fn list_by_status(
        accept: bool,
        handled: bool,
        amount: Option<i64>,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Vec<Application> {

        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
        
                    let mut query = applications::table.into_boxed()
                    .filter(accepted.eq(accept));

                    match handled {
                        true => query = query.filter(closed_at.is_not_null()),
                        false => query = query.filter(closed_at.is_null()),
                    }

                    match amount {
                        Some(amount) => query = query.limit(amount),
                        None => (),
                    }
                    
                    let applications = query
                    .select(Application::as_select())
                    .load(conn)
                    .await
                    .unwrap_or(Vec::new());
                    
                    Ok(applications)
                }.scope_boxed())
                .await
                .unwrap_or(Vec::new())
            },

            Err(_) => Vec::new(),
        }
    }
}


#[derive(Insertable)]
#[diesel(table_name = applications)]
pub struct ApplicationModel<'a> {
    pub user_id: u32,
    pub accepted: bool,
    pub background: &'a str,
    pub motivation: &'a str,
    pub other: Option<&'a str>,
    pub closed_at: Option<NaiveDateTime>,
}

impl ApplicationModel<'_> {
    pub async fn insert(
        &self,
        db: &Pool<AsyncMysqlConnection>,
    ) -> Option<Application> {
        match db.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let _ = diesel::insert_into(applications::table)
                    .values(self)
                    .execute(conn)
                    .await?;
        
                    let application = applications::table
                    .find(last_insert_id())
                    .first::<Application>(conn)
                    .await?;
                    
                    Ok(application)
                }.scope_boxed())
                .await
                .ok()
            },

            Err(_) => None,
        }
    }
}

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = application_reviews)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ApplicationReview {
    pub id: u32,
    pub reviewer_id: u32,
    pub application_id: u32,
}

#[derive(Insertable)]
#[diesel(table_name = application_reviews)]
pub struct ApplicationReviewModel {
    pub reviewer_id: u32,
    pub application_id: u32,
}

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = invites)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Invite {
    pub id: u32,
    pub inviter_id: u32,
    pub application_id: u32,
    pub code: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = invites)]
pub struct InviteModel<'a> {
    pub inviter_id: u32,
    pub application_id: u32,
    pub code: Option<&'a str>,
}


pub async fn list_applications(
    approved: bool,
    handled: bool,
    amount: Option<i64>,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Vec<(Application, User)> {
    match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {

                let mut query = applications::table.into_boxed()
                .inner_join(users::table)
                .filter(accepted.eq(approved));

                match handled {
                    true => query = query.filter(closed_at.is_not_null()),
                    false => query = query.filter(closed_at.is_null()),
                }

                match amount {
                    Some(amount) => query = query.limit(amount),
                    None => (),
                }

                let applications: Vec<(Application, User)> = query
                .select((Application::as_select(), User::as_select()))
                .load::<(Application, User)>(conn)
                .await
                .unwrap_or(Vec::new());

                Ok(applications)
            }.scope_boxed())
            .await
            .unwrap_or(Vec::new())
        },
        Err(_) => Vec::new(),
    }
}

pub async fn close_application(
    reviewer_id: u32,
    user_id: u32,
    application_id: u32,
    resolution: bool,
    conn_pool: &Pool<AsyncMysqlConnection>,
) -> Option<()> {
    let review = ApplicationReviewModel {
        reviewer_id,
        application_id,
    };

    let n_rank = match resolution {
        true => AccessLevel::Member,
        false => AccessLevel::Anonymous,
    };

    let n_usr_rank = UserModel {
        access_level: n_rank as u8,
        username: None,
        email: None,
        password_hash: None,
    };

    match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let _ = diesel::insert_into(application_reviews::table)
                .values(review)
                .execute(conn)
                .await?;

                let _ = diesel::update(applications::table.find(application_id))
                .set((
                    applications::accepted.eq(resolution),
                    applications::closed_at.eq(Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

                User::modify_by_id(user_id, n_usr_rank, &conn_pool)
                .await;

                Ok(())
            }.scope_boxed())
            .await
            .ok()
        },
        Err(_) => None,
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);