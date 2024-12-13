use actix_web::{web, HttpResponse, Responder};


async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/")
            .route(web::get().to(hello)
    ))
    .service(
        web::resource("/test")
            .route(web::get().to(hello))
    );
}