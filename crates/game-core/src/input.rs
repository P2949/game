use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Attack,
    Pause,
    Reset,
    DebugDie,
}

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

#[derive(Debug, Default, Clone, Copy)]
pub struct FrameActions {
    pub action_pressed: bool,
    pub pause_pressed: bool,
    pub reset_pressed: bool,
    pub debug_die_pressed: bool,
}

impl FrameActions {
    pub fn merge(&mut self, other: Self) {
        self.action_pressed |= other.action_pressed;
        self.pause_pressed |= other.pause_pressed;
        self.reset_pressed |= other.reset_pressed;
        self.debug_die_pressed |= other.debug_die_pressed;
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

    pub fn action(&mut self, name: impl Into<String>) -> ActionBindingBuilder<'_> {
        let id = ActionId(self.actions.len() as u32);
        self.actions.push(ActionBinding {
            name: name.into(),
            keys: Vec::new(),
        });
        ActionBindingBuilder { registry: self, id }
    }

    pub fn axis2d(&mut self, name: impl Into<String>) -> Axis2dBindingBuilder<'_> {
        let id = Axis2dId(self.axes2d.len() as u32);
        self.axes2d.push(Axis2dBinding {
            name: name.into(),
            negative_x: Vec::new(),
            positive_x: Vec::new(),
            negative_y: Vec::new(),
            positive_y: Vec::new(),
        });
        Axis2dBindingBuilder { registry: self, id }
    }

    pub fn action_binding(&self, id: ActionId) -> Option<&ActionBinding> {
        self.actions.get(id.0 as usize)
    }

    pub fn axis2d_binding(&self, id: Axis2dId) -> Option<&Axis2dBinding> {
        self.axes2d.get(id.0 as usize)
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

pub struct Input {
    move_axis: Vec2,
    zoom_axis: f32,
    actions: FrameActions,
}

impl Input {
    pub fn new(move_axis: Vec2, zoom_axis: f32, actions: FrameActions) -> Self {
        Self {
            move_axis: sanitize_axis2(move_axis),
            zoom_axis: sanitize_axis(zoom_axis),
            actions,
        }
    }

    pub fn move_axis(&self) -> Vec2 {
        self.move_axis
    }

    pub fn zoom_axis(&self) -> f32 {
        self.zoom_axis
    }

    pub fn pressed(&self, action: Action) -> bool {
        match action {
            Action::Attack => self.actions.action_pressed,
            Action::Pause => self.actions.pause_pressed,
            Action::Reset => self.actions.reset_pressed,
            Action::DebugDie => self.actions.debug_die_pressed,
        }
    }
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
