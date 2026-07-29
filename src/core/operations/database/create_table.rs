use crate::core::utils;

use crate::core::models::Column;
use crate::core::models::Database;
use crate::core::models::Table;
use crate::core::models::TableInfo;

use crate::core::error::TableCreationError;
use std::io::ErrorKind;

impl Database {
    pub fn create_table(
        &self,
        table_id: &str,
        table_columns: Vec<Column>,
    ) -> Result<Table, TableCreationError> {
        let config = self.get_config();
        let database_info = self.get_database_info();

        if !utils::is_id_valid(table_id) {
            return Err(TableCreationError::InvalidId);
        }

        let table_path =
            utils::get_table_path(&config.workdir, &database_info.get_database_id(), table_id);
        let table_config_path = utils::get_table_config_path(
            &config.workdir,
            &database_info.get_database_id(),
            table_id,
        );
        let table_data_path =
            utils::get_table_data_path(&config.workdir, &database_info.get_database_id(), table_id);
        let table_sortfile_path =
            utils::get_sortfile_path(&config.workdir, &database_info.get_database_id(), table_id);

        // A zero-width row also satisfies `% 8 == 0`, but `entry_size` becomes 0
        // and every later read divides the file length by it.
        let row_size = table_columns.iter().fold(0, |acc, x| acc + x.get_size());

        if row_size == 0 || row_size % 8 != 0 {
            return Err(TableCreationError::ColumnsUnfit);
        }

        std::fs::create_dir(table_path).map_err(|e| match e.kind() {
            ErrorKind::AlreadyExists => TableCreationError::AlreadyExists,
            _ => TableCreationError::OtherError(e.to_string()),
        })?;

        let table_info = TableInfo::new(&database_info.get_database_id(), table_id, &table_columns);
        let table = Table::new(&config, &table_info);

        let table_info_serialized = serde_json::to_string(&table_info)
            .map_err(|e| TableCreationError::OtherError(e.to_string()))?;

        std::fs::write(table_config_path, table_info_serialized)
            .map_err(|e| TableCreationError::OtherError(e.to_string()))?;
        std::fs::write(table_data_path, "")
            .map_err(|e| TableCreationError::OtherError(e.to_string()))?;
        std::fs::write(table_sortfile_path, "")
            .map_err(|e| TableCreationError::OtherError(e.to_string()))?;

        Ok(table)
    }
}
