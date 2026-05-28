use eframe::egui::{Color32, Rgba};

pub const PANEL: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
pub const TITLEBAR: Color32 = Color32::from_rgb(0x1c, 0x1c, 0x1c);
pub const ROW: Color32 = Color32::from_rgb(0x1c, 0x1c, 0x1c);
pub const BUTTON: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
pub const BUTTON_HOVER: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0x5B, 0x6E, 0xFF);
pub const RECORD_RED: Color32 = Color32::from_rgb(0xE7, 0x4C, 0x3C);
pub const PAUSE_AMBER: Color32 = Color32::from_rgb(0xF3, 0x9C, 0x12);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x66, 0x66, 0x66);
pub const MOUSE: Color32 = Color32::from_rgb(0x7e, 0xc8, 0xa0);
pub const KEY: Color32 = Color32::from_rgb(0x7a, 0xae, 0xe8);
pub const SCROLL: Color32 = Color32::from_rgb(0xc4, 0xa9, 0x6e);
pub const IDLE_BG: Color32 = Color32::from_rgb(0x0f, 0x0f, 0x0f);
pub const RECORD_BG: Color32 = Color32::from_rgb(0x24, 0x0f, 0x0d);
pub const PLAY_BG: Color32 = Color32::from_rgb(0x0d, 0x15, 0x24);
pub const PAUSE_BG: Color32 = Color32::from_rgb(0x24, 0x19, 0x06);

pub fn separator() -> Color32 {
    Rgba::from_rgba_unmultiplied(1.0, 1.0, 1.0, 0.07).into()
}

pub fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let rgba = Rgba::from(color);
    Rgba::from_rgba_unmultiplied(rgba.r(), rgba.g(), rgba.b(), alpha.clamp(0.0, 1.0)).into()
}
