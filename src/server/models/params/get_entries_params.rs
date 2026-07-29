use crate::server::models::response::ErrorResponse;
use crate::server::proto;
use crate::server::utils::parse_u128;

impl proto::EntriesByValue {
    pub fn get_column_index(&self) -> u64 {
        self.column_index
    }

    pub fn get_value(&self) -> Result<u128, ErrorResponse> {
        parse_u128(&self.value, "value")
    }
}
