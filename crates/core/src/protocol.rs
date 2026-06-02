use serde::{Deserialize, Serialize};

use crate::error::ErrorInfo;
use crate::geom::{Anchor, Percent, Point, Rect, Size};

/// Describes how output coordinates relate to source coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateMapping {
    pub source_origin: Point,
    pub scale_x: f64,
    pub scale_y: f64,
    pub formula: String,
}

/// Common source image information included in all command outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
}

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
    pub recommended_next: String,
    pub reason: String,
    pub suggested_max_side: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewOutput {
    pub output: String,
    pub source: SourceInfo,
    pub result: Size,
    pub scale_factor: f64,
    pub coordinate_mapping: CoordinateMapping,
}

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
    pub spec: CropSpec,
    pub region: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CropSpec {
    Anchor {
        anchor: Anchor,
        requested: Size,
        resolved: Rect,
    },
    Percent {
        percent: Percent,
        resolved: Rect,
    },
    Rect {
        rect: Rect,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleOutput {
    pub source: SourceInfo,
    pub sample: SampleResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SampleResult {
    Point {
        point: Point,
        color: ColorInfo,
    },
    Rect {
        region: Rect,
        average: ColorInfo,
        alpha_stats: AlphaStats,
        pixel_count: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorInfo {
    pub rgba: [u8; 4],
    pub rgb: [u8; 3],
    pub hex: String,
    pub alpha: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaStats {
    pub min: u8,
    pub max: u8,
    pub average: f64,
    pub transparent_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpnessOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub sharpness: SharpnessMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpnessMetrics {
    pub score: f64,
    pub mean_edge_strength: f64,
    pub max_edge_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub histogram: HistogramMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramMetrics {
    pub bins: Vec<u64>,
    pub pixel_count: u64,
    pub mean_luma: f64,
    pub median_luma: u8,
    pub p05_luma: u8,
    pub p95_luma: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippingOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub clipping: ClippingMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippingMetrics {
    pub threshold: u8,
    pub clipped_pixels: u64,
    pub clipped_ratio: f64,
    pub pixel_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub contrast: ContrastMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastMetrics {
    pub rms_contrast: f64,
    pub luma_stddev: f64,
    pub min_luma: u8,
    pub max_luma: u8,
    pub dynamic_range: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorCastOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub color_cast: ColorCastMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorCastMetrics {
    pub channel_means: [f64; 3],
    pub neutral_mean: f64,
    pub cast_vector: [f64; 3],
    pub cast_strength: f64,
    pub dominant_channel: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorCode, ErrorInfo};

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
}
