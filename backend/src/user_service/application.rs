use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::{
    RunQueryDsl, 
    AsyncMysqlConnection, 
    pooled_connection::deadpool::Pool,
    AsyncConnection, 
    scoped_futures::ScopedFutureExt
};

use crate::schema::applications;


#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = applications)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Application {
    pub id: u32,
    pub user_id: u32,
    pub reviewer_id: Option<u32>,
    pub referer_id: Option<u32>,
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
}


#[derive(Insertable)]
#[diesel(table_name = applications)]
pub struct ApplicationModel<'a> {
    pub user_id: u32,
    pub reviewer_id: Option<u32>,
    pub referer_id: Option<u32>,
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

sql_function!(fn last_insert_id() -> Unsigned<Integer>);