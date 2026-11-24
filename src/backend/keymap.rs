use crate::model::LogicalKey;

/// macOS distinguishes between standard Quartz virtual keycodes (`CGKeyCode`)
/// and system-defined media/volume keys (`NX_KEYTYPE_*` via `NSSystemDefined` events).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacOsKeyTarget {
    VirtualKey(u16),
    MediaKey(u8),
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
