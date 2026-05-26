use rmcp::model::{ErrorCode, ErrorData};

pub fn tool_error(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None)
}
