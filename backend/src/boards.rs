pub mod routes {
    use actix_multipart::form::MultipartForm;
    use actix_web::{web, HttpRequest, HttpResponse, Responder};
    use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
    use validator::Validate;

    use crate::users::{authentication::authenticate_user, models::AccessLevel};

    use super::models::{Board, BoardModel, CreateBoardInput, CreateThreadInput};


    pub async fn boards(
        conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    ) -> impl Responder {
        let boards = Board::fetch_boards(&conn_pool).await;

        match boards {
            Ok(boards) => HttpResponse::Ok().json(boards),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
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
            Err(e) => return HttpResponse::UnprocessableEntity().json(&e.to_string()),
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
                        HttpResponse::UnprocessableEntity().json(
                            String::from("Board handle is not unique.")
                        ),
                    _ => HttpResponse::InternalServerError().finish(),
                },
                _ => HttpResponse::InternalServerError().finish(),
            },
        }
    }

    async fn create_thread(
        conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
        MultipartForm(input): MultipartForm<CreateThreadInput>,
        req: HttpRequest,
    ) -> impl Responder {
        HttpResponse::Created()
    }
}


pub mod database {
    use diesel::{result::Error, prelude::*};
    use diesel_async::{
        pooled_connection::deadpool::Pool, 
        scoped_futures::ScopedFutureExt, 
        AsyncConnection, 
        AsyncMysqlConnection,
        RunQueryDsl,
    };

    use crate::schema::{boards, threads};

    use super::models::{Board, BoardModel, Thread, ThreadModel};


    impl Board {
        pub async fn fetch_boards(
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Vec<Board>, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let boards = boards::table
                        .select(Board::as_select())
                        .load(conn)
                        .await?;
                
                        Ok(boards)
                    }.scope_boxed())
                    .await
                },
    
                Err(_) => Err(Error::BrokenTransactionManager),
            }
        }
    }

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

    impl ThreadModel<'_> {
        pub async fn insert(
            &self,
            conn_pool: &Pool<AsyncMysqlConnection>,
        ) -> Result<Thread, Error> {
            match conn_pool.get().await {
                Ok(mut conn) => {
                    conn.transaction::<_, Error, _>(|conn| async move {
                        let _ = diesel::insert_into(threads::table)
                        .values(self)
                        .execute(conn)
                        .await?;
                    
                        let thread = threads::table
                        .find(last_insert_id())
                        .first::<Thread>(conn)
                        .await?;
                
                        Ok(thread)
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
    use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
    use chrono::NaiveDateTime;
    use diesel::prelude::*;
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::schema::{attachments, boards, threads, posts};


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

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = threads)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Thread {
        pub id: u32,
        pub board_id: u32,
        pub title: String,
        pub pinned: bool,
        pub bump_time: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = threads)]
    pub struct ThreadModel<'a> {
        pub board_id: u32,
        pub title: &'a str,
        pub pinned: bool,
    }

    #[derive(Debug, Queryable, Identifiable, Selectable, Serialize)]
    #[diesel(table_name = posts)]
    #[diesel(check_for_backend(diesel::mysql::Mysql))]
    pub struct Post {
        pub id: u32,
        pub user_id: u32,
        pub thread_id: u32,
        pub access_level: u8,
        pub username: bool,
        pub message: String,
        pub created_at: NaiveDateTime,
    }

    #[derive(Insertable, AsChangeset)]
    #[diesel(table_name = posts)]
    pub struct PostModel<'a> {
        pub user_id: u32,
        pub thread_id: u32,
        pub access_level: u8,
        pub username: bool,
        pub message: &'a str,
    }

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

    #[derive(Debug, Deserialize, Validate)]
    pub struct CreateBoardInput {
        #[validate(
            length(
                min = "1",
                max = "100",
                message = "Title must be 1-100 characters long"
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9]+$").unwrap(),
                message = "Title must be alphanumeric"
            )
        )]
        pub title: String,
        #[validate(
            length(
                min = "1",
                max = "8",
                message = "Handle must be 1-8 characters long"
            ),
            regex(
                path = Regex::new(r"^[a-zA-Z0-9]+$").unwrap(),
                message = "Handle must be alphanumeric"
            )
        )]
        pub handle: String,
        pub access_level: u8,
        pub nsfw: bool,
    }

    #[derive(Debug, MultipartForm)]
    pub struct CreateThreadInput {
        pub title: Text<String>,
        pub body: Text<String>,
        pub board: Text<String>,
        pub attachment: TempFile,
    }
}