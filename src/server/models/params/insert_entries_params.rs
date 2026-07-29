use crate::server::models::response::ErrorResponse;
use crate::server::proto;
use crate::server::utils::parse_u128;

impl proto::InsertEntriesRequest {
    pub fn get_data(&self) -> Result<Vec<Vec<u128>>, ErrorResponse> {
        self.data
            .iter()
            .map(|entry| {
                entry
                    .data
                    .iter()
                    .map(|value| parse_u128(value, "data"))
                    .collect()
            })
            .collect()
    }
}
