use crate::state::AppState;
use crate::ui::theme;
use eframe::egui::{self, Button, Color32, RichText};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let mut open = state.show_settings;
    egui::Window::new("Settings")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(360.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Hotkeys").strong().color(theme::TEXT_PRIMARY));
            ui.add_space(6.0);

            hotkey_row(ui, state, "Record", "record");
            hotkey_row(ui, state, "Play / Stop", "play_stop");
            hotkey_row(ui, state, "Pause", "pause");
            hotkey_row(ui, state, "Emergency Stop", "emergency_stop");

            if let Some(warning) = &state.duplicate_hotkey_warning {
                ui.label(RichText::new(warning).color(theme::RECORD_RED));
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            let mut always_on_top = state.config.always_on_top;
            if ui
                .checkbox(&mut always_on_top, "Always on top")
                .on_hover_text("Keep Repeatable above other windows")
                .changed()
            {
                state.toggle_always_on_top();
            }

            let mut start_compact = state.config.start_compact;
            if ui
                .checkbox(&mut start_compact, "Start in compact mode")
                .changed()
            {
                state.set_start_compact(start_compact);
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("Press Escape to cancel rebinding.")
                    .color(theme::TEXT_MUTED)
                    .small(),
            );
        });
    state.show_settings = open;
}

fn hotkey_row(ui: &mut egui::Ui, state: &mut AppState, label: &str, action: &str) {
    ui.horizontal(|ui| {
        ui.set_min_width(320.0);
        ui.label(RichText::new(label).color(theme::TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let listening = state.rebinding_action.as_deref() == Some(action);
            let text = if listening {
                "Press a key...".to_owned()
            } else {
                state.hotkey_value(action).to_owned()
            };
            let fill = if listening {
                theme::PAUSE_AMBER
            } else {
                theme::BUTTON
            };
            let response = ui.add(
                Button::new(RichText::new(text).color(Color32::WHITE))
                    .fill(fill)
                    .rounding(5.0),
            );

            if listening {
                response.request_focus();
            }

            if response.clicked() {
                if listening {
                    state.cancel_rebind();
                } else {
                    state.set_rebinding_action(Some(action.to_owned()));
                    response.request_focus();
                }
            }
        });
    });
}
