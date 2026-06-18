//! Standard beginner actor components.

use game_core::input::Axis2dId;

use crate::helpers::{InputDriven, MovementSpeed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name(pub String);

impl Name {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Player;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Enemy;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Npc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pickup;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Door;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Projectile;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Solid;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Speed(pub f32);

impl Speed {
    pub fn new(units_per_second: f32) -> Self {
        Self(units_per_second)
    }
}

impl MovementSpeed for Speed {
    fn units_per_second(&self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerMovement {
    pub axis: Axis2dId,
}

impl PlayerMovement {
    pub fn axis(axis: Axis2dId) -> Self {
        Self { axis }
    }
}

impl InputDriven for PlayerMovement {
    fn movement_axis(&self) -> Axis2dId {
        self.axis
    }
}
