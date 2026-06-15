use game_kit::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameState {
    pub paused: bool,
    pub player_dead: bool,
}

impl SimulationState for GameState {
    fn paused(&self) -> bool {
        self.paused
    }

    fn dead(&self) -> bool {
        self.player_dead
    }
}
