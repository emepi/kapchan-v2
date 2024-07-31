pub mod routes {
    use actix_web::{web, HttpRequest, HttpResponse, Responder};
    use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
    use regex::Regex;
    use serde::Deserialize;
    use validator::Validate;

    use crate::{users::AccessLevel, utils::{authentication::authenticate_user, models::ErrorOutput}};

    use super::models::BoardModel;

    
    #[derive(Debug, Deserialize, Validate)]
    pub struct CreateBoardInput {
        #[validate(
            length(
                min = "1",
                max = "100",
                message = "fails validation - must be 1-100 characters long"
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9]+$").unwrap(),
                message = "fails validation - is not only alphanumeric"
            )
        )]
        pub title: String,
        #[validate(
            length(
                min = "1",
                max = "8",
                message = "fails validation - must be 1-8 characters long"
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9]+$").unwrap(),
                message = "fails validation - is not only alphanumeric"
            )
        )]
        pub handle: String,
        pub access_level: u8,
        pub nsfw: bool,
    }


    pub async fn create_board(
        conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
        input: web::Json<CreateBoardInput>,
        req: HttpRequest,
    ) -> impl Responder {
        // Check user permissions.
        let conn_info = match authenticate_user(&conn_pool, req).await {
            Ok(conn_info) => conn_info,
            Err(err_res) => return err_res,
        };

        if conn_info.access_level < AccessLevel::Owner as u8 {
            return HttpResponse::Forbidden().finish();
        }

        // Validate input data
        match input.validate() {
            Ok(_) => (),
            Err(e) => return HttpResponse::UnprocessableEntity().json(ErrorOutput {
                err: &e.to_string(),
            }),
        }

        let board = BoardModel {
            handle: &input.handle,
            title: &input.title,
            access_level: input.access_level,
            nsfw: input.nsfw,
        }
        .insert(&conn_pool)
        .await;

        match board {
            Ok(board) => HttpResponse::Created().json(board),
            Err(err) => match err {
                diesel::result::Error::DatabaseError(db_err, _) => match db_err {
                    diesel::result::DatabaseErrorKind::UniqueViolation => 
                        HttpResponse::UnprocessableEntity().json(ErrorOutput {
                            err: "Board handle is not unique.",
                        }),
                    _ => HttpResponse::InternalServerError().finish(),
                },
                _ => HttpResponse::InternalServerError().finish(),
            },
        }
    }
}


pub mod database {
    use diesel::{result::Error, sql_function, QueryDsl};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection,
        RunQueryDsl,
    };

    use crate::schema::boards;

    use super::models::{Board, BoardModel};


    impl BoardModel<'_> {
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Board, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(boards::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let board = boards::table
                        .find(last_insert_id())
                        .first::<Board>(conn)
                        .await?;
                
                        Ok(board)
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

    use crate::schema::boards;


    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = boards)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Board {
        pub id: u32,
        pub handle: String,
        pub title: String,
        pub access_level: u8,
        pub nsfw: bool,
    }

    #[derive(Insertable)]
    #[diesel(table_name = boards)]
    pub struct BoardModel<'a> {
        pub handle: &'a str,
        pub title: &'a str,
        pub access_level: u8,
        pub nsfw: bool,
    }
}