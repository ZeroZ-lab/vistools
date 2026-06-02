use std::fs;
use std::path::Path;

use crate::error::{ErrorCode, ErrorInfo};
use crate::geom::Size;
use crate::guard;
use crate::protocol::SourceInfo;

#[derive(Debug)]
pub struct ImageSource {
    pub image: image::DynamicImage,
    pub info: SourceInfo,
}

#[derive(Debug)]
pub struct RgbaSource {
    pub image: image::RgbaImage,
    pub info: SourceInfo,
}

pub fn load_image_source(input: &Path) -> Result<ImageSource, ErrorInfo> {
    guard::validate_input_path(input)?;

    let image = image::open(input)
        .map_err(|e| ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()))?;

    let info = load_source_info(
        input,
        Size {
            width: image.width(),
            height: image.height(),
        },
    )?;

    Ok(ImageSource { image, info })
}

pub fn load_rgba_source(input: &Path) -> Result<RgbaSource, ErrorInfo> {
    let source = load_image_source(input)?;
    Ok(RgbaSource {
        image: source.image.to_rgba8(),
        info: source.info,
    })
}

pub fn infer_format(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(e) if e == "png" => "png".to_string(),
        Some(e) if e == "jpg" || e == "jpeg" => "jpeg".to_string(),
        Some(e) if e == "webp" => "webp".to_string(),
        Some(e) if e == "tiff" || e == "tif" => "tiff".to_string(),
        Some(e) if e == "bmp" => "bmp".to_string(),
        Some(e) if e == "gif" => "gif".to_string(),
        _ => "unknown".to_string(),
    }
}

fn load_source_info(input: &Path, size: Size) -> Result<SourceInfo, ErrorInfo> {
    let file_meta = fs::metadata(input)
        .map_err(|e| ErrorInfo::with_message(ErrorCode::FileNotFound, e.to_string()))?;

    guard::validate_dimensions(size.width, size.height)?;

    Ok(SourceInfo {
        width: size.width,
        height: size.height,
        format: infer_format(input),
        size_bytes: file_meta.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::MAX_PIXELS;
    use crate::test_support::fixture;

    #[test]
    fn infer_format_variants() {
        assert_eq!(infer_format(Path::new("test.png")), "png");
        assert_eq!(infer_format(Path::new("test.jpg")), "jpeg");
        assert_eq!(infer_format(Path::new("test.JPEG")), "jpeg");
        assert_eq!(infer_format(Path::new("test.webp")), "webp");
        assert_eq!(infer_format(Path::new("test.unknown")), "unknown");
    }

    #[test]
    fn load_image_source_reads_metadata() {
        let source = load_image_source(&fixture("64x64.png")).unwrap();
        assert_eq!(source.info.width, 64);
        assert_eq!(source.info.height, 64);
        assert_eq!(source.info.format, "png");
    }

    #[test]
    fn load_image_source_rejects_missing_file() {
        let err = load_image_source(Path::new("/tmp/__missing_vistools_source__.png")).unwrap_err();
        assert_eq!(err.code, "FILE_NOT_FOUND");
    }

    #[test]
    fn pixel_limit_constant_is_100mp() {
        assert_eq!(MAX_PIXELS, 100_000_000);
    }
}
