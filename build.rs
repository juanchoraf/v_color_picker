use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_NAME: &str = "v_color_picker";
const PUBLISHER: &str = "TheVelasquez.com";
const ICON_SIZES: [usize; 5] = [16, 32, 48, 64, 256];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows") {
        if let Err(error) = embed_windows_resources() {
            if env::var("PROFILE").is_ok_and(|profile| profile == "release") {
                panic!("failed to embed Windows resources: {error}");
            }
            println!("cargo:warning=Windows resources were not embedded: {error}");
        }
    }
}

fn embed_windows_resources() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "OUT_DIR is not set for build script",
        )
    })?);
    let icon_path = out_dir.join(format!("{APP_NAME}.ico"));
    let rc_path = out_dir.join(format!("{APP_NAME}.rc"));

    write_ico(&icon_path)?;
    write_rc_file(&rc_path, &icon_path)?;
    compile_resource(&rc_path, &out_dir)?;
    Ok(())
}

fn compile_resource(rc_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_env == "gnu" {
        let compiler = find_program("windres.exe")
            .or_else(|| find_program("windres"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "windres was not found"))?;
        let object_path = out_dir.join(format!("{APP_NAME}.resource.o"));
        let status = Command::new(&compiler)
            .arg("--input")
            .arg(rc_path)
            .arg("--output")
            .arg(&object_path)
            .arg("--output-format")
            .arg("coff")
            .status()?;

        if !status.success() {
            return Err(format!("{} exited with {status}", compiler.display()).into());
        }

        println!(
            "cargo:rustc-link-arg-bin={APP_NAME}={}",
            object_path.display()
        );
        return Ok(());
    }

    let compiler = find_program("rc.exe")
        .or_else(|| find_program("llvm-rc.exe"))
        .or_else(find_windows_sdk_rc)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "rc.exe or llvm-rc.exe was not found",
            )
        })?;
    let resource_path = out_dir.join(format!("{APP_NAME}.res"));
    let status = Command::new(&compiler)
        .arg("/nologo")
        .arg(format!("/fo{}", resource_path.display()))
        .arg(rc_path)
        .status()?;

    if !status.success() {
        return Err(format!("{} exited with {status}", compiler.display()).into());
    }

    println!(
        "cargo:rustc-link-arg-bin={APP_NAME}={}",
        resource_path.display()
    );
    Ok(())
}

fn find_program(name: &str) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.is_absolute() && candidate.is_file() {
        return Some(candidate.to_path_buf());
    }

    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    })
}

fn find_windows_sdk_rc() -> Option<PathBuf> {
    let base = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .map(|dir| dir.join("Windows Kits").join("10").join("bin"))?;
    let mut matches = fs::read_dir(base)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("x64").join("rc.exe"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    matches.sort();
    matches.pop()
}

fn write_rc_file(rc_path: &Path, icon_path: &Path) -> io::Result<()> {
    let package_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_owned());
    let file_version = version_tuple(&package_version);
    let icon = rc_escape(icon_path);
    let rc = format!(
        r#"1 ICON "{icon}"

1 VERSIONINFO
FILEVERSION {file_version}
PRODUCTVERSION {file_version}
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "CompanyName", "{PUBLISHER}\0"
            VALUE "FileDescription", "v_color_picker\0"
            VALUE "FileVersion", "{package_version}\0"
            VALUE "InternalName", "{APP_NAME}\0"
            VALUE "LegalCopyright", "Copyright (c) {PUBLISHER}\0"
            VALUE "OriginalFilename", "{APP_NAME}.exe\0"
            VALUE "ProductName", "v_color_picker\0"
            VALUE "ProductVersion", "{package_version}\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#
    );
    fs::write(rc_path, rc)
}

fn version_tuple(version: &str) -> String {
    let mut values = [0_u16; 4];
    for (index, part) in version
        .split(|ch| ch == '.' || ch == '-' || ch == '+')
        .take(4)
        .enumerate()
    {
        values[index] = part.parse().unwrap_or(0);
    }
    format!("{},{},{},{}", values[0], values[1], values[2], values[3])
}

