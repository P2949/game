//! Input authoring (Phase 3).
//!
//! [`InputAuthor`] names logical controls (actions and 2D axes) and binds them to
//! keys and controller inputs, without exposing the engine's `InputRegistry`. Reached through
//! [`GameApp::input`].

use anyhow::Result;
use game_core::input::{
    ActionBindingBuilder, ActionId, Axis2dBindingBuilder, Axis2dId, GamepadAxis, GamepadButton,
    InputRegistry, Key, MouseButton,
};

#[derive(Clone, Copy, Debug)]
pub struct TopDownControls {
    pub movement: Axis2dId,
    pub attack: ActionId,
    pub pause: ActionId,
    pub reset: ActionId,
    pub debug_die: ActionId,
    pub debug_overlay: ActionId,
    pub zoom_in: ActionId,
    pub zoom_out: ActionId,
}

/// Declares the logical controls a game uses.
pub struct InputAuthor<'a> {
    registry: &'a mut InputRegistry,
}

impl<'a> InputAuthor<'a> {
    pub(crate) fn new(registry: &'a mut InputRegistry) -> Self {
        Self { registry }
    }

    /// Begins declaring a single action (e.g. `"attack"`), then bind keys with
    /// [`ActionAuthor::key`]/[`ActionAuthor::keys`].
    pub fn action(&mut self, name: impl Into<String>) -> Result<ActionAuthor<'_>> {
        Ok(ActionAuthor {
            builder: self.registry.try_action(name)?,
        })
    }

    /// Begins declaring a 2D movement axis (e.g. `"move"`), then bind directions
    /// with [`Axis2dAuthor::wasd`]/[`Axis2dAuthor::arrows`]/[`Axis2dAuthor::keys`].
    pub fn axis2d(&mut self, name: impl Into<String>) -> Result<Axis2dAuthor<'_>> {
        Ok(Axis2dAuthor {
            builder: self.registry.try_axis2d(name)?,
        })
    }

    pub fn top_down_controls(&mut self) -> Result<TopDownControls> {
        self.top_down_controls_with_gamepad()
    }

    /// Declares the standard top-down controls with keyboard, mouse, and first
    /// controller support. This is also what [`Self::top_down_controls`] uses.
    pub fn top_down_controls_with_gamepad(&mut self) -> Result<TopDownControls> {
        Ok(TopDownControls {
            movement: self.axis2d("move")?.wasd().arrows_and_gamepad_left_stick(),
            attack: self
                .action("attack")?
                .space_or_enter_or_mouse_left_or_gamepad_south(),
            pause: self.action("pause")?.escape_or_p_or_gamepad_start(),
            reset: self
                .action("reset")?
                .key_or_gamepad(Key::R, GamepadButton::Select),
            debug_die: self.action("debug_die")?.key(Key::K),
            debug_overlay: self.action("debug_overlay")?.key(Key::F1),
            zoom_in: self.action("zoom_in")?.key(Key::Plus),
            zoom_out: self.action("zoom_out")?.key(Key::Minus),
        })
    }

    /// Declares controller-only top-down controls using the first connected
    /// gamepad. Use [`Self::top_down_controls_with_gamepad`] for the usual
    /// keyboard-and-controller configuration.
    pub fn top_down_gamepad_controls(&mut self) -> Result<TopDownControls> {
        Ok(TopDownControls {
            movement: self.axis2d("move")?.gamepad_left_stick(),
            attack: self.action("attack")?.gamepad_south(),
            pause: self.action("pause")?.gamepad_start(),
            reset: self.action("reset")?.gamepad_select(),
            debug_die: self.action("debug_die")?.gamepad_west(),
            debug_overlay: self.action("debug_overlay")?.gamepad_north(),
            zoom_in: self.action("zoom_in")?.gamepad_right_shoulder(),
            zoom_out: self.action("zoom_out")?.gamepad_left_shoulder(),
        })
    }
}

/// Binds keys to one logical action.
pub struct ActionAuthor<'a> {
    builder: ActionBindingBuilder<'a>,
}

impl ActionAuthor<'_> {
    /// Binds a single key and finalizes the action.
    pub fn key(self, key: Key) -> ActionId {
        self.builder.bind(key).id()
    }

    /// Binds several keys to the action and finalizes it.
    pub fn keys<const N: usize>(self, keys: [Key; N]) -> ActionId {
        let mut builder = self.builder;
        for key in keys {
            builder = builder.bind(key);
        }
        builder.id()
    }

    pub fn space(self) -> ActionId {
        self.key(Key::Space)
    }

    pub fn enter(self) -> ActionId {
        self.key(Key::Enter)
    }

    pub fn escape(self) -> ActionId {
        self.key(Key::Escape)
    }

    pub fn space_or_enter(self) -> ActionId {
        self.keys([Key::Space, Key::Enter])
    }

    pub fn escape_or_p(self) -> ActionId {
        self.keys([Key::Escape, Key::P])
    }

    pub fn mouse(self, button: MouseButton) -> ActionId {
        self.builder.bind_mouse(button).id()
    }

    pub fn mouse_left(self) -> ActionId {
        self.mouse(MouseButton::Left)
    }

    pub fn mouse_right(self) -> ActionId {
        self.mouse(MouseButton::Right)
    }

    pub fn mouse_middle(self) -> ActionId {
        self.mouse(MouseButton::Middle)
    }

    /// Binds one controller button and finalizes the action.
    pub fn gamepad(self, button: GamepadButton) -> ActionId {
        self.builder.bind_gamepad(button).id()
    }

    pub fn gamepad_south(self) -> ActionId {
        self.gamepad(GamepadButton::South)
    }

    pub fn gamepad_east(self) -> ActionId {
        self.gamepad(GamepadButton::East)
    }

    pub fn gamepad_west(self) -> ActionId {
        self.gamepad(GamepadButton::West)
    }

    pub fn gamepad_north(self) -> ActionId {
        self.gamepad(GamepadButton::North)
    }

    pub fn gamepad_start(self) -> ActionId {
        self.gamepad(GamepadButton::Start)
    }

    pub fn gamepad_select(self) -> ActionId {
        self.gamepad(GamepadButton::Select)
    }

    pub fn gamepad_left_shoulder(self) -> ActionId {
        self.gamepad(GamepadButton::LeftShoulder)
    }

    pub fn gamepad_right_shoulder(self) -> ActionId {
        self.gamepad(GamepadButton::RightShoulder)
    }

    pub fn key_or_gamepad(self, key: Key, button: GamepadButton) -> ActionId {
        self.builder.bind(key).bind_gamepad(button).id()
    }

    fn space_or_enter_or_mouse_left_or_gamepad_south(self) -> ActionId {
        self.builder
            .bind(Key::Space)
            .bind(Key::Enter)
            .bind_mouse(MouseButton::Left)
            .bind_gamepad(GamepadButton::South)
            .id()
    }

    fn escape_or_p_or_gamepad_start(self) -> ActionId {
        self.builder
            .bind(Key::Escape)
            .bind(Key::P)
            .bind_gamepad(GamepadButton::Start)
            .id()
    }
}

