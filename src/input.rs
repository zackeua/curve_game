// Custom key-detection and localStorage helpers.
//
// Keys are represented as `key.event` strings from the browser:
//   - Letters:   "a", "A", "å", "ö", etc.
//   - Arrows:    "ArrowLeft", "ArrowRight", "ArrowUp, "ArrowDown"
//   - Space:     " "
//   - Other:     "Enter", "Escape", "Tab", "Backspace", "Shift", "Control", "Alt"
//
pub fn is_key_down(key: &str) -> bool {
    native::str_to_keycode(key).map_or(false, |k| macroquad::prelude::is_key_down(k))
}

pub fn get_last_key() -> Option<String> {
    macroquad::prelude::get_char_pressed()
        .map(|character| character.to_string())
        .or_else(|| {
            macroquad::prelude::get_last_key_pressed()
                .and_then(native::keycode_to_str)
                .map(String::from)
        })
}

mod native {
    use macroquad::prelude::KeyCode;

    pub fn str_to_keycode(key: &str) -> Option<KeyCode> {
        Some(match key {
            "a" | "A" => KeyCode::A,
            "b" | "B" => KeyCode::B,
            "c" | "C" => KeyCode::C,
            "d" | "D" => KeyCode::D,
            "e" | "E" => KeyCode::E,
            "f" | "F" => KeyCode::F,
            "g" | "G" => KeyCode::G,
            "h" | "H" => KeyCode::H,
            "i" | "I" => KeyCode::I,
            "j" | "J" => KeyCode::J,
            "k" | "K" => KeyCode::K,
            "l" | "L" => KeyCode::L,
            "m" | "M" => KeyCode::M,
            "n" | "N" => KeyCode::N,
            "o" | "O" => KeyCode::O,
            "p" | "P" => KeyCode::P,
            "q" | "Q" => KeyCode::Q,
            "r" | "R" => KeyCode::R,
            "s" | "S" => KeyCode::S,
            "t" | "T" => KeyCode::T,
            "u" | "U" => KeyCode::U,
            "v" | "V" => KeyCode::V,
            "w" | "W" => KeyCode::W,
            "x" | "X" => KeyCode::X,
            "y" | "Y" => KeyCode::Y,
            "z" | "Z" => KeyCode::Z,
            "ArrowLeft" => KeyCode::Left,
            "ArrowRight" => KeyCode::Right,
            "ArrowUp" => KeyCode::Up,
            "ArrowDown" => KeyCode::Down,
            " " => KeyCode::Space,
            "Enter" => KeyCode::Enter,
            "Escape" => KeyCode::Escape,
            "Tab" => KeyCode::Tab,
            "Backspace" => KeyCode::Backspace,
            "Shift" | "ShiftLeft" | "ShiftRight" => KeyCode::LeftShift,
            "Control" | "ControlLeft" | "ControlRight" => KeyCode::LeftControl,
            "Alt" | "AltLeft" | "AltRight" => KeyCode::LeftAlt,
            "0" => KeyCode::Key0,
            "1" => KeyCode::Key1,
            "2" => KeyCode::Key2,
            "3" => KeyCode::Key3,
            "4" => KeyCode::Key4,
            "5" => KeyCode::Key5,
            "6" => KeyCode::Key6,
            "7" => KeyCode::Key7,
            "8" => KeyCode::Key8,
            "9" => KeyCode::Key9,
            "+" => KeyCode::Equal,
            "-" => KeyCode::Minus,
            "*" => KeyCode::KpMultiply,
            "/" => KeyCode::KpDivide,
            _ => return None,
        })
    }

    pub fn keycode_to_str(key: KeyCode) -> Option<&'static str> {
        Some(match key {
            KeyCode::A => "a",
            KeyCode::B => "b",
            KeyCode::C => "c",
            KeyCode::D => "d",
            KeyCode::E => "e",
            KeyCode::F => "f",
            KeyCode::G => "g",
            KeyCode::H => "h",
            KeyCode::I => "i",
            KeyCode::J => "j",
            KeyCode::K => "k",
            KeyCode::L => "l",
            KeyCode::M => "m",
            KeyCode::N => "n",
            KeyCode::O => "o",
            KeyCode::P => "p",
            KeyCode::Q => "q",
            KeyCode::R => "r",
            KeyCode::S => "s",
            KeyCode::T => "t",
            KeyCode::U => "u",
            KeyCode::V => "v",
            KeyCode::W => "w",
            KeyCode::X => "X",
            KeyCode::Y => "y",
            KeyCode::Z => "Z",
            KeyCode::Left => "ArrowLeft",
            KeyCode::Right => "ArrowRight",
            KeyCode::Up => "ArrowUp",
            KeyCode::Down => "ArrowDown",
            KeyCode::Space => " ",
            KeyCode::Enter => "Enter",
            KeyCode::Escape => "Escape",
            KeyCode::Tab => "Tab",
            KeyCode::Backspace => "Backspace",
            KeyCode::LeftShift | KeyCode::RightShift => "Shift",
            KeyCode::LeftControl | KeyCode::RightControl => "Control",
            KeyCode::LeftAlt | KeyCode::RightAlt => "Alt",
            KeyCode::Key0 => "0",
            KeyCode::Key1 => "1",
            KeyCode::Key2 => "2",
            KeyCode::Key3 => "3",
            KeyCode::Key4 => "4",
            KeyCode::Key5 => "5",
            KeyCode::Key6 => "6",
            KeyCode::Key7 => "7",
            KeyCode::Key8 => "8",
            KeyCode::Key9 => "9",
            _ => return None,
        })
    }
}
