use crate::server::service::Service;

pub struct UserService {

}

impl Service for UserService {
    fn data_feed(&self) -> Option<crate::server::service::ServiceDataFeed> {
        None
    }
}