#[derive(Debug, Clone, Copy)]
pub struct Camera2D {
    pub center: glam::Vec2,
    pub zoom: f32,
}

impl Camera2D {
    pub fn view_projection(&self, width: f32, height: f32) -> glam::Mat4 {
        let half_w = width * 0.5 / self.zoom;
        let half_h = height * 0.5 / self.zoom;

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
            zoom: 1.0,
        }
    }
}
