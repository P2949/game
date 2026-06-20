use std::collections::{HashMap, HashSet};

use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Key {
    A,
    D,
    W,
    S,
    Left,
    Right,
    Up,
    Down,
    Space,
    Enter,
    P,
    R,
    K,
    Plus,
    Minus,
    Escape,
    Q,
    E,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Shift,
    Ctrl,
    Tab,
    Backspace,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

const KEY_COUNT: usize = Key::F12 as usize + 1;

impl Key {
    const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

const MOUSE_BUTTON_COUNT: usize = MouseButton::Forward as usize + 1;

impl MouseButton {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Controller buttons using SDL's gamepad-neutral face-button names.
///
/// The first connected controller is currently exposed as one shared gamepad.
/// Per-player controller assignment can build on this without changing content
/// bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum GamepadButton {
    South,
    East,
    West,
    North,
    LeftShoulder,
    RightShoulder,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

const GAMEPAD_BUTTON_COUNT: usize = GamepadButton::DPadRight as usize + 1;

impl GamepadButton {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Controller stick pairs available to logical 2D-axis bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum GamepadAxis {
    LeftStick,
    RightStick,
}

const GAMEPAD_AXIS_COUNT: usize = GamepadAxis::RightStick as usize + 1;
const GAMEPAD_DEADZONE: f32 = 0.2;

impl GamepadAxis {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Raw state for the first connected controller.
#[derive(Debug, Clone, Copy)]
pub struct GamepadState {
    down: [bool; GAMEPAD_BUTTON_COUNT],
    pressed: [bool; GAMEPAD_BUTTON_COUNT],
    raw_axes: [Vec2; GAMEPAD_AXIS_COUNT],
    axes: [Vec2; GAMEPAD_AXIS_COUNT],
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            down: [false; GAMEPAD_BUTTON_COUNT],
            pressed: [false; GAMEPAD_BUTTON_COUNT],
            raw_axes: [Vec2::ZERO; GAMEPAD_AXIS_COUNT],
            axes: [Vec2::ZERO; GAMEPAD_AXIS_COUNT],
        }
    }
}

impl GamepadState {
    fn begin_frame(&mut self) {
        self.pressed = [false; GAMEPAD_BUTTON_COUNT];
    }

    fn set_button(&mut self, button: GamepadButton, down: bool) {
        let index = button.index();
        if down && !self.down[index] {
            self.pressed[index] = true;
        }
        self.down[index] = down;
    }

    fn set_axis(&mut self, axis: GamepadAxis, value: Vec2) {
        let value = sanitize_axis2(value);
        self.raw_axes[axis.index()] = value;
        self.axes[axis.index()] = if value.length_squared() < GAMEPAD_DEADZONE.powi(2) {
            Vec2::ZERO
        } else {
            value
        };
    }

    fn set_axis_component(&mut self, axis: GamepadAxis, component: usize, value: f32) {
        let mut axis_value = self.raw_axes[axis.index()];
        match component {
            0 => axis_value.x = value,
            1 => axis_value.y = value,
            _ => return,
        }
        self.set_axis(axis, axis_value);
    }

    pub fn down(&self, button: GamepadButton) -> bool {
        self.down[button.index()]
    }

    pub fn pressed(&self, button: GamepadButton) -> bool {
        self.pressed[button.index()]
    }

    pub fn axis(&self, axis: GamepadAxis) -> Vec2 {
        self.axes[axis.index()]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    down: [bool; KEY_COUNT],
    pressed: [bool; KEY_COUNT],
    mouse_down: [bool; MOUSE_BUTTON_COUNT],
    mouse_pressed: [bool; MOUSE_BUTTON_COUNT],
    gamepad: GamepadState,
    mouse_position: Vec2,
    viewport_size: Vec2,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            down: [false; KEY_COUNT],
            pressed: [false; KEY_COUNT],
            mouse_down: [false; MOUSE_BUTTON_COUNT],
            mouse_pressed: [false; MOUSE_BUTTON_COUNT],
            gamepad: GamepadState::default(),
            mouse_position: Vec2::ZERO,
            viewport_size: Vec2::ZERO,
        }
    }
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.pressed = [false; KEY_COUNT];
        self.mouse_pressed = [false; MOUSE_BUTTON_COUNT];
        self.gamepad.begin_frame();
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_key(&mut self, key: Key, down: bool) {
        let index = key.index();
        if down && !self.down[index] {
            self.pressed[index] = true;
        }
        self.down[index] = down;
    }

    pub fn down(&self, key: Key) -> bool {
        self.down[key.index()]
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.pressed[key.index()]
    }

    pub fn set_mouse_button(&mut self, button: MouseButton, down: bool) {
        let index = button.index();
        if down && !self.mouse_down[index] {
            self.mouse_pressed[index] = true;
        }
        self.mouse_down[index] = down;
    }

    pub fn mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_down[button.index()]
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed[button.index()]
    }

    pub fn set_gamepad_button(&mut self, button: GamepadButton, down: bool) {
        self.gamepad.set_button(button, down);
    }

    pub fn gamepad_down(&self, button: GamepadButton) -> bool {
        self.gamepad.down(button)
    }

    pub fn gamepad_pressed(&self, button: GamepadButton) -> bool {
        self.gamepad.pressed(button)
    }

    /// Sets a controller stick in its conventional -1.0..=1.0 range. Values
    /// inside the built-in deadzone are reported as zero.
    pub fn set_gamepad_axis(&mut self, axis: GamepadAxis, value: Vec2) {
        self.gamepad.set_axis(axis, value);
    }

    /// Updates one physical stick component while preserving the other one.
    /// Platform adapters should use this for independent X/Y events.
    pub fn set_gamepad_axis_component(&mut self, axis: GamepadAxis, component: usize, value: f32) {
        self.gamepad.set_axis_component(axis, component, value);
    }

    pub fn gamepad_axis(&self, axis: GamepadAxis) -> Vec2 {
        self.gamepad.axis(axis)
    }

    /// Clears only controller state, retaining keyboard and mouse input.
    pub fn clear_gamepad(&mut self) {
        self.gamepad = GamepadState::default();
    }

    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.mouse_position = sanitize_finite_vec2(position);
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn set_viewport_size(&mut self, size: Vec2) {
        self.viewport_size = Vec2::new(
            if size.x.is_finite() && size.x > 0.0 {
                size.x
            } else {
                0.0
            },
            if size.y.is_finite() && size.y > 0.0 {
                size.y
            } else {
                0.0
            },
        );
    }

    pub fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ActionId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Axis2dId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionBinding {
    pub name: String,
    pub keys: Vec<Key>,
    pub mouse_buttons: Vec<MouseButton>,
    pub gamepad_buttons: Vec<GamepadButton>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axis2dBinding {
    pub name: String,
    pub negative_x: Vec<Key>,
    pub positive_x: Vec<Key>,
    pub negative_y: Vec<Key>,
    pub positive_y: Vec<Key>,
    pub gamepad_axes: Vec<GamepadAxis>,
}

#[derive(Default)]
pub struct InputRegistry {
    actions: Vec<ActionBinding>,
    axes2d: Vec<Axis2dBinding>,
}

impl InputRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Low-level convenience wrapper around [`Self::try_action`] that panics on
    /// duplicate action names. Content should use `game-kit::InputAuthor`,
    /// which returns `Result`.
    pub fn action(&mut self, name: impl Into<String>) -> ActionBindingBuilder<'_> {
        self.try_action(name)
            .expect("input action names must be unique")
    }

    pub fn try_action(
        &mut self,
        name: impl Into<String>,
    ) -> anyhow::Result<ActionBindingBuilder<'_>> {
        let name = name.into();
        if self.actions.iter().any(|binding| binding.name == name) {
            anyhow::bail!(
                "Duplicate input action '{name}'.\n\nEach action name must be registered once. Reuse the returned ActionId instead of calling input.action(\"{name}\") again, or choose a different name."
            );
        }
        let id = ActionId(self.actions.len() as u32);
        self.actions.push(ActionBinding {
            name,
            keys: Vec::new(),
            mouse_buttons: Vec::new(),
            gamepad_buttons: Vec::new(),
        });
        Ok(ActionBindingBuilder { registry: self, id })
    }

    /// Low-level convenience wrapper around [`Self::try_axis2d`] that panics on
    /// duplicate axis names. Content should use `game-kit::InputAuthor`, which
    /// returns `Result`.
    pub fn axis2d(&mut self, name: impl Into<String>) -> Axis2dBindingBuilder<'_> {
        self.try_axis2d(name)
            .expect("input axis2d names must be unique")
    }

    pub fn try_axis2d(
        &mut self,
        name: impl Into<String>,
    ) -> anyhow::Result<Axis2dBindingBuilder<'_>> {
        let name = name.into();
        if self.axes2d.iter().any(|binding| binding.name == name) {
            anyhow::bail!(
                "Duplicate input axis2d '{name}'.\n\nEach axis name must be registered once. Reuse the returned Axis2dId instead of calling input.axis2d(\"{name}\") again, or choose a different name."
            );
        }
        let id = Axis2dId(self.axes2d.len() as u32);
        self.axes2d.push(Axis2dBinding {
            name,
            negative_x: Vec::new(),
            positive_x: Vec::new(),
            negative_y: Vec::new(),
            positive_y: Vec::new(),
            gamepad_axes: Vec::new(),
        });
        Ok(Axis2dBindingBuilder { registry: self, id })
    }

    pub fn action_binding(&self, id: ActionId) -> Option<&ActionBinding> {
        self.actions.get(id.0 as usize)
    }

    pub fn axis2d_binding(&self, id: Axis2dId) -> Option<&Axis2dBinding> {
        self.axes2d.get(id.0 as usize)
    }

    pub fn action_id(&self, name: &str) -> Option<ActionId> {
        self.actions
            .iter()
            .position(|binding| binding.name == name)
            .map(|index| ActionId(index as u32))
    }

    pub fn axis2d_id(&self, name: &str) -> Option<Axis2dId> {
        self.axes2d
            .iter()
            .position(|binding| binding.name == name)
            .map(|index| Axis2dId(index as u32))
    }

    /// Registered action bindings in id order (`ActionId(i)` is `actions()[i]`).
    pub fn actions(&self) -> &[ActionBinding] {
        &self.actions
    }

    /// Registered 2D-axis bindings in id order (`Axis2dId(i)` is `axes2d()[i]`).
    pub fn axes2d(&self) -> &[Axis2dBinding] {
        &self.axes2d
    }
}

