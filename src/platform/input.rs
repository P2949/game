use sdl3::keyboard::Keycode;

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
}

impl InputState {
    pub fn set_key(&mut self, keycode: Keycode, pressed: bool) {
        match keycode {
            Keycode::Left | Keycode::A => self.move_left = pressed,
            Keycode::Right | Keycode::D => self.move_right = pressed,
            Keycode::Up | Keycode::W => self.move_up = pressed,
            Keycode::Down | Keycode::S => self.move_down = pressed,
            Keycode::Plus | Keycode::Equals | Keycode::KpPlus => self.zoom_in = pressed,
            Keycode::Minus | Keycode::KpMinus => self.zoom_out = pressed,
            _ => {}
        }
    }
}
