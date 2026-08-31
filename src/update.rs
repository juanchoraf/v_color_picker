use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Result, anyhow};
use semver::Version;
use serde::Deserialize;

pub const DEFAULT_GITHUB_REPO: &str = "juanchoraf/v_color_picker";
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const USER_AGENT: &str = concat!("v_color_picker/", env!("CARGO_PKG_VERSION"));
const DOWNLOAD_CHUNK_SIZE: usize = 256 * 1024;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug)]
pub struct UpdateInstall {
    pub latest_version: Version,
}

#[derive(Clone, Debug)]
pub enum UpdateOutcome {
    Current { current_version: Version },
    Installed(UpdateInstall),
}

#[derive(Clone, Debug)]
pub struct UpdateProgress {
    pub message: &'static str,
    pub fraction: Option<f32>,
}

impl UpdateProgress {
    pub fn checking() -> Self {
        Self {
            message: "Checking for updates",
            fraction: Some(0.05),
        }
    }

    fn downloading(fraction: Option<f32>) -> Self {
        Self {
            message: "Downloading update package",
            fraction,
        }
    }

    fn installing() -> Self {
        Self {
            message: "Installing update in background",
            fraction: None,
        }
    }
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
    install_latest_release_with_progress(|_| {})
}

pub fn install_latest_release_with_progress(
    progress: impl Fn(UpdateProgress),
) -> Result<UpdateOutcome> {
    download_and_launch_update(DEFAULT_GITHUB_REPO, &progress)
}