pub struct ActionBindingBuilder<'a> {
    registry: &'a mut InputRegistry,
    id: ActionId,
}

impl ActionBindingBuilder<'_> {
    pub fn bind(self, key: Key) -> Self {
        self.registry.actions[self.id.0 as usize].keys.push(key);
        self
    }

    pub fn bind_mouse(self, button: MouseButton) -> Self {
        self.registry.actions[self.id.0 as usize]
            .mouse_buttons
            .push(button);
        self
    }

    pub fn bind_gamepad(self, button: GamepadButton) -> Self {
        self.registry.actions[self.id.0 as usize]
            .gamepad_buttons
            .push(button);
        self
    }

    pub fn id(&self) -> ActionId {
        self.id
    }
}

pub struct Axis2dBindingBuilder<'a> {
    registry: &'a mut InputRegistry,
    id: Axis2dId,
}

impl Axis2dBindingBuilder<'_> {
    pub fn negative_x(self, key: Key) -> Self {
        self.registry.axes2d[self.id.0 as usize]
            .negative_x
            .push(key);
        self
    }

    pub fn positive_x(self, key: Key) -> Self {
        self.registry.axes2d[self.id.0 as usize]
            .positive_x
            .push(key);
        self
    }

    pub fn negative_y(self, key: Key) -> Self {
        self.registry.axes2d[self.id.0 as usize]
            .negative_y
            .push(key);
        self
    }

    pub fn positive_y(self, key: Key) -> Self {
        self.registry.axes2d[self.id.0 as usize]
            .positive_y
            .push(key);
        self
    }

    pub fn bind_gamepad_axis(self, axis: GamepadAxis) -> Self {
        self.registry.axes2d[self.id.0 as usize]
            .gamepad_axes
            .push(axis);
        self
    }

    pub fn id(&self) -> Axis2dId {
        self.id
    }
}

