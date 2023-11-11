use crate::server::service::Service;

const USER_SERVICE_ID: u32 = 1;

pub struct UserService {
    
}

impl UserService {
    pub fn new() -> Box<UserService> {

        Box::new(UserService {
            
        })
    }
}

impl Service for UserService {
    fn data_feed(&self) -> Option<crate::server::service::ServiceDataFeed> {
        None
    }

    fn id(&self) -> u32 { USER_SERVICE_ID }
}