fn download_and_launch_update(
    repo: &str,
    progress: &impl Fn(UpdateProgress),
) -> Result<UpdateOutcome> {
    progress(UpdateProgress::checking());
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

    progress(UpdateProgress::downloading(Some(0.10)));
    download_asset(&asset, &destination, progress)?;
    progress(UpdateProgress::installing());
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

fn download_asset(
    asset: &GithubAsset,
    destination: &Path,
    progress: &impl Fn(UpdateProgress),
) -> Result<()> {
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

    let mut file = File::create(destination)
        .with_context(|| format!("failed to write {}", destination.display()))?;
    let mut reader = response.body_mut().as_reader();
    let mut buffer = [0_u8; DOWNLOAD_CHUNK_SIZE];
    let mut downloaded = 0_u64;

    loop {
        let read = reader
            .read(&mut buffer)
            .context("failed to read update asset")?;
        if read == 0 {
            break;
        }

        downloaded += read as u64;
        if downloaded > MAX_DOWNLOAD_BYTES {
            return Err(anyhow!("download exceeded the 512 MiB download limit"));
        }

        file.write_all(&buffer[..read])
            .with_context(|| format!("failed to write {}", destination.display()))?;
        progress(UpdateProgress::downloading(download_fraction(
            asset.size, downloaded,
        )));
    }

    file.flush()
        .with_context(|| format!("failed to write {}", destination.display()))
}

fn download_fraction(size: u64, downloaded: u64) -> Option<f32> {
    if size == 0 {
        return None;
    }

    let raw = (downloaded as f32 / size as f32).clamp(0.0, 1.0);
    Some(0.10 + raw * 0.60)
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
        } else {
            None
        };
    }

    #[cfg(target_os = "macos")]
    {
        return if name.ends_with(".pkg") {
            Some(100)
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
        let installer = powershell_path(path);
        let command = if path.extension().and_then(|ext| ext.to_str()) == Some("msi") {
            let path_arg = path.display().to_string();
            let args = powershell_argument_list(&[
                "/i",
                &path_arg,
                "/quiet",
                "/norestart",
                "REBOOT=ReallySuppress",
            ]);
            format!(
                "$p = Start-Process msiexec.exe -ArgumentList {args} -Verb RunAs -Wait -PassThru; exit $p.ExitCode"
            )
        } else {
            let args = powershell_argument_list(&[
                "/VERYSILENT",
                "/SUPPRESSMSGBOXES",
                "/NORESTART",
                "/CLOSEAPPLICATIONS",
            ]);
            format!(
                "$p = Start-Process {installer} -ArgumentList {args} -Verb RunAs -Wait -PassThru; exit $p.ExitCode"
            )
        };
        let mut process = Command::new("powershell.exe");
        let status = process
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &command,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .context("failed to launch PowerShell update installer")?;
        ensure_success(status, "Windows update installer")?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let command = if path.extension().and_then(|ext| ext.to_str()) == Some("pkg") {
            format!("installer -pkg {} -target /", shell_path(path))
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".tar.gz"))
        {
            format!(
                "/bin/sh -c {} sh {}",
                shell_single_quote(MACOS_TAR_INSTALL_SCRIPT),
                shell_path(path)
            )
        } else {
            return Err(anyhow!(
                "silent automatic updates require a .pkg or .tar.gz asset on macOS"
            ));
        };
        let script = format!(
            "do shell script {} with administrator privileges",
            applescript_double_quoted(&command)
        );
        let status = Command::new("osascript")
            .args(["-e", &script])
            .status()
            .context("failed to run privileged macOS update installer")?;
        ensure_success(status, "macOS update installer")?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if path.extension().and_then(|ext| ext.to_str()) == Some("deb") {
            let status = Command::new("pkexec")
                .arg("dpkg")
                .arg("-i")
                .arg(path)
                .status()
                .context("failed to run privileged deb installer with pkexec")?;
            ensure_success(status, "Linux deb installer")?;
            return Ok(());
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".tar.gz"))
        {
            let status = Command::new("pkexec")
                .arg("sh")
                .arg("-c")
                .arg(UNIX_TAR_INSTALL_SCRIPT)
                .arg("sh")
                .arg(path)
                .status()
                .context("failed to run privileged tar installer with pkexec")?;
            ensure_success(status, "Linux tar installer")?;
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

fn ensure_success(status: ExitStatus, action: &str) -> Result<()> {
    match status.code() {
        Some(0) | Some(3010) => Ok(()),
        Some(code) => Err(anyhow!("{action} exited with code {code}")),
        None => Err(anyhow!("{action} stopped before finishing")),
    }
}

pub fn relaunch_current_app() -> Result<()> {
    let executable = std::env::current_exe().context("failed to locate the current app")?;
    relaunch_executable(&executable)
}

#[cfg(target_os = "windows")]
fn relaunch_executable(path: &Path) -> Result<()> {
    let command = format!(
        "Start-Sleep -Milliseconds 800; Start-Process {}",
        powershell_path(path)
    );
    let mut process = Command::new("powershell.exe");
    process
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &command,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .with_context(|| format!("failed to restart {}", path.display()))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn relaunch_executable(path: &Path) -> Result<()> {
    Command::new("sh")
        .arg("-c")
        .arg("sleep 0.8; exec \"$1\"")
        .arg("sh")
        .arg(path)
        .spawn()
        .with_context(|| format!("failed to restart {}", path.display()))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn powershell_argument_list(args: &[&str]) -> String {
    let quoted = args
        .iter()
        .map(|arg| powershell_single_quote(arg))
        .collect::<Vec<_>>()
        .join(", ");
    format!("@({quoted})")
}

#[cfg(target_os = "windows")]
fn powershell_path(path: &Path) -> String {
    powershell_single_quote(&path.display().to_string())
}

#[cfg(target_os = "windows")]
fn powershell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "macos")]
fn shell_path(path: &Path) -> String {
    shell_single_quote(&path.display().to_string())
}

#[cfg(target_os = "macos")]
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn applescript_double_quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "macos")]
const MACOS_TAR_INSTALL_SCRIPT: &str = r#"set -e
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
tar -xzf "$1" -C "$tmp"
app="$(find "$tmp" -maxdepth 3 -type d -name 'v_color_picker.app' | head -n 1)"
if [ -z "$app" ]; then
  echo "No v_color_picker.app found in update archive" >&2
  exit 1
fi
rm -rf /Applications/v_color_picker.app
cp -R "$app" /Applications/
"#;

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
