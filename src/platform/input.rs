use sdl3::keyboard::Keycode;

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub move_x: f32,
    pub move_y: f32,
    pub action: bool,
    pub pause_pressed: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.pause_pressed = false;
    }

    pub fn set_key(&mut self, keycode: Keycode, pressed: bool) {
        match keycode {
            Keycode::Left | Keycode::A => self.left = pressed,
            Keycode::Right | Keycode::D => self.right = pressed,
            Keycode::Up | Keycode::W => self.up = pressed,
            Keycode::Down | Keycode::S => self.down = pressed,
            Keycode::Space | Keycode::Return => self.action = pressed,
            Keycode::P if pressed => self.pause_pressed = true,
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
}
