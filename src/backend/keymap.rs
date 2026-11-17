use crate::model::LogicalKey;

/// macOS distinguishes between standard Quartz virtual keycodes (`CGKeyCode`)
/// and system-defined media/volume keys (`NX_KEYTYPE_*` via `NSSystemDefined` events).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacOsKeyTarget {
    VirtualKey(u16),
    MediaKey(u8),
}

/// Maps a cross-platform `LogicalKey` to a Linux `<linux/input-event-codes.h>` (`evdev`) code.
#[deny(unreachable_patterns)]
pub const fn to_linux_keycode(key: LogicalKey) -> u16 {
    match key {
        LogicalKey::Escape => 1,
        LogicalKey::Digit1 => 2,
        LogicalKey::Digit2 => 3,
        LogicalKey::Digit3 => 4,
        LogicalKey::Digit4 => 5,
        LogicalKey::Digit5 => 6,
        LogicalKey::Digit6 => 7,
        LogicalKey::Digit7 => 8,
        LogicalKey::Digit8 => 9,
        LogicalKey::Digit9 => 10,
        LogicalKey::Digit0 => 11,
        LogicalKey::Minus => 12,
        LogicalKey::Equal => 13,
        LogicalKey::Backspace => 14,
        LogicalKey::Tab => 15,
        LogicalKey::Q => 16,
        LogicalKey::W => 17,
        LogicalKey::E => 18,
        LogicalKey::R => 19,
        LogicalKey::T => 20,
        LogicalKey::Y => 21,
        LogicalKey::U => 22,
        LogicalKey::I => 23,
        LogicalKey::O => 24,
        LogicalKey::P => 25,
        LogicalKey::BracketLeft => 26,
        LogicalKey::BracketRight => 27,
        LogicalKey::Enter => 28,
        LogicalKey::A => 30,
        LogicalKey::S => 31,
        LogicalKey::D => 32,
        LogicalKey::F => 33,
        LogicalKey::G => 34,
        LogicalKey::H => 35,
        LogicalKey::J => 36,
        LogicalKey::K => 37,
        LogicalKey::L => 38,
        LogicalKey::Semicolon => 39,
        LogicalKey::Quote => 40,
        LogicalKey::Grave => 41,
        LogicalKey::Backslash => 43,
        LogicalKey::Z => 44,
        LogicalKey::X => 45,
        LogicalKey::C => 46,
        LogicalKey::V => 47,
        LogicalKey::B => 48,
        LogicalKey::N => 49,
        LogicalKey::M => 50,
        LogicalKey::Comma => 51,
        LogicalKey::Period => 52,
        LogicalKey::Slash => 53,
        LogicalKey::Space => 57,
        LogicalKey::CapsLock => 58,
        LogicalKey::F1 => 59,
        LogicalKey::F2 => 60,
        LogicalKey::F3 => 61,
        LogicalKey::F4 => 62,
        LogicalKey::F5 => 63,
        LogicalKey::F6 => 64,
        LogicalKey::F7 => 65,
        LogicalKey::F8 => 66,
        LogicalKey::F9 => 67,
        LogicalKey::F10 => 68,
        LogicalKey::ScrollLock => 70,
        LogicalKey::F11 => 87,
        LogicalKey::F12 => 88,
        LogicalKey::PrintScreen => 99,
        LogicalKey::Home => 102,
        LogicalKey::ArrowUp => 103,
        LogicalKey::PageUp => 104,
        LogicalKey::ArrowLeft => 105,
        LogicalKey::ArrowRight => 106,
        LogicalKey::End => 107,
        LogicalKey::ArrowDown => 108,
        LogicalKey::PageDown => 109,
        LogicalKey::Insert => 110,
        LogicalKey::Delete => 111,
        LogicalKey::VolumeMute => 113,
        LogicalKey::VolumeDown => 114,
        LogicalKey::VolumeUp => 115,
        LogicalKey::Pause => 119,
        LogicalKey::ContextMenu => 127,
        LogicalKey::MediaStop => 128,
        LogicalKey::MediaNextTrack => 163,
        LogicalKey::MediaPlayPause => 164,
        LogicalKey::MediaPrevTrack => 165,
        LogicalKey::F13 => 183,
        LogicalKey::F14 => 184,
        LogicalKey::F15 => 185,
        LogicalKey::F16 => 186,
        LogicalKey::F17 => 187,
        LogicalKey::F18 => 188,
        LogicalKey::F19 => 189,
        LogicalKey::F20 => 190,
        LogicalKey::F21 => 191,
        LogicalKey::F22 => 192,
        LogicalKey::F23 => 193,
        LogicalKey::F24 => 194,
    }
}

