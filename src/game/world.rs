pub struct Entity {
    pos: glam::Vec2,
    prev_pos: glam::Vec2,
    vel: glam::Vec2,
    size: glam::Vec2,
    sprite: crate::renderer::TextureId,
}

impl Entity {
    pub fn new_player(pos: glam::Vec2) -> Self {
        Self::new(
            pos,
            glam::vec2(48.0, 48.0),
            crate::renderer::TEST_TEXTURE_ID,
        )
    }

    #[allow(dead_code)]
    pub fn new_solid(pos: glam::Vec2, size: glam::Vec2) -> Self {
        Self::new(pos, size, crate::renderer::TEST_TEXTURE_ID)
    }

    pub fn new(pos: glam::Vec2, size: glam::Vec2, sprite: crate::renderer::TextureId) -> Self {
        let pos = sanitize_vec2(pos, glam::Vec2::ZERO);
        let size = sanitize_size(size, glam::Vec2::ONE);
        Self {
            pos,
            prev_pos: pos,
            vel: glam::Vec2::ZERO,
            size,
            sprite,
        }
    }

    pub fn position(&self) -> glam::Vec2 {
        self.pos
    }

    #[allow(dead_code)]
    pub fn previous_position(&self) -> glam::Vec2 {
        self.prev_pos
    }

    pub fn velocity(&self) -> glam::Vec2 {
        self.vel
    }

    pub fn size(&self) -> glam::Vec2 {
        self.size
    }

    pub fn sprite(&self) -> crate::renderer::TextureId {
        self.sprite
    }

    pub fn set_position(&mut self, pos: glam::Vec2) {
        self.pos = sanitize_vec2(pos, self.pos);
    }

    pub fn set_velocity(&mut self, vel: glam::Vec2) {
        self.vel = sanitize_vec2(vel, glam::Vec2::ZERO);
    }

    pub fn begin_step(&mut self) {
        self.prev_pos = self.pos;
    }

    pub fn aabb(&self) -> crate::game::collision::Aabb {
        crate::game::collision::Aabb::from_pos_size(self.pos, self.size)
    }

    pub fn interpolated_pos(&self, alpha: f32) -> glam::Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

fn sanitize_vec2(value: glam::Vec2, fallback: glam::Vec2) -> glam::Vec2 {
    if value.is_finite() { value } else { fallback }
}

fn sanitize_size(size: glam::Vec2, fallback: glam::Vec2) -> glam::Vec2 {
    if size.is_finite() && size.x > 0.0 && size.y > 0.0 {
        size
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::Entity;

    #[test]
    fn constructor_sanitizes_invalid_position_and_size() {
        let entity = Entity::new_solid(glam::vec2(f32::NAN, 1.0), glam::vec2(-10.0, f32::INFINITY));

        assert_eq!(entity.position(), glam::Vec2::ZERO);
        assert_eq!(entity.previous_position(), glam::Vec2::ZERO);
        assert_eq!(entity.size(), glam::Vec2::ONE);
    }

    #[test]
    fn setters_sanitize_position_and_velocity() {
        let mut entity = Entity::new_player(glam::vec2(2.0, 3.0));
        entity.set_position(glam::vec2(f32::INFINITY, 0.0));
        entity.set_velocity(glam::vec2(f32::NAN, 1.0));

        assert_eq!(entity.position(), glam::vec2(2.0, 3.0));
        assert_eq!(entity.velocity(), glam::Vec2::ZERO);
    }

    #[test]
    fn begin_step_syncs_previous_position() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity.set_position(glam::vec2(10.0, 20.0));
        entity.begin_step();

        assert_eq!(entity.previous_position(), entity.position());
    }
}
