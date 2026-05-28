use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkeys: HotkeyConfig,
    pub always_on_top: bool,
    pub start_compact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub record: String,
    pub play_stop: String,
    pub pause: String,
    pub emergency_stop: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkeys: HotkeyConfig::default(),
            always_on_top: true,
            start_compact: true,
        }
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            record: "F8".to_owned(),
            play_stop: "F10".to_owned(),
            pause: "F11".to_owned(),
            emergency_stop: "F12".to_owned(),
        }
    }
}
