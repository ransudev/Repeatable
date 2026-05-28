use crate::core::scheduler::{next_loop, LoopMode};
use crate::models::{EventKind, InputEvent, MouseButton};
use enigo::{
    Axis, Button as EnigoButton, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings,
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
            if let Some(key) = key_from_rdev_debug(key) {
                let _ = enigo.key(key, Direction::Press);
            }
        }
        EventKind::KeyUp { key } => {
            if let Some(key) = key_from_rdev_debug(key) {
                let _ = enigo.key(key, Direction::Release);
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

fn key_from_rdev_debug(key: &str) -> Option<Key> {
    match key {
        "Space" => Some(Key::Space),
        "Tab" => Some(Key::Tab),
        "Return" | "Enter" => Some(Key::Return),
        "Escape" | "Esc" => Some(Key::Escape),
        "Backspace" => Some(Key::Backspace),
        "Delete" => Some(Key::Delete),
        "LeftArrow" | "ArrowLeft" => Some(Key::LeftArrow),
        "RightArrow" | "ArrowRight" => Some(Key::RightArrow),
        "UpArrow" | "ArrowUp" => Some(Key::UpArrow),
        "DownArrow" | "ArrowDown" => Some(Key::DownArrow),
        "Home" => Some(Key::Home),
        "End" => Some(Key::End),
        "PageUp" => Some(Key::PageUp),
        "PageDown" => Some(Key::PageDown),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "Insert" => Some(Key::Insert),
        "CapsLock" => Some(Key::CapsLock),
        "PrintScreen" => Some(Key::Print),
        "ScrollLock" => Some(Key::Scroll),
        "Pause" => Some(Key::Pause),
        "NumLock" => Some(Key::Numlock),
        "ControlLeft" => Some(Key::LControl),
        "ControlRight" => Some(Key::RControl),
        "Control" => Some(Key::Control),
        "ShiftLeft" => Some(Key::LShift),
        "ShiftRight" => Some(Key::RShift),
        "Shift" => Some(Key::Shift),
        "Alt" | "AltGr" => Some(Key::Alt),
        "MetaLeft" => Some(Key::LWin),
        "MetaRight" => Some(Key::RWin),
        "Meta" => Some(Key::Meta),
        "Minus" => Some(Key::OEMMinus),
        "Equal" => Some(Key::OEMPlus),
        "LeftBracket" => Some(Key::OEM4),
        "RightBracket" => Some(Key::OEM6),
        "BackSlash" | "Backslash" => Some(Key::OEM5),
        "IntlBackslash" => Some(Key::OEM102),
        "SemiColon" | "Semicolon" => Some(Key::OEM1),
        "Quote" => Some(Key::OEM7),
        "Comma" => Some(Key::OEMComma),
        "Dot" | "Period" => Some(Key::OEMPeriod),
        "Slash" => Some(Key::OEM2),
        "Grave" | "BackQuote" => Some(Key::OEM3),
        "KpPlus" => Some(Key::Add),
        "KpMinus" => Some(Key::Subtract),
        "KpMultiply" => Some(Key::Multiply),
        "KpDivide" => Some(Key::Divide),
        "KpDelete" => Some(Key::Decimal),
        "KpReturn" => Some(Key::Return),
        _ => letter_or_digit_key(key),
    }
}

fn letter_or_digit_key(key: &str) -> Option<Key> {
    if let Some(letter) = key.strip_prefix("Key") {
        if letter.len() == 1 {
            return match letter.chars().next()?.to_ascii_uppercase() {
                'A' => Some(Key::A),
                'B' => Some(Key::B),
                'C' => Some(Key::C),
                'D' => Some(Key::D),
                'E' => Some(Key::E),
                'F' => Some(Key::F),
                'G' => Some(Key::G),
                'H' => Some(Key::H),
                'I' => Some(Key::I),
                'J' => Some(Key::J),
                'K' => Some(Key::K),
                'L' => Some(Key::L),
                'M' => Some(Key::M),
                'N' => Some(Key::N),
                'O' => Some(Key::O),
                'P' => Some(Key::P),
                'Q' => Some(Key::Q),
                'R' => Some(Key::R),
                'S' => Some(Key::S),
                'T' => Some(Key::T),
                'U' => Some(Key::U),
                'V' => Some(Key::V),
                'W' => Some(Key::W),
                'X' => Some(Key::X),
                'Y' => Some(Key::Y),
                'Z' => Some(Key::Z),
                _ => None,
            };
        }
    }

    if let Some(digit) = key.strip_prefix("Num") {
        if digit.len() == 1 && digit.chars().all(|ch| ch.is_ascii_digit()) {
            return match digit.chars().next()? {
                '0' => Some(Key::Num0),
                '1' => Some(Key::Num1),
                '2' => Some(Key::Num2),
                '3' => Some(Key::Num3),
                '4' => Some(Key::Num4),
                '5' => Some(Key::Num5),
                '6' => Some(Key::Num6),
                '7' => Some(Key::Num7),
                '8' => Some(Key::Num8),
                '9' => Some(Key::Num9),
                _ => None,
            };
        }
    }

    if let Some(digit) = key.strip_prefix("Kp") {
        if digit.len() == 1 && digit.chars().all(|ch| ch.is_ascii_digit()) {
            return match digit.chars().next()? {
                '0' => Some(Key::Numpad0),
                '1' => Some(Key::Numpad1),
                '2' => Some(Key::Numpad2),
                '3' => Some(Key::Numpad3),
                '4' => Some(Key::Numpad4),
                '5' => Some(Key::Numpad5),
                '6' => Some(Key::Numpad6),
                '7' => Some(Key::Numpad7),
                '8' => Some(Key::Numpad8),
                '9' => Some(Key::Numpad9),
                _ => None,
            };
        }
    }

    if key.len() == 1 {
        return key.chars().next().map(Key::Unicode);
    }

    None
}
