use chrono::NaiveDateTime;
use diesel::{
    prelude::*, 
    result::Error, 
    sql_function, 
    ExpressionMethods, 
    QueryDsl, 
    SelectableHelper
};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};
use serde::{Deserialize, Serialize};

use crate::{schema::{application_reviews, applications, users}, services::time::fi_datetime};

use super::users::User;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = applications)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Application {
    pub id: u32,
    pub user_id: u64,
    pub accepted: bool,
    pub background: String,
    pub motivation: String,
    pub other: String,
    pub created_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>
}

impl Application {
    pub async fn review(
        conn_pool: &Pool<AsyncMysqlConnection>,
        application_id: u32,
        reviewer_id: u64,
        accept: bool,
        review_time: NaiveDateTime,
    ) -> Result<Application, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let _ = diesel::update(applications::table.find(application_id))
                    .set((applications::accepted.eq(accept), applications::closed_at.eq(Some(review_time))))
                    .execute(conn)
                    .await?;

                    let _ = diesel::insert_into(application_reviews::table)
                    .values(ApplicationReviewModel {
                        reviewer_id,
                        application_id,
                    })
                    .execute(conn)
                    .await?;

                    let application = applications::table
                    .find(application_id)
                    .first::<Application>(conn)
                    .await?;

                    Ok(application)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn closed_at(
        conn_pool: &Pool<AsyncMysqlConnection>,
        application_id: u32,
    ) -> Result<Option<NaiveDateTime>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let closed = applications::table
                    .find(application_id)
                    .select(applications::closed_at)
                    .first::<Option<NaiveDateTime>>(conn)
                    .await?;

                    Ok(closed)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn by_id(
        conn_pool: &Pool<AsyncMysqlConnection>,
        application_id: u32,
    ) -> Result<ApplicationView, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let application: (User, Application) = applications::table
                    .find(application_id)
                    .inner_join(users::table)
                    .select((User::as_select(), Application::as_select()))
                    .first::<(User, Application)>(conn)
                    .await?;

                    Ok(ApplicationView {
                        application_id: application.1.id,
                        username: application.0.username.unwrap_or_default(),
                        email: application.0.email.unwrap_or_default(),
                        accepted: application.1.accepted,
                        background: application.1.background,
                        motivation: application.1.motivation,
                        other: application.1.other,
                        submission_time: fi_datetime(application.1.created_at),
                        closed_at: application.1.closed_at.map(|date| fi_datetime(date)),
                    })
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn load_previews(
        conn_pool: &Pool<AsyncMysqlConnection>,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<ApplicationPreview>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let application_previews: Vec<(u32, Option<String>, NaiveDateTime)> = 
                    applications::table
                    .inner_join(users::table)
                    .filter(applications::closed_at.is_null())
                    .limit(page_size)
                    .offset(offset)
                    .select((applications::id, users::username, applications::created_at))
                    .load::<(u32, Option<String>, NaiveDateTime)>(conn)
                    .await?;

                    let application_previews: Vec<ApplicationPreview> = application_previews.into_iter()
                    .map(|preview| ApplicationPreview {
                        username: preview.1.unwrap_or_default(),
                        application_id: preview.0,
                        submission_time: fi_datetime(preview.2),
                    })
                    .collect();
            
                    Ok(application_previews)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }

    pub async fn count_previews(
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<i64, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let count: i64 = applications::table
                    .filter(applications::closed_at.is_null())
                    .count()
                    .get_result(conn)
                    .await?;
            
                    Ok(count)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

impl ApplicationReviewModel {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<ApplicationReview, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(application_reviews::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let application_review = application_reviews::table
                    .find(last_insert_id())
                    .first::<ApplicationReview>(conn)
                    .await?;
            
                    Ok(application_review)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = applications)]
pub struct ApplicationModel<'a> {
    pub user_id: u64,
    pub accepted: bool,
    pub background: &'a str,
    pub motivation: &'a str,
    pub other: &'a str,
    pub closed_at: Option<NaiveDateTime>
}

impl ApplicationModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Application, Error> {
        match conn_pool.get().await {
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
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

#[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
#[diesel(table_name = application_reviews)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ApplicationReview {
    pub id: u32,
    pub reviewer_id: u64,
    pub application_id: u32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = application_reviews)]
pub struct ApplicationReviewModel {
    pub reviewer_id: u64,
    pub application_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationPreview {
    pub username: String,
    pub application_id: u32,
    pub submission_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationView {
    pub application_id: u32,
    pub username: String,
    pub email: String,
    pub accepted: bool,
    pub background: String,
    pub motivation: String,
    pub other: String,
    pub submission_time: String,
    pub closed_at: Option<String>
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);