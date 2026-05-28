use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub timestamp_ms: u64,
    #[serde(flatten)]
    pub kind: EventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventKind {
    MouseMove { x: i32, y: i32 },
    MouseDown { button: MouseButton },
    MouseUp { button: MouseButton },
    Scroll { delta_x: i32, delta_y: i32 },
    KeyDown { key: String },
    KeyUp { key: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineColorHint {
    Mouse,
    Key,
    Scroll,
}

impl InputEvent {
    pub fn new(timestamp_ms: u64, kind: EventKind) -> Self {
        Self { timestamp_ms, kind }
    }

    pub fn label(&self) -> String {
        match &self.kind {
            EventKind::MouseMove { x, y } => format!("Mouse move to {x}, {y}"),
            EventKind::MouseDown { button } => format!("Mouse {button} down"),
            EventKind::MouseUp { button } => format!("Mouse {button} up"),
            EventKind::Scroll { delta_x, delta_y } => {
                format!("Scroll Δx {delta_x}, Δy {delta_y}")
            }
            EventKind::KeyDown { key } => format!("Key {key} down"),
            EventKind::KeyUp { key } => format!("Key {key} up"),
        }
    }

    pub fn color_hint(&self) -> TimelineColorHint {
        match self.kind {
            EventKind::MouseMove { .. }
            | EventKind::MouseDown { .. }
            | EventKind::MouseUp { .. } => TimelineColorHint::Mouse,
            EventKind::Scroll { .. } => TimelineColorHint::Scroll,
            EventKind::KeyDown { .. } | EventKind::KeyUp { .. } => TimelineColorHint::Key,
        }
    }
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseButton::Left => write!(f, "left"),
            MouseButton::Right => write!(f, "right"),
            MouseButton::Middle => write!(f, "middle"),
        }
    }
}
