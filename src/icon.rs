use egui::IconData;

pub fn app_icon() -> IconData {
    const SIZE: usize = 64;
    let mut rgba = vec![0; SIZE * SIZE * 4];

    fill_rect(&mut rgba, SIZE, 0, 0, 64, 64, [14, 18, 26, 255]);

    stroke_line(
        &mut rgba,
        SIZE,
        44.0,
        15.0,
        24.0,
        35.0,
        11.0,
        [136, 143, 246, 255],
    );
    stroke_line(
        &mut rgba,
        SIZE,
        51.0,
        8.0,
        37.0,
        22.0,
        9.0,
        [136, 143, 246, 255],
    );
    stroke_line(
        &mut rgba,
        SIZE,
        24.0,
        35.0,
        16.0,
        43.0,
        5.0,
        [136, 143, 246, 255],
    );
    fill_circle(&mut rgba, SIZE, 15.0, 49.0, 7.0, [38, 166, 154, 255]);

    IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}

fn fill_rect(
    rgba: &mut [u8],
    size: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: [u8; 4],
) {
    for py in y..(y + height).min(size) {
        for px in x..(x + width).min(size) {
            set_pixel(rgba, size, px, py, color);
        }
    }
}

fn fill_circle(rgba: &mut [u8], size: usize, cx: f32, cy: f32, radius: f32, color: [u8; 4]) {
    let radius_sq = radius * radius;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= radius_sq {
                set_pixel(rgba, size, x, y, color);
            }
        }
    }
}

fn stroke_line(
    rgba: &mut [u8],
    size: usize,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: f32,
    color: [u8; 4],
) {
    let steps = ((x2 - x1).abs().max((y2 - y1).abs()) * 2.0) as usize;
    for step in 0..=steps {
        let t = step as f32 / steps.max(1) as f32;
        let x = x1 + (x2 - x1) * t;
        let y = y1 + (y2 - y1) * t;
        fill_circle(rgba, size, x, y, width * 0.5, color);
    }
}

fn set_pixel(rgba: &mut [u8], size: usize, x: usize, y: usize, color: [u8; 4]) {
    let index = (y * size + x) * 4;
    rgba[index..index + 4].copy_from_slice(&color);
}
