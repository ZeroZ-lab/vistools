/// Maximum input image pixels (100 megapixels).
pub const MAX_PIXELS: u64 = 100_000_000;

/// Maximum number of tiles (rows × cols).
pub const MAX_TILE_COUNT: u32 = 64;

/// Threshold above which an overview is recommended.
pub const OVERVIEW_THRESHOLD: u32 = 1568;

/// Default JPEG output quality.
pub const DEFAULT_JPEG_QUALITY: u8 = 95;

/// Default luma threshold for highlight clipping detection.
pub const DEFAULT_HIGHLIGHT_THRESHOLD: u8 = 250;

/// Default luma threshold for shadow clipping detection.
pub const DEFAULT_SHADOW_THRESHOLD: u8 = 5;
