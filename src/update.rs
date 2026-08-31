use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use semver::Version;
use serde::Deserialize;
use v_concat::v_concat_println;

const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const USER_AGENT: &str = concat!("v_color_picker/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

pub fn check_update(repo: &str) -> Result<()> {
    let release = fetch_latest_release(repo)?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let latest = parse_release_version(&release.tag_name)?;

    v_concat_println!("Current version: {current}");
    v_concat_println!("Latest version:  {latest}");
    v_concat_println!("Release page:    {}", release.html_url);

    if latest > current {
        v_concat_println!("Update available.");
    } else {
        v_concat_println!("Already up to date.");
    }

    Ok(())
}

pub fn download_and_launch_update(repo: &str) -> Result<()> {
    let release = fetch_latest_release(repo)?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let latest = parse_release_version(&release.tag_name)?;

    if latest <= current {
        v_concat_println!("Already up to date: {current}");
        return Ok(());
    }

    let asset = select_asset(&release.assets)
        .ok_or_else(|| anyhow!("no release asset matches this OS/architecture"))?;
    let destination = std::env::temp_dir().join(&asset.name);

    v_concat_println!("Downloading {} ({} bytes)", asset.name, asset.size);
    download_asset(asset, &destination)?;
    v_concat_println!("Downloaded update: {}", destination.display());
    launch_installer(&destination)?;

    Ok(())
}

fn fetch_latest_release(repo: &str) -> Result<GithubRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let mut response = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("failed to fetch latest GitHub release from {url}"))?;

    response
        .body_mut()
        .read_json::<GithubRelease>()
        .context("failed to parse GitHub release response")
}

fn parse_release_version(tag: &str) -> Result<Version> {
    Version::parse(tag.trim_start_matches('v'))
        .with_context(|| format!("release tag is not semantic version: {tag}"))
}

fn download_asset(asset: &GithubAsset, destination: &Path) -> Result<()> {
    let mut response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("failed to download {}", asset.browser_download_url))?;

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
    let arch_score = arch_score(&lower);
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

fn arch_score(name: &str) -> i32 {
    match std::env::consts::ARCH {
        "x86_64" => {
            if name.contains("x86_64") || name.contains("amd64") {
                20
            } else {
                0
            }
        }
        "aarch64" => {
            if name.contains("aarch64") || name.contains("arm64") {
                20
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn package_score(name: &str) -> Option<i32> {
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

fn launch_installer(path: &PathBuf) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let command = if path.extension().and_then(|ext| ext.to_str()) == Some("msi") {
            format!(
                "Start-Process msiexec.exe -ArgumentList '/i', '{}', '/passive' -Verb RunAs",
                path.display()
            )
        } else {
            format!("Start-Process '{}' -Verb RunAs", path.display())
        };
        Command::new("powershell.exe")
            .args([
                "-NoExit",
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
        Command::new("open")
            .arg(path)
            .spawn()
            .context("failed to open macOS update asset")?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if path.extension().and_then(|ext| ext.to_str()) == Some("deb") {
            if Command::new("pkexec")
                .arg("dpkg")
                .arg("-i")
                .arg(path)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
            v_concat_println!("Install manually: sudo dpkg -i {}", path.display());
        } else {
            v_concat_println!("Extract and install manually: {}", path.display());
        }
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}
