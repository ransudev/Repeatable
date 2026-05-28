use crate::state::{AppCommand, AppState, AppStatus};
use crate::ui::{loop_label, speed_label, truncate_chars};
use egui_phosphor::regular;
use eframe::egui::{
    self, Align, Button, CentralPanel, Color32, Frame, Label, Layout, RichText, Sense, Stroke,
    Ui,
};

const BG_IDLE: Color32 = Color32::from_rgb(17, 17, 17);
const BG_REC: Color32 = Color32::from_rgb(26, 13, 13);
const BG_PLAY: Color32 = Color32::from_rgb(12, 15, 28);
const BG_PAUSE: Color32 = Color32::from_rgb(24, 19, 9);

const C_PLAY: Color32 = Color32::from_rgb(91, 110, 255);
const C_PAUSE: Color32 = Color32::from_rgb(243, 156, 18);

const C_TEXT: (u8, u8, u8, u8) = (255, 255, 255, 190);
const C_MUTED: (u8, u8, u8, u8) = (255, 255, 255, 64);
const C_DIMMED: (u8, u8, u8, u8) = (255, 255, 255, 35);
const C_BORDER: (u8, u8, u8, u8) = (255, 255, 255, 25);
const C_BTN_BG: (u8, u8, u8, u8) = (255, 255, 255, 15);
const C_BTN_HOV: (u8, u8, u8, u8) = (255, 255, 255, 28);

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let bar_bg = match state.status {
        AppStatus::Recording => BG_REC,
        AppStatus::Playing => BG_PLAY,
        AppStatus::Paused => BG_PAUSE,
        _ => BG_IDLE,
    };

    CentralPanel::default()
        .frame(
            Frame::none()
                .fill(bar_bg)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| {
            let drag = ui.interact(ui.max_rect(), ui.id().with("drag"), Sense::drag());
            if drag.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.set_min_height(48.0);
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            ui.horizontal_centered(|ui| {
                ui.set_min_height(48.0);
                ui.add_space(12.0);

                draw_window_dots(ui, ctx, state);
                draw_sep(ui);
                draw_macro_selector(ui, state);
                draw_sep(ui);
                draw_action_buttons(ui, ctx, state);
                draw_sep(ui);
                draw_quick_controls(ui, state);
                draw_sep(ui);
                draw_status(ui, ctx, state);

                ui.add_space(8.0);
            });

            if !matches!(state.status, AppStatus::Idle) {
                ctx.request_repaint();
            }
        });
}

