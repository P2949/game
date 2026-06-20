use game_core::input::{GamepadAxis, GamepadButton, Key, MouseButton};
use sdl3::gamepad::{Axis as SdlGamepadAxis, Button as SdlGamepadButton};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton as SdlMouseButton;

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
        Keycode::Q => Some(Key::Q),
        Keycode::E => Some(Key::E),
        Keycode::_0 | Keycode::Kp0 => Some(Key::Num0),
        Keycode::_1 | Keycode::Kp1 => Some(Key::Num1),
        Keycode::_2 | Keycode::Kp2 => Some(Key::Num2),
        Keycode::_3 | Keycode::Kp3 => Some(Key::Num3),
        Keycode::_4 | Keycode::Kp4 => Some(Key::Num4),
        Keycode::_5 | Keycode::Kp5 => Some(Key::Num5),
        Keycode::_6 | Keycode::Kp6 => Some(Key::Num6),
        Keycode::_7 | Keycode::Kp7 => Some(Key::Num7),
        Keycode::_8 | Keycode::Kp8 => Some(Key::Num8),
        Keycode::_9 | Keycode::Kp9 => Some(Key::Num9),
        Keycode::LShift | Keycode::RShift => Some(Key::Shift),
        Keycode::LCtrl | Keycode::RCtrl => Some(Key::Ctrl),
        Keycode::Tab => Some(Key::Tab),
        Keycode::Backspace => Some(Key::Backspace),
        Keycode::F1 => Some(Key::F1),
        Keycode::F2 => Some(Key::F2),
        Keycode::F3 => Some(Key::F3),
        Keycode::F4 => Some(Key::F4),
        Keycode::F5 => Some(Key::F5),
        Keycode::F6 => Some(Key::F6),
        Keycode::F7 => Some(Key::F7),
        Keycode::F8 => Some(Key::F8),
        Keycode::F9 => Some(Key::F9),
        Keycode::F10 => Some(Key::F10),
        Keycode::F11 => Some(Key::F11),
        Keycode::F12 => Some(Key::F12),
        _ => None,
    }
}

pub fn mouse_button_from_sdl(button: SdlMouseButton) -> Option<MouseButton> {
    match button {
        SdlMouseButton::Left => Some(MouseButton::Left),
        SdlMouseButton::Right => Some(MouseButton::Right),
        SdlMouseButton::Middle => Some(MouseButton::Middle),
        SdlMouseButton::X1 => Some(MouseButton::Back),
        SdlMouseButton::X2 => Some(MouseButton::Forward),
        SdlMouseButton::Unknown => None,
    }
}

pub fn gamepad_button_from_sdl(button: SdlGamepadButton) -> Option<GamepadButton> {
    match button {
        SdlGamepadButton::South => Some(GamepadButton::South),
        SdlGamepadButton::East => Some(GamepadButton::East),
        SdlGamepadButton::West => Some(GamepadButton::West),
        SdlGamepadButton::North => Some(GamepadButton::North),
        SdlGamepadButton::LeftShoulder => Some(GamepadButton::LeftShoulder),
        SdlGamepadButton::RightShoulder => Some(GamepadButton::RightShoulder),
        SdlGamepadButton::Start => Some(GamepadButton::Start),
        SdlGamepadButton::Back => Some(GamepadButton::Select),
        SdlGamepadButton::DPadUp => Some(GamepadButton::DPadUp),
        SdlGamepadButton::DPadDown => Some(GamepadButton::DPadDown),
        SdlGamepadButton::DPadLeft => Some(GamepadButton::DPadLeft),
        SdlGamepadButton::DPadRight => Some(GamepadButton::DPadRight),
        _ => None,
    }
}

/// Returns the logical stick and component (0 = X, 1 = Y) for an SDL axis.
pub fn gamepad_axis_from_sdl(axis: SdlGamepadAxis) -> Option<(GamepadAxis, usize)> {
    match axis {
        SdlGamepadAxis::LeftX => Some((GamepadAxis::LeftStick, 0)),
        SdlGamepadAxis::LeftY => Some((GamepadAxis::LeftStick, 1)),
        SdlGamepadAxis::RightX => Some((GamepadAxis::RightStick, 0)),
        SdlGamepadAxis::RightY => Some((GamepadAxis::RightStick, 1)),
        _ => None,
    }
}

pub fn normalize_gamepad_axis(value: i16) -> f32 {
    if value >= 0 {
        value as f32 / i16::MAX as f32
    } else {
        value as f32 / -(i16::MIN as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        gamepad_axis_from_sdl, gamepad_button_from_sdl, key_from_sdl, mouse_button_from_sdl,
        normalize_gamepad_axis,
    };
    use game_core::input::{GamepadAxis, GamepadButton, Key, MouseButton};
    use sdl3::gamepad::{Axis as SdlGamepadAxis, Button as SdlGamepadButton};
    use sdl3::keyboard::Keycode;
    use sdl3::mouse::MouseButton as SdlMouseButton;

    #[test]
    fn maps_sdl_keycodes_to_neutral_keys() {
        assert_eq!(key_from_sdl(Keycode::A), Some(Key::A));
        assert_eq!(key_from_sdl(Keycode::Left), Some(Key::Left));
        assert_eq!(key_from_sdl(Keycode::Space), Some(Key::Space));
        assert_eq!(key_from_sdl(Keycode::Return), Some(Key::Enter));
        assert_eq!(key_from_sdl(Keycode::Equals), Some(Key::Plus));
        assert_eq!(key_from_sdl(Keycode::KpMinus), Some(Key::Minus));
        assert_eq!(key_from_sdl(Keycode::Escape), Some(Key::Escape));
        assert_eq!(key_from_sdl(Keycode::Kp7), Some(Key::Num7));
        assert_eq!(key_from_sdl(Keycode::LShift), Some(Key::Shift));
        assert_eq!(key_from_sdl(Keycode::F12), Some(Key::F12));
    }

    #[test]
    fn maps_sdl_mouse_buttons_to_neutral_buttons() {
        assert_eq!(
            mouse_button_from_sdl(SdlMouseButton::Left),
            Some(MouseButton::Left)
        );
        assert_eq!(
            mouse_button_from_sdl(SdlMouseButton::X2),
            Some(MouseButton::Forward)
        );
    }

    #[test]
    fn maps_sdl_gamepad_controls_to_neutral_controls() {
        assert_eq!(
            gamepad_button_from_sdl(SdlGamepadButton::South),
            Some(GamepadButton::South)
        );
        assert_eq!(
            gamepad_button_from_sdl(SdlGamepadButton::Back),
            Some(GamepadButton::Select)
        );
        assert_eq!(
            gamepad_axis_from_sdl(SdlGamepadAxis::LeftX),
            Some((GamepadAxis::LeftStick, 0))
        );
        assert_eq!(
            gamepad_axis_from_sdl(SdlGamepadAxis::RightY),
            Some((GamepadAxis::RightStick, 1))
        );
        assert_eq!(normalize_gamepad_axis(i16::MIN), -1.0);
        assert_eq!(normalize_gamepad_axis(i16::MAX), 1.0);
    }
}
