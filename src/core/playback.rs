use crate::core::scheduler::{next_loop, LoopMode};
use crate::models::{EventKind, InputEvent, MouseButton};
use enigo::{
    Axis, Button as EnigoButton, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings, EXT,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

pub async fn run_playback(
    events: Vec<InputEvent>,
    mut loop_mode: LoopMode,
    speed: f32,
    stop_flag: Arc<AtomicBool>,
    is_playing_flag: Arc<AtomicBool>,
    mut pause_rx: watch::Receiver<bool>,
) {
    is_playing_flag.store(true, Ordering::SeqCst);

    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(error) => {
            eprintln!("Repeatable failed to initialize enigo: {error:?}");
            finish_playback(&is_playing_flag);
            return;
        }
    };

    if events.is_empty() {
        finish_playback(&is_playing_flag);
        return;
    }

    'playback: loop {
        let mut previous_timestamp = 0;

        for event in &events {
            if stop_flag.load(Ordering::SeqCst) {
                break 'playback;
            }

            if !wait_until_unpaused(&stop_flag, &mut pause_rx).await {
                break 'playback;
            }

            let delta_ms = event.timestamp_ms.saturating_sub(previous_timestamp);
            previous_timestamp = event.timestamp_ms;

            if !sleep_scaled(delta_ms, speed, &stop_flag, &mut pause_rx).await {
                break 'playback;
            }

            if stop_flag.load(Ordering::SeqCst) {
                break 'playback;
            }

            dispatch_event(&mut enigo, event);
        }

        if !next_loop(&mut loop_mode) {
            break;
        }
    }

    finish_playback(&is_playing_flag);
}

fn finish_playback(is_playing_flag: &Arc<AtomicBool>) {
    is_playing_flag.store(false, Ordering::SeqCst);
}

async fn wait_until_unpaused(
    stop_flag: &Arc<AtomicBool>,
    pause_rx: &mut watch::Receiver<bool>,
) -> bool {
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            return false;
        }

        if !*pause_rx.borrow() {
            return true;
        }

        if pause_rx.changed().await.is_err() {
            return false;
        }
    }
}

async fn sleep_scaled(
    delta_ms: u64,
    speed: f32,
    stop_flag: &Arc<AtomicBool>,
    pause_rx: &mut watch::Receiver<bool>,
) -> bool {
    let scaled_ms = (delta_ms as f64 / f64::from(speed.max(0.01))).max(0.0);
    let total = Duration::from_secs_f64(scaled_ms / 1000.0);
    let chunk = Duration::from_millis(20);
    let mut slept = Duration::ZERO;

    while slept < total {
        if stop_flag.load(Ordering::SeqCst) {
            return false;
        }
        if !wait_until_unpaused(stop_flag, pause_rx).await {
            return false;
        }

        let remaining = total.saturating_sub(slept);
        let next = remaining.min(chunk);
        tokio::time::sleep(next).await;
        slept += next;
    }

    true
}

fn dispatch_event(enigo: &mut Enigo, event: &InputEvent) {
    match &event.kind {
        EventKind::MouseMove { x, y } => {
            let _ = enigo.move_mouse(*x, *y, Coordinate::Abs);
        }
        EventKind::MouseDown { button } => {
            let _ = enigo.button(map_mouse_button(*button), Direction::Press);
        }
        EventKind::MouseUp { button } => {
            let _ = enigo.button(map_mouse_button(*button), Direction::Release);
        }
        EventKind::Scroll { delta_x, delta_y } => {
            if *delta_x != 0 {
                let _ = enigo.scroll(*delta_x, Axis::Horizontal);
            }
            if *delta_y != 0 {
                let _ = enigo.scroll(*delta_y, Axis::Vertical);
            }
        }
        EventKind::KeyDown { key } => {
            if let Some(scancode) = key_to_scancode(key) {
                let _ = enigo.raw(scancode, Direction::Press);
            }
        }
        EventKind::KeyUp { key } => {
            if let Some(scancode) = key_to_scancode(key) {
                let _ = enigo.raw(scancode, Direction::Release);
            }
        }
    }
}

fn map_mouse_button(button: MouseButton) -> EnigoButton {
    match button {
        MouseButton::Left => EnigoButton::Left,
        MouseButton::Right => EnigoButton::Right,
        MouseButton::Middle => EnigoButton::Middle,
    }
}

