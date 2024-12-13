use actix_web::web;
use handlers::index::index_view;


mod handlers {
    pub mod index;
}

pub fn endpoints(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/")
            .route(web::get().to(index_view))
    );
}