use actix_web::{web, HttpResponse, HttpRequest, Responder, http::header};
use diesel::{sql_function, result::Error, prelude::*};
use diesel_async::{RunQueryDsl, AsyncMysqlConnection, pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{user_service::{user::AccessLevel, authentication::validate_claims}, schema::{board_groups, boards}};

use self::board::{BoardGroupModel, BoardGroup, BoardModel, Board};

mod board;


pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/boards")
        .route(web::get().to(boards))
        .route(web::post().to(create_boards))
    )
    .service(
        web::resource("/groups")
        .route(web::post().to(create_groups))
    );
}


#[derive(Debug, Deserialize)]
struct GroupOptions {
    pub name: String,
}

async fn create_groups(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<GroupOptions>,
    req: HttpRequest,
) -> impl Responder {
    // Check access level.
    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::Unauthorized().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if claims.role < AccessLevel::Owner as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let group = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let _ = diesel::insert_into(board_groups::table)
                .values(BoardGroupModel {
                    name: &input.name,
                })
                .execute(conn)
                .await?;

                let group = board_groups::table
                .find(last_insert_id())
                .first::<BoardGroup>(conn)
                .await?;

                Ok(group)
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match group {
        Ok(group) => {
            HttpResponse::Created().json(group)
        },

        Err(err) => {
            error!("Error with a database insert for board group.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}

#[derive(Serialize)]
struct Category{
    id: u32,
    category: String,
    boards: Vec<CategoryBoard>,
}

#[derive(Serialize)]
struct CategoryBoard {
    id: u32,
    name: String,
    handle: String,
}

async fn boards(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> impl Responder {

    let catalog = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let groups = board_groups::table
                .select(BoardGroup::as_select())
                .load(conn)
                .await?;
    
                let boards = Board::belonging_to(&groups)
                .select(Board::as_select())
                .load(conn)
                .await?;

                let catalog = boards
                .grouped_by(&groups)
                .into_iter()
                .zip(groups)
                .map(|(boards, group)| Category {
                    id: group.id,
                    category: group.name,
                    boards: boards.into_iter().map(|board| CategoryBoard {
                        id: board.id,
                        name: board.title,
                        handle: board.handle,
                    }).collect(),
                })
                .collect::<Vec<Category>>();

                Ok(catalog)
            }.scope_boxed())
            .await
        },

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match catalog {
        Ok(catalog) => {
            HttpResponse::Created().json(catalog)
        },

        Err(err) => {
            error!("Error with a database get for boards.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}


#[derive(Debug, Deserialize)]
struct BoardOptions {
    pub group: u32,
    pub handle: String,
    pub title: String,
    pub description: Option<String>,
}

async fn create_boards(
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    input: web::Json<BoardOptions>,
    req: HttpRequest,
) -> impl Responder {
    // Check access level.
    let auth_token = match req.headers().get(header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => return HttpResponse::NotAcceptable().finish(),
            },
        None => return HttpResponse::Unauthorized().finish(),
    };

    let claims = match validate_claims(auth_token) {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if claims.role < AccessLevel::Owner as u8 {
        return HttpResponse::Forbidden().finish();
    }

    let board = match conn_pool.get().await {
        Ok(mut conn) => {
            conn.transaction::<_, Error, _>(|conn| async move {
                let _ = diesel::insert_into(boards::table)
                .values(BoardModel {
                    board_group_id: input.group,
                    handle: &input.handle,
                    title: &input.title,
                    description: input.description.as_deref(),
                })
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

        Err(err) => {
            error!("Connection pool reported an error.\n {:?}", err);

            return HttpResponse::InternalServerError().finish();
        },
    };

    match board {
        Ok(board) => {
            HttpResponse::Created().json(board)
        },

        Err(err) => {
            error!("Error with a database insert for board.\n {:?}", err);

            HttpResponse::InternalServerError().finish()
        },
    }
}


sql_function!(fn last_insert_id() -> Unsigned<Integer>);