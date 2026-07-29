use crate::server::models::response::ErrorResponse;

/// The rejected value is echoed back to help debugging, but it ends up in the
/// `grpc-message` header, which the peer's header-list limit applies to.
const MAX_ECHOED_VALUE_LENGTH: usize = 64;

/// A column may be up to 128 bits wide, and protobuf has no 128-bit integer, so
/// the values stored in one travel the wire as decimal strings. `field` names
/// the offending field in the rejection.
pub fn parse_u128(value: &str, field: &str) -> Result<u128, ErrorResponse> {
    value.parse().map_err(|_| {
        ErrorResponse::BadRequest(format!(
            "Invalid value for `{field}`: `{}` is not an unsigned 128-bit integer",
            truncate(value)
        ))
    })
}

fn truncate(value: &str) -> String {
    if value.len() <= MAX_ECHOED_VALUE_LENGTH {
        return value.to_string();
    }

    // Never split a character in half — `grpc-message` has to stay valid UTF-8.
    let end = (0..=MAX_ECHOED_VALUE_LENGTH)
        .rev()
        .find(|index| value.is_char_boundary(*index))
        .unwrap_or(0);

    format!("{}…", &value[..end])
}
