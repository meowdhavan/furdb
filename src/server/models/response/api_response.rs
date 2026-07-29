/// Marks a response envelope as successful.
pub const RESULT_SUCCESS: &str = "success";

/// The envelope every response carries alongside its payload. gRPC has its own
/// status codes, but the `statusCode`/`status` pair is part of FurDB's message
/// structure, so successful responses keep reporting it.
#[derive(Clone, Copy)]
pub enum SuccessStatus {
    Ok,
    Created,
}

impl SuccessStatus {
    pub fn get_status_code(&self) -> u32 {
        match self {
            SuccessStatus::Ok => 200,
            SuccessStatus::Created => 201,
        }
    }

    pub fn get_status(&self) -> String {
        match self {
            SuccessStatus::Ok => "OK".to_string(),
            SuccessStatus::Created => "Created".to_string(),
        }
    }
}