fn rc_escape(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

fn write_ico(path: &Path) -> io::Result<()> {
    let images = ICON_SIZES
        .iter()
        .map(|size| {
            let bytes = dib_icon_image(*size);
            (*size, bytes)
        })
        .collect::<Vec<_>>();
    let image_offset = 6 + (images.len() * 16);
    let mut offset = image_offset as u32;
    let mut output = Vec::new();

    write_u16(&mut output, 0);
    write_u16(&mut output, 1);
    write_u16(&mut output, images.len() as u16);

    for (size, bytes) in &images {
        output.push(icon_dimension(*size));
        output.push(icon_dimension(*size));
        output.push(0);
        output.push(0);
        write_u16(&mut output, 1);
        write_u16(&mut output, 32);
        write_u32(&mut output, bytes.len() as u32);
        write_u32(&mut output, offset);
        offset += bytes.len() as u32;
    }

    for (_, bytes) in images {
        output.extend(bytes);
    }

    fs::write(path, output)
}

fn icon_dimension(size: usize) -> u8 {
    if size >= 256 { 0 } else { size as u8 }
}

fn dib_icon_image(size: usize) -> Vec<u8> {
    let rgba = render_icon(size);
    let mut bytes = Vec::new();
    let mask_stride = size.div_ceil(32) * 4;

    write_u32(&mut bytes, 40);
    write_i32(&mut bytes, size as i32);
    write_i32(&mut bytes, (size * 2) as i32);
    write_u16(&mut bytes, 1);
    write_u16(&mut bytes, 32);
    write_u32(&mut bytes, 0);
    write_u32(&mut bytes, (size * size * 4) as u32);
    write_i32(&mut bytes, 0);
    write_i32(&mut bytes, 0);
    write_u32(&mut bytes, 0);
    write_u32(&mut bytes, 0);

    for y in (0..size).rev() {
        for x in 0..size {
            let index = (y * size + x) * 4;
            bytes.push(rgba[index + 2]);
            bytes.push(rgba[index + 1]);
            bytes.push(rgba[index]);
            bytes.push(rgba[index + 3]);
        }
    }

    bytes.resize(bytes.len() + (mask_stride * size), 0);
    bytes
}

fn render_icon(size: usize) -> Vec<u8> {
    let mut rgba = vec![0; size * size * 4];
    fill_rect(&mut rgba, size, 0, 0, size, size, [14, 18, 26, 255]);
    stroke_line(
        &mut rgba,
        size,
        scaled(44.0, size),
        scaled(15.0, size),
        scaled(17.0, size),
        scaled(42.0, size),
        scaled(11.0, size),
        [136, 143, 246, 255],
    );
    stroke_line(
        &mut rgba,
        size,
        scaled(47.0, size),
        scaled(18.0, size),
        scaled(21.0, size),
        scaled(44.0, size),
        scaled(4.0, size),
        [176, 181, 255, 255],
    );
    fill_circle(
        &mut rgba,
        size,
        scaled(20.0, size),
        scaled(44.0, size),
        scaled(6.0, size),
        [35, 209, 139, 255],
    );
    rgba
}

fn scaled(value: f32, size: usize) -> f32 {
    value * (size as f32 / 64.0)
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
    let max_x = (x + width).min(size);
    let max_y = (y + height).min(size);
    for py in y..max_y {
        for px in x..max_x {
            set_pixel(rgba, size, px as i32, py as i32, color);
        }
    }
}

fn stroke_line(
    rgba: &mut [u8],
    size: usize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    radius: f32,
    color: [u8; 4],
) {
    let steps = ((x1 - x0).abs().max((y1 - y0).abs()) as usize).max(1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        fill_circle(
            rgba,
            size,
            x0 + ((x1 - x0) * t),
            y0 + ((y1 - y0) * t),
            radius,
            color,
        );
    }
}

fn fill_circle(rgba: &mut [u8], size: usize, cx: f32, cy: f32, radius: f32, color: [u8; 4]) {
    let min_x = (cx - radius).floor().max(0.0) as i32;
    let max_x = (cx + radius).ceil().min(size as f32 - 1.0) as i32;
    let min_y = (cy - radius).floor().max(0.0) as i32;
    let max_y = (cy + radius).ceil().min(size as f32 - 1.0) as i32;
    let radius_sq = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if (dx * dx) + (dy * dy) <= radius_sq {
                set_pixel(rgba, size, x, y, color);
            }
        }
    }
}

fn set_pixel(rgba: &mut [u8], size: usize, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
        return;
    }
    let index = ((y as usize * size) + x as usize) * 4;
    rgba[index..index + 4].copy_from_slice(&color);
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend(value.to_le_bytes());
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend(value.to_le_bytes());
}

fn write_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}
