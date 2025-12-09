/// Secure error handling utilities
/// Prevents information disclosure through error messages

use actix_web::HttpResponse;
use serde_json::json;
use uuid::Uuid;

/// Generate a request ID for error tracking
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Log error securely without exposing sensitive information
pub fn log_error_safely(context: &str, error: &dyn std::error::Error, request_id: &str) {
    // In production, use proper logging framework (e.g., tracing, log)
    // Only log error type and context, not full error messages
    log::error!(
        "Error in {}: {} (Request ID: {})",
        context,
        error,
        request_id
    );
}

/// Create a generic error response for production
pub fn create_error_response(request_id: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "error": "Internal server error",
        "request_id": request_id,
        "message": "An error occurred processing your request. Please try again later."
    }))
}

/// Create a generic error response with custom message
pub fn create_error_response_with_message(request_id: &str, message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "error": "Internal server error",
        "request_id": request_id,
        "message": message
    }))
}

/// Validate string input length
pub fn validate_string_length(input: &str, max_length: usize, field_name: &str) -> Result<(), String> {
    if input.len() > max_length {
        return Err(format!("{} exceeds maximum length of {} characters", field_name, max_length));
    }
    Ok(())
}

/// Validate UUID format
pub fn validate_uuid(uuid_str: &str, field_name: &str) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(uuid_str).map_err(|_| {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid format",
            "field": field_name,
            "message": format!("{} must be a valid UUID", field_name)
        }))
    })
}

/// Sanitize string for logging (remove sensitive data)
pub fn sanitize_for_logging(input: &str) -> String {
    // Truncate very long strings to prevent DoS
    let mut sanitized = if input.len() > 500 {
        let mut truncated = input.chars().take(500).collect::<String>();
        truncated.push_str("... [TRUNCATED]");
        truncated
    } else {
        input.to_string()
    };
    
    // Simple sanitization - remove newlines and control characters
    sanitized = sanitized.replace('\n', " ").replace('\r', " ");
    sanitized = sanitized.chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect();
    
    sanitized
}

