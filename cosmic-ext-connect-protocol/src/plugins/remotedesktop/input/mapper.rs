//! VNC Keysym to Linux Keycode Mapping
//!
//! Maps X11 keysyms (used by VNC) to Linux input keycodes.
//!
//! ## X11 Keysym Groups
//!
//! - 0x0000-0x00FF: Latin-1 (ASCII + extended)
//! - 0xFF00-0xFFFF: Function keys, special keys
//! - 0xFE00-0xFEFF: Multimedia keys
//!
//! ## References
//!
//! - [X11 Keysym Definitions](https://www.x.org/releases/current/doc/xproto/keysyms.html)
//! - [Linux Input Event Codes](https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h)

use mouse_keyboard_input::*;

/// Map VNC keysym to Linux keycode
///
/// # Arguments
///
/// * `keysym` - X11 keysym value
///
/// # Returns
///
/// Linux keycode (u16) if mapping exists, None otherwise
pub fn keysym_to_keycode(keysym: u32) -> Option<u16> {
    match keysym {
        // Latin-1: Basic ASCII (0x0020-0x007E)
        0x0020 => Some(KEY_SPACE),
        0x0027 => Some(KEY_APOSTROPHE),
        0x002c => Some(KEY_COMMA),
        0x002d => Some(KEY_MINUS),
        0x002e => Some(KEY_DOT),
        0x002f => Some(KEY_SLASH),

        // Numbers (0-9)
        0x0030 => Some(11), // KEY_0 constant doesn't exist, uses raw value
        0x0031 => Some(KEY_1),
        0x0032 => Some(KEY_2),
        0x0033 => Some(KEY_3),
        0x0034 => Some(KEY_4),
        0x0035 => Some(KEY_5),
        0x0036 => Some(KEY_6),
        0x0037 => Some(KEY_7),
        0x0038 => Some(KEY_8),
        0x0039 => Some(KEY_9),

        0x003b => Some(KEY_SEMICOLON),
        0x003d => Some(KEY_EQUAL),

        // Uppercase letters (A-Z)
        0x0041 => Some(KEY_A),
        0x0042 => Some(KEY_B),
        0x0043 => Some(KEY_C),
        0x0044 => Some(KEY_D),
        0x0045 => Some(KEY_E),
        0x0046 => Some(KEY_F),
        0x0047 => Some(KEY_G),
        0x0048 => Some(KEY_H),
        0x0049 => Some(KEY_I),
        0x004a => Some(KEY_J),
        0x004b => Some(KEY_K),
        0x004c => Some(KEY_L),
        0x004d => Some(KEY_M),
        0x004e => Some(KEY_N),
        0x004f => Some(KEY_O),
        0x0050 => Some(KEY_P),
        0x0051 => Some(KEY_Q),
        0x0052 => Some(KEY_R),
        0x0053 => Some(KEY_S),
        0x0054 => Some(KEY_T),
        0x0055 => Some(KEY_U),
        0x0056 => Some(KEY_V),
        0x0057 => Some(KEY_W),
        0x0058 => Some(KEY_X),
        0x0059 => Some(KEY_Y),
        0x005a => Some(KEY_Z),

        0x005b => Some(KEY_LEFTBRACE),
        0x005c => Some(KEY_BACKSLASH),
        0x005d => Some(KEY_RIGHTBRACE),
        0x0060 => Some(KEY_GRAVE),

        // Lowercase letters (a-z) - map to same as uppercase
        0x0061 => Some(KEY_A),
        0x0062 => Some(KEY_B),
        0x0063 => Some(KEY_C),
        0x0064 => Some(KEY_D),
        0x0065 => Some(KEY_E),
        0x0066 => Some(KEY_F),
        0x0067 => Some(KEY_G),
        0x0068 => Some(KEY_H),
        0x0069 => Some(KEY_I),
        0x006a => Some(KEY_J),
        0x006b => Some(KEY_K),
        0x006c => Some(KEY_L),
        0x006d => Some(KEY_M),
        0x006e => Some(KEY_N),
        0x006f => Some(KEY_O),
        0x0070 => Some(KEY_P),
        0x0071 => Some(KEY_Q),
        0x0072 => Some(KEY_R),
        0x0073 => Some(KEY_S),
        0x0074 => Some(KEY_T),
        0x0075 => Some(KEY_U),
        0x0076 => Some(KEY_V),
        0x0077 => Some(KEY_W),
        0x0078 => Some(KEY_X),
        0x0079 => Some(KEY_Y),
        0x007a => Some(KEY_Z),

        // Function keys and special keys (0xFF00-0xFFFF)
        0xff08 => Some(KEY_BACKSPACE),
        0xff09 => Some(KEY_TAB),
        0xff0d => Some(KEY_ENTER),
        0xff1b => Some(KEY_ESC),
        0xff50 => Some(KEY_HOME),
        0xff51 => Some(KEY_LEFT),
        0xff52 => Some(KEY_UP),
        0xff53 => Some(KEY_RIGHT),
        0xff54 => Some(KEY_DOWN),
        0xff55 => Some(KEY_PAGEUP),
        0xff56 => Some(KEY_PAGEDOWN),
        0xff57 => Some(KEY_END),
        0xff63 => Some(KEY_INSERT),
        0xff8d => Some(KEY_KPENTER),
        0xffff => Some(KEY_DELETE),

        // Modifiers
        0xffe1 => Some(KEY_LEFTSHIFT),
        0xffe2 => Some(KEY_RIGHTSHIFT),
        0xffe3 => Some(KEY_LEFTCTRL),
        0xffe4 => Some(KEY_RIGHTCTRL),
        0xffe5 => Some(KEY_CAPSLOCK),
        0xffe7 => Some(KEY_LEFTMETA),
        0xffe8 => Some(KEY_RIGHTMETA),
        0xffe9 => Some(KEY_LEFTALT),
        0xffea => Some(KEY_RIGHTALT),

        // Function keys (F1-F12)
        0xffbe => Some(KEY_F1),
        0xffbf => Some(KEY_F2),
        0xffc0 => Some(KEY_F3),
        0xffc1 => Some(KEY_F4),
        0xffc2 => Some(KEY_F5),
        0xffc3 => Some(KEY_F6),
        0xffc4 => Some(KEY_F7),
        0xffc5 => Some(KEY_F8),
        0xffc6 => Some(KEY_F9),
        0xffc7 => Some(KEY_F10),
        0xffc8 => Some(KEY_F11),
        0xffc9 => Some(KEY_F12),

        // Numpad
        0xffaa => Some(KEY_KPASTERISK),
        0xffab => Some(KEY_KPPLUS),
        0xffad => Some(KEY_KPMINUS),
        0xffae => Some(KEY_KPDOT),
        0xffaf => Some(KEY_KPSLASH),

        // Numpad digits (0-9)
        0xffb0 => Some(KEY_KP0),
        0xffb1 => Some(KEY_KP1),
        0xffb2 => Some(KEY_KP2),
        0xffb3 => Some(KEY_KP3),
        0xffb4 => Some(KEY_KP4),
        0xffb5 => Some(KEY_KP5),
        0xffb6 => Some(KEY_KP6),
        0xffb7 => Some(KEY_KP7),
        0xffb8 => Some(KEY_KP8),
        0xffb9 => Some(KEY_KP9),

        // Multimedia keys
        0x1008ff11 => Some(KEY_VOLUMEDOWN),
        0x1008ff12 => Some(KEY_MUTE),
        0x1008ff13 => Some(KEY_VOLUMEUP),

        // Unknown keysym
        _ => None,
    }
}

