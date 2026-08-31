#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorFormat {
    pub label: &'static str,
    pub value: String,
}

impl RgbaColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            a: alpha_to_u8(alpha),
            ..self
        }
    }
}

pub fn alpha_to_u8(alpha: f32) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn formats(color: RgbaColor, alpha: f32) -> Vec<ColorFormat> {
    let color = color.with_alpha(alpha);
    let alpha_value = color.a as f64 / 255.0;
    let (h, s, l) = rgb_to_hsl(color);
    let (w, bk) = rgb_to_hwb(color);
    let (x, y, z) = rgb_to_xyz(color);
    let (lab_l, lab_a, lab_b) = xyz_to_lab(x, y, z);
    let (lch_l, lch_c, lch_h) = lab_to_lch(lab_l, lab_a, lab_b);
    let (ok_l, ok_a, ok_b) = rgb_to_oklab(color);
    let (oklch_l, oklch_c, oklch_h) = lab_to_lch(ok_l, ok_a, ok_b);

    vec![
        row(
            "HEX",
            format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b),
        ),
        row(
            "HEXA",
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                color.r, color.g, color.b, color.a
            ),
        ),
        row("RGB", format!("rgb({} {} {})", color.r, color.g, color.b)),
        row(
            "RGBA",
            format!(
                "rgba({} {} {} / {})",
                color.r,
                color.g,
                color.b,
                trim(alpha_value, 4)
            ),
        ),
        row(
            "HSL",
            format!(
                "hsl({} {}% {}%)",
                trim(h, 2),
                trim(s * 100.0, 2),
                trim(l * 100.0, 2)
            ),
        ),
        row(
            "HSLA",
            format!(
                "hsl({} {}% {}% / {})",
                trim(h, 2),
                trim(s * 100.0, 2),
                trim(l * 100.0, 2),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "HWB",
            format!(
                "hwb({} {}% {}% / {})",
                trim(h, 2),
                trim(w * 100.0, 2),
                trim(bk * 100.0, 2),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "LAB",
            format!(
                "lab({}% {} {} / {})",
                trim(lab_l, 2),
                trim(lab_a, 2),
                trim(lab_b, 2),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "LCH",
            format!(
                "lch({}% {} {} / {})",
                trim(lch_l, 2),
                trim(lch_c, 2),
                trim(lch_h, 2),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "OKLAB",
            format!(
                "oklab({}% {} {} / {})",
                trim(ok_l * 100.0, 3),
                trim(ok_a, 4),
                trim(ok_b, 4),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "OKLCH",
            format!(
                "oklch({}% {} {} / {})",
                trim(oklch_l * 100.0, 3),
                trim(oklch_c, 4),
                trim(oklch_h, 2),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "SRGB",
            format!(
                "color(srgb {} {} {} / {})",
                trim(color.r as f64 / 255.0, 5),
                trim(color.g as f64 / 255.0, 5),
                trim(color.b as f64 / 255.0, 5),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "SRGB-LINEAR",
            format!(
                "color(srgb-linear {} {} {} / {})",
                trim(linear_channel(color.r), 5),
                trim(linear_channel(color.g), 5),
                trim(linear_channel(color.b), 5),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "XYZ-D65",
            format!(
                "color(xyz-d65 {} {} {} / {})",
                trim(x, 5),
                trim(y, 5),
                trim(z, 5),
                trim(alpha_value, 4)
            ),
        ),
        row(
            "U32",
            format!(
                "0x{:02X}{:02X}{:02X}{:02X}",
                color.r, color.g, color.b, color.a
            ),
        ),
    ]
}

pub fn palette_color(base: RgbaColor, x: f32, y: f32) -> RgbaColor {
    let (base_h, base_s, _) = rgb_to_hsl(base);
    let hue = wrap_degrees(base_h + (f64::from(x.clamp(0.0, 1.0)) - 0.5) * 170.0);
    let saturation = (0.28 + f64::from(x.clamp(0.0, 1.0)) * 0.62).max(base_s * 0.55);
    let lightness = 0.08 + (1.0 - f64::from(y.clamp(0.0, 1.0))) * 0.84;
    hsl_to_rgb(
        hue,
        saturation.clamp(0.0, 1.0),
        lightness.clamp(0.0, 1.0),
        base.a,
    )
}

pub fn similar_swatch(base: RgbaColor, index: usize, total: usize) -> RgbaColor {
    let (h, s, l) = rgb_to_hsl(base);
    let center = (total.saturating_sub(1)) as f64 / 2.0;
    let offset = (index as f64 - center) * 18.0;
    let saturation = (s * (0.82 + index as f64 * 0.035)).clamp(0.0, 1.0);
    let lightness = (l + (index as f64 - center) * 0.018).clamp(0.05, 0.95);
    hsl_to_rgb(wrap_degrees(h + offset), saturation, lightness, base.a)
}

fn row(label: &'static str, value: String) -> ColorFormat {
    ColorFormat { label, value }
}

fn rgb_to_hsl(color: RgbaColor) -> (f64, f64, f64) {
    let r = f64::from(color.r) / 255.0;
    let g = f64::from(color.g) / 255.0;
    let b = f64::from(color.b) / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (max + min) / 2.0;
    let delta = max - min;

    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    (wrap_degrees(hue), saturation, lightness)
}

fn rgb_to_hwb(color: RgbaColor) -> (f64, f64) {
    let r = f64::from(color.r) / 255.0;
    let g = f64::from(color.g) / 255.0;
    let b = f64::from(color.b) / 255.0;
    let white = r.min(g).min(b);
    let black = 1.0 - r.max(g).max(b);
    (white, black)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64, alpha: u8) -> RgbaColor {
    let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = wrap_degrees(h) / 60.0;
    let x = chroma * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = l - chroma / 2.0;
    RgbaColor::new(to_u8(r1 + m), to_u8(g1 + m), to_u8(b1 + m), alpha)
}

fn rgb_to_xyz(color: RgbaColor) -> (f64, f64, f64) {
    let r = linear_channel(color.r);
    let g = linear_channel(color.g);
    let b = linear_channel(color.b);
    (
        r * 0.412_456_4 + g * 0.357_576_1 + b * 0.180_437_5,
        r * 0.212_672_9 + g * 0.715_152_2 + b * 0.072_175,
        r * 0.019_333_9 + g * 0.119_192 + b * 0.950_304_1,
    )
}

fn xyz_to_lab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let fx = lab_f(x / 0.950_47);
    let fy = lab_f(y);
    let fz = lab_f(z / 1.088_83);
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

fn lab_to_lch(lightness: f64, a: f64, b: f64) -> (f64, f64, f64) {
    (
        lightness,
        (a * a + b * b).sqrt(),
        positive_degrees(b.atan2(a).to_degrees()),
    )
}

fn rgb_to_oklab(color: RgbaColor) -> (f64, f64, f64) {
    let r = linear_channel(color.r);
    let g = linear_channel(color.g);
    let b = linear_channel(color.b);
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    (
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    )
}

fn linear_channel(value: u8) -> f64 {
    let c = f64::from(value) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn lab_f(value: f64) -> f64 {
    let delta = 6.0 / 29.0;
    if value > delta * delta * delta {
        value.cbrt()
    } else {
        value / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

fn positive_degrees(value: f64) -> f64 {
    if value < 0.0 { value + 360.0 } else { value }
}

fn wrap_degrees(value: f64) -> f64 {
    let wrapped = value % 360.0;
    positive_degrees(wrapped)
}

fn to_u8(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn trim(value: f64, precision: usize) -> String {
    let mut text = format!("{value:.precision$}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" { "0".to_owned() } else { text }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_expected_hex() {
        let rows = formats(RgbaColor::new(255, 128, 0, 255), 1.0);
        assert_eq!(rows[0].value, "#FF8000");
        assert_eq!(rows[1].value, "#FF8000FF");
    }
}
