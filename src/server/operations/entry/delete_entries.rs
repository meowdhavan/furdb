use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;
use crate::server::proto::delete_entries_request::Entries;

pub fn delete_entries(
    furdb: &FurDB,
    request: proto::DeleteEntriesRequest,
) -> Result<proto::EntriesDeletedResponse, ErrorResponse> {
    let entries = request.entries.as_ref().ok_or_else(|| {
        ErrorResponse::BadRequest("No entry selection given for `entries`".to_string())
    })?;

    let database = furdb.get_database(&request.database_id)?;
    let table = database.get_table(&request.table_id)?;

    match entries {
        Entries::All(_) => table.delete_all_entries(),
        Entries::Indices(entry_indices) => table.delete_entries(entry_indices.indices.to_vec()),
    }?;

    Ok(proto::EntriesDeletedResponse::new())
}