pub const fn from_linux_keycode(code: u16) -> Option<LogicalKey> {
    Some(match code {
        1 => LogicalKey::Escape,
        2 => LogicalKey::Digit1,
        3 => LogicalKey::Digit2,
        4 => LogicalKey::Digit3,
        5 => LogicalKey::Digit4,
        6 => LogicalKey::Digit5,
        7 => LogicalKey::Digit6,
        8 => LogicalKey::Digit7,
        9 => LogicalKey::Digit8,
        10 => LogicalKey::Digit9,
        11 => LogicalKey::Digit0,
        12 => LogicalKey::Minus,
        13 => LogicalKey::Equal,
        14 => LogicalKey::Backspace,
        15 => LogicalKey::Tab,
        16 => LogicalKey::Q,
        17 => LogicalKey::W,
        18 => LogicalKey::E,
        19 => LogicalKey::R,
        20 => LogicalKey::T,
        21 => LogicalKey::Y,
        22 => LogicalKey::U,
        23 => LogicalKey::I,
        24 => LogicalKey::O,
        25 => LogicalKey::P,
        26 => LogicalKey::BracketLeft,
        27 => LogicalKey::BracketRight,
        28 => LogicalKey::Enter,
        30 => LogicalKey::A,
        31 => LogicalKey::S,
        32 => LogicalKey::D,
        33 => LogicalKey::F,
        34 => LogicalKey::G,
        35 => LogicalKey::H,
        36 => LogicalKey::J,
        37 => LogicalKey::K,
        38 => LogicalKey::L,
        39 => LogicalKey::Semicolon,
        40 => LogicalKey::Quote,
        41 => LogicalKey::Grave,
        43 => LogicalKey::Backslash,
        44 => LogicalKey::Z,
        45 => LogicalKey::X,
        46 => LogicalKey::C,
        47 => LogicalKey::V,
        48 => LogicalKey::B,
        49 => LogicalKey::N,
        50 => LogicalKey::M,
        51 => LogicalKey::Comma,
        52 => LogicalKey::Period,
        53 => LogicalKey::Slash,
        57 => LogicalKey::Space,
        58 => LogicalKey::CapsLock,
        59 => LogicalKey::F1,
        60 => LogicalKey::F2,
        61 => LogicalKey::F3,
        62 => LogicalKey::F4,
        63 => LogicalKey::F5,
        64 => LogicalKey::F6,
        65 => LogicalKey::F7,
        66 => LogicalKey::F8,
        67 => LogicalKey::F9,
        68 => LogicalKey::F10,
        70 => LogicalKey::ScrollLock,
        87 => LogicalKey::F11,
        88 => LogicalKey::F12,
        99 => LogicalKey::PrintScreen,
        102 => LogicalKey::Home,
        103 => LogicalKey::ArrowUp,
        104 => LogicalKey::PageUp,
        105 => LogicalKey::ArrowLeft,
        106 => LogicalKey::ArrowRight,
        107 => LogicalKey::End,
        108 => LogicalKey::ArrowDown,
        109 => LogicalKey::PageDown,
        110 => LogicalKey::Insert,
        111 => LogicalKey::Delete,
        113 => LogicalKey::VolumeMute,
        114 => LogicalKey::VolumeDown,
        115 => LogicalKey::VolumeUp,
        119 => LogicalKey::Pause,
        127 => LogicalKey::ContextMenu,
        128 => LogicalKey::MediaStop,
        163 => LogicalKey::MediaNextTrack,
        164 => LogicalKey::MediaPlayPause,
        165 => LogicalKey::MediaPrevTrack,
        183 => LogicalKey::F13,
        184 => LogicalKey::F14,
        185 => LogicalKey::F15,
        186 => LogicalKey::F16,
        187 => LogicalKey::F17,
        188 => LogicalKey::F18,
        189 => LogicalKey::F19,
        190 => LogicalKey::F20,
        191 => LogicalKey::F21,
        192 => LogicalKey::F22,
        193 => LogicalKey::F23,
        194 => LogicalKey::F24,
        _ => return None,
    })
}

