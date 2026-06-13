pub struct Entity {
    pub pos: glam::Vec2,
    pub prev_pos: glam::Vec2,
    pub vel: glam::Vec2,
    pub size: glam::Vec2,
    pub sprite: crate::renderer::TextureId,
}

impl Entity {
    pub fn interpolated_pos(&self, alpha: f32) -> glam::Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}
