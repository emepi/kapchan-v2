use diesel::{result::Error, sql_function, QueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::applications::{Application, ApplicationModel}, schema::applications};


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

sql_function!(fn last_insert_id() -> Unsigned<Integer>);