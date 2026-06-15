use game_core::input::Key;
use sdl3::keyboard::Keycode;

pub fn key_from_sdl(keycode: Keycode) -> Option<Key> {
    match keycode {
        Keycode::A => Some(Key::A),
        Keycode::D => Some(Key::D),
        Keycode::W => Some(Key::W),
        Keycode::S => Some(Key::S),
        Keycode::Left => Some(Key::Left),
        Keycode::Right => Some(Key::Right),
        Keycode::Up => Some(Key::Up),
        Keycode::Down => Some(Key::Down),
        Keycode::Space => Some(Key::Space),
        Keycode::Return => Some(Key::Enter),
        Keycode::P => Some(Key::P),
        Keycode::R => Some(Key::R),
        Keycode::K => Some(Key::K),
        Keycode::Plus | Keycode::Equals | Keycode::KpPlus => Some(Key::Plus),
        Keycode::Minus | Keycode::KpMinus => Some(Key::Minus),
        Keycode::Escape => Some(Key::Escape),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::key_from_sdl;
    use game_core::input::Key;
    use sdl3::keyboard::Keycode;

    #[test]
    fn maps_sdl_keycodes_to_neutral_keys() {
        assert_eq!(key_from_sdl(Keycode::A), Some(Key::A));
        assert_eq!(key_from_sdl(Keycode::Left), Some(Key::Left));
        assert_eq!(key_from_sdl(Keycode::Space), Some(Key::Space));
        assert_eq!(key_from_sdl(Keycode::Return), Some(Key::Enter));
        assert_eq!(key_from_sdl(Keycode::Equals), Some(Key::Plus));
        assert_eq!(key_from_sdl(Keycode::KpMinus), Some(Key::Minus));
        assert_eq!(key_from_sdl(Keycode::Escape), Some(Key::Escape));
    }
}
