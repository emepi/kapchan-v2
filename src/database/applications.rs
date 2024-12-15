use chrono::NaiveDateTime;
use diesel::{result::Error, sql_function, ExpressionMethods, QueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::applications::{Application, ApplicationModel, ApplicationPreview}, schema::{applications::{self, accepted, created_at, id}, users::{self, username}}, services::time::fi_datetime};


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

impl Application {
    pub async fn load_previews(
        conn_pool: &Pool<AsyncMysqlConnection>,
        is_accepted: bool,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<ApplicationPreview>, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let application_previews: Vec<(u32, Option<String>, NaiveDateTime)> = 
                    applications::table
                    .inner_join(users::table)
                    .filter(accepted.eq(is_accepted))
                    .limit(page_size)
                    .offset(offset)
                    .select((id, username, created_at))
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

    pub async fn count(
        conn_pool: &Pool<AsyncMysqlConnection>,
        is_accepted: bool,
    ) -> Result<i64, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {

                    let count: i64 = applications::table
                    .filter(accepted.eq(is_accepted))
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

sql_function!(fn last_insert_id() -> Unsigned<Integer>);