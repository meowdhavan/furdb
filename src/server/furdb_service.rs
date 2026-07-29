use tonic::{Request, Response, Status};

use crate::core::FurDB;

use crate::server::models::response::ErrorResponse;
use crate::server::operations;
use crate::server::proto;

/// Implements the generated `FurDb` service by delegating to the one-per-file
/// handlers in `crate::server::operations`. Every RPC has to be listed here,
/// which is the gRPC equivalent of registering a route.
pub struct FurDbService {
    furdb: FurDB,
}

impl FurDbService {
    pub fn new(furdb: FurDB) -> Self {
        Self { furdb }
    }
}

/// Logs the outcome of an RPC and converts the handler result into the
/// tonic types the generated trait expects.
fn respond<T>(method: &str, result: Result<T, ErrorResponse>) -> Result<Response<T>, Status> {
    match result {
        Ok(response) => {
            log::info!("{method}: OK");
            Ok(Response::new(response))
        }
        Err(error) => {
            log::warn!("{method}: {error}");
            Err(error.into())
        }
    }
}

#[tonic::async_trait]
impl proto::fur_db_server::FurDb for FurDbService {
    async fn get_server_info(
        &self,
        request: Request<proto::GetServerInfoRequest>,
    ) -> Result<Response<proto::ServerInfoResponse>, Status> {
        respond(
            "GetServerInfo",
            operations::get_server_info(&self.furdb, request.into_inner()),
        )
    }

    async fn create_database(
        &self,
        request: Request<proto::CreateDatabaseRequest>,
    ) -> Result<Response<proto::DatabaseCreatedResponse>, Status> {
        respond(
            "CreateDatabase",
            operations::create_database(&self.furdb, request.into_inner()),
        )
    }

    async fn get_database(
        &self,
        request: Request<proto::GetDatabaseRequest>,
    ) -> Result<Response<proto::DatabaseInfoResponse>, Status> {
        respond(
            "GetDatabase",
            operations::get_database(&self.furdb, request.into_inner()),
        )
    }

    async fn delete_database(
        &self,
        request: Request<proto::DeleteDatabaseRequest>,
    ) -> Result<Response<proto::DatabaseDeletedResponse>, Status> {
        respond(
            "DeleteDatabase",
            operations::delete_database(&self.furdb, request.into_inner()),
        )
    }

    async fn create_table(
        &self,
        request: Request<proto::CreateTableRequest>,
    ) -> Result<Response<proto::TableCreatedResponse>, Status> {
        respond(
            "CreateTable",
            operations::create_table(&self.furdb, request.into_inner()),
        )
    }

    async fn get_table(
        &self,
        request: Request<proto::GetTableRequest>,
    ) -> Result<Response<proto::TableInfoResponse>, Status> {
        respond(
            "GetTable",
            operations::get_table(&self.furdb, request.into_inner()),
        )
    }

    async fn delete_table(
        &self,
        request: Request<proto::DeleteTableRequest>,
    ) -> Result<Response<proto::TableDeletedResponse>, Status> {
        respond(
            "DeleteTable",
            operations::delete_table(&self.furdb, request.into_inner()),
        )
    }

    async fn insert_entries(
        &self,
        request: Request<proto::InsertEntriesRequest>,
    ) -> Result<Response<proto::EntriesCreatedResponse>, Status> {
        respond(
            "InsertEntries",
            operations::insert_entries(&self.furdb, request.into_inner()),
        )
    }

    async fn get_entries(
        &self,
        request: Request<proto::GetEntriesRequest>,
    ) -> Result<Response<proto::EntriesResultResponse>, Status> {
        respond(
            "GetEntries",
            operations::get_entries(&self.furdb, request.into_inner()),
        )
    }

