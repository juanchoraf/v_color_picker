use eframe::egui::{self, Color32, Rect, Sense};

use crate::color::{self, RgbaColor};
use crate::drawing::{draw_crosshair, egui_color};

pub const HEIGHT: f32 = 143.0;

const RELATED_CELLS_X: usize = 36;
const RELATED_CELLS_Y: usize = 18;
const DEFAULT_CELLS_X: usize = 72;
const DEFAULT_CELLS_Y: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteTab {
    Related,
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaletteMode {
    Related,
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteAction {
    None,
    Select {
        color: RgbaColor,
        source: &'static str,
        update_palette_base: bool,
        update_default_cursor: bool,
    },
}

pub fn render(
    ui: &mut egui::Ui,
    active: &mut PaletteTab,
    related_cursor: &mut [f32; 2],
    default_cursor: &mut [f32; 2],
    base: RgbaColor,
    alpha: f32,
    size: egui::Vec2,
) -> PaletteAction {
    render_tabs(ui, active);
    ui.add_space(4.0);

    match *active {
        PaletteTab::Related => render_surface(
            ui,
            PaletteMode::Related,
            related_cursor,
            size,
            RELATED_CELLS_X,
            RELATED_CELLS_Y,
            |x, y| color::palette_color(base, x, y).with_alpha(alpha),
        )
        .map_or(PaletteAction::None, |color| PaletteAction::Select {
            color,
            source: "palette",
            update_palette_base: false,
            update_default_cursor: true,
        }),
        PaletteTab::Default => render_surface(
            ui,
            PaletteMode::Default,
            default_cursor,
            size,
            DEFAULT_CELLS_X,
            DEFAULT_CELLS_Y,
            |x, y| color::default_palette_color(x, y, alpha),
        )
        .map_or(PaletteAction::None, |color| PaletteAction::Select {
            color,
            source: "default palette",
            update_palette_base: true,
            update_default_cursor: false,
        }),
    }
}

fn render_tabs(ui: &mut egui::Ui, active: &mut PaletteTab) {
    ui.horizontal(|ui| {
        ui.selectable_value(active, PaletteTab::Related, "Related Palette");
        ui.selectable_value(active, PaletteTab::Default, "Default Palette");
    });
}

fn render_surface(
    ui: &mut egui::Ui,
    mode: PaletteMode,
    cursor: &mut [f32; 2],
    size: egui::Vec2,
    cells_x: usize,
    cells_y: usize,
    mut color_at: impl FnMut(f32, f32) -> RgbaColor,
) -> Option<RgbaColor> {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let painter = ui.painter();

    for y in 0..cells_y {
        for x in 0..cells_x {
            let nx = x as f32 / (cells_x - 1) as f32;
            let ny = y as f32 / (cells_y - 1) as f32;
            let cell = Rect::from_min_max(
                egui::pos2(
                    rect.left() + rect.width() * x as f32 / cells_x as f32,
                    rect.top() + rect.height() * y as f32 / cells_y as f32,
                ),
                egui::pos2(
                    rect.left() + rect.width() * (x + 1) as f32 / cells_x as f32,
                    rect.top() + rect.height() * (y + 1) as f32 / cells_y as f32,
                ),
            );

            painter.rect_filled(cell, 0.0, egui_color(color_at(nx, ny)));
        }
    }

    let selected = handle_pointer(&response, rect, cursor, &mut color_at);
    let cursor_pos = egui::pos2(
        rect.left() + rect.width() * cursor[0],
        rect.top() + rect.height() * cursor[1],
    );
    draw_crosshair(painter, cursor_pos, cursor_color(mode, cursor), 11.0);
    selected
}

fn handle_pointer(
    response: &egui::Response,
    rect: Rect,
    cursor: &mut [f32; 2],
    color_at: &mut impl FnMut(f32, f32) -> RgbaColor,
) -> Option<RgbaColor> {
    if !response.clicked() && !response.dragged() {
        return None;
    }

    let pos = response.interact_pointer_pos()?;
    let nx = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    let ny = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
    *cursor = [nx, ny];
    Some(color_at(nx, ny))
}

fn cursor_color(mode: PaletteMode, cursor: &[f32; 2]) -> Color32 {
    match mode {
        PaletteMode::Default if cursor[1] < 0.35 => Color32::BLACK,
        _ => Color32::WHITE,
    }
}
