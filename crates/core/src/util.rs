//! Shared utility functions.
use std::path::Path;

use crate::types::DEFAULT_JPEG_QUALITY;

/// Save an image to the specified path, inferring format from extension.
///
/// JPEG output uses `DEFAULT_JPEG_QUALITY` (95) via `JpegEncoder`.
/// Other formats delegate to `DynamicImage::save()`.
/// FD6: format from output extension.
pub fn save_image(img: &image::DynamicImage, path: &Path) -> Result<(), String> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => {
            let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
            let mut buf = std::io::BufWriter::new(file);
            let encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, DEFAULT_JPEG_QUALITY);
            img.write_with_encoder(encoder).map_err(|e| e.to_string())
        }
        _ => img.save(path).map_err(|e| e.to_string()),
    }
}
