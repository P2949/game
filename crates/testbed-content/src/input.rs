use game_kit::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct TestbedActions {
    pub attack: ActionId,
    pub pause: ActionId,
    pub reset: ActionId,
    pub debug_die: ActionId,
    pub zoom_in: ActionId,
    pub zoom_out: ActionId,
    pub movement: Axis2dId,
}

pub fn register(input: &mut InputAuthor<'_>) -> Result<TestbedActions> {
    Ok(TestbedActions {
        attack: input.action("attack")?.keys([Key::Space, Key::Enter]),
        pause: input.action("pause")?.key(Key::P),
        reset: input.action("reset")?.key(Key::R),
        debug_die: input.action("debug_die")?.key(Key::K),
        zoom_in: input.action("zoom_in")?.key(Key::Plus),
        zoom_out: input.action("zoom_out")?.key(Key::Minus),
        movement: input.axis2d("move")?.wasd().arrows(),
    })
}
