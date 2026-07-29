use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn insert_entries(
    furdb: &FurDB,
    request: proto::InsertEntriesRequest,
) -> Result<proto::EntriesCreatedResponse, ErrorResponse> {
    let data = request.get_data()?;

    let database = furdb.get_database(&request.database_id)?;
    let table = database.get_table(&request.table_id)?;

    table.insert_entries(&data)?;

    Ok(proto::EntriesCreatedResponse::new())
}
