use crate::core::hotkeys;
use crate::core::playback;
use crate::core::recorder::{spawn_recorder, RecorderControlHandle};
use crate::core::scheduler::LoopMode;
use crate::models::{Config, EventKind, HotkeyConfig, InputEvent, MacroFile};
use crate::storage::macro_store;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use tokio::runtime::Runtime;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub enum AppCommand {
    StartRecording,
    StopRecording,
    TogglePlay,
    TogglePause,
    EmergencyStop,
    RecordedEvent(InputEvent),
    RebindCapture(String),
    PlaybackCompleteFor(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Idle,
    Recording,
    Playing,
    Paused,
    Error(String),
}

pub struct AppState {
    pub macros: Vec<MacroFile>,
    pub selected_macro_index: Option<usize>,
    pub status: AppStatus,
    pub recording_buffer: Vec<InputEvent>,
    pub loop_mode: LoopMode,
    pub speed: f32,
    pub config: Config,
    pub stop_flag: Arc<AtomicBool>,
    pub is_playing_flag: Arc<AtomicBool>,
    pub pause_tx: watch::Sender<bool>,
    pub pause_rx: watch::Receiver<bool>,
    pub show_settings: bool,
    pub rebinding_action: Option<String>,
    pub duplicate_hotkey_warning: Option<String>,
    pub delete_confirm_index: Option<usize>,
    pub rename_index: Option<usize>,
    pub rename_buffer: String,
    pub loop_count_buffer: String,
    pub compact_mode: bool,
    pub positioned: bool,
    pub playback_total_events: usize,
    pub playback_generation: u64,
    pub last_error: Option<String>,
    pub command_tx: mpsc::Sender<AppCommand>,
    pub command_rx: mpsc::Receiver<AppCommand>,
    pub recorder_control: RecorderControlHandle,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (pause_tx, pause_rx) = watch::channel(false);
        let macros = macro_store::load_all();
        let selected_macro_index = if macros.is_empty() { None } else { Some(0) };
        let recorder_control = RecorderControlHandle::new(config.hotkeys.clone());

        Self {
            macros,
            selected_macro_index,
            status: AppStatus::Idle,
            recording_buffer: Vec::new(),
            loop_mode: LoopMode::Once,
            speed: 1.0,
            compact_mode: config.start_compact,
            config,
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_playing_flag: Arc::new(AtomicBool::new(false)),
            pause_tx,
            pause_rx,
            show_settings: false,
            rebinding_action: None,
            duplicate_hotkey_warning: None,
            delete_confirm_index: None,
            rename_index: None,
            rename_buffer: String::new(),
            loop_count_buffer: "2".to_owned(),
            positioned: false,
            playback_total_events: 0,
            playback_generation: 0,
            last_error: None,
            command_tx,
            command_rx,
            recorder_control,
        }
    }

    pub fn start_recorder_thread(&self) {
        let _ = spawn_recorder(
            self.command_tx.clone(),
            self.is_playing_flag.clone(),
            self.recorder_control.clone(),
        );
    }

    pub fn drain_commands(&mut self, runtime: &Runtime) {
        while let Ok(command) = self.command_rx.try_recv() {
            self.handle_command(command, runtime);
        }
    }

    pub fn send_command(&self, command: AppCommand) {
        let _ = self.command_tx.send(command);
    }

    pub fn handle_local_key_press(&mut self, key: String, runtime: &Runtime) {
        if self.rebinding_action.is_some() {
            self.handle_command(AppCommand::RebindCapture(key), runtime);
            return;
        }

        if let Some(command) = hotkeys::command_for_key_press(&key, &self.config.hotkeys) {
            self.handle_command(command, runtime);
            return;
        }

        if self.is_playing_or_paused() {
            return;
        }

        let Some(timestamp_ms) = self.recorder_control.recording_timestamp_ms() else {
            return;
        };

        self.record_event(InputEvent::new(timestamp_ms, EventKind::KeyDown { key }));
    }

    pub fn handle_local_key_release(&mut self, key: String) {
        if self.rebinding_action.is_some() || hotkeys::is_hotkey(&key, &self.config.hotkeys) {
            return;
        }

        if self.is_playing_or_paused() {
            return;
        }

        let Some(timestamp_ms) = self.recorder_control.recording_timestamp_ms() else {
            return;
        };

        self.record_event(InputEvent::new(timestamp_ms, EventKind::KeyUp { key }));
    }

    pub fn handle_command(&mut self, command: AppCommand, runtime: &Runtime) {
        match command {
            AppCommand::StartRecording => self.start_recording(),
            AppCommand::StopRecording => self.stop_recording(),
            AppCommand::TogglePlay => self.toggle_play(runtime),
            AppCommand::TogglePause => self.toggle_pause(),
            AppCommand::EmergencyStop => self.emergency_stop(),
            AppCommand::RecordedEvent(event) => self.record_event(event),
            AppCommand::PlaybackCompleteFor(generation) => self.playback_complete_for(generation),
            AppCommand::RebindCapture(key) => self.capture_rebind(key),
        }
    }

    pub fn selected_macro(&self) -> Option<&MacroFile> {
        self.selected_macro_index
            .and_then(|index| self.macros.get(index))
    }

    pub fn selected_macro_mut(&mut self) -> Option<&mut MacroFile> {
        self.selected_macro_index
            .and_then(|index| self.macros.get_mut(index))
    }

    pub fn selected_events(&self) -> &[InputEvent] {
        if matches!(self.status, AppStatus::Recording) {
            &self.recording_buffer
        } else {
            self.selected_macro()
                .map(|macro_file| macro_file.events.as_slice())
                .unwrap_or(&[])
        }
    }

    pub fn select_previous_macro(&mut self) {
        if self.macros.is_empty() {
            self.selected_macro_index = None;
            return;
        }
        let current = self.selected_macro_index.unwrap_or(0);
        self.selected_macro_index = Some(if current == 0 {
            self.macros.len() - 1
        } else {
            current - 1
        });
        self.cancel_inline_states();
    }

    pub fn select_next_macro(&mut self) {
        if self.macros.is_empty() {
            self.selected_macro_index = None;
            return;
        }
        let current = self.selected_macro_index.unwrap_or(0);
        self.selected_macro_index = Some((current + 1) % self.macros.len());
        self.cancel_inline_states();
    }

    pub fn create_macro(&mut self) {
        let macro_file = MacroFile::new("Untitled");
        if let Err(error) = macro_store::save(&macro_file) {
            self.set_error(format!("Could not save new macro: {error}"));
        }
        self.macros.push(macro_file);
        let index = self.macros.len() - 1;
        self.selected_macro_index = Some(index);
        self.start_rename(index);
    }

    pub fn start_rename(&mut self, index: usize) {
        if let Some(macro_file) = self.macros.get(index) {
            self.rename_index = Some(index);
            self.rename_buffer = macro_file.name.clone();
            self.delete_confirm_index = None;
        }
    }

    pub fn commit_rename(&mut self) {
        let Some(index) = self.rename_index else {
            return;
        };
        let trimmed = self.rename_buffer.trim();
        let new_name = if trimmed.is_empty() {
            "Untitled".to_owned()
        } else {
            trimmed.to_owned()
        };

        let mut to_save = None;
        if let Some(macro_file) = self.macros.get_mut(index) {
            macro_file.name = new_name;
            to_save = Some(macro_file.clone());
        }

        if let Some(macro_file) = to_save {
            if let Err(error) = macro_store::save(&macro_file) {
                self.set_error(format!("Could not rename macro: {error}"));
            }
        }

        self.rename_index = None;
        self.rename_buffer.clear();
    }

    pub fn duplicate_macro(&mut self, index: usize) {
        let Some(original) = self.macros.get(index).cloned() else {
            return;
        };
        let mut duplicate = MacroFile::new(format!("{} (copy)", original.name));
        duplicate.events = original.events;
        if let Err(error) = macro_store::save(&duplicate) {
            self.set_error(format!("Could not duplicate macro: {error}"));
        }
        self.macros.push(duplicate);
        self.selected_macro_index = Some(self.macros.len() - 1);
    }

    pub fn delete_macro(&mut self, index: usize) {
        if index >= self.macros.len() {
            return;
        }

        if let Err(error) = macro_store::delete(&self.macros[index].id) {
            self.set_error(format!("Could not delete macro: {error}"));
            return;
        }

        self.macros.remove(index);
        self.delete_confirm_index = None;
        self.rename_index = None;
        self.selected_macro_index = if self.macros.is_empty() {
            None
        } else if index >= self.macros.len() {
            Some(self.macros.len() - 1)
        } else {
            Some(index)
        };
    }

    pub fn export_macro(&mut self, index: usize) {
        let Some(macro_file) = self.macros.get(index) else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Repeatable macro", &["json"])
            .set_file_name(format!("{}.json", sanitize_file_name(&macro_file.name)))
            .save_file()
        else {
            return;
        };

        match serde_json::to_string_pretty(macro_file) {
            Ok(json) => {
                if let Err(error) = std::fs::write(path, json) {
                    self.set_error(format!("Could not export macro: {error}"));
                }
            }
            Err(error) => self.set_error(format!("Could not serialize macro: {error}")),
        }
    }

    pub fn import_macro(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Repeatable macro", &["json"])
            .pick_file()
        else {
            return;
        };

        match std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<MacroFile>(&text).ok())
        {
            Some(mut macro_file) if macro_file.version == 1 => {
                if self
                    .macros
                    .iter()
                    .any(|existing| existing.id == macro_file.id)
                {
                    macro_file.id = uuid::Uuid::new_v4().to_string();
                }
                if let Err(error) = macro_store::save(&macro_file) {
                    self.set_error(format!("Could not save imported macro: {error}"));
                }
                self.macros.push(macro_file);
                self.selected_macro_index = Some(self.macros.len() - 1);
            }
            _ => self.set_error("Invalid macro file".to_owned()),
        }
    }

    pub fn save_selected_macro(&mut self) {
        let Some(macro_file) = self.selected_macro().cloned() else {
            return;
        };
        if let Err(error) = macro_store::save(&macro_file) {
            self.set_error(format!("Could not save macro: {error}"));
        }
    }

    pub fn toggle_always_on_top(&mut self) {
        self.config.always_on_top = !self.config.always_on_top;
        self.save_config();
    }

    pub fn set_start_compact(&mut self, start_compact: bool) {
        self.config.start_compact = start_compact;
        self.save_config();
    }

    pub fn set_rebinding_action(&mut self, action: Option<String>) {
        self.rebinding_action = action.clone();
        self.duplicate_hotkey_warning = None;
        self.recorder_control.set_rebinding_action(action);
    }

    pub fn cancel_rebind(&mut self) {
        self.set_rebinding_action(None);
    }

    pub fn hotkey_value(&self, action: &str) -> &str {
        match action {
            "record" => &self.config.hotkeys.record,
            "play_stop" => &self.config.hotkeys.play_stop,
            "pause" => &self.config.hotkeys.pause,
            "emergency_stop" => &self.config.hotkeys.emergency_stop,
            _ => "",
        }
    }

    pub fn status_text(&self) -> String {
        match &self.status {
            AppStatus::Idle => "Idle".to_owned(),
            AppStatus::Recording => format!("{} events", self.recording_buffer.len()),
            AppStatus::Playing => "Playing".to_owned(),
            AppStatus::Paused => "Paused".to_owned(),
            AppStatus::Error(message) => message.clone(),
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            self.status,
            AppStatus::Recording | AppStatus::Playing | AppStatus::Paused
        )
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.status, AppStatus::Recording)
    }

    pub fn is_playing_or_paused(&self) -> bool {
        matches!(self.status, AppStatus::Playing | AppStatus::Paused)
    }

    pub fn cycle_speed(&mut self) {
        const SPEEDS: [f32; 5] = [0.25, 0.5, 1.0, 2.0, 4.0];
        let current = SPEEDS
            .iter()
            .position(|candidate| (*candidate - self.speed).abs() < f32::EPSILON)
            .unwrap_or(2);
        self.speed = SPEEDS[(current + 1) % SPEEDS.len()];
    }

    pub fn cycle_compact_loop(&mut self) {
        self.loop_mode = match self.loop_mode {
            LoopMode::Once => LoopMode::Count(2),
            LoopMode::Count(2) => LoopMode::Count(5),
            LoopMode::Count(5) => LoopMode::Infinite,
            _ => LoopMode::Once,
        };
    }

    fn start_recording(&mut self) {
        if self.is_recording() {
            self.stop_recording();
            return;
        }
        if self.is_playing_or_paused() {
            return;
        }
        if self.macros.is_empty() {
            self.create_macro();
        }
        self.stop_flag.store(false, Ordering::SeqCst);
        let _ = self.pause_tx.send(false);
        self.recording_buffer.clear();
        self.playback_total_events = 0;
        self.status = AppStatus::Recording;
        self.last_error = None;
        self.recorder_control.begin_recording();
    }

    fn stop_recording(&mut self) {
        if !self.is_recording() {
            return;
        }

        let mut events = std::mem::take(&mut self.recording_buffer);
        trim_dead_time(&mut events);
        if let Some(macro_file) = self.selected_macro_mut() {
            macro_file.events = events;
            let to_save = macro_file.clone();
            if let Err(error) = macro_store::save(&to_save) {
                self.recorder_control.end_recording();
                self.set_error(format!("Could not save recording: {error}"));
                return;
            }
        }
        self.recorder_control.end_recording();
        self.status = AppStatus::Idle;
    }

    fn toggle_play(&mut self, runtime: &Runtime) {
        match self.status {
            AppStatus::Idle | AppStatus::Error(_) => self.start_playback(runtime),
            AppStatus::Playing | AppStatus::Paused => {
                self.stop_flag.store(true, Ordering::SeqCst);
                let _ = self.pause_tx.send(false);
                self.status = AppStatus::Idle;
            }
            AppStatus::Recording => {}
        }
    }

    fn start_playback(&mut self, runtime: &Runtime) {
        let Some(macro_file) = self.selected_macro().cloned() else {
            return;
        };
        if macro_file.events.is_empty() {
            return;
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        let _ = self.pause_tx.send(false);
        self.playback_total_events = macro_file.events.len();
        self.status = AppStatus::Playing;
        self.playback_generation = self.playback_generation.wrapping_add(1);
        let generation = self.playback_generation;

        let events = macro_file.events;
        let loop_mode = self.loop_mode.clone();
        let speed = self.speed;
        let stop_flag = self.stop_flag.clone();
        let is_playing_flag = self.is_playing_flag.clone();
        let pause_rx = self.pause_rx.clone();
        let sender = self.command_tx.clone();

        runtime.spawn(async move {
            playback::run_playback(
                events,
                loop_mode,
                speed,
                stop_flag,
                is_playing_flag,
                pause_rx,
            )
            .await;
            let _ = sender.send(AppCommand::PlaybackCompleteFor(generation));
        });
    }

    fn toggle_pause(&mut self) {
        match self.status {
            AppStatus::Playing => {
                let _ = self.pause_tx.send(true);
                self.status = AppStatus::Paused;
            }
            AppStatus::Paused => {
                let _ = self.pause_tx.send(false);
                self.status = AppStatus::Playing;
            }
            _ => {}
        }
    }

    fn emergency_stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        let _ = self.pause_tx.send(false);
        self.is_playing_flag.store(false, Ordering::SeqCst);
        self.recorder_control.end_recording();
        self.status = AppStatus::Idle;
        self.cancel_rebind();
        self.cancel_inline_states();
    }

    fn record_event(&mut self, event: InputEvent) {
        if self.is_recording() {
            self.recording_buffer.push(event);
        }
    }

    fn playback_complete(&mut self) {
        self.is_playing_flag.store(false, Ordering::SeqCst);
        if matches!(self.status, AppStatus::Playing | AppStatus::Paused) {
            self.status = AppStatus::Idle;
        }
    }

    fn playback_complete_for(&mut self, generation: u64) {
        if generation == self.playback_generation {
            self.playback_complete();
        }
    }

    fn capture_rebind(&mut self, key: String) {
        let Some(action) = self.rebinding_action.clone() else {
            return;
        };

        if matches!(key.as_str(), "Escape" | "Esc") {
            self.cancel_rebind();
            return;
        }

        if is_modifier_key(&key) {
            self.duplicate_hotkey_warning = Some("Modifier keys cannot be used as hotkeys".into());
            self.recorder_control.set_rebinding_action(Some(action));
            return;
        }

        if self.hotkey_duplicate(&action, &key) {
            self.duplicate_hotkey_warning = Some(format!("{key} is already assigned"));
            self.recorder_control.set_rebinding_action(Some(action));
            return;
        }

        match action.as_str() {
            "record" => self.config.hotkeys.record = key,
            "play_stop" => self.config.hotkeys.play_stop = key,
            "pause" => self.config.hotkeys.pause = key,
            "emergency_stop" => self.config.hotkeys.emergency_stop = key,
            _ => {}
        }

        self.recorder_control
            .set_hotkeys(self.config.hotkeys.clone());
        self.save_config();
        self.set_rebinding_action(None);
    }

    fn hotkey_duplicate(&self, action: &str, key: &str) -> bool {
        let HotkeyConfig {
            record,
            play_stop,
            pause,
            emergency_stop,
        } = &self.config.hotkeys;

        [
            ("record", record),
            ("play_stop", play_stop),
            ("pause", pause),
            ("emergency_stop", emergency_stop),
        ]
        .into_iter()
        .any(|(other_action, other_key)| other_action != action && other_key == key)
    }

    fn save_config(&mut self) {
        if let Err(error) = macro_store::save_config(&self.config) {
            self.set_error(format!("Could not save config: {error}"));
        }
        self.recorder_control
            .set_hotkeys(self.config.hotkeys.clone());
    }

    fn set_error(&mut self, message: String) {
        self.last_error = Some(message.clone());
        self.status = AppStatus::Error(message);
    }

    fn cancel_inline_states(&mut self) {
        self.rename_index = None;
        self.delete_confirm_index = None;
    }
}

fn sanitize_file_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => ch,
        })
        .collect();

    if sanitized.trim().is_empty() {
        "macro".to_owned()
    } else {
        sanitized
    }
}

fn trim_dead_time(events: &mut Vec<InputEvent>) {
    if let Some(first_ts) = events.first().map(|event| event.timestamp_ms) {
        for event in events.iter_mut() {
            event.timestamp_ms = event.timestamp_ms.saturating_sub(first_ts);
        }
    }
}

fn is_modifier_key(key: &str) -> bool {
    matches!(
        key,
        "ShiftLeft"
            | "ShiftRight"
            | "ControlLeft"
            | "ControlRight"
            | "Alt"
            | "AltGr"
            | "MetaLeft"
            | "MetaRight"
            | "CapsLock"
    )
}
