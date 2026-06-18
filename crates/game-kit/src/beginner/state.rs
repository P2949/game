//! Built-in beginner game state.

use crate::helpers::SimulationState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimpleGameState {
    pub paused: bool,
    pub player_dead: bool,
}

impl SimulationState for SimpleGameState {
    fn paused(&self) -> bool {
        self.paused
    }

    fn dead(&self) -> bool {
        self.player_dead
    }
}
