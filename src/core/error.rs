use thiserror::Error;

const DATABASE_CREATION_ERROR_MESSAGE: &str = "Cannot create Database";
const DATABASE_DELETION_ERROR_MESSAGE: &str = "Cannot delete Database";
const DATABASE_READ_ERROR_MESSAGE: &str = "Cannot read Database";

const ENTRY_DELETION_ERROR_MESSAGE: &str = "Cannot delete Entry";
const ENTRY_INSERTION_ERROR_MESSAGE: &str = "Cannot insert Entry";
const ENTRY_READ_ERROR_MESSAGE: &str = "Cannot read Entry";

const FURDB_INITIALIZATION_ERROR_MESSAGE: &str = "Cannot initialize FurDB";

const TABLE_CREATION_ERROR_MESSAGE: &str = "Cannot create Table";
const TABLE_DELETION_ERROR_MESSAGE: &str = "Cannot delete Table";
const TABLE_READ_ERROR_MESSAGE: &str = "Cannot read Table";

#[derive(Error, Debug)]
pub enum DatabaseCreationError {
    #[error("{DATABASE_CREATION_ERROR_MESSAGE}: Database already exists")]
    AlreadyExists,
    #[error("{DATABASE_CREATION_ERROR_MESSAGE}: Invalid Database ID")]
    InvalidId,
    #[error("{DATABASE_CREATION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum DatabaseDeletionError {
    #[error("{DATABASE_DELETION_ERROR_MESSAGE}: Database not found")]
    NotFound,
    #[error("{DATABASE_DELETION_ERROR_MESSAGE}: Invalid Database ID")]
    InvalidId,
    #[error("{DATABASE_DELETION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum DatabaseReadError {
    #[error("{DATABASE_READ_ERROR_MESSAGE}: Database not found")]
    NotFound,
    #[error("{DATABASE_READ_ERROR_MESSAGE}: Invalid Database ID")]
    InvalidId,
    #[error("{DATABASE_READ_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum EntryDeletionError {
    #[error("{ENTRY_DELETION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum EntryInsertionError {
    #[error("{ENTRY_INSERTION_ERROR_MESSAGE}: Number of columns does not match number of values")]
    ColumnMismatch,
    #[error("{ENTRY_INSERTION_ERROR_MESSAGE}: Size of value exceeds column size")]
    ColumnOverflow,
    #[error("{ENTRY_INSERTION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum EntryReadError {
    #[error("{ENTRY_READ_ERROR_MESSAGE}: Invalid index")]
    InvalidIndex,
    #[error("{ENTRY_READ_ERROR_MESSAGE}: Invalid Column")]
    InvalidColumn,
    #[error("{ENTRY_READ_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum FurDBInitializationError {
    #[error("{FURDB_INITIALIZATION_ERROR_MESSAGE}: Permission denied")]
    PermissionDenied,
    #[error("{FURDB_INITIALIZATION_ERROR_MESSAGE}: Invalid path")]
    InvalidPath,
    #[error("{FURDB_INITIALIZATION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum TableCreationError {
    #[error("{TABLE_CREATION_ERROR_MESSAGE}: Table already exists")]
    AlreadyExists,
    #[error("{TABLE_CREATION_ERROR_MESSAGE}: Invalid Table ID")]
    InvalidId,
    #[error("{TABLE_CREATION_ERROR_MESSAGE}: Size of row must be a non-zero multiple of 8")]
    ColumnsUnfit,
    #[error("{TABLE_CREATION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum TableDeletionError {
    #[error("{TABLE_DELETION_ERROR_MESSAGE}: Table not found")]
    NotFound,
    #[error("{TABLE_DELETION_ERROR_MESSAGE}: Invalid Table ID")]
    InvalidId,
    #[error("{TABLE_DELETION_ERROR_MESSAGE}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum TableReadError {
    #[error("{TABLE_READ_ERROR_MESSAGE}: Table not found")]
    NotFound,
    #[error("{TABLE_READ_ERROR_MESSAGE}: Invalid Table ID")]
    InvalidId,
    #[error("{TABLE_READ_ERROR_MESSAGE}")]
    OtherError(String),
}
