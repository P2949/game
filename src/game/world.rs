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
        Self::try_new(pos, size, sprite).expect("entity position/size must be valid")
    }

    pub fn try_new(
        pos: glam::Vec2,
        size: glam::Vec2,
        sprite: crate::renderer::TextureId,
    ) -> anyhow::Result<Self> {
        if !pos.is_finite() {
            anyhow::bail!("entity position must be finite, got {pos:?}");
        }
        if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
            anyhow::bail!("entity size must be finite and positive, got {size:?}");
        }
        crate::game::collision::Aabb::try_from_pos_size(pos, size)
            .map_err(|err| anyhow::anyhow!("invalid entity bounds: {err}"))?;

        Ok(Self {
            pos,
            prev_pos: pos,
            vel: glam::Vec2::ZERO,
            size,
            sprite,
        })
    }

    #[allow(dead_code)]
    pub fn new_sanitized(
        pos: glam::Vec2,
        size: glam::Vec2,
        sprite: crate::renderer::TextureId,
    ) -> Self {
        let (pos, size) = sanitize_entity_geometry(pos, size);
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
        let pos = sanitize_vec2(pos, self.pos);
        if crate::game::collision::Aabb::new(pos, self.size).is_some() {
            self.pos = pos;
        }
    }

    /// Sets the position, returning an error (instead of silently keeping the old
    /// position the way [`Entity::set_position`] does) when `pos` is non-finite or
    /// would make the entity's AABB invalid. Prefer this on runtime/data-driven
    /// paths — level loading, scripted teleports — where an invalid position is a
    /// real failure that should be surfaced rather than swallowed.
    #[allow(dead_code)]
    pub fn try_set_position(&mut self, pos: glam::Vec2) -> anyhow::Result<()> {
        if !pos.is_finite() {
            anyhow::bail!("entity position must be finite, got {pos:?}");
        }
        crate::game::collision::Aabb::try_from_pos_size(pos, self.size)
            .map_err(|err| anyhow::anyhow!("invalid entity position {pos:?}: {err}"))?;
        self.pos = pos;
        Ok(())
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
        self.prev_pos.lerp(self.pos, sanitize_alpha(alpha))
    }
}

fn sanitize_alpha(alpha: f32) -> f32 {
    if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn sanitize_vec2(value: glam::Vec2, fallback: glam::Vec2) -> glam::Vec2 {
    if value.is_finite() { value } else { fallback }
}

#[allow(dead_code)]
fn sanitize_size(size: glam::Vec2, fallback: glam::Vec2) -> glam::Vec2 {
    if size.is_finite() && size.x > 0.0 && size.y > 0.0 {
        size
    } else {
        fallback
    }
}

#[allow(dead_code)]
fn sanitize_entity_geometry(pos: glam::Vec2, size: glam::Vec2) -> (glam::Vec2, glam::Vec2) {
    let pos = sanitize_vec2(pos, glam::Vec2::ZERO);
    let size = sanitize_size(size, glam::Vec2::ONE);

    if crate::game::collision::Aabb::new(pos, size).is_some() {
        (pos, size)
    } else {
        (glam::Vec2::ZERO, glam::Vec2::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::Entity;

    #[test]
    fn try_new_rejects_invalid_position_and_size() {
        assert!(
            Entity::try_new(
                glam::vec2(f32::NAN, 1.0),
                glam::Vec2::ONE,
                crate::renderer::TEST_TEXTURE_ID,
            )
            .is_err()
        );
        assert!(
            Entity::try_new(
                glam::Vec2::ZERO,
                glam::vec2(-10.0, f32::INFINITY),
                crate::renderer::TEST_TEXTURE_ID,
            )
            .is_err()
        );
    }

    #[test]
    fn try_new_rejects_aabb_geometry_that_would_collapse() {
        assert!(
            Entity::try_new(
                glam::Vec2::splat(f32::MAX),
                glam::Vec2::ONE,
                crate::renderer::TEST_TEXTURE_ID,
            )
            .is_err()
        );
    }

    #[test]
    fn new_sanitized_repairs_invalid_position_and_size() {
        let entity = Entity::new_sanitized(
            glam::vec2(f32::NAN, 1.0),
            glam::vec2(-10.0, f32::INFINITY),
            crate::renderer::TEST_TEXTURE_ID,
        );

        assert_eq!(entity.position(), glam::Vec2::ZERO);
        assert_eq!(entity.previous_position(), glam::Vec2::ZERO);
        assert_eq!(entity.size(), glam::Vec2::ONE);
    }

    #[test]
    fn new_sanitized_repairs_aabb_geometry_that_would_collapse() {
        let entity = Entity::new_sanitized(
            glam::Vec2::splat(f32::MAX),
            glam::Vec2::ONE,
            crate::renderer::TEST_TEXTURE_ID,
        );

        assert_eq!(entity.position(), glam::Vec2::ZERO);
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
    fn set_position_rejects_position_that_would_invalidate_aabb() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity.set_position(glam::Vec2::splat(f32::MAX));

        assert_eq!(entity.position(), glam::Vec2::ZERO);
    }

    #[test]
    fn try_set_position_rejects_nan() {
        let mut entity = Entity::new_player(glam::vec2(2.0, 3.0));
        assert!(entity.try_set_position(glam::vec2(f32::NAN, 0.0)).is_err());
        assert!(
            entity
                .try_set_position(glam::vec2(0.0, f32::INFINITY))
                .is_err()
        );
        // Position is unchanged after a rejected update.
        assert_eq!(entity.position(), glam::vec2(2.0, 3.0));
    }

    #[test]
    fn try_set_position_rejects_invalid_aabb() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        // A position at f32::MAX would overflow the AABB max corner.
        assert!(
            entity
                .try_set_position(glam::Vec2::splat(f32::MAX))
                .is_err()
        );
        assert_eq!(entity.position(), glam::Vec2::ZERO);
    }

    #[test]
    fn try_set_position_updates_valid_position() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity
            .try_set_position(glam::vec2(10.0, 20.0))
            .expect("valid position must be accepted");
        assert_eq!(entity.position(), glam::vec2(10.0, 20.0));
    }

    #[test]
    fn begin_step_syncs_previous_position() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity.set_position(glam::vec2(10.0, 20.0));
        entity.begin_step();

        assert_eq!(entity.previous_position(), entity.position());
    }

    #[test]
    fn interpolated_pos_clamps_alpha_below_zero() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity.set_position(glam::vec2(10.0, 20.0));

        assert_eq!(entity.interpolated_pos(-1.0), glam::Vec2::ZERO);
    }

    #[test]
    fn interpolated_pos_clamps_alpha_above_one() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity.set_position(glam::vec2(10.0, 20.0));

        assert_eq!(entity.interpolated_pos(2.0), entity.position());
    }

    #[test]
    fn interpolated_pos_treats_nan_as_zero() {
        let mut entity = Entity::new_player(glam::Vec2::ZERO);
        entity.set_position(glam::vec2(10.0, 20.0));

        assert_eq!(entity.interpolated_pos(f32::NAN), glam::Vec2::ZERO);
    }
}
