use crate::coord;
use crate::error::{ErrorCode, ErrorInfo};
use crate::geom::{Anchor, Percent, Rect, Size};
use crate::protocol::{CoordinateMapping, CropSpec};

pub fn validate_rect(rect: Rect, source: Size, label: &str) -> Result<Rect, ErrorInfo> {
    rect.validate_non_empty_within(source, label)?;
    Ok(rect)
}

pub fn resolve_anchor(
    anchor: Anchor,
    requested: Size,
    source: Size,
) -> Result<(Rect, CropSpec, Option<String>), ErrorInfo> {
    if requested.width == 0 || requested.height == 0 {
        return Err(ErrorInfo::with_message(
            ErrorCode::InvalidDimensions,
            "viewport width and height must be > 0",
        ));
    }

    let warned = if requested.width > source.width || requested.height > source.height {
        Some(format!(
            "viewport ({}x{}) exceeds source ({}x{}); clamped to source bounds",
            requested.width, requested.height, source.width, source.height
        ))
    } else {
        None
    };

    let resolved = coord::anchor_to_rect(anchor, requested, source);
    let spec = CropSpec::Anchor {
        anchor,
        requested,
        resolved,
    };
    Ok((resolved, spec, warned))
}

pub fn resolve_percent(percent: Percent, source: Size) -> Result<(Rect, CropSpec), ErrorInfo> {
    percent.validate_unit_rect()?;
    let resolved = coord::percent_to_rect(percent, source);
    Ok((resolved, CropSpec::Percent { percent, resolved }))
}

pub fn resolve_rect(rect: Rect, source: Size) -> Result<(Rect, CropSpec), ErrorInfo> {
    let rect = validate_rect(rect, source, "rect")?;
    Ok((rect, CropSpec::Rect { rect }))
}

pub fn coordinate_mapping(source_rect: Rect, result_size: Size) -> CoordinateMapping {
    coord::make_mapping(source_rect, result_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_clamps_to_source() {
        let (resolved, _, warning) = resolve_anchor(
            Anchor::Center,
            Size {
                width: 200,
                height: 200,
            },
            Size {
                width: 64,
                height: 64,
            },
        )
        .unwrap();
        assert_eq!(resolved.width, 64);
        assert_eq!(resolved.height, 64);
        assert!(warning.is_some());
    }

    #[test]
    fn percent_rejects_nan() {
        let err = resolve_percent(
            Percent {
                x: f64::NAN,
                y: 0.0,
                w: 0.5,
                h: 0.5,
            },
            Size {
                width: 100,
                height: 100,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "INVALID_PARAMETERS");
    }

    #[test]
    fn rect_rejects_overflow() {
        let err = resolve_rect(
            Rect {
                x: u32::MAX,
                y: 0,
                width: 2,
                height: 1,
            },
            Size {
                width: 100,
                height: 100,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "INVALID_COORDINATES");
    }
}
