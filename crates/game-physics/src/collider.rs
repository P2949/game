#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub half_extents: glam::Vec2,
}

impl Collider {
    pub fn box_of(size: glam::Vec2) -> Self {
        Self {
            half_extents: size * 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Trigger;

#[derive(Clone, Copy, Debug, Default)]
pub struct Solid;
