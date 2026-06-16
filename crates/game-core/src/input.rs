use std::collections::{HashMap, HashSet};

use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

const KEY_COUNT: usize = 16;

impl Key {
    const fn index(self) -> usize {
        match self {
            Key::A => 0,
            Key::D => 1,
            Key::W => 2,
            Key::S => 3,
            Key::Left => 4,
            Key::Right => 5,
            Key::Up => 6,
            Key::Down => 7,
            Key::Space => 8,
            Key::Enter => 9,
            Key::P => 10,
            Key::R => 11,
            Key::K => 12,
            Key::Plus => 13,
            Key::Minus => 14,
            Key::Escape => 15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    down: [bool; KEY_COUNT],
    pressed: [bool; KEY_COUNT],
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            down: [false; KEY_COUNT],
            pressed: [false; KEY_COUNT],
        }
    }
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.pressed = [false; KEY_COUNT];
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ActionId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Axis2dId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionBinding {
    pub name: String,
    pub keys: Vec<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axis2dBinding {
    pub name: String,
    pub negative_x: Vec<Key>,
    pub positive_x: Vec<Key>,
    pub negative_y: Vec<Key>,
    pub positive_y: Vec<Key>,
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
            anyhow::bail!("duplicate input action '{name}'");
        }
        let id = ActionId(self.actions.len() as u32);
        self.actions.push(ActionBinding {
            name,
            keys: Vec::new(),
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
            anyhow::bail!("duplicate input axis2d '{name}'");
        }
        let id = Axis2dId(self.axes2d.len() as u32);
        self.axes2d.push(Axis2dBinding {
            name,
            negative_x: Vec::new(),
            positive_x: Vec::new(),
            negative_y: Vec::new(),
            positive_y: Vec::new(),
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

    /// Builds the continuous part of the input (held actions and axes) by
    /// evaluating every registered binding against the raw key state. Edge-pressed
    /// actions are added separately via [`Self::set_pressed`] so the runtime can
    /// deliver a frame's key presses to exactly one fixed step.
    pub fn evaluate_continuous(registry: &InputRegistry, state: &InputState) -> Self {
        let mut down = HashSet::new();
        for (index, binding) in registry.actions().iter().enumerate() {
            if binding.keys.iter().any(|key| state.down(*key)) {
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
        }
    }

    /// Set of actions newly pressed this frame, for the runtime's per-frame edge
    /// accumulation.
    pub fn pressed_this_frame(registry: &InputRegistry, state: &InputState) -> HashSet<ActionId> {
        let mut pressed = HashSet::new();
        for (index, binding) in registry.actions().iter().enumerate() {
            if binding.keys.iter().any(|key| state.pressed(*key)) {
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
    sanitize_axis2(Vec2::new(
        axis(&binding.negative_x, &binding.positive_x),
        axis(&binding.negative_y, &binding.positive_y),
    ))
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

#[cfg(test)]
mod tests {
    use super::{Input, InputRegistry, InputState, Key};

    fn registry() -> (InputRegistry, super::ActionId, super::Axis2dId) {
        let mut registry = InputRegistry::new();
        let attack = registry
            .action("attack")
            .bind(Key::Space)
            .bind(Key::Enter)
            .id();
        let movement = registry
            .axis2d("move")
            .negative_x(Key::A)
            .positive_x(Key::D)
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

        let mut input = Input::evaluate_continuous(&registry, &state);
        input.set_pressed(Input::pressed_this_frame(&registry, &state));

        assert!(input.pressed(attack));
        assert!(input.down(attack));
        assert_eq!(input.axis2d(movement), glam::vec2(1.0, 0.0));
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
        assert!(err.to_string().contains("duplicate input action"));

        registry.axis2d("move");
        let err = registry
            .try_axis2d("move")
            .err()
            .expect("duplicate axis should be rejected");
        assert!(err.to_string().contains("duplicate input axis2d"));
    }
}
