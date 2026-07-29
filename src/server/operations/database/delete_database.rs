use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn delete_database(
    furdb: &FurDB,
    request: proto::DeleteDatabaseRequest,
) -> Result<proto::DatabaseDeletedResponse, ErrorResponse> {
    furdb.delete_database(&request.database_id)?;

    Ok(proto::DatabaseDeletedResponse::new())
}
