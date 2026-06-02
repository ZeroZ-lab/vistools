//! Coordinate calculation and mapping primitives.
//!
//! Decisions: PD2 (unified coordinate system), FD2 (coordinate mapping per operation).
use crate::geom::{Anchor, Percent, Point, Rect, Size};
use crate::protocol::CoordinateMapping;

/// Convert a nine-position anchor to a pixel crop rectangle.
///
/// The anchor selects which region of the source image to extract.
/// `viewport_w` and `viewport_h` define the crop size.
pub fn anchor_to_rect(anchor: Anchor, requested: Size, source: Size) -> Rect {
    let width = requested.width.min(source.width);
    let height = requested.height.min(source.height);
    let x = match anchor {
        Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => 0,
        Anchor::Top | Anchor::Center | Anchor::Bottom => source.width.saturating_sub(width) / 2,
        Anchor::TopRight | Anchor::Right | Anchor::BottomRight => {
            source.width.saturating_sub(width)
        }
    };
    let y = match anchor {
        Anchor::TopLeft | Anchor::Top | Anchor::TopRight => 0,
        Anchor::Left | Anchor::Center | Anchor::Right => source.height.saturating_sub(height) / 2,
        Anchor::BottomLeft | Anchor::Bottom | Anchor::BottomRight => {
            source.height.saturating_sub(height)
        }
    };
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// Convert percentage-based region to pixel rectangle.
///
/// `px = pct.x * source.width`, clamped to source bounds.
pub fn percent_to_rect(pct: Percent, source: Size) -> Rect {
    let x = (pct.x * source.width as f64).round() as u32;
    let y = (pct.y * source.height as f64).round() as u32;
    let w = (pct.w * source.width as f64).round() as u32;
    let h = (pct.h * source.height as f64).round() as u32;
    // Clamp to source bounds
    let x = x.min(source.width);
    let y = y.min(source.height);
    let w = w.min(source.width.saturating_sub(x));
    let h = h.min(source.height.saturating_sub(y));
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

/// Build a coordinate mapping from source crop region to result image.
///
/// Used by viewport and overview to describe how
/// output coordinates relate back to the source image.
pub fn make_mapping(source_rect: Rect, result_size: Size) -> CoordinateMapping {
    let scale_x = if source_rect.width > 0 {
        result_size.width as f64 / source_rect.width as f64
    } else {
        1.0
    };
    let scale_y = if source_rect.height > 0 {
        result_size.height as f64 / source_rect.height as f64
    } else {
        1.0
    };
    let has_scale = (scale_x - 1.0).abs() > f64::EPSILON || (scale_y - 1.0).abs() > f64::EPSILON;
    let formula = if has_scale && source_rect.x == 0 && source_rect.y == 0 {
        format!("source_x = result_x / {scale_x:.6}, source_y = result_y / {scale_y:.6}")
    } else if has_scale {
        format!(
            "source_x = result_x / {:.6} + {}, source_y = result_y / {:.6} + {}",
            scale_x, source_rect.x, scale_y, source_rect.y
        )
    } else if source_rect.x == 0 && source_rect.y == 0 {
        "source_x = result_x, source_y = result_y".to_string()
    } else {
        format!(
            "source_x = result_x + {}, source_y = result_y + {}",
            source_rect.x, source_rect.y
        )
    };

    CoordinateMapping {
        source_origin: Point {
            x: source_rect.x,
            y: source_rect.y,
        },
        scale_x,
        scale_y,
        formula,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_right_on_6000x4000() {
        // Contract verification: anchor right, w=2000, h=4000 → x=4000, y=0
        let rect = anchor_to_rect(
            Anchor::Right,
            Size {
                width: 2000,
                height: 4000,
            },
            Size {
                width: 6000,
                height: 4000,
            },
        );
        assert_eq!(rect.x, 4000);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 2000);
        assert_eq!(rect.height, 4000);
    }

    #[test]
    fn anchor_center_on_1000x1000() {
        let rect = anchor_to_rect(
            Anchor::Center,
            Size {
                width: 500,
                height: 500,
            },
            Size {
                width: 1000,
                height: 1000,
            },
        );
        assert_eq!(rect.x, 250);
        assert_eq!(rect.y, 250);
    }

    #[test]
    fn anchor_bottom_left() {
        let rect = anchor_to_rect(
            Anchor::BottomLeft,
            Size {
                width: 200,
                height: 300,
            },
            Size {
                width: 1000,
                height: 800,
            },
        );
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 500);
    }

    #[test]
    fn percent_half_on_1000x1000() {
        // Contract: percent (0.5, 0.5, 0.5, 0.5) on 1000x1000 → (500, 500, 500, 500)
        let rect = percent_to_rect(
            Percent {
                x: 0.5,
                y: 0.5,
                w: 0.5,
                h: 0.5,
            },
            Size {
                width: 1000,
                height: 1000,
            },
        );
        assert_eq!(rect.x, 500);
        assert_eq!(rect.y, 500);
        assert_eq!(rect.width, 500);
        assert_eq!(rect.height, 500);
    }

    #[test]
    fn percent_full() {
        let rect = percent_to_rect(
            Percent {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            Size {
                width: 6000,
                height: 4000,
            },
        );
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 6000);
        assert_eq!(rect.height, 4000);
    }

    #[test]
    fn mapping_crop_only() {
        let mapping = make_mapping(
            Rect {
                x: 100,
                y: 200,
                width: 500,
                height: 500,
            },
            Size {
                width: 500,
                height: 500,
            },
        );
        assert_eq!(mapping.source_origin.x, 100);
        assert_eq!(mapping.source_origin.y, 200);
        assert_eq!(mapping.scale_x, 1.0);
        assert_eq!(mapping.scale_y, 1.0);
        assert!(mapping.formula.contains("100"));
    }

    #[test]
    fn mapping_with_scale() {
        let mapping = make_mapping(
            Rect {
                x: 0,
                y: 0,
                width: 6000,
                height: 4000,
            },
            Size {
                width: 1200,
                height: 800,
            },
        );
        assert_eq!(mapping.source_origin.x, 0);
        assert_eq!(mapping.source_origin.y, 0);
        assert!((mapping.scale_x - 0.2).abs() < 0.01);
        assert!((mapping.scale_y - 0.2).abs() < 0.01);
    }
}
