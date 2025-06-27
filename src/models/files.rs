use serde::{Deserialize, Serialize};
use diesel::{
    prelude::*, 
    result::Error, 
    sql_function, 
    ExpressionMethods, 
    QueryDsl, 
    SelectableHelper
};
use diesel_async::{
    pooled_connection::deadpool::Pool, 
    scoped_futures::ScopedFutureExt, 
    AsyncConnection, 
    AsyncMysqlConnection, 
    RunQueryDsl
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub file_name: String,
    pub file_type: String,
}