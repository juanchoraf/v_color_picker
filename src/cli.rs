use anyhow::{Result, anyhow};
use v_concat::{v_concat_eprintln, v_concat_println};

use crate::update;

pub fn handle_cli<I>(args: I) -> Result<bool>
where
    I: IntoIterator<Item = String>,
{
    let mut check_update = false;
    let mut run_update = false;
    let mut github_repo = std::env::var("V_COLOR_PICKER_GITHUB_REPO").ok();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(true);
            }
            "-V" | "--version" | "version" => {
                v_concat_println!("v_color_picker {}", env!("CARGO_PKG_VERSION"));
                return Ok(true);
            }
            "--check-update" => check_update = true,
            "--update" => run_update = true,
            "--github-repo" => {
                github_repo = Some(
                    iter.next()
                        .ok_or_else(|| anyhow!("--github-repo expects owner/repo"))?,
                );
            }
            "--uninstall" => {
                print_uninstall_hint();
                return Ok(true);
            }
            unknown => {
                v_concat_eprintln!("Unknown option: {unknown}");
                print_help();
                return Ok(true);
            }
        }
    }

    if check_update || run_update {
        let repo = github_repo
            .ok_or_else(|| anyhow!("set --github-repo owner/repo or V_COLOR_PICKER_GITHUB_REPO"))?;

        if run_update {
            update::download_and_launch_update(&repo)?;
        } else {
            update::check_update(&repo)?;
        }

        return Ok(true);
    }

    Ok(false)
}

fn print_help() {
    const HELP: &str = concat!(
        "v_color_picker ",
        env!("CARGO_PKG_VERSION"),
        "\n\nUSAGE:\n",
        "  v_color_picker\n",
        "  v_color_picker --check-update --github-repo owner/repo\n",
        "  v_color_picker --update --github-repo owner/repo\n",
        "  v_color_picker --version\n",
        "  v_color_picker --uninstall\n\n",
        "ENV:\n",
        "  V_COLOR_PICKER_GITHUB_REPO=owner/repo\n\n",
        "The GUI starts when no CLI operation is requested."
    );
    v_concat_println!("{HELP}");
}

fn print_uninstall_hint() {
    #[cfg(target_os = "windows")]
    v_concat_println!("Run scripts\\uninstall_windows.ps1 from an elevated PowerShell.");

    #[cfg(target_os = "macos")]
    v_concat_println!("Run sudo scripts/uninstall_macos.sh.");

    #[cfg(all(unix, not(target_os = "macos")))]
    v_concat_println!("Run sudo scripts/uninstall_linux.sh.");
}
