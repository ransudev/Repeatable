use crate::core::hotkeys;
use crate::models::{EventKind, HotkeyConfig, InputEvent, MouseButton};
use crate::state::AppCommand;
use rdev::{listen, Button as RdevButton, Event, EventType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone)]
struct RecorderControl {
    hotkeys: HotkeyConfig,
    rebinding_action: Option<String>,
    recording_origin: Option<Instant>,
    last_mouse_time: Option<Instant>,
}

#[derive(Clone)]
pub struct RecorderControlHandle {
    inner: Arc<Mutex<RecorderControl>>,
}

impl RecorderControlHandle {
    pub fn new(hotkeys: HotkeyConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RecorderControl {
                hotkeys,
                rebinding_action: None,
                recording_origin: None,
                last_mouse_time: None,
            })),
        }
    }

    pub fn set_hotkeys(&self, hotkeys: HotkeyConfig) {
        if let Ok(mut control) = self.inner.lock() {
            control.hotkeys = hotkeys;
        }
    }

    pub fn set_rebinding_action(&self, action: Option<String>) {
        if let Ok(mut control) = self.inner.lock() {
            control.rebinding_action = action;
        }
    }

    pub fn begin_recording(&self) {
        if let Ok(mut control) = self.inner.lock() {
            control.recording_origin = Some(Instant::now());
            control.last_mouse_time = None;
        }
    }

    pub fn end_recording(&self) {
        if let Ok(mut control) = self.inner.lock() {
            control.recording_origin = None;
            control.last_mouse_time = None;
        }
    }

    pub fn update_last_mouse_time(&self, t: Instant) {
        if let Ok(mut control) = self.inner.lock() {
            control.last_mouse_time = Some(t);
        }
    }

    pub fn recording_timestamp_ms(&self) -> Option<u64> {
        timestamp_ms(&self.snapshot())
    }

    fn snapshot(&self) -> RecorderControl {
        self.inner
            .lock()
            .map(|control| control.clone())
            .unwrap_or_else(|_| RecorderControl {
                hotkeys: HotkeyConfig::default(),
                rebinding_action: None,
                recording_origin: None,
                last_mouse_time: None,
            })
    }
}

pub fn spawn_recorder(
    sender: Sender<AppCommand>,
    is_playing_flag: Arc<AtomicBool>,
    control: RecorderControlHandle,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        create_windows_message_queue();

        let listen_result = listen(move |event| {
            handle_event(&sender, &is_playing_flag, &control, event);
        });

        if let Err(error) = listen_result {
            eprintln!("Repeatable recorder listener stopped: {error:?}");
        }
    })
}

#[cfg(target_os = "windows")]
fn create_windows_message_queue() {
    use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, MSG, PM_NOREMOVE};

    // rdev installs a Windows hook on this thread. Touching the thread's
    // message queue first ensures keyboard events are delivered even when no
    // egui text input currently has focus.
    unsafe {
        let mut msg = std::mem::zeroed::<MSG>();
        let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
    }
}

#[cfg(not(target_os = "windows"))]
fn create_windows_message_queue() {}

