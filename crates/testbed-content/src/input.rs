use game_kit::prelude::*;

pub type TestbedActions = TopDownControls;

pub fn register(input: &mut InputAuthor<'_>) -> Result<TestbedActions> {
    input.top_down_controls()
}
