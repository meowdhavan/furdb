use std::path::Path;
use std::path::PathBuf;

pub fn get_database_path(fur_directory: &Path, database_id: &str) -> PathBuf {
    fur_directory.join(database_id)
}

pub fn get_database_config_path(fur_directory: &Path, database_id: &str) -> PathBuf {
    get_database_path(fur_directory, database_id).join("database_config.json")
}

pub fn get_all_tables_path(fur_directory: &Path, database_id: &str) -> PathBuf {
    get_database_path(fur_directory, database_id).join("tables")
}

pub fn get_table_path(fur_directory: &Path, database_id: &str, table_id: &str) -> PathBuf {
    get_all_tables_path(fur_directory, database_id).join(table_id)
}

pub fn get_table_config_path(fur_directory: &Path, database_id: &str, table_id: &str) -> PathBuf {
    get_table_path(fur_directory, database_id, table_id).join("table_config.json")
}

pub fn get_table_data_path(fur_directory: &Path, database_id: &str, table_id: &str) -> PathBuf {
    get_table_path(fur_directory, database_id, table_id).join("data")
}

pub fn get_sortfile_path(fur_directory: &Path, database_id: &str, table_id: &str) -> PathBuf {
    get_table_path(fur_directory, database_id, table_id).join("sortfile")
}

pub fn is_id_valid(id: &str) -> bool {
    // `all()` over an empty string is vacuously true, and an empty ID resolves
    // to the parent directory rather than to anything inside it.
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
