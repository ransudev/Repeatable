use crate::state::AppState;
use crate::ui::{theme, truncate_chars};
use eframe::egui::{self, Button, Frame, Key, RichText, ScrollArea, Sense, Stroke, TextEdit, Ui};

enum MacroLibraryAction {
    Duplicate(usize),
    Delete(usize),
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    Frame::none()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0, theme::separator()))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new("Macro Library")
                    .color(theme::TEXT_PRIMARY)
                    .strong(),
            );
            ui.add_space(6.0);

            let available_height = (ui.available_height() - 34.0).max(80.0);
            ScrollArea::vertical()
                .id_source("macro_library_scroll")
                .max_height(available_height)
                .show(ui, |ui| {
                    let count = state.macros.len();
                    let mut pending_action = None;
                    for index in 0..count {
                        if pending_action.is_none() {
                            pending_action = macro_row(ui, state, index);
                        } else {
                            macro_row(ui, state, index);
                        }
                    }

                    if let Some(action) = pending_action {
                        match action {
                            MacroLibraryAction::Duplicate(index) => state.duplicate_macro(index),
                            MacroLibraryAction::Delete(index) => state.delete_macro(index),
                        }
                    }
                });

            ui.add_space(8.0);
            if ui
                .add_enabled(!state.is_busy(), Button::new("New Macro").rounding(5.0))
                .clicked()
            {
                state.create_macro();
            }
        });
}

fn macro_row(ui: &mut Ui, state: &mut AppState, index: usize) -> Option<MacroLibraryAction> {
    let selected = state.selected_macro_index == Some(index);
    let name = state
        .macros
        .get(index)
        .map(|macro_file| macro_file.name.clone())
        .unwrap_or_default();

    let fill = if selected {
        theme::with_alpha(theme::ACCENT_BLUE, 0.16)
    } else {
        theme::ROW
    };
    let mut pending_action = None;
    let mut row_response = None;

    Frame::none()
        .fill(fill)
        .stroke(if selected {
            Stroke::new(1.0, theme::ACCENT_BLUE)
        } else {
            Stroke::new(1.0, theme::separator())
        })
        .rounding(5.0)
        .inner_margin(6.0)
        .show(ui, |ui| {
            if state.rename_index == Some(index) {
                let response = ui.add(
                    TextEdit::singleline(&mut state.rename_buffer)
                        .desired_width(155.0)
                        .hint_text("Macro name"),
                );
                if response.lost_focus() || ui.input(|input| input.key_pressed(Key::Enter)) {
                    state.commit_rename();
                }
            } else {
                let response = ui.add(
                    egui::Label::new(
                        RichText::new(truncate_chars(&name, 22)).color(theme::TEXT_PRIMARY),
                    )
                    .sense(Sense::click()),
                );
                row_response = Some(response);
            }

            if state.delete_confirm_index == Some(index) {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("Delete {name}?"))
                        .color(theme::TEXT_MUTED)
                        .small(),
                );
                ui.horizontal(|ui| {
                    if ui.small_button("Cancel").clicked() {
                        state.delete_confirm_index = None;
                    }
                    if ui
                        .add(
                            Button::new(RichText::new("Delete").color(egui::Color32::WHITE))
                                .fill(theme::RECORD_RED),
                        )
                        .clicked()
                    {
                        pending_action = Some(MacroLibraryAction::Delete(index));
                    }
                });
            }
        });

    let Some(response) = row_response else {
        ui.add_space(5.0);
        return pending_action;
    };

    if response.clicked() && !state.is_busy() {
        state.selected_macro_index = Some(index);
    }

    response.context_menu(|ui| {
        if ui.button("Rename").clicked() {
            state.start_rename(index);
            ui.close_menu();
        }
        if ui.button("Duplicate").clicked() {
            pending_action = Some(MacroLibraryAction::Duplicate(index));
            ui.close_menu();
        }
        if ui.button("Export JSON").clicked() {
            state.export_macro(index);
            ui.close_menu();
        }
        if ui
            .button(RichText::new("Delete").color(theme::RECORD_RED))
            .clicked()
        {
            state.delete_confirm_index = Some(index);
            ui.close_menu();
        }
    });

    ui.add_space(5.0);
    pending_action
}
