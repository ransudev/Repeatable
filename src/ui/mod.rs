pub mod compact;
pub mod expanded;
pub mod macro_library;
pub mod settings;
pub mod theme;
pub mod timeline;
pub mod toolbar;

use crate::core::scheduler::LoopMode;

pub fn speed_label(speed: f32) -> &'static str {
    if (speed - 0.25).abs() < f32::EPSILON {
        "0.25×"
    } else if (speed - 0.5).abs() < f32::EPSILON {
        "0.5×"
    } else if (speed - 2.0).abs() < f32::EPSILON {
        "2×"
    } else if (speed - 4.0).abs() < f32::EPSILON {
        "4×"
    } else {
        "1×"
    }
}

pub fn loop_label(loop_mode: &LoopMode) -> String {
    match loop_mode {
        LoopMode::Once => "×1".to_owned(),
        LoopMode::Count(count) => format!("×{count}"),
        LoopMode::Infinite => "∞".to_owned(),
    }
}

pub fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_owned()
    } else {
        let mut out = value
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>();
        out.push('…');
        out
    }
}
