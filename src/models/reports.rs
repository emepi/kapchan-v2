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
use serde::Serialize;

use crate::schema::reports;


#[derive(Debug, Queryable, Identifiable, Selectable, Serialize, Clone)]
#[diesel(table_name = reports)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Report {
    pub id: u32,
    pub post_id: u32,
    pub reason: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = reports)]
pub struct ReportModel<'a> {
    pub post_id: u32,
    pub reason: &'a str,
}
