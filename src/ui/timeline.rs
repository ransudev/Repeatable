use crate::models::TimelineColorHint;
use crate::state::{AppState, AppStatus};
use crate::ui::theme;
use eframe::egui::{Align, Frame, Layout, RichText, ScrollArea, Stroke, Ui};

pub fn show(ui: &mut Ui, state: &AppState, _ctx: &eframe::egui::Context) {
    Frame::none()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0, theme::separator()))
        .inner_margin(8.0)
        .show(ui, |ui| {
            let title = state
                .selected_macro()
                .map(|macro_file| macro_file.name.clone())
                .unwrap_or_else(|| "Timeline".to_owned());
            ui.label(RichText::new(title).color(theme::TEXT_PRIMARY).strong());
            ui.add_space(6.0);

            let events = state.selected_events();
            if events.is_empty() {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space((ui.available_height() * 0.4).max(24.0));
                    ui.label(RichText::new("No events").color(theme::TEXT_MUTED));
                });
                return;
            }

            ScrollArea::vertical()
                .id_source("timeline_scroll")
                .stick_to_bottom(matches!(state.status, AppStatus::Recording))
                .show(ui, |ui| {
                    for event in events {
                        let color = match event.color_hint() {
                            TimelineColorHint::Mouse => theme::MOUSE,
                            TimelineColorHint::Key => theme::KEY,
                            TimelineColorHint::Scroll => theme::SCROLL,
                        };

                        Frame::none()
                            .fill(theme::ROW)
                            .rounding(5.0)
                            .inner_margin(6.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{:>7} ms", event.timestamp_ms))
                                            .monospace()
                                            .color(theme::TEXT_MUTED),
                                    );
                                    ui.label(RichText::new(event.label()).color(color));
                                });
                            });
                        ui.add_space(4.0);
                    }
                });
        });
}
