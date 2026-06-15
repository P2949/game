use game_kit::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ArenaActions {
    pub attack: ActionId,
    pub pause: ActionId,
    pub reset: ActionId,
    pub debug_die: ActionId,
    pub zoom_in: ActionId,
    pub zoom_out: ActionId,
    pub movement: Axis2dId,
}

pub fn register(input: &mut InputAuthor<'_>) -> ArenaActions {
    let attack = input.action("attack").keys([Key::Space, Key::Enter]);
    let pause = input.action("pause").key(Key::P);
    let reset = input.action("reset").key(Key::R);
    let debug_die = input.action("debug_die").key(Key::K);
    let zoom_in = input.action("zoom_in").key(Key::Plus);
    let zoom_out = input.action("zoom_out").key(Key::Minus);
    let movement = input.axis2d("move").wasd().arrows();

    ArenaActions {
        attack,
        pause,
        reset,
        debug_die,
        zoom_in,
        zoom_out,
        movement,
    }
}