/// Binds directional keys to one logical 2D axis.
pub struct Axis2dAuthor<'a> {
    builder: Axis2dBindingBuilder<'a>,
}

impl<'a> Axis2dAuthor<'a> {
    /// Adds W/A/S/D directional bindings (returns `self` so arrows can be added
    /// too: `axis.wasd().arrows()`).
    pub fn wasd(self) -> Self {
        Self {
            builder: self
                .builder
                .negative_x(Key::A)
                .positive_x(Key::D)
                .negative_y(Key::W)
                .positive_y(Key::S),
        }
    }

    /// Adds arrow-key directional bindings and finalizes the axis.
    pub fn arrows(self) -> Axis2dId {
        self.builder
            .negative_x(Key::Left)
            .positive_x(Key::Right)
            .negative_y(Key::Up)
            .positive_y(Key::Down)
            .id()
    }

    /// Binds the first controller's left stick and finalizes the axis.
    pub fn gamepad_left_stick(self) -> Axis2dId {
        self.builder.bind_gamepad_axis(GamepadAxis::LeftStick).id()
    }

    /// Binds the first controller's right stick and finalizes the axis.
    pub fn gamepad_right_stick(self) -> Axis2dId {
        self.builder.bind_gamepad_axis(GamepadAxis::RightStick).id()
    }

    fn arrows_and_gamepad_left_stick(self) -> Axis2dId {
        self.builder
            .negative_x(Key::Left)
            .positive_x(Key::Right)
            .negative_y(Key::Up)
            .positive_y(Key::Down)
            .bind_gamepad_axis(GamepadAxis::LeftStick)
            .id()
    }

    /// Adds explicit directional bindings and finalizes the axis.
    pub fn keys(self, left: Key, right: Key, up: Key, down: Key) -> Axis2dId {
        self.builder
            .negative_x(left)
            .positive_x(right)
            .negative_y(up)
            .positive_y(down)
            .id()
    }
}

#[cfg(test)]
mod tests {
    use game_core::input::{GamepadAxis, GamepadButton, InputRegistry, Key, MouseButton};

    use super::InputAuthor;

    #[test]
    fn action_aliases_bind_expected_inputs() {
        let mut registry = InputRegistry::new();
        let mut input = InputAuthor::new(&mut registry);

        let attack = input.action("attack").unwrap().space_or_enter();
        let pause = input.action("pause").unwrap().escape_or_p();
        let shoot = input.action("shoot").unwrap().mouse_left();

        assert_eq!(
            registry.action_binding(attack).unwrap().keys,
            [Key::Space, Key::Enter]
        );
        assert_eq!(
            registry.action_binding(pause).unwrap().keys,
            [Key::Escape, Key::P]
        );
        assert_eq!(
            registry.action_binding(shoot).unwrap().mouse_buttons,
            [MouseButton::Left]
        );
    }

    #[test]
    fn top_down_controls_register_standard_names() {
        let mut registry = InputRegistry::new();
        let mut input = InputAuthor::new(&mut registry);

        let controls = input.top_down_controls().unwrap();

        assert_eq!(registry.action_id("attack"), Some(controls.attack));
        assert_eq!(registry.axis2d_id("move"), Some(controls.movement));
        assert_eq!(
            registry.action_binding(controls.pause).unwrap().keys,
            [Key::Escape, Key::P]
        );
        assert_eq!(
            registry
                .action_binding(controls.attack)
                .unwrap()
                .gamepad_buttons,
            [GamepadButton::South]
        );
        assert_eq!(
            registry
                .axis2d_binding(controls.movement)
                .unwrap()
                .gamepad_axes,
            [GamepadAxis::LeftStick]
        );
    }

    #[test]
    fn gamepad_authors_bind_named_controls() {
        let mut registry = InputRegistry::new();
        let mut input = InputAuthor::new(&mut registry);

        let attack = input.action("attack").unwrap().gamepad_south();
        let movement = input.axis2d("move").unwrap().gamepad_left_stick();

        assert_eq!(
            registry.action_binding(attack).unwrap().gamepad_buttons,
            [GamepadButton::South]
        );
        assert_eq!(
            registry.axis2d_binding(movement).unwrap().gamepad_axes,
            [GamepadAxis::LeftStick]
        );
    }
}
