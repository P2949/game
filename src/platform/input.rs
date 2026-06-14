use sdl3::keyboard::Keycode;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub move_x: f32,
    pub move_y: f32,
    pub action_pressed: bool,
    pub pause_pressed: bool,
    pub reset_pressed: bool,
    pub debug_die_pressed: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.action_pressed = false;
        self.pause_pressed = false;
        self.reset_pressed = false;
        self.debug_die_pressed = false;
    }

    /// Clears all held-key and movement state. Called on window focus loss: a key
    /// released while we are unfocused never delivers a key-up event, so without
    /// this a held movement/zoom key would stay "down" and the player would keep
    /// drifting or zooming after refocusing.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn take_frame_actions(&mut self) -> FrameActions {
        let actions = FrameActions {
            action_pressed: self.action_pressed,
            pause_pressed: self.pause_pressed,
            reset_pressed: self.reset_pressed,
            debug_die_pressed: self.debug_die_pressed,
        };

        self.action_pressed = false;
        self.pause_pressed = false;
        self.reset_pressed = false;
        self.debug_die_pressed = false;

        actions
    }

    pub fn set_key(&mut self, keycode: Keycode, pressed: bool) {
        match keycode {
            Keycode::Left | Keycode::A => self.left = pressed,
            Keycode::Right | Keycode::D => self.right = pressed,
            Keycode::Up | Keycode::W => self.up = pressed,
            Keycode::Down | Keycode::S => self.down = pressed,
            // Action is purely edge-triggered (press → one-shot); there is no
            // held-action state to track.
            Keycode::Space | Keycode::Return if pressed => self.action_pressed = true,
            Keycode::P if pressed => self.pause_pressed = true,
            Keycode::R if pressed => self.reset_pressed = true,
            Keycode::K if pressed => self.debug_die_pressed = true,
            Keycode::Plus | Keycode::Equals | Keycode::KpPlus => self.zoom_in = pressed,
            Keycode::Minus | Keycode::KpMinus => self.zoom_out = pressed,
            _ => {}
        }

        self.move_x = axis(self.left, self.right);
        self.move_y = axis(self.up, self.down);
    }

    pub fn movement(&self) -> glam::Vec2 {
        let v = glam::vec2(self.move_x, self.move_y);
        if v.length_squared() > 1.0 {
            v.normalize()
        } else {
            v
        }
    }
}

fn axis(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::InputState;
    use sdl3::keyboard::Keycode;

    #[test]
    fn diagonal_movement_is_normalized() {
        let mut input = InputState::default();
        input.set_key(Keycode::Right, true);
        input.set_key(Keycode::Down, true);

        let movement = input.movement();
        assert!((movement.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn take_frame_actions_consumes_edges_once() {
        let mut input = InputState::default();
        input.set_key(Keycode::P, true);
        input.set_key(Keycode::Space, true);

        let actions = input.take_frame_actions();
        assert!(actions.pause_pressed);
        assert!(actions.action_pressed);

        let actions = input.take_frame_actions();
        assert!(!actions.pause_pressed);
        assert!(!actions.action_pressed);
    }

    #[test]
    fn reset_clears_held_movement_keys() {
        let mut input = InputState::default();
        input.set_key(Keycode::Right, true);
        input.set_key(Keycode::Up, true);
        assert!(input.movement().length() > 0.0);

        // Simulates losing focus while keys are held.
        input.reset();
        assert_eq!(input.movement(), glam::Vec2::ZERO);

        // A late key-up for an already-cleared key must keep movement at rest, not
        // drive it negative.
        input.set_key(Keycode::Right, false);
        assert_eq!(input.movement(), glam::Vec2::ZERO);
    }
}
