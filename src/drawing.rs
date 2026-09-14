use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::color::RgbaColor;

pub fn draw_logo_mark(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 6.0, Color32::from_rgb(27, 33, 43));

    let dropper = Color32::from_rgb(136, 143, 246);
    let picked = Color32::from_rgb(38, 166, 154);
    let top = rect.min + egui::vec2(size * 0.7, size * 0.23);
    let bottom = rect.min + egui::vec2(size * 0.36, size * 0.57);
    let tip = rect.min + egui::vec2(size * 0.25, size * 0.68);
    let picked_color = rect.min + egui::vec2(size * 0.23, size * 0.78);

    painter.line_segment([top, bottom], Stroke::new(size * 0.16, dropper));
    painter.line_segment(
        [
            rect.min + egui::vec2(size * 0.8, size * 0.13),
            rect.min + egui::vec2(size * 0.61, size * 0.32),
        ],
        Stroke::new(size * 0.14, dropper),
    );
    painter.line_segment([bottom, tip], Stroke::new(size * 0.08, dropper));
    painter.circle_filled(picked_color, size * 0.11, picked);
}

pub fn draw_swatch(ui: &mut egui::Ui, color: RgbaColor, alpha: f32, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    draw_checkerboard(ui.painter(), rect, 10.0);
    ui.painter()
        .rect_filled(rect, 0.0, egui_color(color.with_alpha(alpha)));
}

pub fn small_swatch(ui: &mut egui::Ui, color: RgbaColor) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    ui.painter().rect_filled(rect, 0.0, egui_color(color));
    response
}

#[derive(Clone, Copy, Debug)]
pub enum ButtonIcon {
    Copy,
    DefaultPalette,
    Pick,
    RelatedPalette,
    Trash,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Active,
    Normal,
    Success,
}

pub fn icon_button(
    ui: &mut egui::Ui,
    icon: ButtonIcon,
    label: &str,
    min_width: f32,
) -> egui::Response {
    icon_button_with_state(ui, icon, label, min_width, ButtonState::Normal)
}

pub fn icon_button_with_state(
    ui: &mut egui::Ui,
    icon: ButtonIcon,
    label: &str,
    min_width: f32,
    state: ButtonState,
) -> egui::Response {
    let enabled = ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let height = 38.0;
    let width = min_width.max(92.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), sense);

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let accent = Color32::from_rgb(0x3F, 0xB3, 0xD8);
        let success = Color32::from_rgb(0x22, 0xAB, 0x94);
        let muted = Color32::from_gray(105);
        let content_color = match (enabled, icon) {
            (false, _) => muted,
            (true, ButtonIcon::Pick) => accent,
            (true, ButtonIcon::Update) if response.hovered() => {
                Color32::from_rgb(0x6A, 0xE0, 0xB5)
            }
            _ => visuals.text_color(),
        };
        let border_color = match (enabled, icon) {
            (false, _) => Color32::from_rgb(54, 62, 74),
            (true, ButtonIcon::Pick) => accent,
            _ => Color32::from_rgb(74, 84, 98),
        };
        let (fill, border_color, content_color) = match state {
            ButtonState::Active => (Color32::from_rgb(0x18, 0x73, 0x92), accent, Color32::WHITE),
            ButtonState::Normal => {
                let fill = if matches!(icon, ButtonIcon::Update) {
                    Color32::from_rgb(0x1A, 0x20, 0x2A)
                } else if enabled && response.hovered() {
                    Color32::from_rgb(53, 61, 73)
                } else {
                    Color32::from_rgb(38, 44, 54)
                };
                (fill, border_color, content_color)
            }
            ButtonState::Success => (success, success, Color32::WHITE),
        };
        let stroke = Stroke::new(1.0, border_color);

        ui.painter().rect_filled(rect, 6.0, fill);
        ui.painter()
            .rect_stroke(rect, 6.0, stroke, StrokeKind::Inside);
        let font_id = FontId::proportional(14.0);
        let text_width = ui
            .painter()
            .layout_no_wrap(label.to_owned(), font_id.clone(), content_color)
            .size()
            .x;
        let icon_width = 20.0;
        let gap = 8.0;
        let content_width = icon_width + gap + text_width;
        let content_left = rect.center().x - content_width / 2.0;
        let icon_center = egui::pos2(content_left + icon_width / 2.0, rect.center().y);

        draw_button_icon(ui.painter(), icon, icon_center, content_color, fill);
        ui.painter().text(
            egui::pos2(content_left + icon_width + gap, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font_id,
            content_color,
        );
    }

    response
}

pub fn icon_only_button(
    ui: &mut egui::Ui,
    icon: ButtonIcon,
    size: Vec2,
    tooltip: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let border_color = Color32::from_rgb(74, 84, 98);
        let fill = if response.hovered() {
            Color32::from_rgb(53, 61, 73)
        } else {
            Color32::from_rgb(38, 44, 54)
        };

        ui.painter().rect_filled(rect, 6.0, fill);
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, border_color),
            StrokeKind::Inside,
        );
        draw_button_icon(
            ui.painter(),
            icon,
            rect.center(),
            visuals.text_color(),
            fill,
        );
    }

    response.on_hover_text(tooltip)
}

pub fn text_button(ui: &mut egui::Ui, label: &str, size: Vec2) -> egui::Response {
    let enabled = ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let border_color = Color32::from_rgb(74, 84, 98);
        let content_color = if enabled {
            visuals.text_color()
        } else {
            Color32::from_gray(105)
        };
        let fill = if enabled && response.hovered() {
            Color32::from_rgb(53, 61, 73)
        } else {
            Color32::from_rgb(38, 44, 54)
        };

        ui.painter().rect_filled(rect, 6.0, fill);
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, border_color),
            StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(14.0),
            content_color,
        );
    }

    response
}

