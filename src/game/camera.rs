/// Zoom used whenever an invalid (non-finite or non-positive) zoom is supplied.
const DEFAULT_ZOOM: f32 = 1.0;
/// Smallest allowed zoom. Keeps the visible half-extents from exploding toward
/// infinity as zoom approaches zero.
const MIN_ZOOM: f32 = 0.01;
/// Largest allowed zoom. Keeps `right - left` (and `top - bottom`) safely above
/// zero so the orthographic projection never collapses to a degenerate matrix.
const MAX_ZOOM: f32 = 100_000.0;

/// Maps any zoom into a safe, strictly-positive, finite range. Non-finite
/// (NaN/inf) or non-positive values fall back to [`DEFAULT_ZOOM`]; otherwise the
/// value is clamped to `[MIN_ZOOM, MAX_ZOOM]`. This makes division by `zoom` in
/// [`Camera2D::view_projection`] and the resulting projection always well-defined.
fn sanitize_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    } else {
        DEFAULT_ZOOM
    }
}

/// Maps a viewport dimension to a finite, strictly-positive value. A non-finite
/// (NaN/inf) or non-positive width/height would otherwise produce a degenerate or
/// non-finite projection; fall back to 1.0 so the matrix stays well-defined.
fn sanitize_dimension(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera2D {
    pub center: glam::Vec2,
    pub zoom: f32,
}

impl Camera2D {
    /// Creates a camera, sanitizing `zoom` into the safe range so callers cannot
    /// construct one that divides by zero or produces a non-finite projection.
    pub fn new(center: glam::Vec2, zoom: f32) -> Self {
        Self {
            center,
            zoom: sanitize_zoom(zoom),
        }
    }

    /// Sets the zoom, sanitizing it into the safe range.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = sanitize_zoom(zoom);
    }

    pub fn view_projection(&self, width: f32, height: f32) -> glam::Mat4 {
        // Sanitize at the point of use as well, so even a directly-mutated public
        // `zoom` field or a bogus viewport size can never produce a
        // divide-by-zero, degenerate, or non-finite matrix.
        let zoom = sanitize_zoom(self.zoom);
        let width = sanitize_dimension(width);
        let height = sanitize_dimension(height);
        let half_w = width * 0.5 / zoom;
        let half_h = height * 0.5 / zoom;

        let left = self.center.x - half_w;
        let right = self.center.x + half_w;
        let bottom = self.center.y - half_h;
        let top = self.center.y + half_h;

        glam::Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0)
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            center: glam::Vec2::ZERO,
            zoom: DEFAULT_ZOOM,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera2D, DEFAULT_ZOOM, MAX_ZOOM, MIN_ZOOM, sanitize_zoom};

    #[test]
    fn sanitize_zoom_rejects_invalid_values() {
        assert_eq!(sanitize_zoom(0.0), DEFAULT_ZOOM);
        assert_eq!(sanitize_zoom(-2.0), DEFAULT_ZOOM);
        assert_eq!(sanitize_zoom(f32::NAN), DEFAULT_ZOOM);
        assert_eq!(sanitize_zoom(f32::INFINITY), DEFAULT_ZOOM);
        assert_eq!(sanitize_zoom(f32::NEG_INFINITY), DEFAULT_ZOOM);
    }

    #[test]
    fn sanitize_zoom_preserves_and_clamps_valid_values() {
        assert_eq!(sanitize_zoom(2.0), 2.0);
        assert_eq!(sanitize_zoom(MIN_ZOOM / 2.0), MIN_ZOOM);
        assert_eq!(sanitize_zoom(MAX_ZOOM * 2.0), MAX_ZOOM);
    }

    #[test]
    fn constructor_and_setter_sanitize() {
        let camera = Camera2D::new(glam::Vec2::ZERO, 0.0);
        assert_eq!(camera.zoom, DEFAULT_ZOOM);

        let mut camera = Camera2D::new(glam::Vec2::ZERO, 3.0);
        assert_eq!(camera.zoom, 3.0);
        camera.set_zoom(f32::NAN);
        assert_eq!(camera.zoom, DEFAULT_ZOOM);
    }

    #[test]
    fn view_projection_is_finite_even_for_poisoned_zoom() {
        // Directly poison the public field, bypassing the constructor/setter.
        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        camera.zoom = 0.0;
        let m = camera.view_projection(1280.0, 720.0);
        assert!(m.to_cols_array().iter().all(|v| v.is_finite()));
    }

    #[test]
    fn view_projection_is_finite_for_non_finite_or_zero_dimensions() {
        let camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        for (w, h) in [
            (0.0, 0.0),
            (f32::NAN, 720.0),
            (1280.0, f32::INFINITY),
            (-1280.0, -720.0),
        ] {
            let m = camera.view_projection(w, h);
            assert!(
                m.to_cols_array().iter().all(|v| v.is_finite()),
                "non-finite matrix for size {w}x{h}"
            );
        }
    }
}
