use crate::models::HotkeyConfig;
use crate::state::AppCommand;

pub fn command_for_key_press(key: &str, hotkeys: &HotkeyConfig) -> Option<AppCommand> {
    if key == hotkeys.record {
        Some(AppCommand::StartRecording)
    } else if key == hotkeys.play_stop {
        Some(AppCommand::TogglePlay)
    } else if key == hotkeys.pause {
        Some(AppCommand::TogglePause)
    } else if key == hotkeys.emergency_stop {
        Some(AppCommand::EmergencyStop)
    } else {
        None
    }
}

pub fn is_hotkey(key: &str, hotkeys: &HotkeyConfig) -> bool {
    key == hotkeys.record
        || key == hotkeys.play_stop
        || key == hotkeys.pause
        || key == hotkeys.emergency_stop
}
