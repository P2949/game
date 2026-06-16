use game_kit::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name(pub String);

impl Name {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerController {
    pub move_axis: Axis2dId,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyTag;

#[derive(Clone, Copy, Debug)]
pub struct MoveSpeed(pub f32);

impl InputDriven for PlayerController {
    fn movement_axis(&self) -> Axis2dId {
        self.move_axis
    }
}

impl MovementSpeed for MoveSpeed {
    fn units_per_second(&self) -> f32 {
        self.0
    }
}
