use crate::state::{AppCommand, AppState, AppStatus};
use crate::ui::theme;
use eframe::egui::{Button, Color32, RichText, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        let is_idle = matches!(state.status, AppStatus::Idle | AppStatus::Error(_));
        let is_recording = matches!(state.status, AppStatus::Recording);
        let record_label = if is_recording { "Stop Recording" } else { "Record" };
        let record_enabled = is_idle || is_recording;

        if ui
            .add_enabled(
                record_enabled,
                Button::new(RichText::new(record_label).color(Color32::WHITE))
                    .fill(theme::RECORD_RED)
                    .rounding(5.0),
            )
            .clicked()
        {
            state.send_command(if is_recording {
                AppCommand::StopRecording
            } else {
                AppCommand::StartRecording
            });
        }

        let is_stoppable = state.is_playing_or_paused();
        let play_label = if is_stoppable { "Stop" } else { "Play" };
        let play_enabled = is_stoppable
            || (matches!(state.status, AppStatus::Idle | AppStatus::Error(_))
                && state
                    .selected_macro()
                    .map(|macro_file| !macro_file.events.is_empty())
                    .unwrap_or(false));
        if ui
            .add_enabled(
                play_enabled,
                Button::new(RichText::new(play_label).color(Color32::WHITE))
                    .fill(if is_stoppable {
                        theme::RECORD_RED
                    } else {
                        theme::ACCENT_BLUE
                    })
                    .rounding(5.0),
            )
            .clicked()
        {
            state.send_command(AppCommand::TogglePlay);
        }

        if ui
            .add_enabled(
                state.is_playing_or_paused(),
                Button::new("Pause").rounding(5.0),
            )
            .clicked()
        {
            state.send_command(AppCommand::TogglePause);
        }

        let save_enabled = matches!(state.status, AppStatus::Idle | AppStatus::Error(_))
            && state
                .selected_macro()
                .map(|macro_file| !macro_file.events.is_empty())
                .unwrap_or(false);
        if ui
            .add_enabled(save_enabled, Button::new("Save").rounding(5.0))
            .clicked()
        {
            state.save_selected_macro();
        }

        if ui
            .add_enabled(
                matches!(state.status, AppStatus::Idle | AppStatus::Error(_)),
                Button::new("Import").rounding(5.0),
            )
            .clicked()
        {
            state.import_macro();
        }

        if ui.button("Settings").clicked() {
            state.show_settings = true;
        }

        ui.with_layout(
            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
            |ui| {
                if ui.button("Collapse").clicked() {
                    state.compact_mode = true;
                    state.positioned = false;
                }
            },
        );
    });
}