    async fn delete_entries(
        &self,
        request: Request<proto::DeleteEntriesRequest>,
    ) -> Result<Response<proto::EntriesDeletedResponse>, Status> {
        respond(
            "DeleteEntries",
            operations::delete_entries(&self.furdb, request.into_inner()),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;
    use tonic::{Code, Request, Status};

    use super::*;
    use crate::core::FurDBConfig;
    use crate::server::proto::fur_db_server::FurDb;

    /// Every test gets its own workdir — `cargo test` runs them in parallel, so
    /// a shared path would produce cross-talk.
    fn service() -> (TempDir, FurDbService) {
        let workdir = TempDir::new().unwrap();
        let service = service_at(workdir.path());

        (workdir, service)
    }

    /// Opens a service over an existing workdir. Used to prove state survives a
    /// restart, since nothing is cached in memory between calls.
    fn service_at(workdir: &Path) -> FurDbService {
        let furdb = FurDB::new(&FurDBConfig {
            workdir: workdir.join("furdb"),
        })
        .unwrap();

        FurDbService::new(furdb)
    }

    fn column(size: u64) -> proto::Column {
        proto::Column { size }
    }

    fn entry_data(data: &[&str]) -> proto::EntryData {
        proto::EntryData {
            data: data.iter().map(|value| value.to_string()).collect(),
        }
    }

    async fn seed(service: &FurDbService) {
        seed_with(service, &[column(5), column(3)]).await;
    }

    /// Creates `my_database`/`my_table` with the given column widths.
    async fn seed_with(service: &FurDbService, table_columns: &[proto::Column]) {
        service
            .create_database(Request::new(proto::CreateDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap();

        service
            .create_table(Request::new(proto::CreateTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                table_columns: table_columns.to_vec(),
            }))
            .await
            .unwrap();
    }

    async fn insert(service: &FurDbService, rows: &[&[&str]]) -> Result<(), Status> {
        service
            .insert_entries(Request::new(proto::InsertEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                data: rows.iter().map(|row| entry_data(row)).collect(),
            }))
            .await
            .map(|_| ())
    }

    async fn get_all(service: &FurDbService) -> proto::EntriesResult {
        service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::All(
                    proto::AllEntries {},
                )),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap()
    }

    async fn query(
        service: &FurDbService,
        column_index: u64,
        value: &str,
    ) -> Result<proto::EntriesResult, Status> {
        service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::Value(
                    proto::EntriesByValue {
                        column_index,
                        value: value.to_string(),
                    },
                )),
            }))
            .await
            .map(|response| response.into_inner().response.unwrap())
    }

    async fn delete_indices(service: &FurDbService, indices: Vec<u64>) -> Result<(), Status> {
        service
            .delete_entries(Request::new(proto::DeleteEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::delete_entries_request::Entries::Indices(
                    proto::EntryIndices { indices },
                )),
            }))
            .await
            .map(|_| ())
    }

    /// The rows of a result as `(index, data)`, which is what most assertions
    /// about ordering and renumbering care about.
    fn rows(result: &proto::EntriesResult) -> Vec<(u64, Vec<String>)> {
        result
            .results
            .iter()
            .map(|entry| (entry.index, entry.data.to_vec()))
            .collect()
    }

    fn data_file(workdir: &TempDir) -> PathBuf {
        workdir
            .path()
            .join("furdb")
            .join("my_database")
            .join("tables")
            .join("my_table")
            .join("data")
    }

    /// Asserts the gRPC code and the `statusCode`/`status` pair echoed in the
    /// trailers, which is the whole error contract a client sees.
    fn assert_error(error: &Status, code: Code, status_code: &str, status: &str) {
        assert_eq!(error.code(), code, "message was {:?}", error.message());

        let metadata = error.metadata();
        assert_eq!(
            metadata.get("x-furdb-status-code").unwrap().as_bytes(),
            status_code.as_bytes()
        );
        assert_eq!(metadata.get("x-furdb-status").unwrap(), status);
    }

    /// Asserts the success envelope every response message carries.
    macro_rules! assert_envelope {
        ($response:expr, $status_code:expr, $status:expr) => {{
            let response = $response;
            assert_eq!(response.result, "success");
            assert_eq!(response.status_code, $status_code);
            assert_eq!(response.status, $status);
            response
        }};
    }

    #[tokio::test]
    async fn reports_server_info() {
        let (_workdir, service) = service();

        let response = service
            .get_server_info(Request::new(proto::GetServerInfoRequest {}))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.result, "success");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.status, "OK");
        assert!(response.response.unwrap().workdir.ends_with("furdb"));
    }

    #[tokio::test]
    async fn creates_and_reads_back_a_table() {
        let (_workdir, service) = service();
        seed(&service).await;

        let response = service
            .get_table(Request::new(proto::GetTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.status_code, 200);

        let table_info = response.response.unwrap();
        assert_eq!(table_info.database_id, "my_database");
        assert_eq!(table_info.table_id, "my_table");
        assert_eq!(table_info.table_columns, vec![column(5), column(3)]);
    }

    #[tokio::test]
    async fn inserts_then_queries_entries() {
        let (_workdir, service) = service();
        seed(&service).await;

        let inserted = service
            .insert_entries(Request::new(proto::InsertEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                data: vec![
                    entry_data(&["21", "0"]),
                    entry_data(&["17", "1"]),
                    entry_data(&["23", "2"]),
                ],
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(inserted.status_code, 201);

        let all = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::All(
                    proto::AllEntries {},
                )),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();

        assert_eq!(all.result_count, 3);
        assert_eq!(all.results[0].index, 0);
        assert_eq!(all.results[0].data, ["21", "0"]);

        let queried = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::Value(
                    proto::EntriesByValue {
                        column_index: 0,
                        value: "23".to_string(),
                    },
                )),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();

        assert_eq!(queried.result_count, 1);
        assert_eq!(queried.results[0].index, 2);

        service
            .delete_entries(Request::new(proto::DeleteEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::delete_entries_request::Entries::Indices(
                    proto::EntryIndices { indices: vec![0] },
                )),
            }))
            .await
            .unwrap();

        let remaining = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::All(
                    proto::AllEntries {},
                )),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();

        assert_eq!(remaining.result_count, 2);
        assert_eq!(remaining.results[0].data, ["17", "1"]);
    }

    #[tokio::test]
    async fn maps_core_errors_onto_grpc_codes() {
        let (_workdir, service) = service();
        seed(&service).await;

        let missing = service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "absent".to_string(),
            }))
            .await
            .unwrap_err();
        assert_eq!(missing.code(), Code::NotFound);

        let duplicate = service
            .create_database(Request::new(proto::CreateDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap_err();
        assert_eq!(duplicate.code(), Code::AlreadyExists);

        let unfit = service
            .create_table(Request::new(proto::CreateTableRequest {
                database_id: "my_database".to_string(),
                table_id: "unfit".to_string(),
                table_columns: vec![column(3)],
            }))
            .await
            .unwrap_err();
        assert_eq!(unfit.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn echoes_the_status_pair_in_error_trailers() {
        let (_workdir, service) = service();

        let error = service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "absent".to_string(),
            }))
            .await
            .unwrap_err();

        let metadata = error.metadata();
        assert_eq!(
            metadata.get("x-furdb-status-code").unwrap().as_bytes(),
            b"404"
        );
        assert_eq!(metadata.get("x-furdb-status").unwrap(), "Not Found");
    }

    #[tokio::test]
    async fn validates_a_request_before_looking_anything_up() {
        let (_workdir, service) = service();

        let error = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "absent".to_string(),
                table_id: "absent".to_string(),
                entries: Some(proto::get_entries_request::Entries::Value(
                    proto::EntriesByValue {
                        column_index: 0,
                        value: "not a number".to_string(),
                    },
                )),
            }))
            .await
            .unwrap_err();

        assert_eq!(error.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn rejects_values_that_are_not_u128() {
        let (_workdir, service) = service();
        seed(&service).await;

        let error = service
            .insert_entries(Request::new(proto::InsertEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                data: vec![entry_data(&["not a number", "0"])],
            }))
            .await
            .unwrap_err();

        assert_eq!(error.code(), Code::InvalidArgument);
        assert!(error.message().contains("128-bit"));
    }

    #[tokio::test]
    async fn does_not_echo_an_oversized_value_into_the_status_message() {
        let (_workdir, service) = service();
        seed(&service).await;

        let error = service
            .insert_entries(Request::new(proto::InsertEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                // `€` is 3 bytes wide and so does not divide the truncation
                // limit — slicing naively would split it and panic.
                data: vec![entry_data(&["€".repeat(8192).as_str(), "0"])],
            }))
            .await
            .unwrap_err();

        assert_eq!(error.code(), Code::InvalidArgument);
        assert!(
            error.message().len() < 256,
            "status message was {} bytes",
            error.message().len()
        );
    }

    #[tokio::test]
    async fn rejects_a_request_with_no_entry_selection() {
        let (_workdir, service) = service();
        seed(&service).await;

        let error = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: None,
            }))
            .await
            .unwrap_err();

        assert_eq!(error.code(), Code::InvalidArgument);
    }

    //
    // Response envelope
    //

    #[tokio::test]
    async fn every_rpc_reports_the_success_envelope() {
        let (_workdir, service) = service();

        assert_envelope!(
            service
                .get_server_info(Request::new(proto::GetServerInfoRequest {}))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );

        assert_envelope!(
            service
                .create_database(Request::new(proto::CreateDatabaseRequest {
                    database_id: "my_database".to_string(),
                }))
                .await
                .unwrap()
                .into_inner(),
            201,
            "Created"
        );

        assert_envelope!(
            service
                .get_database(Request::new(proto::GetDatabaseRequest {
                    database_id: "my_database".to_string(),
                }))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );

        assert_envelope!(
            service
                .create_table(Request::new(proto::CreateTableRequest {
                    database_id: "my_database".to_string(),
                    table_id: "my_table".to_string(),
                    table_columns: vec![column(5), column(3)],
                }))
                .await
                .unwrap()
                .into_inner(),
            201,
            "Created"
        );

        assert_envelope!(
            service
                .get_table(Request::new(proto::GetTableRequest {
                    database_id: "my_database".to_string(),
                    table_id: "my_table".to_string(),
                }))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );

        assert_envelope!(
            service
                .insert_entries(Request::new(proto::InsertEntriesRequest {
                    database_id: "my_database".to_string(),
                    table_id: "my_table".to_string(),
                    data: vec![entry_data(&["1", "1"])],
                }))
                .await
                .unwrap()
                .into_inner(),
            201,
            "Created"
        );

        assert_envelope!(
            service
                .get_entries(Request::new(proto::GetEntriesRequest {
                    database_id: "my_database".to_string(),
                    table_id: "my_table".to_string(),
                    entries: Some(proto::get_entries_request::Entries::All(
                        proto::AllEntries {},
                    )),
                }))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );

        assert_envelope!(
            service
                .delete_entries(Request::new(proto::DeleteEntriesRequest {
                    database_id: "my_database".to_string(),
                    table_id: "my_table".to_string(),
                    entries: Some(proto::delete_entries_request::Entries::All(
                        proto::AllEntries {},
                    )),
                }))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );

        assert_envelope!(
            service
                .delete_table(Request::new(proto::DeleteTableRequest {
                    database_id: "my_database".to_string(),
                    table_id: "my_table".to_string(),
                }))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );

        assert_envelope!(
            service
                .delete_database(Request::new(proto::DeleteDatabaseRequest {
                    database_id: "my_database".to_string(),
                }))
                .await
                .unwrap()
                .into_inner(),
            200,
            "OK"
        );
    }

    #[tokio::test]
    async fn payload_carrying_responses_always_populate_response() {
        let (_workdir, service) = service();
        seed(&service).await;

        // `response` is an optional message on the wire, so an absent payload
        // would decode as `None` rather than failing.
        assert!(service
            .get_server_info(Request::new(proto::GetServerInfoRequest {}))
            .await
            .unwrap()
            .into_inner()
            .response
            .is_some());

        assert!(service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .is_some());

        assert!(service
            .get_table(Request::new(proto::GetTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .is_some());
    }

    #[tokio::test]
    async fn database_info_lists_its_tables() {
        let (_workdir, service) = service();
        seed(&service).await;

        let info = service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();

        assert_eq!(info.database_id, "my_database");
        assert_eq!(info.database_tables, vec!["my_table".to_string()]);
    }

    //
    // Error paths: every ErrorResponse variant and its gRPC code
    //

    #[tokio::test]
    async fn not_found_maps_onto_not_found() {
        let (_workdir, service) = service();
        seed(&service).await;

        let missing_database = service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "absent".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&missing_database, Code::NotFound, "404", "Not Found");

        let missing_table = service
            .get_table(Request::new(proto::GetTableRequest {
                database_id: "my_database".to_string(),
                table_id: "absent".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&missing_table, Code::NotFound, "404", "Not Found");

        let delete_missing_table = service
            .delete_table(Request::new(proto::DeleteTableRequest {
                database_id: "my_database".to_string(),
                table_id: "absent".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&delete_missing_table, Code::NotFound, "404", "Not Found");

        let delete_missing_database = service
            .delete_database(Request::new(proto::DeleteDatabaseRequest {
                database_id: "absent".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&delete_missing_database, Code::NotFound, "404", "Not Found");
    }

    #[tokio::test]
    async fn conflict_maps_onto_already_exists() {
        let (_workdir, service) = service();
        seed(&service).await;

        let duplicate_database = service
            .create_database(Request::new(proto::CreateDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&duplicate_database, Code::AlreadyExists, "409", "Conflict");

        let duplicate_table = service
            .create_table(Request::new(proto::CreateTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                table_columns: vec![column(8)],
            }))
            .await
            .unwrap_err();
        assert_error(&duplicate_table, Code::AlreadyExists, "409", "Conflict");
    }

    #[tokio::test]
    async fn bad_request_maps_onto_invalid_argument() {
        let (_workdir, service) = service();
        seed(&service).await;

        // ColumnsUnfit: a row that is not a whole number of bytes.
        let unfit = service
            .create_table(Request::new(proto::CreateTableRequest {
                database_id: "my_database".to_string(),
                table_id: "unfit".to_string(),
                table_columns: vec![column(3)],
            }))
            .await
            .unwrap_err();
        assert_error(&unfit, Code::InvalidArgument, "400", "Bad Request");

        // InvalidId.
        let invalid_id = service
            .create_database(Request::new(proto::CreateDatabaseRequest {
                database_id: "not.valid".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&invalid_id, Code::InvalidArgument, "400", "Bad Request");

        // ColumnMismatch: two columns declared, one value supplied.
        let mismatch = insert(&service, &[&["1"]]).await.unwrap_err();
        assert_error(&mismatch, Code::InvalidArgument, "400", "Bad Request");

        // ColumnOverflow: 32 does not fit in 5 bits.
        let overflow = insert(&service, &[&["32", "0"]]).await.unwrap_err();
        assert_error(&overflow, Code::InvalidArgument, "400", "Bad Request");

        // InvalidIndex.
        insert(&service, &[&["1", "1"]]).await.unwrap();
        let invalid_index = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::Indices(
                    proto::EntryIndices { indices: vec![99] },
                )),
            }))
            .await
            .unwrap_err();
        assert_error(&invalid_index, Code::InvalidArgument, "400", "Bad Request");

        // InvalidColumn: the table has two columns, so index 2 is out of range.
        let invalid_column = query(&service, 2, "1").await.unwrap_err();
        assert_error(&invalid_column, Code::InvalidArgument, "400", "Bad Request");
    }

    #[tokio::test]
    async fn internal_error_maps_onto_internal_and_leaks_nothing() {
        let (workdir, service) = service();
        seed(&service).await;

        // A config the engine cannot parse is the reachable route to
        // `OtherError`, which is the only source of InternalServerError.
        let config = workdir
            .path()
            .join("furdb")
            .join("my_database")
            .join("database_config.json");
        std::fs::write(&config, "{ this is not json").unwrap();

        let error = service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap_err();

        assert_error(&error, Code::Internal, "500", "Internal Server Error");

        // Internal detail is deliberately swallowed: no serde text, no paths.
        assert_eq!(error.message(), "Internal Server Error");
        assert!(!error.message().contains("json"));
        assert!(!error.message().to_lowercase().contains("furdb/my_database"));
    }

    #[tokio::test]
    async fn rejects_ids_that_would_escape_the_workdir() {
        let (_workdir, service) = service();
        seed(&service).await;

        for database_id in ["..", "../..", "../escape", "a/b", "/etc", "."] {
            let error = service
                .create_database(Request::new(proto::CreateDatabaseRequest {
                    database_id: database_id.to_string(),
                }))
                .await
                .unwrap_err();

            assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
        }

        for table_id in ["..", "../..", "a/b", "."] {
            let error = service
                .create_table(Request::new(proto::CreateTableRequest {
                    database_id: "my_database".to_string(),
                    table_id: table_id.to_string(),
                    table_columns: vec![column(8)],
                }))
                .await
                .unwrap_err();

            assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
        }
    }

    #[tokio::test]
    async fn rejects_an_empty_database_id() {
        let (_workdir, service) = service();

        // `is_id_valid("")` is `all()` over no characters, so an empty id passes
        // validation and resolves to the workdir itself.
        let error = service
            .create_database(Request::new(proto::CreateDatabaseRequest {
                database_id: String::new(),
            }))
            .await
            .unwrap_err();

        assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
    }

    #[tokio::test]
    async fn rejects_a_delete_request_with_no_entry_selection() {
        let (_workdir, service) = service();
        seed(&service).await;

        let error = service
            .delete_entries(Request::new(proto::DeleteEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: None,
            }))
            .await
            .unwrap_err();

        assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
    }

    //
    // u128-as-decimal-string boundary
    //

    #[tokio::test]
    async fn round_trips_the_bounds_of_a_column() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["0"], &["255"], &["1"]]).await.unwrap();

        let all = get_all(&service).await;
        assert_eq!(
            rows(&all),
            vec![
                (0, vec!["0".to_string()]),
                (1, vec!["255".to_string()]),
                (2, vec!["1".to_string()]),
            ]
        );
    }

    #[tokio::test]
    async fn rejects_a_value_one_past_the_column_width() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        // 255 fits in 8 bits, 256 does not.
        insert(&service, &[&["255"]]).await.unwrap();

        let error = insert(&service, &[&["256"]]).await.unwrap_err();
        assert_error(&error, Code::InvalidArgument, "400", "Bad Request");

        // The rejected insert must not have appended anything.
        assert_eq!(get_all(&service).await.result_count, 1);
    }

    #[tokio::test]
    async fn round_trips_a_127_bit_value() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(127), column(1)]).await;

        let max_127 = (u128::pow(2, 127) - 1).to_string();
        insert(&service, &[&[&max_127, "1"], &["0", "0"]])
            .await
            .unwrap();

        let all = get_all(&service).await;
        assert_eq!(all.results[0].data, [max_127.as_str(), "1"]);
        assert_eq!(all.results[1].data, ["0", "0"]);
    }

    #[tokio::test]
    async fn round_trips_u128_max_in_a_128_bit_column() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(128)]).await;

        insert(&service, &[&["0"], &[&u128::MAX.to_string()]])
            .await
            .unwrap();

        let all = get_all(&service).await;
        assert_eq!(
            rows(&all),
            vec![(0, vec!["0".to_string()]), (1, vec![u128::MAX.to_string()]),]
        );
    }

    #[tokio::test]
    async fn rejects_values_that_are_not_a_decimal_u128() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        let cases = [
            ("empty", ""),
            ("non numeric", "abc"),
            ("negative", "-1"),
            ("fractional", "1.5"),
            ("leading space", " 1"),
            ("trailing space", "1 "),
            ("hexadecimal", "0xff"),
            ("underscore separated", "1_0"),
            (
                "one past u128::MAX",
                "340282366920938463463374607431768211456",
            ),
        ];

        for (name, value) in cases {
            let error = insert(&service, &[&[value]]).await.unwrap_err_or_else(name);
            assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
            assert!(
                error.message().contains("128-bit"),
                "{name}: message was {:?}",
                error.message()
            );
        }

        // Accepted by `u128::from_str`, so accepted here: leading zeros and an
        // explicit `+` sign both parse.
        insert(&service, &[&["007"]]).await.unwrap();
        insert(&service, &[&["+9"]]).await.unwrap();
        assert_eq!(
            rows(&get_all(&service).await),
            vec![(0, vec!["7".to_string()]), (1, vec!["9".to_string()])]
        );
    }

    #[tokio::test]
    async fn rejects_a_non_numeric_query_value() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;
        insert(&service, &[&["1"]]).await.unwrap();

        for value in ["", "abc", "-1", "340282366920938463463374607431768211456"] {
            let error = query(&service, 0, value).await.unwrap_err();
            assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
        }
    }

    #[tokio::test]
    async fn rejects_a_whole_batch_if_one_value_is_malformed() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        // The second row is bad; nothing from the batch may be persisted.
        let error = insert(&service, &[&["1"], &["nope"], &["3"]])
            .await
            .unwrap_err();
        assert_error(&error, Code::InvalidArgument, "400", "Bad Request");

        assert_eq!(get_all(&service).await.result_count, 0);
    }

    #[tokio::test]
    async fn rejects_a_whole_batch_if_one_value_overflows() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        let error = insert(&service, &[&["1"], &["256"], &["3"]])
            .await
            .unwrap_err();
        assert_error(&error, Code::InvalidArgument, "400", "Bad Request");

        assert_eq!(get_all(&service).await.result_count, 0);
    }

    //
    // Table shape
    //

    #[tokio::test]
    async fn rejects_a_table_that_cannot_address_its_entries() {
        // A zero-bit row satisfies `% 8 == 0` but makes `entry_size` zero, and
        // every later read divides the file length by it. It has to be refused
        // at creation — otherwise reading the table panics, which takes the
        // connection down and is remotely reachable.
        let (_workdir, service) = service();

        service
            .create_database(Request::new(proto::CreateDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap();

        // A zero-bit row divides into the data file zero times, so no entry can
        // ever be addressed. Both shapes pass the `% 8` check.
        for (name, table_columns) in [
            ("no columns", vec![]),
            ("one zero width column", vec![column(0)]),
            ("all zero width columns", vec![column(0), column(0)]),
        ] {
            let error = service
                .create_table(Request::new(proto::CreateTableRequest {
                    database_id: "my_database".to_string(),
                    table_id: format!("t{}", table_columns.len()),
                    table_columns,
                }))
                .await
                .unwrap_err_or_else(name);

            assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
        }
    }

    //
    // Statefulness: index renumbering after a delete
    //

    #[tokio::test]
    async fn renumbers_every_index_after_a_deleted_one() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["10"], &["11"], &["12"], &["13"], &["14"]])
            .await
            .unwrap();

        // Remove the middle entry.
        delete_indices(&service, vec![2]).await.unwrap();

        assert_eq!(
            rows(&get_all(&service).await),
            vec![
                (0, vec!["10".to_string()]),
                (1, vec!["11".to_string()]),
                (2, vec!["13".to_string()]),
                (3, vec!["14".to_string()]),
            ],
            "surviving entries must close the gap and be renumbered"
        );

        // The old index 4 is now index 3, and index 4 no longer exists.
        let error = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::Indices(
                    proto::EntryIndices { indices: vec![4] },
                )),
            }))
            .await
            .unwrap_err();
        assert_error(&error, Code::InvalidArgument, "400", "Bad Request");
    }

    #[tokio::test]
    async fn deletes_several_indices_in_one_request() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["10"], &["11"], &["12"], &["13"], &["14"]])
            .await
            .unwrap();

        // Indices refer to the state before the delete, not to each other.
        delete_indices(&service, vec![0, 2, 4]).await.unwrap();

        assert_eq!(
            rows(&get_all(&service).await),
            vec![(0, vec!["11".to_string()]), (1, vec!["13".to_string()])]
        );
    }

    #[tokio::test]
    async fn deleting_the_same_index_twice_removes_it_once() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["10"], &["11"], &["12"]])
            .await
            .unwrap();

        delete_indices(&service, vec![1, 1, 1]).await.unwrap();

        assert_eq!(
            rows(&get_all(&service).await),
            vec![(0, vec!["10".to_string()]), (1, vec!["12".to_string()])]
        );
    }

    #[tokio::test]
    async fn deleting_an_out_of_range_index_changes_nothing() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["10"], &["11"]]).await.unwrap();

        // There is no error variant for this, so it is a silent no-op — pinned
        // here so a change to that contract is visible.
        delete_indices(&service, vec![99]).await.unwrap();

        assert_eq!(
            rows(&get_all(&service).await),
            vec![(0, vec!["10".to_string()]), (1, vec!["11".to_string()])]
        );
    }

    #[tokio::test]
    async fn deletes_from_an_empty_table_and_deletes_everything() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        delete_indices(&service, vec![]).await.unwrap();
        assert_eq!(get_all(&service).await.result_count, 0);

        insert(&service, &[&["10"], &["11"]]).await.unwrap();

        service
            .delete_entries(Request::new(proto::DeleteEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::delete_entries_request::Entries::All(
                    proto::AllEntries {},
                )),
            }))
            .await
            .unwrap();

        assert_eq!(get_all(&service).await.result_count, 0);

        // The table stays usable and numbering restarts at 0.
        insert(&service, &[&["12"]]).await.unwrap();
        assert_eq!(
            rows(&get_all(&service).await),
            vec![(0, vec!["12".to_string()])]
        );
    }

    #[tokio::test]
    async fn keeps_the_index_usable_across_insert_delete_insert() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["10"], &["20"], &["30"]])
            .await
            .unwrap();
        delete_indices(&service, vec![0]).await.unwrap();
        insert(&service, &[&["40"]]).await.unwrap();

        assert_eq!(
            rows(&get_all(&service).await),
            vec![
                (0, vec!["20".to_string()]),
                (1, vec!["30".to_string()]),
                (2, vec!["40".to_string()]),
            ]
        );

        // The sortfile is rebuilt after each write, so the query index must
        // report the new numbering.
        let found = query(&service, 0, "30").await.unwrap();
        assert_eq!(found.result_count, 1);
        assert_eq!(found.results[0].index, 1);

        let gone = query(&service, 0, "10").await.unwrap();
        assert_eq!(gone.result_count, 0);
    }

    #[tokio::test]
    async fn empty_insert_is_a_no_op() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[]).await.unwrap();
        assert_eq!(get_all(&service).await.result_count, 0);

        insert(&service, &[&["1"]]).await.unwrap();
        insert(&service, &[]).await.unwrap();
        assert_eq!(get_all(&service).await.result_count, 1);
    }

    //
    // Querying
    //

    #[tokio::test]
    async fn finds_every_value_of_every_column() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8), column(8), column(8)]).await;

        // Six entries; each column holds six distinct ascending values, so
        // every value must be reachable by an exact-match query.
        let entries: Vec<Vec<String>> = (0..6u64)
            .map(|i| vec![i.to_string(), (10 + i).to_string(), (100 + i).to_string()])
            .collect();
        let borrowed: Vec<Vec<&str>> = entries
            .iter()
            .map(|entry| entry.iter().map(String::as_str).collect())
            .collect();
        let batch: Vec<&[&str]> = borrowed.iter().map(Vec::as_slice).collect();
        insert(&service, &batch).await.unwrap();

        // Collected rather than asserted per case, so a failure reports every
        // unreachable value at once instead of stopping at the first.
        let mut unreachable = Vec::new();

        for (entry_index, entry) in entries.iter().enumerate() {
            for (column_index, value) in entry.iter().enumerate() {
                let found = query(&service, column_index as u64, value).await.unwrap();

                if found.result_count != 1 || found.results[0].index != entry_index as u64 {
                    unreachable.push(format!(
                        "column {column_index} value {value} (entry {entry_index}) matched {} entries",
                        found.result_count
                    ));
                }
            }
        }

        assert!(
            unreachable.is_empty(),
            "values that could not be found:\n  {}",
            unreachable.join("\n  ")
        );
    }

    #[tokio::test]
    async fn finds_the_only_entry_by_any_column() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8), column(8)]).await;

        insert(&service, &[&["7", "9"]]).await.unwrap();

        assert_eq!(query(&service, 0, "7").await.unwrap().result_count, 1);
        assert_eq!(query(&service, 1, "9").await.unwrap().result_count, 1);
    }

    #[tokio::test]
    async fn returns_every_entry_sharing_a_value() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["5"], &["5"], &["7"], &["5"]])
            .await
            .unwrap();

        let found = query(&service, 0, "5").await.unwrap();
        assert_eq!(found.result_count, 3);

        let mut indices = found
            .results
            .iter()
            .map(|entry| entry.index)
            .collect::<Vec<u64>>();
        indices.sort_unstable();
        assert_eq!(indices, vec![0, 1, 3]);
    }

    #[tokio::test]
    async fn a_query_that_matches_nothing_returns_an_empty_result() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        // Empty table.
        let empty = query(&service, 0, "1").await.unwrap();
        assert_eq!(empty.result_count, 0);
        assert!(empty.results.is_empty());

        insert(&service, &[&["1"], &["2"]]).await.unwrap();

        let absent = query(&service, 0, "200").await.unwrap();
        assert_eq!(absent.result_count, 0);
        assert!(absent.results.is_empty());
    }

    #[tokio::test]
    async fn reads_entries_by_explicit_index() {
        let (_workdir, service) = service();
        seed_with(&service, &[column(8)]).await;

        insert(&service, &[&["10"], &["11"], &["12"]])
            .await
            .unwrap();

        let selected = service
            .get_entries(Request::new(proto::GetEntriesRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
                entries: Some(proto::get_entries_request::Entries::Indices(
                    proto::EntryIndices {
                        indices: vec![2, 0],
                    },
                )),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();

        // The requested order is preserved.
        assert_eq!(
            rows(&selected),
            vec![(2, vec!["12".to_string()]), (0, vec!["10".to_string()])]
        );
    }

    //
    // Persistence
    //

    #[tokio::test]
    async fn packs_entries_msb_first_with_no_header_or_delimiter() {
        let (workdir, service) = service();
        seed(&service).await;

        // Columns are 5 and 3 bits: 21 -> 10101, 0 -> 000  => 0b10101000.
        //                           17 -> 10001, 1 -> 001  => 0b10001001.
        //                           23 -> 10111, 2 -> 010  => 0b10111010.
        insert(&service, &[&["21", "0"], &["17", "1"], &["23", "2"]])
            .await
            .unwrap();

        let bytes = std::fs::read(data_file(&workdir)).unwrap();
        assert_eq!(bytes, vec![0b1010_1000, 0b1000_1001, 0b1011_1010]);
    }

    #[tokio::test]
    async fn state_survives_a_restart() {
        let (workdir, service) = service();
        seed_with(&service, &[column(8), column(8)]).await;

        insert(&service, &[&["1", "10"], &["2", "20"], &["3", "30"]])
            .await
            .unwrap();
        delete_indices(&service, vec![1]).await.unwrap();

        drop(service);

        let reopened = service_at(workdir.path());

        let table = reopened
            .get_table(Request::new(proto::GetTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();
        assert_eq!(table.table_columns, vec![column(8), column(8)]);

        assert_eq!(
            rows(&get_all(&reopened).await),
            vec![
                (0, vec!["1".to_string(), "10".to_string()]),
                (1, vec!["3".to_string(), "30".to_string()]),
            ]
        );

        // The secondary index survives too.
        let found = query(&reopened, 0, "3").await.unwrap();
        assert_eq!(found.result_count, 1);
        assert_eq!(found.results[0].index, 1);
    }

    #[tokio::test]
    async fn deleting_a_table_leaves_the_database_and_its_siblings_intact() {
        let (_workdir, service) = service();
        seed(&service).await;

        service
            .create_table(Request::new(proto::CreateTableRequest {
                database_id: "my_database".to_string(),
                table_id: "other_table".to_string(),
                table_columns: vec![column(8)],
            }))
            .await
            .unwrap();

        service
            .delete_table(Request::new(proto::DeleteTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
            }))
            .await
            .unwrap();

        let info = service
            .get_database(Request::new(proto::GetDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap()
            .into_inner()
            .response
            .unwrap();
        assert_eq!(info.database_tables, vec!["other_table".to_string()]);

        let error = service
            .get_table(Request::new(proto::GetTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&error, Code::NotFound, "404", "Not Found");
    }

    #[tokio::test]
    async fn deleting_a_database_removes_its_tables() {
        let (_workdir, service) = service();
        seed(&service).await;
        insert(&service, &[&["1", "1"]]).await.unwrap();

        service
            .delete_database(Request::new(proto::DeleteDatabaseRequest {
                database_id: "my_database".to_string(),
            }))
            .await
            .unwrap();

        let error = service
            .get_table(Request::new(proto::GetTableRequest {
                database_id: "my_database".to_string(),
                table_id: "my_table".to_string(),
            }))
            .await
            .unwrap_err();
        assert_error(&error, Code::NotFound, "404", "Not Found");

        // The id is reusable, and the new table starts empty.
        seed(&service).await;
        assert_eq!(get_all(&service).await.result_count, 0);
    }

    /// `unwrap_err` with the case name in the panic message, so a table-driven
    /// failure says which row failed.
    trait UnwrapErrOrElse<T, E> {
        fn unwrap_err_or_else(self, name: &str) -> E;
    }

    impl<T, E> UnwrapErrOrElse<T, E> for Result<T, E> {
        fn unwrap_err_or_else(self, name: &str) -> E {
            match self {
                Ok(_) => panic!("{name}: expected an error, got a success"),
                Err(error) => error,
            }
        }
    }
}
