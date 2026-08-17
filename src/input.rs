// Custom key-detection and localStorage helpers.
//
// Keys are represented as `key.event` strings from the browser:
//   - Letters:   "a", "A", "å", "ö", etc.
//   - Arrows:    "ArrowLeft", "ArrowRight", "ArrowUp, "ArrowDown"
//   - Space:     " "
//   - Other:     "Enter", "Escape", "Tab", "Backspace", "Shift", "Control", "Alt"
//
// On WASM the implementation talks to JS functions injected by gl.js.
// On native it falls back to mackroquad's KeyCode system (best-effort for
// ASCII / common keys; special layout chars like å are not detectable).

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Returns 1 if the key string (UTF-8, *not* NUL-terminated) is currently held down.
    fn custom_key_is_down(ptr: *const u8, len: u32) -> i32;
    /// Writes the last key pressed since the previous frame into `buf_ptr`.
    /// Returns the byte length written, or -1 if no key was pressed.
    fn custom_get_last_key(buf_ptr: *mut u8, buf_len: u32) -> i32;
    /// Saves `value` under `key` in localStorage.
    fn storage_save(key_ptr: *const u8, key_len: u32, val_ptr: *const u8, val_len: u32);
    /// Loads the value stored under `key` from localStorage into `buf_ptr`.
    /// Returns the byte length of the value, or -1 if the key does not exist.
    fn storage_load(key_ptr: *const u8, key_len: u32, buf_ptr: *mut u8, buf_len: u32) -> i32;
}

// -- WASM implementations --------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub fn is_key_down(key: &str) -> bool {
    unsafe { custom_key_is_down(key.as_ptr(), key.len() as u32) != 0 }
}

#[cfg(target_arch = "wasm32")]
pub fn get_last_key() -> Option<String> {
    let mut buf = [0u8, 64];
    let len = unsafe { custom_get_last_key(buf.as_mut_ptr(), buf.len() as u32) };
    if len < 0 {
        None
    } else {
        String::from_utf8(buf[..len as usize].to_vec()).ok()
    }
}

#[cfg(target_arch = "wasm32")]
pub fn storage_save_str(key: &str, value: &str) {
    unsafe {
        storage_save(
            key.as_ptr(),
            key.len() as u32,
            value.as_ptr(),
            value.len() as u32,
        );
    }
}

#[cfg(target_arch = "wasm32")]
pub fn storage_load_str(key: &str) -> Option<String> {
    let mut buf = vec![0u8; 65536];
    let len = unsafe {
        storage_load(
            key.as_ptr(),
            key.len() as u32,
            buf.as_mut_ptr(),
            buf.len() as u32,
        )
    };
    if len < 0 {
        None
    } else {
        String::from_utf8(buf[..len as usize].to_vec()).ok()
    }
}

// -- Native (non-WASM) implementations ------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
pub fn is_key_down(key: &str) -> bool {
    native::str_to_keycode(key).map_or(false, |k| macroquad::prelude::is_key_down(k))
}


#[cfg(not(target_arch = "wasm32"))]
pub fn get_last_key() -> Option<String> {
    macroquad::prelude::get_last_key_pressed()
        .and_then(native::keycode_to_string)
        .map(String::from)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn storage_save_str(key: &str, _value: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn storage_load_str(key: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
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