fn key_to_scancode(key: &str) -> Option<u16> {
    match key {
        "Escape" | "Esc" => Some(0x01),
        "Num1" => Some(0x02),
        "Num2" => Some(0x03),
        "Num3" => Some(0x04),
        "Num4" => Some(0x05),
        "Num5" => Some(0x06),
        "Num6" => Some(0x07),
        "Num7" => Some(0x08),
        "Num8" => Some(0x09),
        "Num9" => Some(0x0A),
        "Num0" => Some(0x0B),
        "Minus" => Some(0x0C),
        "Equal" => Some(0x0D),
        "Backspace" => Some(0x0E),
        "Tab" => Some(0x0F),
        "KeyQ" => Some(0x10),
        "KeyW" => Some(0x11),
        "KeyE" => Some(0x12),
        "KeyR" => Some(0x13),
        "KeyT" => Some(0x14),
        "KeyY" => Some(0x15),
        "KeyU" => Some(0x16),
        "KeyI" => Some(0x17),
        "KeyO" => Some(0x18),
        "KeyP" => Some(0x19),
        "LeftBracket" => Some(0x1A),
        "RightBracket" => Some(0x1B),
        "Return" | "Enter" => Some(0x1C),
        "ControlLeft" | "Control" => Some(0x1D),
        "KeyA" => Some(0x1E),
        "KeyS" => Some(0x1F),
        "KeyD" => Some(0x20),
        "KeyF" => Some(0x21),
        "KeyG" => Some(0x22),
        "KeyH" => Some(0x23),
        "KeyJ" => Some(0x24),
        "KeyK" => Some(0x25),
        "KeyL" => Some(0x26),
        "SemiColon" | "Semicolon" => Some(0x27),
        "Quote" => Some(0x28),
        "Grave" | "BackQuote" => Some(0x29),
        "ShiftLeft" | "Shift" => Some(0x2A),
        "BackSlash" | "Backslash" => Some(0x2B),
        "KeyZ" => Some(0x2C),
        "KeyX" => Some(0x2D),
        "KeyC" => Some(0x2E),
        "KeyV" => Some(0x2F),
        "KeyB" => Some(0x30),
        "KeyN" => Some(0x31),
        "KeyM" => Some(0x32),
        "Comma" => Some(0x33),
        "Dot" | "Period" => Some(0x34),
        "Slash" => Some(0x35),
        "ShiftRight" => Some(0x36),
        "KpMultiply" => Some(0x37),
        "Alt" => Some(0x38),
        "Space" => Some(0x39),
        "CapsLock" => Some(0x3A),
        "F1" => Some(0x3B),
        "F2" => Some(0x3C),
        "F3" => Some(0x3D),
        "F4" => Some(0x3E),
        "F5" => Some(0x3F),
        "F6" => Some(0x40),
        "F7" => Some(0x41),
        "F8" => Some(0x42),
        "F9" => Some(0x43),
        "F10" => Some(0x44),
        "NumLock" => Some(0x45),
        "ScrollLock" => Some(0x46),
        "Kp7" => Some(0x47),
        "Kp8" => Some(0x48),
        "Kp9" => Some(0x49),
        "KpMinus" => Some(0x4A),
        "Kp4" => Some(0x4B),
        "Kp5" => Some(0x4C),
        "Kp6" => Some(0x4D),
        "KpPlus" => Some(0x4E),
        "Kp1" => Some(0x4F),
        "Kp2" => Some(0x50),
        "Kp3" => Some(0x51),
        "Kp0" => Some(0x52),
        "KpDelete" => Some(0x53),
        "IntlBackslash" => Some(0x56),
        "F11" => Some(0x57),
        "F12" => Some(0x58),
        "KpReturn" => Some(0x1C | EXT),
        "ControlRight" => Some(0x1D | EXT),
        "KpDivide" => Some(0x35 | EXT),
        "PrintScreen" => Some(0x37 | EXT),
        "AltGr" => Some(0x38 | EXT),
        "Home" => Some(0x47 | EXT),
        "UpArrow" | "ArrowUp" => Some(0x48 | EXT),
        "PageUp" => Some(0x49 | EXT),
        "LeftArrow" | "ArrowLeft" => Some(0x4B | EXT),
        "RightArrow" | "ArrowRight" => Some(0x4D | EXT),
        "End" => Some(0x4F | EXT),
        "DownArrow" | "ArrowDown" => Some(0x50 | EXT),
        "PageDown" => Some(0x51 | EXT),
        "Insert" => Some(0x52 | EXT),
        "Delete" => Some(0x53 | EXT),
        "MetaLeft" | "Meta" => Some(0x5B | EXT),
        "MetaRight" => Some(0x5C | EXT),
        "Pause" => Some(0x45 | EXT),
        _ => None,
    }
}
