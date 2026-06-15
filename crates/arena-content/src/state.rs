#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameState {
    pub paused: bool,
    pub player_dead: bool,
}
