//! Core types for vistools.
//!
//! All types implement Debug, Clone, Serialize, Deserialize.
//! Coordinate system: origin top-left, x→right, y→down.
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Coordinate primitives
// ---------------------------------------------------------------------------

/// Pixel coordinate point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

/// Pixel rectangle region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Percentage-based region (0.0–1.0 relative to source image).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Percent {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Nine-position semantic anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

/// Image dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Coordinate mapping
// ---------------------------------------------------------------------------

/// Describes how output coordinates relate to source coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateMapping {
    /// [x, y] offset of the crop/viewport origin in source coordinates.
    pub crop_origin_in_source: [u32; 2],
    /// Scale factor (None for crop-only operations with no resize).
    pub scale_factor: Option<f64>,
    /// Human-readable formula for coordinate conversion.
    pub formula: String,
}

// ---------------------------------------------------------------------------
// Source info
// ---------------------------------------------------------------------------

/// Common source image information included in all command outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
}

// ---------------------------------------------------------------------------
// Unified output wrapper
// ---------------------------------------------------------------------------

/// Every command returns this JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult<T> {
    pub ok: bool,
    pub operation: String,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub elapsed_ms: u64,
}

impl<T> CommandResult<T> {
    pub fn ok(operation: impl Into<String>, input: impl Into<String>, data: T) -> Self {
        Self {
            ok: true,
            operation: operation.into(),
            input: input.into(),
            data: Some(data),
            error: None,
            warnings: Vec::new(),
            elapsed_ms: 0,
        }
    }

    pub fn err(operation: impl Into<String>, input: impl Into<String>, error: ErrorInfo) -> Self {
        Self {
            ok: false,
            operation: operation.into(),
            input: input.into(),
            data: None,
            error: Some(error),
            warnings: Vec::new(),
            elapsed_ms: 0,
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    pub fn with_elapsed_ms(mut self, ms: u64) -> Self {
        self.elapsed_ms = ms;
        self
    }
}

// ---------------------------------------------------------------------------
// Command-specific output types
// ---------------------------------------------------------------------------

// --- inspect ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectOutput {
    pub source: SourceInfo,
    pub suggestion: Suggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub needs_overview: bool,
    pub max_tile_rows: u32,
    pub max_tile_cols: u32,
}

// --- overview ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewOutput {
    pub output: String,
    pub source: SourceInfo,
    pub result: Size,
    pub scale_factor: f64,
    pub coordinate_mapping: CoordinateMapping,
}

// --- tile ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileOutput {
    pub source: SourceInfo,
    pub rows: u32,
    pub cols: u32,
    pub tiles: Vec<TileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileInfo {
    pub path: String,
    pub row: u32,
    pub col: u32,
    pub width: u32,
    pub height: u32,
    pub source_region: Rect,
}

// --- viewport ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportOutput {
    pub output: String,
    pub source: SourceInfo,
    pub crop: CropInfo,
    pub result: Size,
    pub coordinate_mapping: CoordinateMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropInfo {
    /// "anchor" | "percent" | "rect"
    pub mode: String,
    /// The resolved pixel region that was cropped.
    pub region: Rect,
    /// The original parameters the user passed (varies by mode).
    pub params: serde_json::Value,
}

// --- resize ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeOutput {
    pub output: String,
    pub source: SourceInfo,
    pub result: Size,
    pub scale_factor: f64,
    pub coordinate_mapping: CoordinateMapping,
}

// --- rotate ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateOutput {
    pub output: String,
    pub source: SourceInfo,
    pub result: Size,
    pub degrees: u32,
    pub coordinate_mapping: CoordinateMapping,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum input image pixels (100 megapixels).
pub const MAX_PIXELS: u64 = 100_000_000;

/// Maximum number of tiles (rows × cols).
pub const MAX_TILE_COUNT: u32 = 64;

/// Threshold above which an overview is recommended (from Claude docs).
pub const OVERVIEW_THRESHOLD: u32 = 1568;

/// Default JPEG output quality.
pub const DEFAULT_JPEG_QUALITY: u8 = 95;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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

    #[test]
    fn command_result_ok_serializes() {
        let result: CommandResult<()> = CommandResult::ok("inspect", "test.png", ());
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"operation\":\"inspect\""));
    }

    #[test]
    fn command_result_err_serializes() {
        let result = CommandResult::<()>::err(
            "inspect",
            "missing.png",
            ErrorInfo::with_message(ErrorCode::FileNotFound, "file not found"),
        );
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("FILE_NOT_FOUND"));
    }

    #[test]
    fn rect_fields() {
        let r = Rect {
            x: 10,
            y: 20,
            width: 100,
            height: 200,
        };
        assert_eq!(r.x, 10);
        assert_eq!(r.right(), 110);
        assert_eq!(r.bottom(), 220);
    }

    #[test]
    fn percent_boundary() {
        let p = Percent {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        };
        assert_eq!(p.x, 0.0);
        assert_eq!(p.w, 1.0);
    }
}
