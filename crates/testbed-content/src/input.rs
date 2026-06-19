use game_kit::advanced::prelude::*;

pub type TestbedActions = TopDownControls;

pub fn register(input: &mut InputAuthor<'_>) -> Result<TestbedActions> {
    input.top_down_controls()
}
