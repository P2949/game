use game_core::input::{ActionId, Axis2dId, InputRegistry, Key};

#[derive(Clone, Copy, Debug)]
pub struct TestbedActions {
    pub attack: ActionId,
    pub pause: ActionId,
    pub reset: ActionId,
    pub debug_die: ActionId,
    pub movement: Axis2dId,
}

pub fn register(input: &mut InputRegistry) -> TestbedActions {
    let attack = input
        .action("attack")
        .bind(Key::Space)
        .bind(Key::Enter)
        .id();
    let pause = input.action("pause").bind(Key::P).id();
    let reset = input.action("reset").bind(Key::R).id();
    let debug_die = input.action("debug_die").bind(Key::K).id();
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

    TestbedActions {
        attack,
        pause,
        reset,
        debug_die,
        movement,
    }
}

#[cfg(test)]
mod tests {
    use super::register;
    use game_core::input::{InputRegistry, Key};

    #[test]
    fn testbed_input_registers_bindings() {
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
                .positive_x,
            vec![Key::D, Key::Right]
        );
    }
}