/// Get keysym name for debugging
pub fn keysym_name(keysym: u32) -> String {
    match keysym {
        0x0020 => "Space".to_string(),
        0x0041..=0x005a => format!("'{}'", (keysym as u8) as char),
        0x0061..=0x007a => format!("'{}'", (keysym as u8) as char),
        0x0030..=0x0039 => format!("'{}'", (keysym as u8) as char),
        0xff08 => "Backspace".to_string(),
        0xff09 => "Tab".to_string(),
        0xff0d => "Enter".to_string(),
        0xff1b => "Escape".to_string(),
        0xff50 => "Home".to_string(),
        0xff51 => "Left".to_string(),
        0xff52 => "Up".to_string(),
        0xff53 => "Right".to_string(),
        0xff54 => "Down".to_string(),
        0xff55 => "PageUp".to_string(),
        0xff56 => "PageDown".to_string(),
        0xff57 => "End".to_string(),
        0xffff => "Delete".to_string(),
        0xffe1 => "LeftShift".to_string(),
        0xffe2 => "RightShift".to_string(),
        0xffe3 => "LeftCtrl".to_string(),
        0xffe4 => "RightCtrl".to_string(),
        0xffe9 => "LeftAlt".to_string(),
        0xffea => "RightAlt".to_string(),
        0xffbe..=0xffc9 => format!("F{}", keysym - 0xffbd),
        _ => format!("0x{:08x}", keysym),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_letters() {
        // Lowercase
        assert!(keysym_to_keycode(0x0061).is_some()); // 'a'
        assert!(keysym_to_keycode(0x007a).is_some()); // 'z'

        // Uppercase
        assert!(keysym_to_keycode(0x0041).is_some()); // 'A'
        assert!(keysym_to_keycode(0x005a).is_some()); // 'Z'
    }

    #[test]
    fn test_numbers() {
        assert!(keysym_to_keycode(0x0030).is_some()); // '0'
        assert!(keysym_to_keycode(0x0039).is_some()); // '9'
    }

    #[test]
    fn test_special_keys() {
        assert!(keysym_to_keycode(0xff08).is_some()); // Backspace
        assert!(keysym_to_keycode(0xff09).is_some()); // Tab
        assert!(keysym_to_keycode(0xff0d).is_some()); // Enter
        assert!(keysym_to_keycode(0xff1b).is_some()); // Escape
        assert!(keysym_to_keycode(0xffff).is_some()); // Delete
    }

    #[test]
    fn test_function_keys() {
        assert!(keysym_to_keycode(0xffbe).is_some()); // F1
        assert!(keysym_to_keycode(0xffc9).is_some()); // F12
    }

    #[test]
    fn test_modifiers() {
        assert!(keysym_to_keycode(0xffe1).is_some()); // Left Shift
        assert!(keysym_to_keycode(0xffe3).is_some()); // Left Ctrl
        assert!(keysym_to_keycode(0xffe9).is_some()); // Left Alt
    }

    #[test]
    fn test_arrow_keys() {
        assert!(keysym_to_keycode(0xff51).is_some()); // Left
        assert!(keysym_to_keycode(0xff52).is_some()); // Up
        assert!(keysym_to_keycode(0xff53).is_some()); // Right
        assert!(keysym_to_keycode(0xff54).is_some()); // Down
    }

    #[test]
    fn test_unknown_keysym() {
        assert!(keysym_to_keycode(0x123456).is_none());
    }

    #[test]
    fn test_keysym_name() {
        assert_eq!(keysym_name(0x0041), "'A'");
        assert_eq!(keysym_name(0xff08), "Backspace");
        assert_eq!(keysym_name(0xffbe), "F1");
        assert_eq!(keysym_name(0x123456), "0x00123456");
    }
}
