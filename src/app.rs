use std::time::{Duration, Instant};

use arboard::Clipboard;
use eframe::egui::{self, Align, Color32, Layout, RichText};

use crate::color::{self, ColorFormat, RgbaColor};
use crate::drawing::{
    ButtonIcon, ButtonState, draw_logo_mark, draw_swatch, icon_button, icon_button_with_state,
    small_swatch,
};
use crate::gui_update::{self, GuiUpdate};
use crate::history::{self, PickRecord};
use crate::history_view::{self, HistoryAction};
use crate::icon;
use crate::palette::{self, PaletteAction, PaletteTab};
use crate::picker_overlay::{OverlayAction, PickerOverlay};
use crate::screen::{ScreenCapture, ScreenSample};

const PICKER_VIEWPORT: &str = "v_color_picker_picker_overlay";
const DEFAULT_FORMAT: &str = "HEX";
const OUTER_MARGIN: i8 = 24;
const WORKSPACE_WIDTH: f32 = 520.0;
const SLIDER_WIDTH: f32 = 220.0;
const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 25.0;
const DEFAULT_ZOOM: f32 = 17.0;
const PICK_HIDE_SETTLE_MS: u64 = 16;
const COPY_FEEDBACK_MS: u64 = 1_500;

pub const APP_TITLE: &str = "V_Color_Picker";

pub struct VColorPickerApp {
    history: Vec<PickRecord>,
    selected: usize,
    current_color: RgbaColor,
    alpha: f32,
    zoom: f32,
    palette_base: RgbaColor,
    related_palette_cursor: [f32; 2],
    default_palette_cursor: [f32; 2],
    selected_palette: PaletteTab,
    selected_format: String,
    overlay: Option<PickerOverlay>,
    main_window_hidden_for_picker: bool,
    main_focus_retry_frames: u8,
    copy_feedback_until: Option<Instant>,
    updates: GuiUpdate,
    status: String,
}

