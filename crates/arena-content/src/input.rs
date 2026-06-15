use crate::engine::input::{ActionId, Axis2dId, InputRegistry, Key};

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

pub fn register(input: &mut InputRegistry) -> ArenaActions {
    let attack = input
        .action("attack")
        .bind(Key::Space)
        .bind(Key::Enter)
        .id();
    let pause = input.action("pause").bind(Key::P).id();
    let reset = input.action("reset").bind(Key::R).id();
    let debug_die = input.action("debug_die").bind(Key::K).id();
    let zoom_in = input.action("zoom_in").bind(Key::Plus).id();
    let zoom_out = input.action("zoom_out").bind(Key::Minus).id();
    let movement = input
        .axis2d("move")
        .negative_x(Key::A)
        .positive_x(Key::D)
        .negative_y(Key::W)
        .positive_y(Key::S)
        .negative_x(Key::Left)
        .positive_x(Key::Right)
        .negative_y(Key::Up)
        .positive_y(Key::Down)
        .id();

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

#[cfg(test)]
mod tests {
    use super::register;
    use crate::engine::input::{InputRegistry, Key};

    #[test]
    fn arena_input_registers_current_bindings() {
        let mut registry = InputRegistry::new();
        let actions = register(&mut registry);

        assert_eq!(
            registry.action_binding(actions.attack).unwrap().keys,
            vec![Key::Space, Key::Enter]
        );
        assert_eq!(
            registry
                .axis2d_binding(actions.movement)
                .unwrap()
                .negative_x,
            vec![Key::A, Key::Left]
        );
    }
}
