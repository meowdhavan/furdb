use serde::{Deserialize, Serialize};

use crate::core::error::TableReadError;

use crate::core::furdb_config::FurDBConfig;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    config: FurDBConfig,
    database_info: DatabaseInfo,
}

impl Database {
    pub fn new(config: &FurDBConfig, database_info: &DatabaseInfo) -> Self {
        Self {
            config: config.to_owned(),
            database_info: database_info.to_owned(),
        }
    }

    pub fn get_config(&self) -> FurDBConfig {
        self.config.to_owned()
    }

    pub fn get_database_info(&self) -> DatabaseInfo {
        self.database_info.to_owned()
    }

    pub fn get_database_info_full(&self) -> Result<DatabaseInfoExtra, TableReadError> {
        Ok(DatabaseInfoExtra::new(
            &self.database_info,
            self.get_all_tables()?,
        ))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    database_id: String,
}

impl DatabaseInfo {
    pub fn new(database_id: &str) -> Self {
        Self {
            database_id: database_id.to_string(),
        }
    }

    pub fn get_database_id(&self) -> String {
        self.database_id.to_owned()
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfoExtra {
    #[serde(flatten)]
    database_info: DatabaseInfo,
    database_tables: Vec<String>,
}

impl DatabaseInfoExtra {
    pub fn new(database_info: &DatabaseInfo, database_tables: Vec<String>) -> Self {
        Self {
            database_info: database_info.to_owned(),
            database_tables,
        }
    }

    pub fn get_database_info(&self) -> DatabaseInfo {
        self.database_info.to_owned()
    }

    pub fn get_database_tables(&self) -> Vec<String> {
        self.database_tables.to_owned()
    }
}
