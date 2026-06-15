use game_core::input::Axis2dId;

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
pub struct MoveSpeed(pub f32);