/// Maps a cross-platform `LogicalKey` to a Windows Win32 Virtual-Key (`VK_*`) code.
#[deny(unreachable_patterns)]
pub const fn to_win32_vk(key: LogicalKey) -> u16 {
    match key {
        LogicalKey::Backspace => 0x08,   // VK_BACK
        LogicalKey::Tab => 0x09,         // VK_TAB
        LogicalKey::Enter => 0x0D,       // VK_RETURN
        LogicalKey::Pause => 0x13,       // VK_PAUSE
        LogicalKey::CapsLock => 0x14,    // VK_CAPITAL
        LogicalKey::Escape => 0x1B,      // VK_ESCAPE
        LogicalKey::Space => 0x20,       // VK_SPACE
        LogicalKey::PageUp => 0x21,      // VK_PRIOR
        LogicalKey::PageDown => 0x22,    // VK_NEXT
        LogicalKey::End => 0x23,         // VK_END
        LogicalKey::Home => 0x24,        // VK_HOME
        LogicalKey::ArrowLeft => 0x25,   // VK_LEFT
        LogicalKey::ArrowUp => 0x26,     // VK_UP
        LogicalKey::ArrowRight => 0x27,  // VK_RIGHT
        LogicalKey::ArrowDown => 0x28,   // VK_DOWN
        LogicalKey::PrintScreen => 0x2C, // VK_SNAPSHOT
        LogicalKey::Insert => 0x2D,      // VK_INSERT
        LogicalKey::Delete => 0x2E,      // VK_DELETE
        LogicalKey::Digit0 => 0x30,
        LogicalKey::Digit1 => 0x31,
        LogicalKey::Digit2 => 0x32,
        LogicalKey::Digit3 => 0x33,
        LogicalKey::Digit4 => 0x34,
        LogicalKey::Digit5 => 0x35,
        LogicalKey::Digit6 => 0x36,
        LogicalKey::Digit7 => 0x37,
        LogicalKey::Digit8 => 0x38,
        LogicalKey::Digit9 => 0x39,
        LogicalKey::A => 0x41,
        LogicalKey::B => 0x42,
        LogicalKey::C => 0x43,
        LogicalKey::D => 0x44,
        LogicalKey::E => 0x45,
        LogicalKey::F => 0x46,
        LogicalKey::G => 0x47,
        LogicalKey::H => 0x48,
        LogicalKey::I => 0x49,
        LogicalKey::J => 0x4A,
        LogicalKey::K => 0x4B,
        LogicalKey::L => 0x4C,
        LogicalKey::M => 0x4D,
        LogicalKey::N => 0x4E,
        LogicalKey::O => 0x4F,
        LogicalKey::P => 0x50,
        LogicalKey::Q => 0x51,
        LogicalKey::R => 0x52,
        LogicalKey::S => 0x53,
        LogicalKey::T => 0x54,
        LogicalKey::U => 0x55,
        LogicalKey::V => 0x56,
        LogicalKey::W => 0x57,
        LogicalKey::X => 0x58,
        LogicalKey::Y => 0x59,
        LogicalKey::Z => 0x5A,
        LogicalKey::ContextMenu => 0x5D, // VK_APPS
        LogicalKey::F1 => 0x70,
        LogicalKey::F2 => 0x71,
        LogicalKey::F3 => 0x72,
        LogicalKey::F4 => 0x73,
        LogicalKey::F5 => 0x74,
        LogicalKey::F6 => 0x75,
        LogicalKey::F7 => 0x76,
        LogicalKey::F8 => 0x77,
        LogicalKey::F9 => 0x78,
        LogicalKey::F10 => 0x79,
        LogicalKey::F11 => 0x7A,
        LogicalKey::F12 => 0x7B,
        LogicalKey::F13 => 0x7C,
        LogicalKey::F14 => 0x7D,
        LogicalKey::F15 => 0x7E,
        LogicalKey::F16 => 0x7F,
        LogicalKey::F17 => 0x80,
        LogicalKey::F18 => 0x81,
        LogicalKey::F19 => 0x82,
        LogicalKey::F20 => 0x83,
        LogicalKey::F21 => 0x84,
        LogicalKey::F22 => 0x85,
        LogicalKey::F23 => 0x86,
        LogicalKey::F24 => 0x87,
        LogicalKey::ScrollLock => 0x91,     // VK_SCROLL
        LogicalKey::VolumeMute => 0xAD,     // VK_VOLUME_MUTE
        LogicalKey::VolumeDown => 0xAE,     // VK_VOLUME_DOWN
        LogicalKey::VolumeUp => 0xAF,       // VK_VOLUME_UP
        LogicalKey::MediaNextTrack => 0xB0, // VK_MEDIA_NEXT_TRACK
        LogicalKey::MediaPrevTrack => 0xB1, // VK_MEDIA_PREV_TRACK
        LogicalKey::MediaStop => 0xB2,      // VK_MEDIA_STOP
        LogicalKey::MediaPlayPause => 0xB3, // VK_MEDIA_PLAY_PAUSE
        LogicalKey::Semicolon => 0xBA,      // VK_OEM_1
        LogicalKey::Equal => 0xBB,          // VK_OEM_PLUS
        LogicalKey::Comma => 0xBC,          // VK_OEM_COMMA
        LogicalKey::Minus => 0xBD,          // VK_OEM_MINUS
        LogicalKey::Period => 0xBE,         // VK_OEM_PERIOD
        LogicalKey::Slash => 0xBF,          // VK_OEM_2
        LogicalKey::Grave => 0xC0,          // VK_OEM_3
        LogicalKey::BracketLeft => 0xDB,    // VK_OEM_4
        LogicalKey::Backslash => 0xDC,      // VK_OEM_5
        LogicalKey::BracketRight => 0xDD,   // VK_OEM_6
        LogicalKey::Quote => 0xDE,          // VK_OEM_7
    }
}