/// Resolved per-step input, keyed by the content-defined [`ActionId`]/[`Axis2dId`]
/// handles minted by an [`InputRegistry`]. The runtime builds this each step from
/// the registry bindings and the raw [`InputState`], so gameplay never refers to
/// physical keys — it asks `input.pressed(actions.attack)` /
/// `input.axis2d(controller.move_axis)`. A second demo can bind entirely
/// different keys to the same logical handles without touching the runtime.
#[derive(Clone, Default)]
pub struct Input {
    pressed: HashSet<ActionId>,
    down: HashSet<ActionId>,
    axes2d: HashMap<Axis2dId, Vec2>,
    mouse_position: Vec2,
    viewport_size: Vec2,
}

impl Input {
    /// True if any key bound to `action` transitioned to pressed this frame
    /// (edge-triggered).
    pub fn pressed(&self, action: ActionId) -> bool {
        self.pressed.contains(&action)
    }

    /// True while any key bound to `action` is held (level-triggered); used for
    /// continuous inputs such as zoom.
    pub fn down(&self, action: ActionId) -> bool {
        self.down.contains(&action)
    }

    /// Sanitized value of a 2D axis (clamped to the unit disc), or zero if the
    /// axis is unbound.
    pub fn axis2d(&self, axis: Axis2dId) -> Vec2 {
        self.axes2d.get(&axis).copied().unwrap_or(Vec2::ZERO)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }

