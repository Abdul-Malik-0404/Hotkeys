use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
        meta: false,
    };

    pub const fn new(ctrl: bool, alt: bool, shift: bool, meta: bool) -> Self {
        Self {
            ctrl,
            alt,
            shift,
            meta,
        }
    }

    pub const fn is_empty(&self) -> bool {
        !self.ctrl && !self.alt && !self.shift && !self.meta
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Super");
        }
        write!(f, "{}", parts.join(" + "))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalKey {
    // Media & Volume
    MediaPlayPause,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    VolumeUp,
    VolumeDown,
    VolumeMute,

    // Function Keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Digits
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    // Navigation & Editing
    Escape,
    Tab,
    CapsLock,
    Space,
    Enter,
    Backspace,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Symbols & Special
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Grave,
    Comma,
    Period,
    Slash,
    ContextMenu,
    PrintScreen,
    ScrollLock,
    Pause,
}

impl LogicalKey {
    pub const ALL: &'static [LogicalKey] = &[
        // Media & Volume
        Self::MediaPlayPause,
        Self::MediaNextTrack,
        Self::MediaPrevTrack,
        Self::MediaStop,
        Self::VolumeUp,
        Self::VolumeDown,
        Self::VolumeMute,
        // Function Keys
        Self::F1,
        Self::F2,
        Self::F3,
        Self::F4,
        Self::F5,
        Self::F6,
        Self::F7,
        Self::F8,
        Self::F9,
        Self::F10,
        Self::F11,
        Self::F12,
        Self::F13,
        Self::F14,
        Self::F15,
        Self::F16,
        Self::F17,
        Self::F18,
        Self::F19,
        Self::F20,
        Self::F21,
        Self::F22,
        Self::F23,
        Self::F24,
        // Letters
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
        Self::I,
        Self::J,
        Self::K,
        Self::L,
        Self::M,
        Self::N,
        Self::O,
        Self::P,
        Self::Q,
        Self::R,
        Self::S,
        Self::T,
        Self::U,
        Self::V,
        Self::W,
        Self::X,
        Self::Y,
        Self::Z,
        // Digits
        Self::Digit0,
        Self::Digit1,
        Self::Digit2,
        Self::Digit3,
        Self::Digit4,
        Self::Digit5,
        Self::Digit6,
        Self::Digit7,
        Self::Digit8,
        Self::Digit9,
        // Navigation & Editing
        Self::Escape,
        Self::Tab,
        Self::CapsLock,
        Self::Space,
        Self::Enter,
        Self::Backspace,
        Self::Insert,
        Self::Delete,
        Self::Home,
        Self::End,
        Self::PageUp,
        Self::PageDown,
        Self::ArrowUp,
        Self::ArrowDown,
        Self::ArrowLeft,
        Self::ArrowRight,
        // Symbols & Special
        Self::Minus,
        Self::Equal,
        Self::BracketLeft,
        Self::BracketRight,
        Self::Backslash,
        Self::Semicolon,
        Self::Quote,
        Self::Grave,
        Self::Comma,
        Self::Period,
        Self::Slash,
        Self::ContextMenu,
        Self::PrintScreen,
        Self::ScrollLock,
        Self::Pause,
    ];

    pub const fn label(&self) -> &'static str {
        match self {
            Self::MediaPlayPause => "Media Play/Pause",
            Self::MediaNextTrack => "Media Next Track",
            Self::MediaPrevTrack => "Media Prev Track",
            Self::MediaStop => "Media Stop",
            Self::VolumeUp => "Volume Up",
            Self::VolumeDown => "Volume Down",
            Self::VolumeMute => "Volume Mute",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::F13 => "F13",
            Self::F14 => "F14",
            Self::F15 => "F15",
            Self::F16 => "F16",
            Self::F17 => "F17",
            Self::F18 => "F18",
            Self::F19 => "F19",
            Self::F20 => "F20",
            Self::F21 => "F21",
            Self::F22 => "F22",
            Self::F23 => "F23",
            Self::F24 => "F24",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::Escape => "Esc",
            Self::Tab => "Tab",
            Self::CapsLock => "CapsLock",
            Self::Space => "Space",
            Self::Enter => "Enter",
            Self::Backspace => "Backspace",
            Self::Insert => "Insert",
            Self::Delete => "Delete",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::ArrowUp => "Up",
            Self::ArrowDown => "Down",
            Self::ArrowLeft => "Left",
            Self::ArrowRight => "Right",
            Self::Minus => "-",
            Self::Equal => "=",
            Self::BracketLeft => "[",
            Self::BracketRight => "]",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::Grave => "`",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::ContextMenu => "Menu",
            Self::PrintScreen => "PrintScreen",
            Self::ScrollLock => "ScrollLock",
            Self::Pause => "Pause",
        }
    }

    pub const fn category(&self) -> &'static str {
        match self {
            Self::MediaPlayPause
            | Self::MediaNextTrack
            | Self::MediaPrevTrack
            | Self::MediaStop
            | Self::VolumeUp
            | Self::VolumeDown
            | Self::VolumeMute => "Media & Volume",
            Self::F1
            | Self::F2
            | Self::F3
            | Self::F4
            | Self::F5
            | Self::F6
            | Self::F7
            | Self::F8
            | Self::F9
            | Self::F10
            | Self::F11
            | Self::F12
            | Self::F13
            | Self::F14
            | Self::F15
            | Self::F16
            | Self::F17
            | Self::F18
            | Self::F19
            | Self::F20
            | Self::F21
            | Self::F22
            | Self::F23
            | Self::F24 => "Function Keys",
            Self::A
            | Self::B
            | Self::C
            | Self::D
            | Self::E
            | Self::F
            | Self::G
            | Self::H
            | Self::I
            | Self::J
            | Self::K
            | Self::L
            | Self::M
            | Self::N
            | Self::O
            | Self::P
            | Self::Q
            | Self::R
            | Self::S
            | Self::T
            | Self::U
            | Self::V
            | Self::W
            | Self::X
            | Self::Y
            | Self::Z => "Letters",
            Self::Digit0
            | Self::Digit1
            | Self::Digit2
            | Self::Digit3
            | Self::Digit4
            | Self::Digit5
            | Self::Digit6
            | Self::Digit7
            | Self::Digit8
            | Self::Digit9 => "Digits",
            Self::Escape
            | Self::Tab
            | Self::CapsLock
            | Self::Space
            | Self::Enter
            | Self::Backspace
            | Self::Insert
            | Self::Delete
            | Self::Home
            | Self::End
            | Self::PageUp
            | Self::PageDown
            | Self::ArrowUp
            | Self::ArrowDown
            | Self::ArrowLeft
            | Self::ArrowRight => "Navigation & Editing",
            Self::Minus
            | Self::Equal
            | Self::BracketLeft
            | Self::BracketRight
            | Self::Backslash
            | Self::Semicolon
            | Self::Quote
            | Self::Grave
            | Self::Comma
            | Self::Period
            | Self::Slash
            | Self::ContextMenu
            | Self::PrintScreen
            | Self::ScrollLock
            | Self::Pause => "Symbols & Special",
        }
    }

    pub fn from_egui_key(key: egui::Key) -> Option<Self> {
        use egui::Key;
        Some(match key {
            Key::ArrowDown => Self::ArrowDown,
            Key::ArrowLeft => Self::ArrowLeft,
            Key::ArrowRight => Self::ArrowRight,
            Key::ArrowUp => Self::ArrowUp,
            Key::Escape => Self::Escape,
            Key::Tab => Self::Tab,
            Key::Backspace => Self::Backspace,
            Key::Enter => Self::Enter,
            Key::Space => Self::Space,
            Key::Insert => Self::Insert,
            Key::Delete => Self::Delete,
            Key::Home => Self::Home,
            Key::End => Self::End,
            Key::PageUp => Self::PageUp,
            Key::PageDown => Self::PageDown,
            Key::Minus => Self::Minus,
            Key::Equals => Self::Equal,
            Key::OpenBracket => Self::BracketLeft,
            Key::CloseBracket => Self::BracketRight,
            Key::Backslash => Self::Backslash,
            Key::Semicolon => Self::Semicolon,
            Key::Quote => Self::Quote,
            Key::Backtick => Self::Grave,
            Key::Comma => Self::Comma,
            Key::Period => Self::Period,
            Key::Slash => Self::Slash,
            Key::Num0 => Self::Digit0,
            Key::Num1 => Self::Digit1,
            Key::Num2 => Self::Digit2,
            Key::Num3 => Self::Digit3,
            Key::Num4 => Self::Digit4,
            Key::Num5 => Self::Digit5,
            Key::Num6 => Self::Digit6,
            Key::Num7 => Self::Digit7,
            Key::Num8 => Self::Digit8,
            Key::Num9 => Self::Digit9,
            Key::A => Self::A,
            Key::B => Self::B,
            Key::C => Self::C,
            Key::D => Self::D,
            Key::E => Self::E,
            Key::F => Self::F,
            Key::G => Self::G,
            Key::H => Self::H,
            Key::I => Self::I,
            Key::J => Self::J,
            Key::K => Self::K,
            Key::L => Self::L,
            Key::M => Self::M,
            Key::N => Self::N,
            Key::O => Self::O,
            Key::P => Self::P,
            Key::Q => Self::Q,
            Key::R => Self::R,
            Key::S => Self::S,
            Key::T => Self::T,
            Key::U => Self::U,
            Key::V => Self::V,
            Key::W => Self::W,
            Key::X => Self::X,
            Key::Y => Self::Y,
            Key::Z => Self::Z,
            Key::F1 => Self::F1,
            Key::F2 => Self::F2,
            Key::F3 => Self::F3,
            Key::F4 => Self::F4,
            Key::F5 => Self::F5,
            Key::F6 => Self::F6,
            Key::F7 => Self::F7,
            Key::F8 => Self::F8,
            Key::F9 => Self::F9,
            Key::F10 => Self::F10,
            Key::F11 => Self::F11,
            Key::F12 => Self::F12,
            Key::F13 => Self::F13,
            Key::F14 => Self::F14,
            Key::F15 => Self::F15,
            Key::F16 => Self::F16,
            Key::F17 => Self::F17,
            Key::F18 => Self::F18,
            Key::F19 => Self::F19,
            Key::F20 => Self::F20,
            Key::F21 => Self::F21,
            Key::F22 => Self::F22,
            Key::F23 => Self::F23,
            Key::F24 => Self::F24,
            _ => return None,
        })
    }
}

impl fmt::Display for LogicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub modifiers: Modifiers,
    pub key: LogicalKey,
}

impl KeyCombo {
    pub const fn new(modifiers: Modifiers, key: LogicalKey) -> Self {
        Self { modifiers, key }
    }

    pub const fn bare(key: LogicalKey) -> Self {
        Self {
            modifiers: Modifiers::NONE,
            key,
        }
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{} + {}", self.modifiers, self.key)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeybindRule {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub trigger: KeyCombo,
    pub target: KeyCombo,
}

impl KeybindRule {
    pub fn new(name: impl Into<String>, trigger: KeyCombo, target: KeyCombo) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            enabled: true,
            trigger,
            target,
        }
    }
}
