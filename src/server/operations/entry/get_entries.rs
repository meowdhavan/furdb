use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;
use crate::server::proto::get_entries_request::Entries;

/// The request's entry selection, with its values already parsed.
enum Selection {
    All,
    Indices(Vec<u64>),
    Value(u64, u128),
}

pub fn get_entries(
    furdb: &FurDB,
    request: proto::GetEntriesRequest,
) -> Result<proto::EntriesResultResponse, ErrorResponse> {
    let entries = request.entries.as_ref().ok_or_else(|| {
        ErrorResponse::BadRequest("No entry selection given for `entries`".to_string())
    })?;

    // The request is validated before anything is looked up, so a malformed
    // value is reported as such rather than as a missing database.
    let selection = match entries {
        Entries::All(_) => Selection::All,
        Entries::Indices(entry_indices) => Selection::Indices(entry_indices.indices.to_vec()),
        Entries::Value(entries_by_value) => Selection::Value(
            entries_by_value.get_column_index(),
            entries_by_value.get_value()?,
        ),
    };

    let database = furdb.get_database(&request.database_id)?;
    let table = database.get_table(&request.table_id)?;

    let entries_result = match selection {
        Selection::All => table.get_all_entries(),
        Selection::Indices(indices) => table.get_entries(indices),
        Selection::Value(column_index, value) => table.query(column_index, value),
    }?;

    Ok(proto::EntriesResultResponse::new(entries_result.into()))
}
