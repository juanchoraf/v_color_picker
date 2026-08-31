use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, TextureHandle, Vec2};

use crate::color;
use crate::drawing::{draw_crosshair, egui_color};
use crate::screen::{ScreenCapture, ScreenSample};

pub(crate) struct PickerOverlay {
    pub(crate) capture: ScreenCapture,
    texture: TextureHandle,
    wait_for_release: bool,
}

pub(crate) enum OverlayAction {
    None,
    Pick(ScreenSample),
    Cancel,
}

impl PickerOverlay {
    pub(crate) fn new(
        capture: ScreenCapture,
        texture: TextureHandle,
        wait_for_release: bool,
    ) -> Self {
        Self {
            capture,
            texture,
            wait_for_release,
        }
    }

    pub(crate) fn render(&mut self, ui: &mut egui::Ui, zoom: f32, action: &mut OverlayAction) {
        let rect = ui.max_rect();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let uv = Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0));
        ui.painter()
            .image(self.texture.id(), rect, uv, Color32::WHITE);

        let cancelled = ui.input(|input| {
            input.key_pressed(egui::Key::Escape) || input.viewport().close_requested()
        });
        if cancelled {
            *action = OverlayAction::Cancel;
            return;
        }

        let hover_pos = ui.input(|input| input.pointer.hover_pos());
        let Some(pos) = hover_pos.filter(|pos| rect.contains(*pos)) else {
            return;
        };

        let Some((virtual_x, virtual_y)) = self.virtual_pos(rect, pos) else {
            return;
        };

        if let Some(sample) = self.capture.sample_virtual(virtual_x, virtual_y) {
            draw_crosshair(ui.painter(), pos, Color32::WHITE, 18.0);
            self.draw_zoom(ui, rect, pos, virtual_x, virtual_y, zoom, &sample);

            if self.wait_for_release {
                if !ui.input(|input| input.pointer.primary_down()) {
                    self.wait_for_release = false;
                }
                return;
            }

            if response.clicked() {
                *action = OverlayAction::Pick(sample);
            }
        }
    }

    fn virtual_pos(&self, rect: Rect, pos: Pos2) -> Option<(u32, u32)> {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return None;
        }

        let x = ((pos.x - rect.left()) / rect.width() * self.capture.width as f32).floor();
        let y = ((pos.y - rect.top()) / rect.height() * self.capture.height as f32).floor();
        Some((
            x.clamp(0.0, self.capture.width.saturating_sub(1) as f32) as u32,
            y.clamp(0.0, self.capture.height.saturating_sub(1) as f32) as u32,
        ))
    }

    fn draw_zoom(
        &self,
        ui: &mut egui::Ui,
        viewport: Rect,
        cursor: Pos2,
        virtual_x: u32,
        virtual_y: u32,
        zoom: f32,
        sample: &ScreenSample,
    ) {
        let grid = zoom.round().clamp(5.0, 21.0) as i32;
        let radius = grid / 2;
        let padding = 8.0;
        let max_panel_width = (viewport.width() - 8.0).max(48.0);
        let panel_width = (grid as f32 * 8.0 + padding * 2.0)
            .max(96.0)
            .min(max_panel_width);
        let text_width = (panel_width - padding * 2.0).max(32.0);
        let cell = ((panel_width - padding * 2.0) / grid as f32)
            .min(8.0)
            .max(1.0);
        let grid_size = grid as f32 * cell;

        let painter = ui.painter();
        let hex = color::formats(sample.color, sample.color.a as f32 / 255.0)
            .into_iter()
            .find(|row| row.label == "HEX")
            .map(|row| row.value)
            .unwrap_or_else(|| "#000000".to_owned());
        let info = painter.layout(
            format!("{hex}\n{},{}", sample.global_x, sample.global_y),
            FontId::monospace(13.0),
            Color32::WHITE,
            text_width,
        );
        let monitor = painter.layout(
            sample.monitor_name.clone(),
            FontId::monospace(12.0),
            Color32::from_gray(190),
            text_width,
        );
        let text_gap = 4.0;
        let text_height = info.size().y + text_gap + monitor.size().y;
        let panel_height = padding + grid_size + 8.0 + text_height + padding;
        let panel_size = Vec2::new(panel_width, panel_height);
        let mut panel_origin = cursor + egui::vec2(24.0, 24.0);

        if panel_origin.x + panel_size.x > viewport.right() {
            panel_origin.x = cursor.x - panel_size.x - 24.0;
        }
        if panel_origin.y + panel_size.y > viewport.bottom() {
            panel_origin.y = cursor.y - panel_size.y - 24.0;
        }
        let (min_x, min_y) = (viewport.left() + 4.0, viewport.top() + 4.0);
        panel_origin.x = panel_origin
            .x
            .clamp(min_x, (viewport.right() - panel_size.x - 4.0).max(min_x));
        panel_origin.y = panel_origin
            .y
            .clamp(min_y, (viewport.bottom() - panel_size.y - 4.0).max(min_y));

        let panel_rect = Rect::from_min_size(panel_origin, panel_size);
        painter.rect_filled(
            panel_rect,
            6.0,
            Color32::from_rgba_unmultiplied(12, 15, 20, 235),
        );
        let origin = panel_origin + egui::vec2((panel_width - grid_size) * 0.5, padding);

        for y in -radius..=radius {
            for x in -radius..=radius {
                let sx = virtual_x as i32 + x;
                let sy = virtual_y as i32 + y;
                let color = if sx >= 0 && sy >= 0 {
                    self.capture
                        .sample_virtual(sx as u32, sy as u32)
                        .map(|sample| egui_color(sample.color))
                        .unwrap_or(Color32::BLACK)
                } else {
                    Color32::BLACK
                };
                let cell_rect = Rect::from_min_size(
                    origin + egui::vec2((x + radius) as f32 * cell, (y + radius) as f32 * cell),
                    Vec2::splat(cell),
                );
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }

        let center = origin
            + egui::vec2(
                radius as f32 * cell + cell / 2.0,
                radius as f32 * cell + cell / 2.0,
            );
        draw_crosshair(painter, center, Color32::WHITE, cell * 1.5);
        let clipped = painter.with_clip_rect(panel_rect.shrink(padding));
        let text_origin = egui::pos2(
            panel_rect.left() + padding,
            panel_rect.top() + padding + grid_size + 8.0,
        );
        clipped.galley(text_origin, info, Color32::WHITE);
        clipped.galley(
            text_origin + egui::vec2(0.0, text_height - monitor.size().y),
            monitor,
            Color32::from_gray(190),
        );
    }
}
