use crate::core::models::Column;

use crate::server::proto;

impl proto::CreateTableRequest {
    pub fn get_table_columns(&self) -> Vec<Column> {
        self.table_columns
            .iter()
            .map(|column| Column::new(column.size as u128))
            .collect()
    }
}