pub fn draw_crosshair(painter: &egui::Painter, center: Pos2, color: Color32, radius: f32) {
    let stroke = Stroke::new(1.5, color);
    painter.line_segment(
        [
            center + egui::vec2(-radius, 0.0),
            center + egui::vec2(radius, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(0.0, -radius),
            center + egui::vec2(0.0, radius),
        ],
        stroke,
    );
}

pub fn egui_color(color: RgbaColor) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn draw_checkerboard(painter: &egui::Painter, rect: Rect, cell: f32) {
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;

    for y in 0..rows {
        for x in 0..cols {
            let color = if (x + y) % 2 == 0 {
                Color32::from_rgb(210, 214, 221)
            } else {
                Color32::from_rgb(145, 151, 162)
            };
            let tile = Rect::from_min_size(
                rect.min + egui::vec2(x as f32 * cell, y as f32 * cell),
                Vec2::splat(cell),
            )
            .intersect(rect);
            painter.rect_filled(tile, 0.0, color);
        }
    }
}

fn draw_button_icon(
    painter: &egui::Painter,
    icon: ButtonIcon,
    center: Pos2,
    color: Color32,
    fill: Color32,
) {
    match icon {
        ButtonIcon::Copy => {
            let back =
                Rect::from_center_size(center + egui::vec2(-2.5, -2.5), Vec2::new(10.0, 12.0));
            let front =
                Rect::from_center_size(center + egui::vec2(2.5, 2.5), Vec2::new(10.0, 12.0));
            painter.rect_stroke(back, 2.0, Stroke::new(1.4, color), StrokeKind::Inside);
            painter.rect_filled(front, 2.0, fill);
            painter.rect_stroke(front, 2.0, Stroke::new(1.4, color), StrokeKind::Inside);
        }
        ButtonIcon::DefaultPalette => {
            let stroke = Stroke::new(1.4, color);
            painter.circle_stroke(center, 8.5, stroke);
            painter.circle_filled(center + egui::vec2(3.8, 3.0), 3.2, fill);
            painter.circle_stroke(center + egui::vec2(3.8, 3.0), 3.2, stroke);
            painter.circle_filled(center + egui::vec2(-4.2, -3.5), 1.7, color);
            painter.circle_filled(center + egui::vec2(1.2, -5.1), 1.7, color);
            painter.circle_filled(center + egui::vec2(-5.6, 2.0), 1.7, color);
        }
        ButtonIcon::Pick => {
            painter.circle_stroke(center, 7.0, Stroke::new(1.6, color));
            painter.line_segment(
                [
                    center + egui::vec2(-10.0, 0.0),
                    center + egui::vec2(-3.0, 0.0),
                ],
                Stroke::new(1.5, color),
            );
            painter.line_segment(
                [
                    center + egui::vec2(3.0, 0.0),
                    center + egui::vec2(10.0, 0.0),
                ],
                Stroke::new(1.5, color),
            );
            painter.line_segment(
                [
                    center + egui::vec2(0.0, -10.0),
                    center + egui::vec2(0.0, -3.0),
                ],
                Stroke::new(1.5, color),
            );
            painter.line_segment(
                [
                    center + egui::vec2(0.0, 3.0),
                    center + egui::vec2(0.0, 10.0),
                ],
                Stroke::new(1.5, color),
            );
        }
        ButtonIcon::RelatedPalette => {
            let handle = Stroke::new(2.0, color);
            let ferrule = Stroke::new(1.4, color);
            painter.line_segment(
                [
                    center + egui::vec2(-7.0, 7.0),
                    center + egui::vec2(2.0, -2.0),
                ],
                handle,
            );
            painter.line_segment(
                [
                    center + egui::vec2(0.0, -4.0),
                    center + egui::vec2(5.0, 1.0),
                ],
                ferrule,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-2.5, -1.5),
                    center + egui::vec2(2.5, 3.5),
                ],
                ferrule,
            );
            let bristles = vec![
                center + egui::vec2(2.3, -6.8),
                center + egui::vec2(8.4, -9.0),
                center + egui::vec2(6.3, -2.6),
            ];
            painter.add(egui::Shape::convex_polygon(bristles, color, Stroke::NONE));
        }
        ButtonIcon::Trash => {
            let stroke = Stroke::new(1.5, color);
            let body = Rect::from_min_max(
                center + egui::vec2(-6.0, -3.0),
                center + egui::vec2(6.0, 8.0),
            );
            painter.line_segment(
                [
                    center + egui::vec2(-8.0, -6.0),
                    center + egui::vec2(8.0, -6.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-3.5, -8.0),
                    center + egui::vec2(3.5, -8.0),
                ],
                stroke,
            );
            painter.rect_stroke(body, 1.5, stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    center + egui::vec2(-2.5, -1.0),
                    center + egui::vec2(-2.5, 6.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(2.5, -1.0),
                    center + egui::vec2(2.5, 6.0),
                ],
                stroke,
            );
        }
        ButtonIcon::Update => {
            let stroke = Stroke::new(1.5, color);
            painter.line_segment(
                [
                    center + egui::vec2(0.0, -8.0),
                    center + egui::vec2(0.0, 3.5),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-4.5, -1.0),
                    center + egui::vec2(0.0, 3.5),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(4.5, -1.0),
                    center + egui::vec2(0.0, 3.5),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-7.0, 8.0),
                    center + egui::vec2(7.0, 8.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-7.0, 4.0),
                    center + egui::vec2(-7.0, 8.0),
                ],
                stroke,
            );
            painter.line_segment(
                [center + egui::vec2(7.0, 4.0), center + egui::vec2(7.0, 8.0)],
                stroke,
            );
        }
    }
}
