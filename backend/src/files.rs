pub mod database {
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection, 
        RunQueryDsl
    };

    use crate::schema::attachments;

    use super::models::{Attachment, AttachmentModel};


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
                        .find(last_insert_id())
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


    sql_function!(fn last_insert_id() -> Unsigned<Integer>);
}


pub mod models {
    use diesel::prelude::*;
    use serde::Serialize;

    use crate::schema::attachments;


    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = attachments)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Attachment {
        pub id: u32,
        pub file_name: String,
        pub file_location: String,
        pub thumbnail_location: String,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = attachments)]
    pub struct AttachmentModel<'a> {
        pub file_name: &'a str,
        pub file_location: &'a str,
        pub thumbnail_location: &'a str,
    }
}