fn handle_event(
    sender: &Sender<AppCommand>,
    is_playing_flag: &Arc<AtomicBool>,
    control: &RecorderControlHandle,
    event: Event,
) {
    match event.event_type {
        EventType::KeyPress(key) => {
            let key_string = rdev_key_to_string(&key);
            process_key_press(sender, is_playing_flag, control, key_string);
        }
        EventType::KeyRelease(key) => {
            let key_string = rdev_key_to_string(&key);
            process_key_release(sender, is_playing_flag, control, key_string);
        }
        EventType::ButtonPress(button) => {
            if is_playing_flag.load(Ordering::SeqCst) {
                return;
            }

            let snapshot = control.snapshot();
            let Some(timestamp_ms) = timestamp_ms(&snapshot) else {
                return;
            };
            if let Some(button) = map_mouse_button(button) {
                let _ = sender.send(AppCommand::RecordedEvent(InputEvent::new(
                    timestamp_ms,
                    EventKind::MouseDown { button },
                )));
            }
        }
        EventType::ButtonRelease(button) => {
            if is_playing_flag.load(Ordering::SeqCst) {
                return;
            }

            let snapshot = control.snapshot();
            let Some(timestamp_ms) = timestamp_ms(&snapshot) else {
                return;
            };
            if let Some(button) = map_mouse_button(button) {
                let _ = sender.send(AppCommand::RecordedEvent(InputEvent::new(
                    timestamp_ms,
                    EventKind::MouseUp { button },
                )));
            }
        }
        EventType::MouseMove { x, y } => {
            if is_playing_flag.load(Ordering::SeqCst) {
                return;
            }

            let snapshot = control.snapshot();
            let Some(timestamp_ms) = timestamp_ms(&snapshot) else {
                return;
            };

            let now = Instant::now();
            if let Some(last) = snapshot.last_mouse_time {
                if now.duration_since(last).as_millis() < 16 {
                    return;
                }
            }
            control.update_last_mouse_time(now);

            let _ = sender.send(AppCommand::RecordedEvent(InputEvent::new(
                timestamp_ms,
                EventKind::MouseMove {
                    x: f64_to_i32(x),
                    y: f64_to_i32(y),
                },
            )));
        }
        EventType::Wheel { delta_x, delta_y } => {
            if is_playing_flag.load(Ordering::SeqCst) {
                return;
            }

            let snapshot = control.snapshot();
            let Some(timestamp_ms) = timestamp_ms(&snapshot) else {
                return;
            };
            let _ = sender.send(AppCommand::RecordedEvent(InputEvent::new(
                timestamp_ms,
                EventKind::Scroll {
                    delta_x: i64_to_i32(delta_x),
                    delta_y: i64_to_i32(delta_y),
                },
            )));
        }
    }
}

fn process_key_press(
    sender: &Sender<AppCommand>,
    is_playing_flag: &Arc<AtomicBool>,
    control: &RecorderControlHandle,
    key_string: String,
) {
    let snapshot = control.snapshot();

    if snapshot.rebinding_action.is_some() {
        let _ = sender.send(AppCommand::RebindCapture(key_string));
        return;
    }

    if let Some(command) = hotkeys::command_for_key_press(&key_string, &snapshot.hotkeys) {
        let _ = sender.send(command);
        return;
    }

    if is_playing_flag.load(Ordering::SeqCst) {
        return;
    }

    let Some(timestamp_ms) = timestamp_ms(&snapshot) else {
        return;
    };

    let _ = sender.send(AppCommand::RecordedEvent(InputEvent::new(
        timestamp_ms,
        EventKind::KeyDown { key: key_string },
    )));
}

fn process_key_release(
    sender: &Sender<AppCommand>,
    is_playing_flag: &Arc<AtomicBool>,
    control: &RecorderControlHandle,
    key_string: String,
) {
    let snapshot = control.snapshot();

    if snapshot.rebinding_action.is_some() || hotkeys::is_hotkey(&key_string, &snapshot.hotkeys) {
        return;
    }

    if is_playing_flag.load(Ordering::SeqCst) {
        return;
    }

    let Some(timestamp_ms) = timestamp_ms(&snapshot) else {
        return;
    };

    let _ = sender.send(AppCommand::RecordedEvent(InputEvent::new(
        timestamp_ms,
        EventKind::KeyUp { key: key_string },
    )));
}

