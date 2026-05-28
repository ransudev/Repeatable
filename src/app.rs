use crate::state::{AppState, AppStatus};
use crate::ui;
use eframe::egui::{self, Color32, FontFamily, FontId, TextStyle, Visuals};
use eframe::{App, CreationContext, Frame};
use std::time::Duration;
use tokio::runtime::Runtime;
use egui_phosphor::Variant;

pub struct RepeatableApp {
    pub state: AppState,
    runtime: Runtime,
}

impl RepeatableApp {
    pub fn new(cc: &CreationContext<'_>, runtime: Runtime, config: crate::models::Config) -> Self {
        configure_fonts_and_theme(&cc.egui_ctx);
        let state = AppState::new(config);
        state.start_recorder_thread();
        Self { state, runtime }
    }
}

impl App for RepeatableApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.state.drain_commands(&self.runtime);

        capture_keyboard_from_egui(ctx, &mut self.state, &self.runtime);

        apply_viewport_mode(ctx, &mut self.state);

        if self.state.compact_mode {
            ui::compact::show(ctx, &mut self.state);
        } else {
            ui::expanded::show(ctx, &mut self.state);
        }

        if !matches!(self.state.status, AppStatus::Idle) {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

fn capture_keyboard_from_egui(ctx: &egui::Context, state: &mut AppState, runtime: &Runtime) {
    let key_events = ctx.input(|input| {
        input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } => {
                    if *pressed && *repeat {
                        return None;
                    }

                    egui_key_to_hotkey(*key).map(|mapped| (mapped, *pressed))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    });

    for (key, pressed) in key_events {
        if pressed {
            state.handle_local_key_press(key, runtime);
        } else {
            state.handle_local_key_release(key);
        }
    }
}

fn egui_key_to_hotkey(key: egui::Key) -> Option<String> {
    let key = match key {
        egui::Key::F1 => "F1",
        egui::Key::F2 => "F2",
        egui::Key::F3 => "F3",
        egui::Key::F4 => "F4",
        egui::Key::F5 => "F5",
        egui::Key::F6 => "F6",
        egui::Key::F7 => "F7",
        egui::Key::F8 => "F8",
        egui::Key::F9 => "F9",
        egui::Key::F10 => "F10",
        egui::Key::F11 => "F11",
        egui::Key::F12 => "F12",
        egui::Key::Enter => "Return",
        egui::Key::Escape => "Escape",
        egui::Key::Backspace => "Backspace",
        egui::Key::Tab => "Tab",
        egui::Key::Space => "Space",
        egui::Key::Delete => "Delete",
        egui::Key::Insert => "Insert",
        egui::Key::Home => "Home",
        egui::Key::End => "End",
        egui::Key::PageUp => "PageUp",
        egui::Key::PageDown => "PageDown",
        egui::Key::ArrowUp => "UpArrow",
        egui::Key::ArrowDown => "DownArrow",
        egui::Key::ArrowLeft => "LeftArrow",
        egui::Key::ArrowRight => "RightArrow",
        egui::Key::A => "KeyA",
        egui::Key::B => "KeyB",
        egui::Key::C => "KeyC",
        egui::Key::D => "KeyD",
        egui::Key::E => "KeyE",
        egui::Key::F => "KeyF",
        egui::Key::G => "KeyG",
        egui::Key::H => "KeyH",
        egui::Key::I => "KeyI",
        egui::Key::J => "KeyJ",
        egui::Key::K => "KeyK",
        egui::Key::L => "KeyL",
        egui::Key::M => "KeyM",
        egui::Key::N => "KeyN",
        egui::Key::O => "KeyO",
        egui::Key::P => "KeyP",
        egui::Key::Q => "KeyQ",
        egui::Key::R => "KeyR",
        egui::Key::S => "KeyS",
        egui::Key::T => "KeyT",
        egui::Key::U => "KeyU",
        egui::Key::V => "KeyV",
        egui::Key::W => "KeyW",
        egui::Key::X => "KeyX",
        egui::Key::Y => "KeyY",
        egui::Key::Z => "KeyZ",
        egui::Key::Num0 => "Num0",
        egui::Key::Num1 => "Num1",
        egui::Key::Num2 => "Num2",
        egui::Key::Num3 => "Num3",
        egui::Key::Num4 => "Num4",
        egui::Key::Num5 => "Num5",
        egui::Key::Num6 => "Num6",
        egui::Key::Num7 => "Num7",
        egui::Key::Num8 => "Num8",
        egui::Key::Num9 => "Num9",
        _ => return None,
    };

    Some(key.to_owned())
}

fn apply_viewport_mode(ctx: &egui::Context, state: &mut AppState) {
    let level = if state.config.always_on_top {
        egui::viewport::WindowLevel::AlwaysOnTop
    } else {
        egui::viewport::WindowLevel::Normal
    };
    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));

    if state.compact_mode {
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(480.0, 52.0)));
        if !state.positioned {
            let screen = ctx.input(|input| input.screen_rect());
            let x = screen.center().x - 240.0;
            let y = screen.top() + 8.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(x, y)));
            state.positioned = true;
        }
    } else {
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(780.0, 560.0)));
    }
}

fn configure_fonts_and_theme(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, Variant::Regular);
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();
    style.visuals.window_fill = crate::ui::theme::PANEL;
    style.visuals.panel_fill = crate::ui::theme::PANEL;
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 10, 10);
    style.visuals.widgets.inactive.bg_fill = crate::ui::theme::BUTTON;
    style.visuals.widgets.hovered.bg_fill = crate::ui::theme::BUTTON_HOVER;
    style.visuals.widgets.active.bg_fill = crate::ui::theme::BUTTON_HOVER;
    style.visuals.selection.bg_fill = crate::ui::theme::ACCENT_BLUE;

    for text_style in [
        TextStyle::Body,
        TextStyle::Button,
        TextStyle::Heading,
        TextStyle::Small,
    ] {
        let size = match text_style {
            TextStyle::Heading => 18.0,
            TextStyle::Small => 11.0,
            _ => 13.0,
        };
        style
            .text_styles
            .insert(text_style, FontId::new(size, FontFamily::Proportional));
    }
    ctx.set_style(style);
}