impl VColorPickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let history = history::load(cc.storage);
        let selected_color = history
            .first()
            .map(|record| record.color)
            .unwrap_or_else(|| PickRecord::starter().color);

        Self {
            history,
            selected: 0,
            current_color: selected_color,
            alpha: selected_color.a as f32 / 255.0,
            zoom: DEFAULT_ZOOM,
            palette_base: selected_color,
            related_palette_cursor: [0.5, 0.5],
            default_palette_cursor: color::default_palette_cursor(selected_color),
            selected_palette: PaletteTab::Related,
            selected_format: DEFAULT_FORMAT.to_owned(),
            overlay: None,
            main_window_hidden_for_picker: false,
            main_focus_retry_frames: 0,
            copy_feedback_until: None,
            updates: GuiUpdate::new(),
            status: "Ready".to_owned(),
        }
    }

    fn selected_color(&self) -> RgbaColor {
        self.current_color
    }

    fn selected_formats(&self) -> Vec<ColorFormat> {
        color::formats(self.selected_color(), self.alpha)
    }

    fn start_pick(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.overlay.is_some() {
            return;
        }

        self.status = "Capturing screens".to_owned();

        if let Some(window) = frame.winit_window() {
            window.set_visible(false);
            self.main_window_hidden_for_picker = true;
            std::thread::sleep(Duration::from_millis(PICK_HIDE_SETTLE_MS));
        }

        let result = ScreenCapture::capture_all();

        match result {
            Ok(capture) => {
                let image = capture.to_color_image();
                let texture = ctx.load_texture(
                    "v_color_picker_screen",
                    image,
                    egui::TextureOptions::NEAREST,
                );
                let count = capture.monitors.len();
                self.overlay = Some(PickerOverlay::new(capture, texture, true));
                self.status = format!("Picker armed across {count} monitor(s)");
                ctx.request_repaint();
            }
            Err(error) => {
                if let Some(window) = frame.winit_window() {
                    window.set_visible(true);
                    window.request_redraw();
                }
                self.main_window_hidden_for_picker = false;
                self.status = format!("Capture failed: {error}");
            }
        }
    }

    fn restore_main_window(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.bring_main_window_forward(ctx, frame);
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::RequestUserAttention(egui::UserAttentionType::Informational),
        );
        self.main_window_hidden_for_picker = false;
        self.main_focus_retry_frames = 6;
        ctx.request_repaint();
    }

    fn bring_main_window_forward(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(window) = frame.winit_window() {
            window.set_visible(true);
            window.set_minimized(false);
            window.request_redraw();
            window.focus_window();
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::Minimized(false),
        );
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
    }

    fn show_main_window_behind_picker(&mut self, ctx: &egui::Context) {
        if !self.main_window_hidden_for_picker {
            return;
        }

        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::Minimized(false),
        );
        self.main_window_hidden_for_picker = false;
        ctx.request_repaint();
    }

    fn apply_pending_main_focus(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.main_focus_retry_frames == 0 {
            return;
        }

        self.bring_main_window_forward(ctx, frame);
        self.main_focus_retry_frames -= 1;

        if self.main_focus_retry_frames > 0 {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn add_sample(&mut self, sample: ScreenSample) {
        let color = sample.color;
        self.history.insert(0, PickRecord::from_sample(sample));
        history::clamp(&mut self.history);
        self.selected = 0;
        self.current_color = color;
        self.alpha = color.a as f32 / 255.0;
        self.palette_base = color;
        self.related_palette_cursor = [0.5, 0.5];
        self.default_palette_cursor = color::default_palette_cursor(color);
        self.status = "Color picked".to_owned();
    }

    fn copy_text(&mut self, ctx: &egui::Context, value: &str) {
        match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(value.to_owned())) {
            Ok(()) => self.status = format!("Copied {value}"),
            Err(_) => {
                ctx.copy_text(value.to_owned());
                self.status = format!("Copied {value}");
            }
        }

        self.copy_feedback_until = Some(Instant::now() + Duration::from_millis(COPY_FEEDBACK_MS));
        ctx.request_repaint_after(Duration::from_millis(COPY_FEEDBACK_MS));
    }

    fn copy_selected_format(&mut self, ctx: &egui::Context) {
        let formats = self.selected_formats();
        self.ensure_selected_format(&formats);

        if let Some(value) = formats
            .iter()
            .find(|row| row.label == self.selected_format.as_str())
            .or_else(|| formats.first())
            .map(|row| row.value.clone())
        {
            self.copy_text(ctx, &value);
        }
    }

    fn copy_button_state(&mut self, ctx: &egui::Context) -> ButtonState {
        let Some(until) = self.copy_feedback_until else {
            return ButtonState::Normal;
        };

        let now = Instant::now();
        if now >= until {
            self.copy_feedback_until = None;
            return ButtonState::Normal;
        }

        ctx.request_repaint_after(until - now);
        ButtonState::Success
    }

    fn update_current_color(
        &mut self,
        color: RgbaColor,
        source: &str,
        update_palette_base: bool,
        update_default_cursor: bool,
    ) {
        self.current_color = color;
        if update_default_cursor {
            self.default_palette_cursor = color::default_palette_cursor(color);
        }
        if update_palette_base {
            self.palette_base = color;
            self.related_palette_cursor = [0.5, 0.5];
        }

        self.status = format!("{source} color selected");
    }

    fn render_main(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(16, 19, 24));

        egui::Frame::new()
            .inner_margin(egui::Margin::same(OUTER_MARGIN))
            .show(ui, |ui| {
                ui.set_width(WORKSPACE_WIDTH);
                ui.spacing_mut().item_spacing = egui::vec2(14.0, 10.0);
                self.render_header(ui, frame);
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(12.0);
                self.render_color_workspace(ui, frame);
            });
    }

    fn render_header(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            draw_logo_mark(ui, 42.0);
            ui.heading(APP_TITLE).on_hover_text(self.status.as_str());

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let pick = icon_button(ui, ButtonIcon::Pick, "Pick", 104.0);
                if pick.is_pointer_button_down_on() || pick.clicked() {
                    self.start_pick(ui.ctx(), frame);
                }
            });
        });
    }

    fn render_color_workspace(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let color = self.selected_color();
        ui.label(RichText::new("Selected Color").strong());
        ui.add_space(4.0);
        draw_swatch(
            ui,
            color,
            self.alpha,
            egui::vec2(ui.available_width().min(WORKSPACE_WIDTH), 64.0),
        );

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.set_height(24.0);
            let alpha_changed = ui
                .add_sized(
                    egui::vec2(SLIDER_WIDTH, 20.0),
                    egui::Slider::new(&mut self.alpha, 0.0..=1.0).text("Alpha"),
                )
                .changed();
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.add_sized(
                    egui::vec2(SLIDER_WIDTH, 20.0),
                    egui::Slider::new(&mut self.zoom, MIN_ZOOM..=MAX_ZOOM).text("Zoom"),
                );
            });

            if alpha_changed {
                self.select_alpha_format_if_needed();
            }
        });

        ui.add_space(10.0);
        self.render_formats(ui);

        ui.add_space(10.0);
        self.render_palettes(ui);

        ui.add_space(10.0);
        ui.label(RichText::new("Alternatives").strong());
        ui.horizontal_wrapped(|ui| {
            for index in 0..12 {
                let swatch = color::similar_swatch(color, index, 12);
                let response = small_swatch(ui, swatch);
                if response.clicked() {
                    self.update_current_color(swatch.with_alpha(self.alpha), "swatch", true, true);
                }
            }
        });

        ui.add_space(10.0);
        match history_view::render(ui, &self.history, self.selected) {
            HistoryAction::None => {}
            HistoryAction::Select(index) => self.select_history(index),
            HistoryAction::Clear => self.clear_history(frame.storage_mut()),
        }

        ui.add_space(14.0);
        gui_update::render_bottom_right(ui, &mut self.updates, &mut self.status);
    }

    fn render_palettes(&mut self, ui: &mut egui::Ui) {
        let size = egui::vec2(ui.available_width().min(WORKSPACE_WIDTH), palette::HEIGHT);
        match palette::render(
            ui,
            &mut self.selected_palette,
            &mut self.related_palette_cursor,
            &mut self.default_palette_cursor,
            self.palette_base,
            self.alpha,
            size,
        ) {
            PaletteAction::None => {}
            PaletteAction::Select {
                color,
                source,
                update_palette_base,
                update_default_cursor,
            } => {
                self.update_current_color(color, source, update_palette_base, update_default_cursor)
            }
        }
    }

    fn render_formats(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Formats").strong());
        let formats = self.selected_formats();
        self.ensure_selected_format(&formats);

        let row = formats
            .iter()
            .find(|row| row.label == self.selected_format)
            .or_else(|| formats.first())
            .cloned();

        if let Some(row) = row {
            let copy_button_state = self.copy_button_state(ui.ctx());
            let copy_button_label = match copy_button_state {
                ButtonState::Active => "Copy",
                ButtonState::Normal => "Copy",
                ButtonState::Success => "Copied!",
            };

            ui.horizontal(|ui| {
                ui.set_height(42.0);
                self.render_format_dropdown(ui, &formats);
                let copy_width = 96.0;
                let value_width =
                    (ui.available_width() - copy_width - ui.spacing().item_spacing.x).max(130.0);
                ui.add_sized(
                    egui::vec2(value_width, 38.0),
                    egui::Label::new(RichText::new(&row.value).monospace()).truncate(),
                );
                if icon_button_with_state(
                    ui,
                    ButtonIcon::Copy,
                    copy_button_label,
                    copy_width,
                    copy_button_state,
                )
                .clicked()
                {
                    self.copy_text(ui.ctx(), &row.value);
                }
            });
        }
    }

    fn ensure_selected_format(&mut self, formats: &[ColorFormat]) {
        if !formats.iter().any(|row| row.label == self.selected_format) {
            self.selected_format = DEFAULT_FORMAT.to_owned();
        }
    }

    fn select_alpha_format_if_needed(&mut self) {
        if let Some(format) = Self::closest_alpha_format(&self.selected_format) {
            self.selected_format = format.to_owned();
        }
    }

    fn closest_alpha_format(format: &str) -> Option<&'static str> {
        match format {
            "HEX" => Some("HEXA"),
            "RGB" => Some("RGBA"),
            "HSL" => Some("HSLA"),
            _ => None,
        }
    }

    fn render_format_dropdown(&mut self, ui: &mut egui::Ui, formats: &[ColorFormat]) {
        let selected = self.selected_format.clone();
        let width = Self::format_dropdown_width(formats);

        ui.scope(|ui| {
            let stroke = egui::Stroke::new(1.0, Color32::from_rgb(74, 84, 98));
            let radius = egui::CornerRadius::same(6);

            ui.spacing_mut().button_padding = egui::vec2(12.0, 10.0);
            ui.spacing_mut().interact_size.y = 38.0;
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::from_rgb(38, 44, 54);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::from_rgb(53, 61, 73);
            ui.visuals_mut().widgets.active.weak_bg_fill = Color32::from_rgb(53, 61, 73);
            ui.visuals_mut().widgets.open.weak_bg_fill = Color32::from_rgb(53, 61, 73);
            ui.visuals_mut().widgets.inactive.bg_stroke = stroke;
            ui.visuals_mut().widgets.hovered.bg_stroke = stroke;
            ui.visuals_mut().widgets.active.bg_stroke = stroke;
            ui.visuals_mut().widgets.open.bg_stroke = stroke;
            ui.visuals_mut().widgets.inactive.corner_radius = radius;
            ui.visuals_mut().widgets.hovered.corner_radius = radius;
            ui.visuals_mut().widgets.active.corner_radius = radius;
            ui.visuals_mut().widgets.open.corner_radius = radius;

            egui::ComboBox::from_id_salt("format_selector")
                .selected_text(selected)
                .width(width)
                .height(320.0)
                .show_ui(ui, |ui| {
                    for row in formats {
                        ui.selectable_value(
                            &mut self.selected_format,
                            row.label.to_owned(),
                            row.label,
                        );
                    }
                });
        });
    }

    fn format_dropdown_width(formats: &[ColorFormat]) -> f32 {
        let longest = formats
            .iter()
            .map(|row| row.label.chars().count())
            .max()
            .unwrap_or(DEFAULT_FORMAT.len()) as f32;

        (longest * 8.0 + 36.0).ceil().max(84.0)
    }

    fn select_history(&mut self, index: usize) {
        if let Some(record) = self.history.get(index) {
            self.selected = index;
            self.current_color = record.color;
            self.alpha = record.color.a as f32 / 255.0;
            self.palette_base = record.color;
            self.related_palette_cursor = [0.5, 0.5];
            self.default_palette_cursor = color::default_palette_cursor(record.color);
        }
    }

    fn clear_history(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        let color = PickRecord::starter().color;
        self.history.clear();
        self.selected = 0;
        self.current_color = color;
        self.alpha = 1.0;
        self.palette_base = color;
        self.related_palette_cursor = [0.5, 0.5];
        self.default_palette_cursor = color::default_palette_cursor(color);
        self.status = "History deleted".to_owned();

        if let Some(storage) = storage {
            history::save(storage, &self.history);
        }
    }

    fn render_overlay(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut action = OverlayAction::None;
        let zoom = self.zoom;
        let picker_id = egui::ViewportId::from_hash_of(PICKER_VIEWPORT);

        if let Some(overlay) = self.overlay.as_mut() {
            let capture = &overlay.capture;
            let builder = egui::ViewportBuilder::default()
                .with_title(format!("{APP_TITLE} picker"))
                .with_app_id("com.thevelasquez.v_color_picker.picker")
                .with_position(egui::pos2(capture.origin_x as f32, capture.origin_y as f32))
                .with_inner_size(egui::vec2(capture.width as f32, capture.height as f32))
                .with_clamp_size_to_monitor_size(false)
                .with_decorations(false)
                .with_resizable(false)
                .with_taskbar(false)
                .with_always_on_top()
                .with_active(true)
                .with_icon(icon::app_icon());

            ctx.show_viewport_immediate(picker_id, builder, |ui, _class| {
                overlay.render(ui, zoom, &mut action)
            });
            self.show_main_window_behind_picker(ctx);
        }

        match action {
            OverlayAction::None => {}
            OverlayAction::Pick(sample) => {
                self.overlay = None;
                ctx.send_viewport_cmd_to(picker_id, egui::ViewportCommand::Close);
                self.add_sample(sample);
                self.copy_selected_format(ctx);
                self.restore_main_window(ctx, frame);
            }
            OverlayAction::Cancel => {
                self.overlay = None;
                ctx.send_viewport_cmd_to(picker_id, egui::ViewportCommand::Close);
                self.status = "Picker cancelled".to_owned();
                self.restore_main_window(ctx, frame);
            }
        }
    }
}

impl eframe::App for VColorPickerApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.render_main(ui, frame);
        self.render_overlay(ui.ctx(), frame);
        self.apply_pending_main_focus(ui.ctx(), frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        history::save(storage, &self.history);
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}
