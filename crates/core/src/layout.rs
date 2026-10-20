//! Window geometry: the account strip on the left, the active session filling the rest.

/// Width of the account strip in logical pixels.
pub const STRIP_WIDTH: f64 = 68.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub strip: Rect,
    pub content: Rect,
}

/// Splits a window's logical inner size into the strip and content areas.
/// Sizes never go negative, even while the window is being minimised.
pub fn split(width: f64, height: f64) -> Layout {
    let width = width.max(0.0);
    let height = height.max(0.0);
    let strip_width = STRIP_WIDTH.min(width);
    Layout {
        strip: Rect {
            x: 0.0,
            y: 0.0,
            width: strip_width,
            height,
        },
        content: Rect {
            x: strip_width,
            y: 0.0,
            width: width - strip_width,
            height,
        },
    }
}
