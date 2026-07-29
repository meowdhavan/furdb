use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn get_database(
    furdb: &FurDB,
    request: proto::GetDatabaseRequest,
) -> Result<proto::DatabaseInfoResponse, ErrorResponse> {
    let database = furdb.get_database(&request.database_id)?;

    let database_info_full = database.get_database_info_full()?;

    Ok(proto::DatabaseInfoResponse::new(database_info_full.into()))
}
