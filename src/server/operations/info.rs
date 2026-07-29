use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::proto;

pub fn get_server_info(
    furdb: &FurDB,
    _request: proto::GetServerInfoRequest,
) -> Result<proto::ServerInfoResponse, ErrorResponse> {
    let furdb_config = furdb.get_config();

    Ok(proto::ServerInfoResponse::new(furdb_config.into()))
}
