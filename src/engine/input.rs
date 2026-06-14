use glam::Vec2;

use crate::platform::input::{FrameActions, InputState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Attack,
    Pause,
    Reset,
    DebugDie,
}

pub struct Input {
    move_axis: Vec2,
    zoom_axis: f32,
    actions: FrameActions,
}

impl Input {
    pub fn new(state: &InputState, actions: FrameActions) -> Self {
        let zoom_axis = match (state.zoom_out, state.zoom_in) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        Self {
            move_axis: state.movement(),
            zoom_axis,
            actions,
        }
    }

    pub fn move_axis(&self) -> Vec2 {
        self.move_axis
    }

    pub fn zoom_axis(&self) -> f32 {
        self.zoom_axis
    }

    pub fn pressed(&self, action: Action) -> bool {
        match action {
            Action::Attack => self.actions.action_pressed,
            Action::Pause => self.actions.pause_pressed,
            Action::Reset => self.actions.reset_pressed,
            Action::DebugDie => self.actions.debug_die_pressed,
        }
    }
}
