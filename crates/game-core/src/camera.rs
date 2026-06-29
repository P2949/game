/// Zoom used whenever an invalid (non-finite or non-positive) zoom is supplied.
const DEFAULT_ZOOM: f32 = 1.0;
/// Smallest allowed zoom. Keeps the visible half-extents from exploding toward
/// infinity as zoom approaches zero.
const MIN_ZOOM: f32 = 0.01;
/// Largest allowed zoom. Keeps `right - left` and `top - bottom` safely above
/// zero so the orthographic projection never collapses to a degenerate matrix.
const MAX_ZOOM: f32 = 100_000.0;

fn sanitize_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    } else {
        DEFAULT_ZOOM
    }
}

fn sanitize_dimension(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

fn sanitize_center(center: glam::Vec2) -> glam::Vec2 {
    if center.is_finite() {
        center
    } else {
        glam::Vec2::ZERO
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera2D {
    center: glam::Vec2,
    zoom: f32,
}

impl Camera2D {
    pub fn new(center: glam::Vec2, zoom: f32) -> Self {
        Self {
            center: sanitize_center(center),
            zoom: sanitize_zoom(zoom),
        }
    }

    #[allow(dead_code)]
    pub fn center(&self) -> glam::Vec2 {
        self.center
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = sanitize_zoom(zoom);
    }

    pub fn set_center(&mut self, center: glam::Vec2) {
        self.center = sanitize_center(center);
    }

    pub fn view_projection(&self, width: f32, height: f32) -> glam::Mat4 {
        let zoom = sanitize_zoom(self.zoom);
        let center = sanitize_center(self.center);
        let width = sanitize_dimension(width);
        let height = sanitize_dimension(height);
        let half_w = width * 0.5 / zoom;
        let half_h = height * 0.5 / zoom;

        glam::camera::rh::proj::directx::orthographic(
            center.x - half_w,
            center.x + half_w,
            center.y - half_h,
            center.y + half_h,
            -1.0,
            1.0,
        )
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
        let camera = Camera2D::new(glam::vec2(f32::NAN, 0.0), 0.0);
        assert_eq!(camera.zoom(), DEFAULT_ZOOM);
        assert_eq!(camera.center(), glam::Vec2::ZERO);

        let mut camera = Camera2D::new(glam::Vec2::ZERO, 3.0);
        assert_eq!(camera.zoom(), 3.0);
        camera.set_zoom(f32::NAN);
        assert_eq!(camera.zoom(), DEFAULT_ZOOM);
    }

    #[test]
    fn view_projection_stays_finite_for_bad_inputs() {
        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        camera.set_zoom(0.0);
        camera.set_center(glam::vec2(f32::NAN, f32::INFINITY));

        for (w, h) in [
            (1280.0, 720.0),
            (0.0, 0.0),
            (f32::NAN, 720.0),
            (1280.0, f32::INFINITY),
        ] {
            let m = camera.view_projection(w, h);
            assert!(m.to_cols_array().iter().all(|v| v.is_finite()));
        }
    }
}
