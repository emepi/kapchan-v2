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

use crate::schema::{applications::{self, accepted, closed_at}, users, application_reviews};

use super::user::{User, UserModel};


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

sql_function!(fn last_insert_id() -> Unsigned<Integer>);