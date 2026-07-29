use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn get_table(
    furdb: &FurDB,
    request: proto::GetTableRequest,
) -> Result<proto::TableInfoResponse, ErrorResponse> {
    let database = furdb.get_database(&request.database_id)?;
    let table = database.get_table(&request.table_id)?;

    let table_info = table.get_table_info();

    Ok(proto::TableInfoResponse::new(table_info.into()))
}
