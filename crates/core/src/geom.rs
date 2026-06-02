use serde::{Deserialize, Serialize};

use crate::error::{ErrorCode, ErrorInfo};

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
    pub fn checked_right(&self) -> Option<u32> {
        self.x.checked_add(self.width)
    }

    pub fn checked_bottom(&self) -> Option<u32> {
        self.y.checked_add(self.height)
    }

    pub fn right(&self) -> u32 {
        self.checked_right()
            .expect("rect right overflowed u32; validate before use")
    }

    pub fn bottom(&self) -> u32 {
        self.checked_bottom()
            .expect("rect bottom overflowed u32; validate before use")
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn fits_within(&self, size: Size) -> bool {
        match (self.checked_right(), self.checked_bottom()) {
            (Some(right), Some(bottom)) => right <= size.width && bottom <= size.height,
            _ => false,
        }
    }

    pub fn validate_non_empty_within(&self, size: Size, label: &str) -> Result<(), ErrorInfo> {
        if self.is_empty() {
            return Err(ErrorInfo::with_message(
                ErrorCode::InvalidDimensions,
                format!("{label} width and height must be > 0"),
            ));
        }
        if !self.fits_within(size) {
            return Err(ErrorInfo::with_message(
                ErrorCode::InvalidCoordinates,
                format!(
                    "{label} ({},{},{},{}) exceeds source ({}x{})",
                    self.x, self.y, self.width, self.height, size.width, size.height
                ),
            ));
        }
        Ok(())
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

impl Percent {
    pub fn validate_unit_rect(&self) -> Result<(), ErrorInfo> {
        if !self.x.is_finite() || !self.y.is_finite() || !self.w.is_finite() || !self.h.is_finite()
        {
            return Err(ErrorInfo::with_message(
                ErrorCode::InvalidParameters,
                "percent values must be finite numbers",
            ));
        }
        if self.x < 0.0
            || self.x > 1.0
            || self.y < 0.0
            || self.y > 1.0
            || self.w <= 0.0
            || self.w > 1.0
            || self.h <= 0.0
            || self.h > 1.0
        {
            return Err(ErrorInfo::with_message(
                ErrorCode::InvalidParameters,
                "percent x,y must be within 0..1 and w,h must be within 0..1",
            ));
        }
        if self.x + self.w > 1.0 + f64::EPSILON || self.y + self.h > 1.0 + f64::EPSILON {
            return Err(ErrorInfo::with_message(
                ErrorCode::InvalidCoordinates,
                "percent region exceeds source bounds",
            ));
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn rect_fits_within_rejects_overflow() {
        let rect = Rect {
            x: u32::MAX,
            y: 0,
            width: 2,
            height: 1,
        };
        assert!(!rect.fits_within(Size {
            width: 10,
            height: 10
        }));
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
        assert!(p.validate_unit_rect().is_ok());
    }
}