    /// Test/builder helper: marks `action` as both pressed and held.
    pub fn with_pressed(mut self, action: ActionId) -> Self {
        self.pressed.insert(action);
        self.down.insert(action);
        self
    }

    /// Test/builder helper: marks `action` as held (not edge-pressed).
    pub fn with_down(mut self, action: ActionId) -> Self {
        self.down.insert(action);
        self
    }

    /// Test/builder helper: sets a 2D axis value (sanitized to the unit disc).
    pub fn with_axis2d(mut self, axis: Axis2dId, value: Vec2) -> Self {
        self.axes2d.insert(axis, sanitize_axis2(value));
        self
    }

    pub fn with_mouse_position(mut self, position: Vec2, viewport_size: Vec2) -> Self {
        self.mouse_position = sanitize_finite_vec2(position);
        self.viewport_size = sanitize_viewport_size(viewport_size);
        self
    }

    /// Builds the continuous part of the input (held actions and axes) by
    /// evaluating every registered binding against the raw key state. Edge-pressed
    /// actions are added separately via [`Self::set_pressed`] so the runtime can
    /// deliver a frame's key presses to exactly one fixed step.
    pub fn evaluate_continuous(registry: &InputRegistry, state: &InputState) -> Self {
        let mut down = HashSet::new();
        for (index, binding) in registry.actions().iter().enumerate() {
            if binding.keys.iter().any(|key| state.down(*key))
                || binding
                    .mouse_buttons
                    .iter()
                    .any(|button| state.mouse_down(*button))
                || binding
                    .gamepad_buttons
                    .iter()
                    .any(|button| state.gamepad_down(*button))
            {
                down.insert(ActionId(index as u32));
            }
        }

        let mut axes2d = HashMap::new();
        for (index, binding) in registry.axes2d().iter().enumerate() {
            axes2d.insert(Axis2dId(index as u32), evaluate_axis2d(binding, state));
        }

        Self {
            pressed: HashSet::new(),
            down,
            axes2d,
            mouse_position: state.mouse_position(),
            viewport_size: state.viewport_size(),
        }
    }

    /// Set of actions newly pressed this frame, for the runtime's per-frame edge
    /// accumulation.
    pub fn pressed_this_frame(registry: &InputRegistry, state: &InputState) -> HashSet<ActionId> {
        let mut pressed = HashSet::new();
        for (index, binding) in registry.actions().iter().enumerate() {
            if binding.keys.iter().any(|key| state.pressed(*key))
                || binding
                    .mouse_buttons
                    .iter()
                    .any(|button| state.mouse_pressed(*button))
                || binding
                    .gamepad_buttons
                    .iter()
                    .any(|button| state.gamepad_pressed(*button))
            {
                pressed.insert(ActionId(index as u32));
            }
        }
        pressed
    }

    /// Overwrites the edge-pressed action set (used by the runtime to deliver
    /// accumulated presses to the first fixed step of a frame).
    pub fn set_pressed(&mut self, pressed: HashSet<ActionId>) {
        self.pressed = pressed;
    }
}

fn evaluate_axis2d(binding: &Axis2dBinding, state: &InputState) -> Vec2 {
    let axis = |negative: &[Key], positive: &[Key]| -> f32 {
        let neg = negative.iter().any(|key| state.down(*key));
        let pos = positive.iter().any(|key| state.down(*key));
        f32::from(pos) - f32::from(neg)
    };
    let keyboard = Vec2::new(
        axis(&binding.negative_x, &binding.positive_x),
        axis(&binding.negative_y, &binding.positive_y),
    );
    let gamepad = binding
        .gamepad_axes
        .iter()
        .fold(Vec2::ZERO, |value, axis| value + state.gamepad_axis(*axis));
    sanitize_axis2(keyboard + gamepad)
}

