use std::env;

use diesel::{result::Error, QueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    AsyncMysqlConnection, 
    AsyncConnection, 
    RunQueryDsl, 
    scoped_futures::ScopedFutureExt,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

use crate::schema::users;

use super::user::User;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,          // Expiration time (as UTC timestamp)
    pub iat: usize,          // Issued at (as UTC timestamp)
    pub sub: String,         // Subject (whom token refers to)
}

pub async fn authenticate(
    token: &str, 
    conn_pool: &Pool<AsyncMysqlConnection>
) -> Option<User> {

    let jwt_secret = env::var("JWT_SECRET")
    .expect(".env variable `JWT_SECRET` must be set");

    let claims = match decode::<Claims>(
        token, 
        &DecodingKey::from_secret(jwt_secret.as_ref()), 
        &Validation::default(),
    ) {
        Ok(data) =>  Some(data.claims),

        Err(err) => {
            // TODO: match specific errors
            match err.kind() {
                _ => None,
            }
        },
    }?;

    let user_id = claims.sub.parse::<u32>().ok()?;

    let mut connection = conn_pool.get().await.ok()?;

    connection.transaction::<_, Error, _>(|conn| async move {

        let user = users::table
        .find(user_id)
        .first::<User>(conn)
        .await?;
        
        Ok(user)
    }.scope_boxed())
    .await
    .ok()
}