fn draw_window_dots(ui: &mut Ui, ctx: &egui::Context, state: &mut AppState) {
    ui.add_space(4.0);

    let btn_style = |icon: &str, color: Color32| {
        Button::new(RichText::new(icon).color(color).size(12.0))
            .fill(Color32::TRANSPARENT)
            .frame(false)
            .min_size(egui::vec2(22.0, 32.0))
    };

    if ui
        .add(btn_style(regular::X, Color32::from_rgb(231, 76, 60)))
        .clicked()
    {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    if ui
        .add(btn_style(regular::MINUS, Color32::from_rgb(232, 160, 32)))
        .clicked()
    {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
    if ui
        .add(btn_style(regular::SQUARE, Color32::from_rgb(40, 200, 100)))
        .clicked()
    {
        state.compact_mode = false;
        state.positioned = false;
    }

    ui.add_space(2.0);
}

fn draw_sep(ui: &mut Ui) {
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 24.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::Rounding::ZERO, rgba(C_BORDER));
    ui.add_space(4.0);
}

fn draw_macro_selector(ui: &mut Ui, state: &mut AppState) {
    let disabled = matches!(state.status, AppStatus::Recording | AppStatus::Playing);
    let selector_width = (ui.available_width() - 310.0).max(80.0);

    ui.allocate_ui_with_layout(
        egui::vec2(selector_width, 48.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            let arrow_color = if disabled { rgba(C_DIMMED) } else { rgba(C_MUTED) };
            let left = Button::new(RichText::new("<").color(arrow_color).size(12.5))
                .fill(Color32::TRANSPARENT)
                .frame(false)
                .min_size(egui::vec2(20.0, 32.0));
            if ui.add_enabled(!disabled, left).clicked() {
                state.select_previous_macro();
            }

            let name_width = (selector_width - 40.0).max(20.0);
            ui.allocate_ui_with_layout(
                egui::vec2(name_width, 32.0),
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| match state.selected_macro() {
                    Some(macro_file) => {
                        let color = match state.status {
                            AppStatus::Idle | AppStatus::Error(_) => rgba(C_TEXT),
                            AppStatus::Recording => {
                                Color32::from_rgba_unmultiplied(255, 120, 120, 178)
                            }
                            AppStatus::Playing => {
                                Color32::from_rgba_unmultiplied(130, 150, 255, 204)
                            }
                            AppStatus::Paused => {
                                Color32::from_rgba_unmultiplied(243, 180, 80, 178)
                            }
                        };
                        let max_chars = ((name_width / 7.0).floor() as usize).max(1);
                        ui.add_sized(
                            egui::vec2(name_width, 32.0),
                            Label::new(
                                RichText::new(truncate_chars(&macro_file.name, max_chars))
                                    .color(color)
                                    .size(12.5),
                            )
                            .wrap(false),
                        );
                    }
                    None => {
                        ui.add_sized(
                            egui::vec2(name_width, 32.0),
                            Label::new(
                                RichText::new("no macros")
                                    .color(rgba(C_DIMMED))
                                    .italics()
                                    .size(12.5),
                            )
                            .wrap(false),
                        );
                    }
                },
            );

            let right = Button::new(RichText::new(">").color(arrow_color).size(12.5))
                .fill(Color32::TRANSPARENT)
                .frame(false)
                .min_size(egui::vec2(20.0, 32.0));
            if ui.add_enabled(!disabled, right).clicked() {
                state.select_next_macro();
            }
        },
    );
}

fn draw_action_buttons(ui: &mut Ui, ctx: &egui::Context, state: &mut AppState) {
    let is_idle = matches!(state.status, AppStatus::Idle | AppStatus::Error(_));
    let is_recording = matches!(state.status, AppStatus::Recording);
    let is_playing = matches!(state.status, AppStatus::Playing);
    let is_paused = matches!(state.status, AppStatus::Paused);
    let has_playable_macro = state
        .selected_macro()
        .map(|macro_file| !macro_file.events.is_empty())
        .unwrap_or(false);

    let rec_button = if is_recording {
        let alpha = pulse_alpha(ctx, 170, 85);
        action_button(
            RichText::new("⏺ Stop")
                .color(Color32::from_rgba_unmultiplied(255, 107, 107, alpha))
                .size(11.5),
            Color32::from_rgba_unmultiplied(231, 76, 60, 46),
            Color32::from_rgba_unmultiplied(231, 76, 60, 102),
        )
        .min_size(egui::vec2(58.0, 32.0))
    } else {
        action_button(
            RichText::new("⏺ Rec")
                .color(if is_idle {
                    Color32::from_rgba_unmultiplied(231, 76, 60, 178)
                } else {
                    rgba(C_DIMMED)
                })
                .size(11.5),
            if is_idle { Color32::TRANSPARENT } else { rgba(C_BTN_BG) },
            if is_idle {
                Color32::from_rgba_unmultiplied(231, 76, 60, 80)
            } else {
                rgba(C_BORDER)
            },
        )
        .min_size(egui::vec2(58.0, 32.0))
    };
    let rec_enabled = is_idle || is_recording;
    let rec_response = ui.add_enabled(rec_enabled, rec_button);
    if rec_response.clicked() {
        state.send_command(if is_recording {
            AppCommand::StopRecording
        } else {
            AppCommand::StartRecording
        });
    }

    ui.add_space(4.0);

    let play_label = if is_playing || is_paused {
        RichText::new("⏹ Stop")
            .color(Color32::from_rgb(255, 107, 107))
            .size(11.5)
    } else {
        RichText::new("▶ Play")
            .color(if is_idle {
                Color32::from_rgba_unmultiplied(91, 110, 255, 204)
            } else {
                rgba(C_DIMMED)
            })
            .size(11.5)
    };
    let play_button = action_button(
        play_label,
        if is_playing || is_paused {
            Color32::from_rgba_unmultiplied(255, 80, 80, 26)
        } else if is_idle {
            Color32::TRANSPARENT
        } else {
            rgba(C_BTN_BG)
        },
        if is_playing || is_paused {
            Color32::from_rgba_unmultiplied(255, 80, 80, 76)
        } else if is_idle {
            Color32::from_rgba_unmultiplied(91, 110, 255, 80)
        } else {
            rgba(C_BORDER)
        },
    )
    .min_size(egui::vec2(58.0, 32.0));
    let play_enabled = (is_idle && has_playable_macro) || is_playing || is_paused;
    if ui.add_enabled(play_enabled, play_button).clicked() {
        state.send_command(AppCommand::TogglePlay);
    }

    ui.add_space(4.0);

    let (pause_label, pause_fill, pause_stroke) = match state.status {
        AppStatus::Playing => (
            RichText::new("⏸")
                .color(Color32::from_rgba_unmultiplied(243, 156, 18, 204))
                .size(12.0),
            Color32::TRANSPARENT,
            Color32::from_rgba_unmultiplied(243, 156, 18, 80),
        ),
        AppStatus::Paused => (
            RichText::new("▶")
                .color(Color32::from_rgb(243, 156, 18))
                .size(12.0),
            Color32::from_rgba_unmultiplied(243, 156, 18, 31),
            Color32::from_rgba_unmultiplied(243, 156, 18, 102),
        ),
        _ => (
            RichText::new("⏸").color(rgba(C_DIMMED)).size(12.0),
            Color32::TRANSPARENT,
            rgba(C_BORDER),
        ),
    };
    let pause_button = action_button(pause_label, pause_fill, pause_stroke)
        .min_size(egui::vec2(32.0, 32.0));
    if ui
        .add_enabled(is_playing || is_paused, pause_button)
        .clicked()
    {
        state.send_command(AppCommand::TogglePause);
    }
}

fn draw_quick_controls(ui: &mut Ui, state: &mut AppState) {
    let idle = matches!(state.status, AppStatus::Idle | AppStatus::Error(_));
    let spd_label = speed_label(state.speed);
    let spd_color = if idle { rgba(C_TEXT) } else { rgba(C_DIMMED) };
    if ui
        .add_enabled(
            idle,
            Button::new(RichText::new(spd_label).color(spd_color).size(11.5))
                .fill(Color32::TRANSPARENT)
                .frame(false)
                .min_size(egui::vec2(32.0, 28.0)),
        )
        .clicked()
    {
        state.cycle_speed();
    }

    ui.label(RichText::new("·").color(rgba(C_DIMMED)).size(11.0));

    let loop_text = loop_label(&state.loop_mode);
    let loop_color = if idle { rgba(C_TEXT) } else { rgba(C_DIMMED) };
    if ui
        .add_enabled(
            idle,
            Button::new(RichText::new(loop_text).color(loop_color).size(11.5))
                .fill(Color32::TRANSPARENT)
                .frame(false)
                .min_size(egui::vec2(28.0, 28.0)),
        )
        .clicked()
    {
        state.cycle_compact_loop();
    }
}

fn draw_status(ui: &mut Ui, ctx: &egui::Context, state: &AppState) {
    ui.allocate_ui_with_layout(
        egui::vec2(80.0, 32.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            let (dot_color, status_text, text_color) = match state.status {
                AppStatus::Idle => (
                    Color32::from_rgb(58, 58, 58),
                    "Idle".to_owned(),
                    rgba(C_MUTED),
                ),
                AppStatus::Recording => (
                    Color32::from_rgba_unmultiplied(231, 76, 60, pulse_alpha(ctx, 100, 155)),
                    format!("{} ev", state.recording_buffer.len()),
                    Color32::from_rgba_unmultiplied(255, 100, 100, 127),
                ),
                AppStatus::Playing => (
                    C_PLAY,
                    loop_label(&state.loop_mode),
                    Color32::from_rgba_unmultiplied(100, 120, 255, 153),
                ),
                AppStatus::Paused => (
                    C_PAUSE,
                    "Paused".to_owned(),
                    Color32::from_rgba_unmultiplied(243, 156, 18, 127),
                ),
                AppStatus::Error(_) => (rgba(C_DIMMED), "—".to_owned(), rgba(C_DIMMED)),
            };

            let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 3.5, dot_color);
            ui.add_space(5.0);
            ui.label(RichText::new(status_text).color(text_color).size(11.0));
        },
    );
}

fn action_button<'a>(label: RichText, fill: Color32, stroke_color: Color32) -> Button<'a> {
    Button::new(label)
        .fill(fill)
        .stroke(Stroke::new(0.5, stroke_color))
        .rounding(egui::Rounding::same(5.0))
}

fn pulse_alpha(ctx: &egui::Context, base: u8, range: u8) -> u8 {
    let t = (ctx.input(|i| i.time * 3.0).sin() as f32 * 0.5 + 0.5).clamp(0.0, 1.0);
    base.saturating_add((t * range as f32) as u8)
}

fn rgba((r, g, b, a): (u8, u8, u8, u8)) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}