pub fn sanitize_axis(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn sanitize_axis2(value: Vec2) -> Vec2 {
    let v = Vec2::new(sanitize_axis(value.x), sanitize_axis(value.y));
    if v.length_squared() > 1.0 {
        v.normalize()
    } else {
        v
    }
}

fn sanitize_finite_vec2(value: Vec2) -> Vec2 {
    Vec2::new(
        if value.x.is_finite() { value.x } else { 0.0 },
        if value.y.is_finite() { value.y } else { 0.0 },
    )
}

fn sanitize_viewport_size(value: Vec2) -> Vec2 {
    Vec2::new(
        if value.x.is_finite() && value.x > 0.0 {
            value.x
        } else {
            0.0
        },
        if value.y.is_finite() && value.y > 0.0 {
            value.y
        } else {
            0.0
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{GamepadAxis, GamepadButton, Input, InputRegistry, InputState, Key, MouseButton};

    fn registry() -> (InputRegistry, super::ActionId, super::Axis2dId) {
        let mut registry = InputRegistry::new();
        let attack = registry
            .action("attack")
            .bind(Key::Space)
            .bind(Key::Enter)
            .bind_mouse(MouseButton::Left)
            .bind_gamepad(GamepadButton::South)
            .id();
        let movement = registry
            .axis2d("move")
            .negative_x(Key::A)
            .positive_x(Key::D)
            .bind_gamepad_axis(GamepadAxis::LeftStick)
            .id();
        (registry, attack, movement)
    }

    #[test]
    fn evaluates_actions_and_axes_from_registry_bindings() {
        let (registry, attack, movement) = registry();
        let mut state = InputState::default();
        state.begin_frame();
        state.set_key(Key::Enter, true); // edge press on a bound key
        state.set_key(Key::D, true); // held: positive x
        state.set_mouse_position(glam::vec2(100.0, 50.0));
        state.set_viewport_size(glam::vec2(800.0, 600.0));

        let mut input = Input::evaluate_continuous(&registry, &state);
        input.set_pressed(Input::pressed_this_frame(&registry, &state));

        assert!(input.pressed(attack));
        assert!(input.down(attack));
        assert_eq!(input.axis2d(movement), glam::vec2(1.0, 0.0));
        assert_eq!(input.mouse_position(), glam::vec2(100.0, 50.0));
        assert_eq!(input.viewport_size(), glam::vec2(800.0, 600.0));
    }

    #[test]
    fn mouse_button_bindings_drive_actions() {
        let (registry, attack, _movement) = registry();
        let mut state = InputState::default();
        state.begin_frame();
        state.set_mouse_button(MouseButton::Left, true);

        let mut input = Input::evaluate_continuous(&registry, &state);
        input.set_pressed(Input::pressed_this_frame(&registry, &state));

        assert!(input.pressed(attack));
        assert!(input.down(attack));
    }

    #[test]
    fn gamepad_button_bindings_drive_actions() {
        let (registry, attack, _movement) = registry();
        let mut state = InputState::default();
        state.begin_frame();
        state.set_gamepad_button(GamepadButton::South, true);

        let mut input = Input::evaluate_continuous(&registry, &state);
        input.set_pressed(Input::pressed_this_frame(&registry, &state));

        assert!(input.pressed(attack));
        assert!(input.down(attack));
    }

    #[test]
    fn gamepad_stick_combines_with_keyboard_and_clamps_to_unit_disc() {
        let (registry, _attack, movement) = registry();
        let mut state = InputState::default();
        state.set_key(Key::D, true);
        state.set_gamepad_axis(GamepadAxis::LeftStick, glam::vec2(0.5, -0.5));

        let input = Input::evaluate_continuous(&registry, &state);
        assert_eq!(input.axis2d(movement), glam::vec2(1.0, -0.5).normalize());
    }

    #[test]
    fn gamepad_deadzone_zeros_small_stick_values() {
        let mut state = InputState::default();
        state.set_gamepad_axis(GamepadAxis::LeftStick, glam::vec2(0.1, 0.1));

        assert_eq!(state.gamepad_axis(GamepadAxis::LeftStick), glam::Vec2::ZERO);
    }

    #[test]
    fn held_key_is_down_but_not_edge_pressed() {
        let (registry, attack, _movement) = registry();
        let mut state = InputState::default();
        state.set_key(Key::Space, true); // pressed on a previous frame
        state.begin_frame(); // new frame: still down, no fresh edge

        let mut input = Input::evaluate_continuous(&registry, &state);
        input.set_pressed(Input::pressed_this_frame(&registry, &state));

        assert!(input.down(attack));
        assert!(!input.pressed(attack));
    }

    #[test]
    fn unbound_axis_reads_zero() {
        let (registry, _attack, _movement) = registry();
        let input = Input::evaluate_continuous(&registry, &InputState::default());
        assert_eq!(input.axis2d(super::Axis2dId(7)), glam::Vec2::ZERO);
    }

    #[test]
    fn registry_rejects_duplicate_binding_names() {
        let mut registry = InputRegistry::new();
        registry.action("attack");
        let err = registry
            .try_action("attack")
            .err()
            .expect("duplicate action should be rejected");
        assert!(err.to_string().contains("Duplicate input action"));

        registry.axis2d("move");
        let err = registry
            .try_axis2d("move")
            .err()
            .expect("duplicate axis should be rejected");
        assert!(err.to_string().contains("Duplicate input axis2d"));
    }
}
