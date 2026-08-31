use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use semver::Version;
use serde::Deserialize;

pub const DEFAULT_GITHUB_REPO: &str = "juanchoraf/v_color_picker";
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const USER_AGENT: &str = concat!("v_color_picker/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug)]
pub struct UpdateInstall {
    pub latest_version: Version,
}

#[derive(Clone, Debug)]
pub enum UpdateOutcome {
    Current { current_version: Version },
    Installed(UpdateInstall),
}

#[derive(Clone, Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn install_latest_release() -> Result<UpdateOutcome> {
    download_and_launch_update(DEFAULT_GITHUB_REPO)
}

fn download_and_launch_update(repo: &str) -> Result<UpdateOutcome> {
    let release = fetch_latest_release(repo)?;
    let current = Version::parse(current_version())?;
    let latest = parse_release_version(&release.tag_name)?;

    if latest <= current {
        return Ok(UpdateOutcome::Current {
            current_version: current,
        });
    }

    let asset = select_asset(&release.assets)
        .ok_or_else(|| anyhow!("no release asset matches this OS/architecture"))?
        .clone();
    let destination = std::env::temp_dir().join(format!(
        "v_color_picker_update_{}_{}",
        std::process::id(),
        asset.name
    ));

    download_asset(&asset, &destination)?;
    launch_installer(&destination)?;

    Ok(UpdateOutcome::Installed(UpdateInstall {
        latest_version: latest,
    }))
}

fn fetch_latest_release(repo: &str) -> Result<GithubRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let mut response = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .context("failed to fetch latest release")?;

    response
        .body_mut()
        .read_json::<GithubRelease>()
        .context("failed to parse latest release response")
}

fn parse_release_version(tag: &str) -> Result<Version> {
    Version::parse(tag.trim_start_matches('v'))
        .with_context(|| format!("release tag is not semantic version: {tag}"))
}

fn download_asset(asset: &GithubAsset, destination: &Path) -> Result<()> {
    if asset.size > MAX_DOWNLOAD_BYTES {
        return Err(anyhow!(
            "release asset {} is larger than the 512 MiB download limit",
            asset.name
        ));
    }

    let mut response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("failed to download {}", asset.name))?;

    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_DOWNLOAD_BYTES)
        .read_to_vec()
        .context("failed to read update asset")?;

    fs::write(destination, bytes)
        .with_context(|| format!("failed to write {}", destination.display()))
}

fn select_asset(assets: &[GithubAsset]) -> Option<&GithubAsset> {
    assets
        .iter()
        .filter_map(|asset| asset_score(&asset.name).map(|score| (score, asset)))
        .max_by_key(|(score, _)| *score)
        .map(|(_, asset)| asset)
}

fn asset_score(name: &str) -> Option<i32> {
    let lower = name.to_ascii_lowercase();
    let os_score = os_score(&lower)?;
    let arch_score = arch_score(&lower)?;
    let package_score = package_score(&lower)?;
    Some(os_score + arch_score + package_score)
}

fn os_score(name: &str) -> Option<i32> {
    #[cfg(target_os = "windows")]
    {
        return (name.contains("windows") || name.contains("win")).then_some(40);
    }

    #[cfg(target_os = "macos")]
    {
        return (name.contains("macos") || name.contains("darwin") || name.contains("apple"))
            .then_some(40);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return (name.contains("linux") || name.contains("unix") || name.ends_with(".deb"))
            .then_some(40);
    }

    #[allow(unreachable_code)]
    None
}

fn arch_score(name: &str) -> Option<i32> {
    let has_x86_64 = name.contains("x86_64") || name.contains("amd64");
    let has_aarch64 = name.contains("aarch64") || name.contains("arm64");

    match std::env::consts::ARCH {
        "x86_64" if has_aarch64 => None,
        "x86_64" if has_x86_64 => Some(20),
        "aarch64" if has_x86_64 => None,
        "aarch64" if has_aarch64 => Some(20),
        _ => Some(0),
    }
}

fn package_score(name: &str) -> Option<i32> {
    if name.contains("portable") {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        return if name.ends_with(".msi") {
            Some(100)
        } else if name.ends_with(".exe") {
            Some(90)
        } else if name.ends_with(".zip") {
            Some(50)
        } else {
            None
        };
    }

    #[cfg(target_os = "macos")]
    {
        return if name.ends_with(".pkg") {
            Some(100)
        } else if name.ends_with(".dmg") {
            Some(90)
        } else if name.ends_with(".tar.gz") {
            Some(60)
        } else {
            None
        };
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return if name.ends_with(".deb") {
            Some(100)
        } else if name.ends_with(".tar.gz") {
            Some(60)
        } else {
            None
        };
    }

    #[allow(unreachable_code)]
    None
}

fn launch_installer(path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let installer = powershell_single_quoted(path);
        let command = if path.extension().and_then(|ext| ext.to_str()) == Some("msi") {
            format!(
                "Start-Process msiexec.exe -ArgumentList '/i', {installer}, '/passive' -Verb RunAs"
            )
        } else {
            format!("Start-Process {installer} -Verb RunAs")
        };
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &command,
            ])
            .spawn()
            .context("failed to launch PowerShell update installer")?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        if path.extension().and_then(|ext| ext.to_str()) == Some("pkg") {
            let command = format!("installer -pkg {} -target /", shell_single_quoted(path));
            let script = format!(
                "do shell script {} with administrator privileges",
                applescript_double_quoted(&command)
            );
            Command::new("osascript")
                .args(["-e", &script])
                .spawn()
                .context("failed to launch privileged macOS package installer")?;
        } else {
            Command::new("open")
                .arg(path)
                .spawn()
                .context("failed to open macOS update asset")?;
        }
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if path.extension().and_then(|ext| ext.to_str()) == Some("deb") {
            Command::new("pkexec")
                .arg("dpkg")
                .arg("-i")
                .arg(path)
                .spawn()
                .context("failed to launch privileged deb installer with pkexec")?;
            return Ok(());
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".tar.gz"))
        {
            Command::new("pkexec")
                .arg("sh")
                .arg("-c")
                .arg(UNIX_TAR_INSTALL_SCRIPT)
                .arg("sh")
                .arg(path)
                .spawn()
                .context("failed to launch privileged tar installer with pkexec")?;
            return Ok(());
        }

        return Err(anyhow!(
            "no automatic installer is available for {}",
            path.display()
        ));
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(target_os = "windows")]
fn powershell_single_quoted(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

#[cfg(target_os = "macos")]
fn shell_single_quoted(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn applescript_double_quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(all(unix, not(target_os = "macos")))]
const UNIX_TAR_INSTALL_SCRIPT: &str = r#"set -e
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
tar -xzf "$1" -C "$tmp"
bin="$(find "$tmp" -maxdepth 1 -type f \( -name 'v_color_picker' -o -name 'v_color_picker_v*_*.bin' \) | head -n 1)"
if [ -z "$bin" ]; then
  echo "No v_color_picker binary found in update archive" >&2
  exit 1
fi
install -Dm755 "$bin" /opt/v_color_picker/v_color_picker
if [ -f "$tmp/logo.svg" ]; then
  install -Dm644 "$tmp/logo.svg" /usr/share/icons/hicolor/scalable/apps/v_color_picker.svg
fi
install -Dm644 /dev/stdin /usr/share/applications/com.thevelasquez.v_color_picker.desktop <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=v_color_picker
Comment=Pick and inspect exact screen colors
Exec=/opt/v_color_picker/v_color_picker
Icon=v_color_picker
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
"#;
