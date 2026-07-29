use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn create_table(
    furdb: &FurDB,
    request: proto::CreateTableRequest,
) -> Result<proto::TableCreatedResponse, ErrorResponse> {
    let table_columns = request.get_table_columns();

    let database = furdb.get_database(&request.database_id)?;

    let table = database.create_table(&request.table_id, table_columns)?;

    let table_info = table.get_table_info();

    Ok(proto::TableCreatedResponse::new(table_info.into()))
}
