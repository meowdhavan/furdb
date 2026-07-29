use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn delete_table(
    furdb: &FurDB,
    request: proto::DeleteTableRequest,
) -> Result<proto::TableDeletedResponse, ErrorResponse> {
    let database = furdb.get_database(&request.database_id)?;

    database.delete_table(&request.table_id)?;

    Ok(proto::TableDeletedResponse::new())
}
