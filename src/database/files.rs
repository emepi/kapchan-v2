use diesel::{result::Error, sql_function, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

use crate::{models::posts::{Attachment, AttachmentModel}, schema::attachments};


impl Attachment {
    pub async fn by_id(
        id: u32,
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Attachment, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let attachment = attachments::table
                    .find(id)
                    .first::<Attachment>(conn)
                    .await?;
        
                    Ok(attachment)
                }.scope_boxed())
                .await
            },
    
            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}

impl AttachmentModel<'_> {
    pub async fn insert(
        &self, 
        conn_pool: &Pool<AsyncMysqlConnection>,
    ) -> Result<Attachment, Error> {
        match conn_pool.get().await {
            Ok(mut conn) => {
                conn.transaction::<_, Error, _>(|conn| async move {
                    let _ = diesel::insert_into(attachments::table)
                    .values(self)
                    .execute(conn)
                    .await?;
                
                    let attachment = attachments::table
                    .find(self.id)
                    .first::<Attachment>(conn)
                    .await?;
            
                    Ok(attachment)
                }.scope_boxed())
                .await
            },

            Err(_) => Err(Error::BrokenTransactionManager),
        }
    }
}