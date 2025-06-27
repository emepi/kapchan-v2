use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{models::users::AccessLevel, services::{applications::submit_application, authentication::resolve_user, users::register_user}, views::{application_view::{self, ApplicationTemplate}, register_view::{self, RegisterTemplate}}};


pub async fn register() -> actix_web::Result<HttpResponse> {
    register_view::render(RegisterTemplate {
        errors: vec![],
    }).await
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(
        length(
            min = "1",
            max = "16",
            message = "Käyttäjänimi täytyy olla 1-16 merkkiä pitkä."
        ),
        regex(
            path = Regex::new(r"^[a-zA-Z0-9.-]+$").unwrap(),
            message = "Käyttäjänimi sisältää kiellettyjä merkkejä!"
        )
    )]
    username: String,
    #[validate(
        length(
            min = "1",
            max = "128",
            message = "Sähköposti täytyy olla 1-128 merkkiä pitkä."
        ),
        regex(
            path = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap(),
            message = "Sähköpostiosoite on virheellinen!"
        )
    )]
    email: String,
    #[validate(length(
        min = "5",
        max = "128",
        message = "Salasana täytyy olla 5-128 merkkiä pitkä."
    ))]
    pwd: String,
}

pub async fn handle_registration(
    user: Option<Identity>,
    input: web::Form<RegisterForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level != AccessLevel::Anonymous as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    match input.validate() {
        Ok(_) => (),
        Err(e) => {
            let errors = e.field_errors()
            .iter()
            .map(|err| err.1.iter().map(|k| k.to_string()).collect::<Vec<String>>())
            .flat_map(|errors| errors)
            .collect();

            let template = RegisterTemplate {
                errors,
            };

            return register_view::render(template).await;
        },
    };

    let result = register_user(
        &conn_pool, 
        user_data.id, 
        &input.username, 
        &input.email, 
        &input.pwd
    ).await;

    match result {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/apply")).finish()),
        Err(e) => match e {
            diesel::result::Error::DatabaseError(e_type, _) => match e_type {
                diesel::result::DatabaseErrorKind::UniqueViolation => 
                {
                    let template = RegisterTemplate {
                        errors: vec!["Käyttäjänimi tai sähköposti on jo olemassa!".to_string()]
                    };

                    register_view::render(template).await
                },
                _ => Ok(HttpResponse::InternalServerError().finish()),
            },
            _ => Ok(HttpResponse::InternalServerError().finish()),
        },
    }
}

pub async fn application(
    user: Option<Identity>,
    req: HttpRequest,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level != AccessLevel::Registered as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    application_view::render(ApplicationTemplate {
        errors: vec![],
    }).await
}

#[derive(Serialize, Deserialize)]
pub struct ApplicationForm {
    background: String,
    motivation: String,
    other: String,
}

pub async fn handle_application(
    user: Option<Identity>,
    input: web::Form<ApplicationForm>,
    conn_pool: web::Data<Pool<AsyncMysqlConnection>>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_data = match resolve_user(user, req, &conn_pool).await {
        Ok(usr_data) => usr_data,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    if user_data.access_level != AccessLevel::Registered as u8 {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let res = submit_application(
        &conn_pool, 
        user_data.id, 
        &input.background, 
        &input.motivation, 
        &input.other
    )
    .await;

    match res {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(_) => {
            let template = ApplicationTemplate {
                errors: vec!["Server error.".to_string()]
            };
    
            application_view::render(template).await
        },
    }
}