/// Maps a cross-platform `LogicalKey` to macOS Carbon/Quartz `CGKeyCode` or `NX_KEYTYPE_*`.
#[deny(unreachable_patterns)]
pub const fn to_macos_key(key: LogicalKey) -> MacOsKeyTarget {
    use MacOsKeyTarget::{MediaKey, VirtualKey};
    match key {
        // IOKit / hidsystem NX_KEYTYPE constants for macOS media & volume keys
        LogicalKey::VolumeUp => MediaKey(0),       // NX_KEYTYPE_SOUND_UP
        LogicalKey::VolumeDown => MediaKey(1),     // NX_KEYTYPE_SOUND_DOWN
        LogicalKey::VolumeMute => MediaKey(7),     // NX_KEYTYPE_MUTE
        LogicalKey::MediaPlayPause => MediaKey(16), // NX_KEYTYPE_PLAY
        LogicalKey::MediaNextTrack => MediaKey(17), // NX_KEYTYPE_NEXT
        LogicalKey::MediaPrevTrack => MediaKey(18), // NX_KEYTYPE_PREVIOUS
        LogicalKey::MediaStop => MediaKey(19),      // NX_KEYTYPE_FAST (or Stop)

        // ANSI Quartz Virtual Keycodes (Events.h)
        LogicalKey::A => VirtualKey(0x00),
        LogicalKey::S => VirtualKey(0x01),
        LogicalKey::D => VirtualKey(0x02),
        LogicalKey::F => VirtualKey(0x03),
        LogicalKey::H => VirtualKey(0x04),
        LogicalKey::G => VirtualKey(0x05),
        LogicalKey::Z => VirtualKey(0x06),
        LogicalKey::X => VirtualKey(0x07),
        LogicalKey::C => VirtualKey(0x08),
        LogicalKey::V => VirtualKey(0x09),
        LogicalKey::B => VirtualKey(0x0B),
        LogicalKey::Q => VirtualKey(0x0C),
        LogicalKey::W => VirtualKey(0x0D),
        LogicalKey::E => VirtualKey(0x0E),
        LogicalKey::R => VirtualKey(0x0F),
        LogicalKey::Y => VirtualKey(0x10),
        LogicalKey::T => VirtualKey(0x11),
        LogicalKey::Digit1 => VirtualKey(0x12),
        LogicalKey::Digit2 => VirtualKey(0x13),
        LogicalKey::Digit3 => VirtualKey(0x14),
        LogicalKey::Digit4 => VirtualKey(0x15),
        LogicalKey::Digit6 => VirtualKey(0x16),
        LogicalKey::Digit5 => VirtualKey(0x17),
        LogicalKey::Equal => VirtualKey(0x18),
        LogicalKey::Digit9 => VirtualKey(0x19),
        LogicalKey::Digit7 => VirtualKey(0x1A),
        LogicalKey::Minus => VirtualKey(0x1B),
        LogicalKey::Digit8 => VirtualKey(0x1C),
        LogicalKey::Digit0 => VirtualKey(0x1D),
        LogicalKey::BracketRight => VirtualKey(0x1E),
        LogicalKey::O => VirtualKey(0x1F),
        LogicalKey::U => VirtualKey(0x20),
        LogicalKey::BracketLeft => VirtualKey(0x21),
        LogicalKey::I => VirtualKey(0x22),
        LogicalKey::P => VirtualKey(0x23),
        LogicalKey::Enter => VirtualKey(0x24),
        LogicalKey::L => VirtualKey(0x25),
        LogicalKey::J => VirtualKey(0x26),
        LogicalKey::Quote => VirtualKey(0x27),
        LogicalKey::K => VirtualKey(0x28),
        LogicalKey::Semicolon => VirtualKey(0x29),
        LogicalKey::Backslash => VirtualKey(0x2A),
        LogicalKey::Comma => VirtualKey(0x2B),
        LogicalKey::Slash => VirtualKey(0x2C),
        LogicalKey::N => VirtualKey(0x2D),
        LogicalKey::M => VirtualKey(0x2E),
        LogicalKey::Period => VirtualKey(0x2F),
        LogicalKey::Tab => VirtualKey(0x30),
        LogicalKey::Space => VirtualKey(0x31),
        LogicalKey::Grave => VirtualKey(0x32),
        LogicalKey::Backspace => VirtualKey(0x33),
        LogicalKey::Escape => VirtualKey(0x35),
        LogicalKey::CapsLock => VirtualKey(0x39),
        LogicalKey::F17 => VirtualKey(0x40),
        LogicalKey::F18 => VirtualKey(0x4F),
        LogicalKey::F19 => VirtualKey(0x50),
        LogicalKey::F20 => VirtualKey(0x5A),
        LogicalKey::F5 => VirtualKey(0x60),
        LogicalKey::F6 => VirtualKey(0x61),
        LogicalKey::F7 => VirtualKey(0x62),
        LogicalKey::F3 => VirtualKey(0x63),
        LogicalKey::F8 => VirtualKey(0x64),
        LogicalKey::F9 => VirtualKey(0x65),
        LogicalKey::F11 => VirtualKey(0x67),
        LogicalKey::PrintScreen => VirtualKey(0x69), // kVK_F13 / PrintScreen on Mac keyboards
        LogicalKey::F16 => VirtualKey(0x6A),
        LogicalKey::ScrollLock => VirtualKey(0x6B),  // kVK_F14 / ScrollLock
        LogicalKey::F10 => VirtualKey(0x6D),
        LogicalKey::ContextMenu => VirtualKey(0x6E), // kVK_ContextualMenu
        LogicalKey::F12 => VirtualKey(0x6F),
        LogicalKey::Pause => VirtualKey(0x71),       // kVK_F15 / Pause
        LogicalKey::Insert => VirtualKey(0x72),      // kVK_Help / Insert
        LogicalKey::Home => VirtualKey(0x73),
        LogicalKey::PageUp => VirtualKey(0x74),
        LogicalKey::Delete => VirtualKey(0x75),      // kVK_ForwardDelete
        LogicalKey::F4 => VirtualKey(0x76),
        LogicalKey::End => VirtualKey(0x77),
        LogicalKey::F2 => VirtualKey(0x78),
        LogicalKey::PageDown => VirtualKey(0x79),
        LogicalKey::F1 => VirtualKey(0x7A),
        LogicalKey::ArrowLeft => VirtualKey(0x7B),
        LogicalKey::ArrowRight => VirtualKey(0x7C),
        LogicalKey::ArrowDown => VirtualKey(0x7D),
        LogicalKey::ArrowUp => VirtualKey(0x7E),
        LogicalKey::F13 => VirtualKey(0x80),
        LogicalKey::F14 => VirtualKey(0x81),
        LogicalKey::F15 => VirtualKey(0x82),
        LogicalKey::F21 => VirtualKey(0x83),
        LogicalKey::F22 => VirtualKey(0x84),
        LogicalKey::F23 => VirtualKey(0x85),
        LogicalKey::F24 => VirtualKey(0x86),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn test_linux_keycode_bijective_mapping() {
        let mut seen = HashSet::new();
        for &key in LogicalKey::ALL {
            let code = to_linux_keycode(key);
            assert!(
                seen.insert(code),
                "Duplicate Linux keycode {} for {:?}",
                code,
                key
            );
            assert_eq!(from_linux_keycode(code), Some(key));
        }
    }

    #[test]
    fn test_win32_vk_unique_mapping() {
        let mut seen = HashSet::new();
        for &key in LogicalKey::ALL {
            let vk = to_win32_vk(key);
            assert!(
                seen.insert(vk),
                "Duplicate Win32 VK {:#x} for {:?}",
                vk,
                key
            );
        }
    }

    #[test]
    fn test_macos_key_unique_mapping() {
        let mut seen = HashSet::new();
        for &key in LogicalKey::ALL {
            let target = to_macos_key(key);
            assert!(
                seen.insert(target),
                "Duplicate macOS key target {:?} for {:?}",
                target,
                key
            );
        }
    }
}
