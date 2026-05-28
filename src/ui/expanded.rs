use crate::core::scheduler::LoopMode;
use crate::state::{AppState, AppStatus};
use crate::ui::{macro_library, settings, speed_label, theme, timeline, toolbar};
use eframe::egui::{self, CentralPanel, ComboBox, Frame, RichText, TopBottomPanel};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    TopBottomPanel::top("toolbar")
        .frame(Frame::none().fill(theme::TITLEBAR).inner_margin(8.0))
        .show(ctx, |ui| toolbar::show(ui, state));

    TopBottomPanel::bottom("status_bar")
        .frame(Frame::none().fill(theme::TITLEBAR).inner_margin(8.0))
        .show(ctx, |ui| {
            controls_row(ui, state);
            ui.separator();
            status_bar(ui, state, ctx.input(|input| input.time));
        });

    CentralPanel::default()
        .frame(Frame::none().fill(theme::PANEL).inner_margin(8.0))
        .show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].set_width(185.0);
                macro_library::show(&mut columns[0], state);
                timeline::show(&mut columns[1], state, ctx);
            });
        });

    if state.show_settings {
        settings::show(ctx, state);
    }
}

fn controls_row(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_enabled_ui(!state.is_busy(), |ui| {
        ui.horizontal(|ui| {
            if let LoopMode::Count(count) = state.loop_mode {
                state.loop_count_buffer = count.to_string();
            }

            let selected_loop_text = match state.loop_mode {
                LoopMode::Once => "Once".to_owned(),
                LoopMode::Count(2) => "2×".to_owned(),
                LoopMode::Count(5) => "5×".to_owned(),
                LoopMode::Count(10) => "10×".to_owned(),
                LoopMode::Infinite => "Infinite".to_owned(),
                LoopMode::Count(_) => format!("{}×", state.loop_count_buffer),
            };

            ui.label(RichText::new("Loop").color(theme::TEXT_MUTED));
            ComboBox::from_id_source("loop_dropdown")
                .selected_text(selected_loop_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.loop_mode, LoopMode::Once, "Once");
                    ui.selectable_value(&mut state.loop_mode, LoopMode::Count(2), "2×");
                    ui.selectable_value(&mut state.loop_mode, LoopMode::Count(5), "5×");
                    ui.selectable_value(&mut state.loop_mode, LoopMode::Count(10), "10×");
                    ui.selectable_value(&mut state.loop_mode, LoopMode::Infinite, "Infinite");
                });

            ui.separator();
            ui.label(RichText::new("Speed").color(theme::TEXT_MUTED));
            ComboBox::from_id_source("speed_dropdown")
                .selected_text(speed_label(state.speed))
                .show_ui(ui, |ui| {
                    for speed in [0.25, 0.5, 1.0, 2.0, 4.0] {
                        ui.selectable_value(&mut state.speed, speed, speed_label(speed));
                    }
                });
        });
    });
}

fn status_bar(ui: &mut egui::Ui, state: &AppState, time: f64) {
    ui.horizontal(|ui| {
        let (color, text) = match &state.status {
            AppStatus::Idle => (
                theme::TEXT_MUTED,
                format!(
                    "Idle — {} to record · {} to play",
                    state.config.hotkeys.record, state.config.hotkeys.play_stop
                ),
            ),
            AppStatus::Recording => {
                let alpha = 0.55 + 0.35 * (time * 3.0).sin() as f32;
                (
                    theme::with_alpha(theme::RECORD_RED, alpha),
                    format!("Recording — {} events", state.recording_buffer.len()),
                )
            }
            AppStatus::Playing => (theme::ACCENT_BLUE, "Playing".to_owned()),
            AppStatus::Paused => (theme::PAUSE_AMBER, "Paused".to_owned()),
            AppStatus::Error(message) => (theme::RECORD_RED, message.clone()),
        };
        let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.label(RichText::new(text).color(theme::TEXT_PRIMARY));
    });
}
