use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn create_database(
    furdb: &FurDB,
    request: proto::CreateDatabaseRequest,
) -> Result<proto::DatabaseCreatedResponse, ErrorResponse> {
    let database = furdb.create_database(&request.database_id)?;

    let database_info = database.get_database_info();

    Ok(proto::DatabaseCreatedResponse::new(database_info.into()))
}
