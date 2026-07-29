//! Conversions from the core models into their wire representation.
//!
//! Column values are `u128`, which protobuf has no type for, so they are
//! rendered as decimal strings.

use crate::core::models::{
    Column, DatabaseInfo, DatabaseInfoExtra, EntriesResult, Entry, TableInfo,
};
use crate::core::FurDBConfig;

use crate::server::proto;

impl From<FurDBConfig> for proto::FurDbConfig {
    fn from(config: FurDBConfig) -> Self {
        Self {
            workdir: config.workdir.to_string_lossy().to_string(),
        }
    }
}

impl From<Column> for proto::Column {
    fn from(column: Column) -> Self {
        Self {
            // A column narrow enough to have been accepted always fits in a
            // `u64`; saturating keeps a hand-edited `table_config.json` from
            // wrapping round to a plausible-looking width.
            size: u64::try_from(column.get_size()).unwrap_or(u64::MAX),
        }
    }
}

impl From<DatabaseInfo> for proto::DatabaseInfo {
    fn from(database_info: DatabaseInfo) -> Self {
        Self {
            database_id: database_info.get_database_id(),
        }
    }
}

impl From<DatabaseInfoExtra> for proto::DatabaseInfoExtra {
    fn from(database_info_extra: DatabaseInfoExtra) -> Self {
        Self {
            database_id: database_info_extra.get_database_info().get_database_id(),
            database_tables: database_info_extra.get_database_tables(),
        }
    }
}

impl From<TableInfo> for proto::TableInfo {
    fn from(table_info: TableInfo) -> Self {
        Self {
            database_id: table_info.get_database_id(),
            table_id: table_info.get_table_id(),
            table_columns: table_info
                .get_table_columns()
                .into_iter()
                .map(proto::Column::from)
                .collect(),
        }
    }
}

impl From<Entry> for proto::Entry {
    fn from(entry: Entry) -> Self {
        Self {
            index: entry.get_index() as u64,
            data: entry
                .get_data()
                .into_iter()
                .map(|value| value.to_string())
                .collect(),
        }
    }
}

impl From<EntriesResult> for proto::EntriesResult {
    fn from(entries_result: EntriesResult) -> Self {
        Self {
            result_count: entries_result.get_result_count() as u64,
            results: entries_result
                .get_results()
                .into_iter()
                .map(proto::Entry::from)
                .collect(),
        }
    }
}