fn rdev_key_to_string(key: &rdev::Key) -> String {
    match key {
        rdev::Key::F1 => "F1".into(),
        rdev::Key::F2 => "F2".into(),
        rdev::Key::F3 => "F3".into(),
        rdev::Key::F4 => "F4".into(),
        rdev::Key::F5 => "F5".into(),
        rdev::Key::F6 => "F6".into(),
        rdev::Key::F7 => "F7".into(),
        rdev::Key::F8 => "F8".into(),
        rdev::Key::F9 => "F9".into(),
        rdev::Key::F10 => "F10".into(),
        rdev::Key::F11 => "F11".into(),
        rdev::Key::F12 => "F12".into(),
        rdev::Key::Return => "Return".into(),
        rdev::Key::Escape => "Escape".into(),
        rdev::Key::Backspace => "Backspace".into(),
        rdev::Key::Tab => "Tab".into(),
        rdev::Key::Space => "Space".into(),
        rdev::Key::Delete => "Delete".into(),
        rdev::Key::Insert => "Insert".into(),
        rdev::Key::Home => "Home".into(),
        rdev::Key::End => "End".into(),
        rdev::Key::PageUp => "PageUp".into(),
        rdev::Key::PageDown => "PageDown".into(),
        rdev::Key::UpArrow => "UpArrow".into(),
        rdev::Key::DownArrow => "DownArrow".into(),
        rdev::Key::LeftArrow => "LeftArrow".into(),
        rdev::Key::RightArrow => "RightArrow".into(),
        rdev::Key::CapsLock => "CapsLock".into(),
        rdev::Key::ShiftLeft => "ShiftLeft".into(),
        rdev::Key::ShiftRight => "ShiftRight".into(),
        rdev::Key::ControlLeft => "ControlLeft".into(),
        rdev::Key::ControlRight => "ControlRight".into(),
        rdev::Key::Alt => "Alt".into(),
        rdev::Key::AltGr => "AltGr".into(),
        rdev::Key::MetaLeft => "MetaLeft".into(),
        rdev::Key::MetaRight => "MetaRight".into(),
        rdev::Key::PrintScreen => "PrintScreen".into(),
        rdev::Key::ScrollLock => "ScrollLock".into(),
        rdev::Key::Pause => "Pause".into(),
        rdev::Key::NumLock => "NumLock".into(),
        rdev::Key::KeyA => "KeyA".into(),
        rdev::Key::KeyB => "KeyB".into(),
        rdev::Key::KeyC => "KeyC".into(),
        rdev::Key::KeyD => "KeyD".into(),
        rdev::Key::KeyE => "KeyE".into(),
        rdev::Key::KeyF => "KeyF".into(),
        rdev::Key::KeyG => "KeyG".into(),
        rdev::Key::KeyH => "KeyH".into(),
        rdev::Key::KeyI => "KeyI".into(),
        rdev::Key::KeyJ => "KeyJ".into(),
        rdev::Key::KeyK => "KeyK".into(),
        rdev::Key::KeyL => "KeyL".into(),
        rdev::Key::KeyM => "KeyM".into(),
        rdev::Key::KeyN => "KeyN".into(),
        rdev::Key::KeyO => "KeyO".into(),
        rdev::Key::KeyP => "KeyP".into(),
        rdev::Key::KeyQ => "KeyQ".into(),
        rdev::Key::KeyR => "KeyR".into(),
        rdev::Key::KeyS => "KeyS".into(),
        rdev::Key::KeyT => "KeyT".into(),
        rdev::Key::KeyU => "KeyU".into(),
        rdev::Key::KeyV => "KeyV".into(),
        rdev::Key::KeyW => "KeyW".into(),
        rdev::Key::KeyX => "KeyX".into(),
        rdev::Key::KeyY => "KeyY".into(),
        rdev::Key::KeyZ => "KeyZ".into(),
        rdev::Key::Num0 => "Num0".into(),
        rdev::Key::Num1 => "Num1".into(),
        rdev::Key::Num2 => "Num2".into(),
        rdev::Key::Num3 => "Num3".into(),
        rdev::Key::Num4 => "Num4".into(),
        rdev::Key::Num5 => "Num5".into(),
        rdev::Key::Num6 => "Num6".into(),
        rdev::Key::Num7 => "Num7".into(),
        rdev::Key::Num8 => "Num8".into(),
        rdev::Key::Num9 => "Num9".into(),
        rdev::Key::Minus => "Minus".into(),
        rdev::Key::Equal => "Equal".into(),
        rdev::Key::LeftBracket => "LeftBracket".into(),
        rdev::Key::RightBracket => "RightBracket".into(),
        rdev::Key::BackSlash => "BackSlash".into(),
        rdev::Key::SemiColon => "SemiColon".into(),
        rdev::Key::Quote => "Quote".into(),
        rdev::Key::Comma => "Comma".into(),
        rdev::Key::Dot => "Dot".into(),
        rdev::Key::Slash => "Slash".into(),
        rdev::Key::BackQuote => "Grave".into(),
        _ => format!("{key:?}"),
    }
}

fn timestamp_ms(snapshot: &RecorderControl) -> Option<u64> {
    snapshot
        .recording_origin
        .map(|origin| origin.elapsed().as_millis().min(u128::from(u64::MAX)) as u64)
}

fn map_mouse_button(button: RdevButton) -> Option<MouseButton> {
    match button {
        RdevButton::Left => Some(MouseButton::Left),
        RdevButton::Right => Some(MouseButton::Right),
        RdevButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

fn f64_to_i32(value: f64) -> i32 {
    if value.is_nan() {
        0
    } else {
        value
            .round()
            .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
    }
}

fn i64_to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
