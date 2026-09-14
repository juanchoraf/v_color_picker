use eframe::egui::{self, Align, Rect, RichText, Sense};

use crate::drawing::{ButtonIcon, draw_swatch, icon_only_button};
use crate::history::PickRecord;

const COLUMN_LIMIT: usize = 5;
const VISIBLE_LIMIT: usize = 10;
const MAX_HEIGHT: f32 = 132.0;
const CLEAR_BUTTON_SIZE: egui::Vec2 = egui::vec2(32.0, 24.0);
const ROW_HEIGHT: f32 = 24.0;
const LABEL_HEIGHT: f32 = 22.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryAction {
    None,
    Select(usize),
    Clear,
}

pub fn render(ui: &mut egui::Ui, history: &[PickRecord], selected: usize) -> HistoryAction {
    let mut action = HistoryAction::None;

    ui.horizontal(|ui| {
        ui.label(RichText::new("History").strong());
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if icon_only_button(ui, ButtonIcon::Trash, CLEAR_BUTTON_SIZE, "Delete history")
                .clicked()
            {
                action = HistoryAction::Clear;
            }
        });
    });

    ui.columns(2, |columns| {
        capture_action(
            &mut action,
            render_column(&mut columns[0], history, selected, 0),
        );
        capture_action(
            &mut action,
            render_column(&mut columns[1], history, selected, COLUMN_LIMIT),
        );
    });

    action
}

fn capture_action(action: &mut HistoryAction, new_action: HistoryAction) {
    if matches!(action, HistoryAction::None) {
        *action = new_action;
    }
}

fn render_column(
    ui: &mut egui::Ui,
    history: &[PickRecord],
    selected: usize,
    start: usize,
) -> HistoryAction {
    ui.set_min_height(MAX_HEIGHT);
    ui.spacing_mut().item_spacing.y = 4.0;

    let mut action = HistoryAction::None;
    let end = history.len().min(start + COLUMN_LIMIT).min(VISIBLE_LIMIT);

    for index in start..end {
        if render_history_item(ui, &history[index], selected == index) {
            action = HistoryAction::Select(index);
        }
    }

    action
}

fn render_history_item(ui: &mut egui::Ui, record: &PickRecord, selected: bool) -> bool {
    let mut clicked = false;

    ui.horizontal(|ui| {
        ui.set_height(ROW_HEIGHT);
        ui.spacing_mut().item_spacing.x = 14.0;
        draw_swatch(
            ui,
            record.color,
            record.color.a as f32 / 255.0,
            egui::vec2(28.0, LABEL_HEIGHT),
        );

        let label_width = ui.available_width().max(48.0);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(label_width, LABEL_HEIGHT), Sense::click());
        let visuals = ui.style().interact_selectable(&response, selected);

        if selected || response.hovered() {
            ui.painter().rect_filled(rect, 3.0, visuals.weak_bg_fill);
        }

        let text_rect = Rect::from_min_max(
            rect.left_center() + egui::vec2(8.0, -LABEL_HEIGHT / 2.0),
            rect.right_center() + egui::vec2(-8.0, LABEL_HEIGHT / 2.0),
        );

        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(text_rect)
                .layout(egui::Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_width(text_rect.width());
                ui.add(
                    egui::Label::new(RichText::new(record.summary()).monospace())
                        .truncate()
                        .halign(Align::Min)
                        .selectable(false),
                );
            },
        );

        clicked = response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked();
    });

    clicked
}
