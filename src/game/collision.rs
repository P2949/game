use crate::game::world::Entity;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: glam::Vec2,
    pub max: glam::Vec2,
}

impl Aabb {
    pub fn from_pos_size(pos: glam::Vec2, size: glam::Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

pub fn move_with_collision(entity: &mut Entity, solids: &[Aabb], dt: f32) {
    entity.prev_pos = entity.pos;

    entity.pos.x += entity.vel.x * dt;
    for solid in solids {
        let aabb = Aabb::from_pos_size(entity.pos, entity.size);
        if aabb.overlaps(*solid) {
            if entity.vel.x > 0.0 {
                entity.pos.x = solid.min.x - entity.size.x;
            } else if entity.vel.x < 0.0 {
                entity.pos.x = solid.max.x;
            }
            entity.vel.x = 0.0;
        }
    }

    entity.pos.y += entity.vel.y * dt;
    for solid in solids {
        let aabb = Aabb::from_pos_size(entity.pos, entity.size);
        if aabb.overlaps(*solid) {
            if entity.vel.y > 0.0 {
                entity.pos.y = solid.min.y - entity.size.y;
            } else if entity.vel.y < 0.0 {
                entity.pos.y = solid.max.y;
            }
            entity.vel.y = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Aabb;

    #[test]
    fn overlapping_aabbs_intersect() {
        let a = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(10.0, 10.0));
        let b = Aabb::from_pos_size(glam::vec2(5.0, 5.0), glam::vec2(10.0, 10.0));

        assert!(a.overlaps(b));
    }

    #[test]
    fn separated_aabbs_do_not_intersect() {
        let a = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(10.0, 10.0));
        let b = Aabb::from_pos_size(glam::vec2(10.0, 0.0), glam::vec2(10.0, 10.0));

        assert!(!a.overlaps(b));
    }
}
