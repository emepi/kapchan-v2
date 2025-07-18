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
use serde::Serialize;

use crate::schema::reports;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Clone)]
#[diesel(table_name = reports)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Report {
    pub id: u32,
    pub post_id: u32,
    pub reason: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = reports)]
pub struct ReportModel<'a> {
    pub post_id: u32,
    pub reason: &'a str,
}

impl ReportModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Report, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(reports::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let report = reports::table
                    .find(last_insert_id())
                    .first::<Report>(conn)
                    .await?;
            
                    Ok(report)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

sql_function!(fn last_insert_id() -> Unsigned<Integer>);
