pub mod board;
pub mod post;


use std::{fs, path::PathBuf, str::FromStr};

use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDateTime;
use diesel::{result::{DatabaseErrorKind, Error::{self, DatabaseError, NotFound}}, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection, AsyncMysqlConnection, RunQueryDsl};
use log::info;
use post::{File, FileModel, Post, PostModel, Thread, ThreadModel};
use serde::{Deserialize, Serialize};

use crate::{schema::{files, posts::{self, created_at, op_id}, threads::{self, bump_date, pinned}}, user_service::authentication::{authenticate_user, AccessLevel}};

use self::board::{Board, BoardModel};


/// API endpoints exposed by the board service.
pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/files/{id}")
        .route(web::get().to(serve_files))
    )
    .service(
        web::resource("/boards")
        .route(web::get().to(boards))
        .route(web::post().to(create_board))
    )
    .service(
        web::resource("/boards/{id}/threads")
        .route(web::get().to(fetch_threads))
    )
    .service(
        web::resource("/threads")
        .route(web::post().to(create_thread))
    )
    .service(
        web::resource("/threads/{id}")
        .route(web::get().to(serve_thread))
        .route(web::post().to(create_post))
    );
}


/// Handler for `GET /boards` request.
async fn boards(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> impl Responder {
    let boards = Board::fetch_boards(&conn_pool).await;

    match boards {
        Ok(boards) => HttpResponse::Ok().json(boards),
        Err(err) => match err {
            NotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

/// JSON body accepted by `POST /boards` method.
#[derive(Debug, Deserialize)]
struct CreateBoardInput {
    pub title: String,
    pub handle: String,
    pub access_level: u8,
    pub bump_limit: u32,
    pub nsfw: bool,
}

/// Handler for `POST /boards` request.
async fn create_board(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<CreateBoardInput>,
    req: HttpRequest,
) -> impl Responder {
    // Check user permissions.
    let conn_info = match authenticate_user(&conn_pool, req).await {
        Ok(conn_info) => conn_info,
        Err(mut err_res) => return err_res.finish(),
    };

    if conn_info.access_level < AccessLevel::Owner as u8 {
        return HttpResponse::Forbidden().finish();
    }

    // Insert the board into database.
    let board = BoardModel {
        handle: &input.handle,
        title: &input.title,
        access_level: input.access_level,
        bump_limit: input.bump_limit,
        nsfw: input.nsfw,
    }
    .insert(&conn_pool)
    .await;

    match board {
        Ok(board) => HttpResponse::Created().json(board),
        Err(err) => match err {
            DatabaseError(db_err, _) => match db_err {
                DatabaseErrorKind::UniqueViolation => HttpResponse::BadRequest().finish(),
                _ => HttpResponse::InternalServerError().finish(),
            },
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

/// Multipart form accepted by `POST /threads` method.
#[derive(Debug, MultipartForm)]
struct CreateThreadInput {
    pub title: Text<String>,
    pub body: Text<String>,
    pub board: Text<String>,
    pub attachment: TempFile,
}

/// Handler for `POST /threads` request.
async fn create_thread(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    MultipartForm(input): MultipartForm<CreateThreadInput>,
    req: HttpRequest,
) -> impl Responder {
    // Fetch board info
    let handle = input.board.into_inner();
    let board = Board::by_handle(&handle, &conn_pool).await;

    let board = match board {
        Ok(board) => board,
        Err(_) => {
            println!("board errored");
            return HttpResponse::InternalServerError().finish()
        },
    };


    // Check permissions
    if board.access_level > AccessLevel::Anonymous as u8 {
        let conn_info = match authenticate_user(&conn_pool, req).await {
            Ok(conn_info) => conn_info,
            Err(mut err_res) => return err_res.finish(),
        };
    
        if conn_info.access_level < board.access_level {
            return HttpResponse::Forbidden().finish();
        }
    }

    // Create a thread
    let op_post = PostModel {
        op_id: None,
        body: input.body.into_inner(),
        access_level: board.access_level,
    }
    .insert(&conn_pool)
    .await;

    let op_post = match op_post {
        Ok(op_post) => op_post,
        Err(e) => {
            println!("Op post errored {:?}",e);
            return HttpResponse::InternalServerError().finish()
        },
    };

    let thread = ThreadModel {
        id: op_post.id,
        board: board.id,
        title: input.title.into_inner(),
        pinned: false,
    };

    match thread.insert(&conn_pool).await {
        Ok(_) => (),
        Err(e) => {
            println!("thread errored {:?}",e);
            return HttpResponse::InternalServerError().finish()
        },
    }

    let mime = match input.attachment.content_type {
        Some(mime) => mime,
        None => {
            println!("Mime errored");
            return HttpResponse::InternalServerError().finish()
        },
    };

    match mime.type_() {
        mime::IMAGE => {
            let dir_path = format!("target/files/{}", op_post.id);

            match tokio::fs::create_dir_all(&dir_path).await {
                Ok(_) => (),
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }

            let file = input.attachment.file;
            let file_name = match input.attachment.file_name {
                Some(name) => name,
                None => String::from("file"),
            };

            let file_path = format!("{}/file.{}", dir_path, mime.subtype().as_str()); // TODO

            match file.persist(&file_path) {
                Ok(_) => (),
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }

            // TODO: create a thumbnail

            let file_model = FileModel {
                id: op_post.id,
                file_name,
                thumbnail: String::default(),
                file_path,
                file_type: mime.type_().to_string(),
            };

            match file_model.insert(&conn_pool).await {
                Ok(_) => (),
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }
        },
        _ => ()
    };
    

    HttpResponse::Created().finish()
}

#[derive(Debug, Serialize)]
pub struct ThreadOutput {
    pub title: String,
    pub pinned: bool,
    pub op_post: PostOutput,
}

#[derive(Debug, Serialize)]
pub struct PostOutput{
    post_id: u32,
    body: String,
    attachment: Option<String>,
    created_at: NaiveDateTime,
}

/// Handler for `GET /boards/{id}/threads` request.
async fn fetch_threads(
    board: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {

    let board_id = board.into_inner().0;
    println!("Requested threads for the board id: {}", board_id);

    //TODO: check permissions

    let threads = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let thread: Vec<(Thread, (Post, Option<File>))> = threads::table
                .filter(threads::board.eq(board_id))
                .order((pinned.eq(true), bump_date.desc()))
                .inner_join((posts::table).left_join(files::table))
                .select((Thread::as_select(), (Post::as_select(), Option::<File>::as_select())))
                .load::<(Thread, (Post, Option<File>))>(conn)
                .await?;
        
                Ok(thread)
            }.scope_boxed())
            .await
        },

        // Failed to get a connection from the pool.
        Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
    };

    let threads = match threads {
        Ok(threads) => threads.into_iter()
        .map(|db_res| {
            ThreadOutput {
                title: db_res.0.title,
                pinned: db_res.0.pinned,
                op_post: PostOutput {
                    post_id: db_res.1.0.id,
                    body: db_res.1.0.body,
                    attachment: db_res.1.1.map(|file| file.file_type),
                    created_at: db_res.1.0.created_at,
                },
            }
        }).collect::<Vec<ThreadOutput>>(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(threads)
}

async fn serve_files(
    file: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<NamedFile> {
    let file_id = file.into_inner().0;

    let file_info = match File::by_id(file_id, &conn_pool).await {
        Ok(info) => info,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    let path: PathBuf = match file_info.file_path.parse() {
        Ok(path) => path,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    Ok(NamedFile::open(path)?)
}

#[derive(Debug, Serialize)]
pub struct ThreadResponseOutput {
    pub title: String,
    pub pinned: bool,
    pub op_post: PostOutput,
    pub responses: Vec<PostOutput>,
}

async fn serve_thread(
    thread: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {
    let thread_id = thread.into_inner().0;

    let thread = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let thread: (Thread, (Post, Option<File>)) = threads::table
                .find(thread_id)
                .inner_join((posts::table).left_join(files::table))
                .select((Thread::as_select(), (Post::as_select(), Option::<File>::as_select())))
                .first::<(Thread, (Post, Option<File>))>(conn)
                .await?;
        
                Ok(thread)
            }.scope_boxed())
            .await
        },

        // Failed to get a connection from the pool.
        Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
    };

    // TODO: check not found
    let thread = match thread {
        Ok(thread) => thread,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    //TODO check permissions

    let responses = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let thread: Vec<(Post, Option<File>)> = posts::table
                .filter(op_id.eq(thread.0.id))
                .order(created_at.asc())
                .left_join(files::table)
                .select((Post::as_select(), Option::<File>::as_select()))
                .load::<(Post, Option<File>)>(conn)
                .await?;
        
                Ok(thread)
            }.scope_boxed())
            .await
        },

        // Failed to get a connection from the pool.
        Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
    };

    let responses = match responses {
        Ok(responses) => responses,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }
    .into_iter()
    .map(|db_res| {
        PostOutput {
            post_id: db_res.0.id,
            body: db_res.0.body,
            attachment: db_res.1.map(|file| file.file_type),
            created_at: db_res.0.created_at,
        }
    })
    .collect::<Vec<PostOutput>>();

    HttpResponse::Ok().json(ThreadResponseOutput {
        title: thread.0.title,
        pinned: thread.0.pinned,
        op_post: PostOutput {
            post_id: thread.1.0.id,
            body: thread.1.0.body,
            attachment: thread.1.1.map(|file| file.file_type),
            created_at: thread.1.0.created_at,
        },
        responses,
    })
}

/// Multipart form accepted by `POST /threads/{id}` method.
#[derive(Debug, MultipartForm)]
struct CreatePostInput {
    pub body: Text<String>,
    pub attachment: TempFile,
}

async fn create_post(
    thread: web::Path<(u32,)>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    MultipartForm(input): MultipartForm<CreatePostInput>,
    req: HttpRequest,
) -> impl Responder {
    let thread_id = thread.into_inner().0;

    let op_post = match Post::by_id(thread_id, &conn_pool).await {
        Ok(post) => post,
        Err(_) => return HttpResponse::InternalServerError().finish(), //TODO: check not found
    };

    let post = PostModel {
        op_id: Some(op_post.id),
        body: input.body.into_inner(),
        access_level: op_post.access_level,
    }
    .insert(&conn_pool)
    .await;

    let post = match post {
        Ok(post) => post,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let mime = match input.attachment.content_type {
        Some(mime) => mime,
        None => {
            println!("Mime errored");
            return HttpResponse::InternalServerError().finish()
        },
    };

    match mime.type_() {
        mime::IMAGE => {
            let dir_path = format!("target/files/{}", post.id);

            match tokio::fs::create_dir_all(&dir_path).await {
                Ok(_) => (),
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }

            let file = input.attachment.file;
            let file_name = match input.attachment.file_name {
                Some(name) => name,
                None => String::from("file"),
            };

            let file_path = format!("{}/file.{}", dir_path, mime.subtype().as_str()); // TODO

            match file.persist(&file_path) {
                Ok(_) => (),
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }

            // TODO: create a thumbnail

            let file_model = FileModel {
                id: post.id,
                file_name,
                thumbnail: String::default(),
                file_path,
                file_type: mime.type_().to_string(),
            };

            match file_model.insert(&conn_pool).await {
                Ok(_) => (),
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }
        },
        _ => ()
    };

    let _ = Thread::bump_by_id(thread_id, &conn_pool).await;

    HttpResponse::Created().finish()
}