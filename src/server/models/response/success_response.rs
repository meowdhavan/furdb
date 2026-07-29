use crate::server::models::response::api_response::{SuccessStatus, RESULT_SUCCESS};
use crate::server::proto;

/// Constructors for every successful response message. This is the single place
/// that decides which status a given operation reports, so a new RPC means a
/// new constructor here.
macro_rules! success_response {
    ($response:ident, $status:expr, $payload:ty) => {
        impl proto::$response {
            pub fn new(response: $payload) -> Self {
                let status = $status;

                Self {
                    result: RESULT_SUCCESS.to_string(),
                    status_code: status.get_status_code(),
                    status: status.get_status(),
                    response: Some(response),
                }
            }
        }
    };
    ($response:ident, $status:expr) => {
        impl proto::$response {
            pub fn new() -> Self {
                let status = $status;

                Self {
                    result: RESULT_SUCCESS.to_string(),
                    status_code: status.get_status_code(),
                    status: status.get_status(),
                }
            }
        }
    };
}

success_response!(ServerInfoResponse, SuccessStatus::Ok, proto::FurDbConfig);

success_response!(
    DatabaseCreatedResponse,
    SuccessStatus::Created,
    proto::DatabaseInfo
);
success_response!(
    DatabaseInfoResponse,
    SuccessStatus::Ok,
    proto::DatabaseInfoExtra
);
success_response!(DatabaseDeletedResponse, SuccessStatus::Ok);

success_response!(
    TableCreatedResponse,
    SuccessStatus::Created,
    proto::TableInfo
);
success_response!(TableInfoResponse, SuccessStatus::Ok, proto::TableInfo);
success_response!(TableDeletedResponse, SuccessStatus::Ok);

success_response!(EntriesCreatedResponse, SuccessStatus::Created);
success_response!(
    EntriesResultResponse,
    SuccessStatus::Ok,
    proto::EntriesResult
);
success_response!(EntriesDeletedResponse, SuccessStatus::Ok);
