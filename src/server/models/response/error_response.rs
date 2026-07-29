use std::fmt::{Display, Formatter, Result};

use tonic::metadata::MetadataValue;
use tonic::{Code, Status};

/// gRPC carries failures as statuses rather than as a response envelope, so the
/// `statusCode`/`status` pair successful responses report is echoed in these
/// trailers instead of being dropped.
const STATUS_CODE_METADATA_KEY: &str = "x-furdb-status-code";
const STATUS_METADATA_KEY: &str = "x-furdb-status";

#[derive(Debug, Clone)]
pub enum ErrorResponse {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    InternalServerError,
}

impl ErrorResponse {
    fn get_code(&self) -> Code {
        match self {
            ErrorResponse::NotFound(_) => Code::NotFound,
            ErrorResponse::BadRequest(_) => Code::InvalidArgument,
            ErrorResponse::Conflict(_) => Code::AlreadyExists,
            ErrorResponse::InternalServerError => Code::Internal,
        }
    }

    fn get_status_code(&self) -> &'static str {
        match self {
            ErrorResponse::NotFound(_) => "404",
            ErrorResponse::BadRequest(_) => "400",
            ErrorResponse::Conflict(_) => "409",
            ErrorResponse::InternalServerError => "500",
        }
    }

    fn get_status(&self) -> &'static str {
        match self {
            ErrorResponse::NotFound(_) => "Not Found",
            ErrorResponse::BadRequest(_) => "Bad Request",
            ErrorResponse::Conflict(_) => "Conflict",
            ErrorResponse::InternalServerError => "Internal Server Error",
        }
    }

    fn get_message(&self) -> String {
        match self {
            ErrorResponse::NotFound(message)
            | ErrorResponse::BadRequest(message)
            | ErrorResponse::Conflict(message) => message.to_owned(),
            ErrorResponse::InternalServerError => "Internal Server Error".to_string(),
        }
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ErrorResponse::InternalServerError => write!(f, "{}", self.get_status()),
            _ => write!(f, "{}: {}", self.get_status(), self.get_message()),
        }
    }
}

impl From<ErrorResponse> for Status {
    fn from(error: ErrorResponse) -> Self {
        let mut status = Status::new(error.get_code(), error.get_message());

        let metadata = status.metadata_mut();
        metadata.insert(
            STATUS_CODE_METADATA_KEY,
            MetadataValue::from_static(error.get_status_code()),
        );
        metadata.insert(
            STATUS_METADATA_KEY,
            MetadataValue::from_static(error.get_status()),
        );

        status
    }
}
