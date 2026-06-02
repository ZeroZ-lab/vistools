use serde::{Deserialize, Serialize};

/// Stable error codes that agents can pattern-match on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    FileNotFound,
    UnsupportedFormat,
    InvalidDimensions,
    InvalidCoordinates,
    InvalidParameters,
    OutputWriteError,
    PathEscape,
    OutputSameAsInput,
    PixelLimitExceeded,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::UnsupportedFormat => "UNSUPPORTED_FORMAT",
            Self::InvalidDimensions => "INVALID_DIMENSIONS",
            Self::InvalidCoordinates => "INVALID_COORDINATES",
            Self::InvalidParameters => "INVALID_PARAMETERS",
            Self::OutputWriteError => "OUTPUT_WRITE_ERROR",
            Self::PathEscape => "PATH_ESCAPE",
            Self::OutputSameAsInput => "OUTPUT_SAME_AS_INPUT",
            Self::PixelLimitExceeded => "PIXEL_LIMIT_EXCEEDED",
        }
    }
}

/// Structured error info returned in JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
}

impl From<&ErrorCode> for ErrorInfo {
    fn from(code: &ErrorCode) -> Self {
        Self {
            code: code.as_str().to_string(),
            message: String::new(),
        }
    }
}

impl ErrorInfo {
    pub fn with_message(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.as_str().to_string(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_strings() {
        assert_eq!(ErrorCode::FileNotFound.as_str(), "FILE_NOT_FOUND");
        assert_eq!(ErrorCode::PathEscape.as_str(), "PATH_ESCAPE");
        assert_eq!(
            ErrorCode::PixelLimitExceeded.as_str(),
            "PIXEL_LIMIT_EXCEEDED"
        );
    }
}
