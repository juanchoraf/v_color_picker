use anyhow::{Context, Result};
use eframe::egui::{Color32, ColorImage};
use image::RgbaImage;
use xcap::Monitor;

use crate::color::RgbaColor;

#[derive(Clone, Debug)]
pub struct ScreenSample {
    pub color: RgbaColor,
    pub global_x: i32,
    pub global_y: i32,
    pub monitor_name: String,
}

#[derive(Clone, Debug)]
pub struct ScreenCapture {
    pub monitors: Vec<MonitorCapture>,
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct MonitorCapture {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub primary: bool,
    image: RgbaImage,
}

impl ScreenCapture {
    pub fn capture_all() -> Result<Self> {
        let monitors = Monitor::all().context("failed to enumerate monitors")?;
        let mut captures = Vec::with_capacity(monitors.len());

        for monitor in monitors {
            let x = monitor.x().context("failed to read monitor x coordinate")?;
            let y = monitor.y().context("failed to read monitor y coordinate")?;
            let name = monitor
                .friendly_name()
                .or_else(|_| monitor.name())
                .unwrap_or_else(|_| "Unknown monitor".to_owned());
            let scale_factor = monitor.scale_factor().unwrap_or(1.0);
            let primary = monitor.is_primary().unwrap_or(false);
            let image = monitor
                .capture_image()
                .with_context(|| format!("failed to capture monitor {name}"))?;
            let width = image.width();
            let height = image.height();

            captures.push(MonitorCapture {
                name,
                x,
                y,
                width,
                height,
                scale_factor,
                primary,
                image,
            });
        }

        let first = captures
            .first()
            .context("no monitors were returned by the system")?;
        let min_x = captures
            .iter()
            .map(|monitor| monitor.x)
            .min()
            .unwrap_or(first.x);
        let min_y = captures
            .iter()
            .map(|monitor| monitor.y)
            .min()
            .unwrap_or(first.y);
        let max_x = captures
            .iter()
            .map(|monitor| monitor.x + monitor.width as i32)
            .max()
            .unwrap_or(first.x + first.width as i32);
        let max_y = captures
            .iter()
            .map(|monitor| monitor.y + monitor.height as i32)
            .max()
            .unwrap_or(first.y + first.height as i32);

        Ok(Self {
            monitors: captures,
            origin_x: min_x,
            origin_y: min_y,
            width: (max_x - min_x).max(1) as u32,
            height: (max_y - min_y).max(1) as u32,
        })
    }

    pub fn sample_virtual(&self, x: u32, y: u32) -> Option<ScreenSample> {
        if x >= self.width || y >= self.height {
            return None;
        }

        self.sample_global(self.origin_x + x as i32, self.origin_y + y as i32)
    }

    pub fn sample_global(&self, x: i32, y: i32) -> Option<ScreenSample> {
        self.monitors
            .iter()
            .find_map(|monitor| monitor.sample(x, y))
    }

    pub fn to_color_image(&self) -> ColorImage {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut pixels = vec![Color32::from_rgb(9, 11, 15); width * height];

        for monitor in &self.monitors {
            let offset_x = (monitor.x - self.origin_x) as usize;
            let offset_y = (monitor.y - self.origin_y) as usize;

            for (x, y, pixel) in monitor.image.enumerate_pixels() {
                let tx = offset_x + x as usize;
                let ty = offset_y + y as usize;
                if tx < width && ty < height {
                    let [r, g, b, a] = pixel.0;
                    pixels[ty * width + tx] = Color32::from_rgba_unmultiplied(r, g, b, a);
                }
            }
        }

        ColorImage::new([width, height], pixels)
    }
}

impl MonitorCapture {
    fn sample(&self, global_x: i32, global_y: i32) -> Option<ScreenSample> {
        let right = self.x + self.width as i32;
        let bottom = self.y + self.height as i32;

        if global_x < self.x || global_y < self.y || global_x >= right || global_y >= bottom {
            return None;
        }

        let local_x = (global_x - self.x) as u32;
        let local_y = (global_y - self.y) as u32;
        let [r, g, b, a] = self.image.get_pixel(local_x, local_y).0;

        let monitor_name = if self.primary {
            format!("{} primary {:.2}x", self.name, self.scale_factor)
        } else {
            format!("{} {:.2}x", self.name, self.scale_factor)
        };

        Some(ScreenSample {
            color: RgbaColor::new(r, g, b, a),
            global_x,
            global_y,
            monitor_name,
        })
    }
}
