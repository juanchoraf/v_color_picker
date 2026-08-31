# v_color_picker

[![Release_Badge]][Release_Url]
[![Build_Badge]][Build_Url]
[![Dependencies_Badge]][Dependencies_Url]
![License_Badge](https://img.shields.io/badge/License-Apache--2.0_or_MIT-blue)

`v_color_picker` is a Rust desktop color picker app for Linux, macOS, BSD/Unix-like systems, and Windows. It captures every visible monitor, lets you zoom into the captured desktop, and stores picked colors with copy-ready formats.

Made with AI (Codex) 🤖

## Supported OS

| Platform  | Support |
| --------- |:-------:|
| Linux     |   ✅    |
| Windows   |   ✅    |
| macOS     |   ✅    |
| Unix      |   ✅    |


## Features

||
|---------|
| Pick colors from any visible point on any connected monitor.
| Works even when the target window is not focused.
| Shows HEX, HEXA, RGB, RGBA, HSL, HSLA, HWB, LAB, LCH, OKLAB, OKLCH, SRGB, SRGB linear, XYZ-D65, and packed U32 values.
| Alpha slider, zoom slider, related palette, alternatives, copy buttons, and pick history.
| CLI support for version, update checks, update launching, and uninstall hints.
| Release scripts for 64-bit Linux, macOS, and Windows packaging.

## Run

Start the desktop app:

```sh
cargo run --release
```

Installed app:

```sh
v_color_picker
```

Example flow:

1. Open V_Color_Picker.
2. Press `Pick`.
3. Click any pixel on any monitor.
4. Select the format you need and copy it.

On Windows, installed shortcuts launch the app through PowerShell with `-NoExit`
so command output remains visible.

## CLI

```sh
v_color_picker --version
v_color_picker --check-update --github-repo owner/repo
v_color_picker --update --github-repo owner/repo
v_color_picker --uninstall
```

You can set `V_COLOR_PICKER_GITHUB_REPO=owner/repo` instead of passing
`--github-repo`.

## Build Release Packages

The project is Rust 2024 and uses the current stable toolchain.
Release scripts accept `--locked` to use `Cargo.lock` exactly as-is and
`--no-update` to skip `cargo update`.

### GitHub Release Builds

GitHub Actions builds release packages automatically when you push a matching
`v*` tag. The tag version must match the version in `Cargo.toml`.

```sh
scripts/bump_version.sh 0.1.1 --no-build
git add Cargo.toml Cargo.lock
git commit -m "Release v0.1.1"
git tag v0.1.1
git push origin main v0.1.1
```

That release workflow builds Linux x86_64, Linux arm64, macOS x86_64,
macOS arm64, and Windows x86_64, then publishes the artifacts to the GitHub
Release. You can also run `.github/workflows/release.yml` manually from GitHub
Actions by passing a tag such as `v0.1.1`.

### Local Release Builds

Linux:

```sh
scripts/build_binaries.sh
```

On Debian/Ubuntu, `sudo scripts/install_linux_build_deps.sh` installs common
packaging tools and the XCB runtime library if your machine is missing them.

Outputs are written to `versions/v_color_picker_vVERSION/`:

- `v_color_picker_vVERSION_linux_x86_64.bin` or `v_color_picker_vVERSION_linux_arm64.bin`
- `v_color_picker_vVERSION_linux_x86_64.deb` or `v_color_picker_vVERSION_linux_arm64.deb`
- `v_color_picker_vVERSION_linux_x86_64.tar.gz` or `v_color_picker_vVERSION_linux_arm64.tar.gz`
- install and uninstall helper scripts

macOS:

```sh
scripts/build_binaries.sh
```

Outputs include:

- `v_color_picker_vVERSION_macos_x86_64.tar.gz` or `v_color_picker_vVERSION_macos_arm64.tar.gz`
- `v_color_picker_vVERSION_macos_x86_64.pkg` or `v_color_picker_vVERSION_macos_arm64.pkg`
- install and uninstall helper scripts

BSD/Unix-like:

```sh
scripts/build_binaries.sh
```

Outputs include:

- `v_color_picker_vVERSION_unix_x86_64.bin` or `v_color_picker_vVERSION_unix_arm64.bin`
- `v_color_picker_vVERSION_unix_x86_64.tar.gz` or `v_color_picker_vVERSION_unix_arm64.tar.gz`
- install and uninstall helper scripts

Windows PowerShell:

```powershell
.\scripts\build_binaries_windows.ps1
```

Outputs include:

- `v_color_picker_vVERSION_windows_x86_64.exe`
- `v_color_picker_vVERSION_windows_x86_64.msi` when WiX `wix.exe` is installed
- install and uninstall helper scripts
- a local self-signed code-signing certificate under `certs/`

The Windows script imports the generated certificate into the current user's
Trusted Publisher and Root stores, then signs assets when `signtool.exe` is
available. It builds an `.exe` installer when Inno Setup `ISCC.exe` is
available; otherwise it keeps a signed portable `.exe` and prints a warning.
For WiX v7 MSI builds, the script passes `-acceptEula wix7`; override that with
`V_COLOR_PICKER_WIX_EULA_ID` if WiX changes the required EULA ID.

To bump version and package for the current OS:

```sh
scripts/bump_version.sh 0.1.1
```

## Install

Debian/Ubuntu:

```sh
sudo apt install ./versions/v_color_picker_vVERSION/v_color_picker_vVERSION_linux_x86_64.deb
```

The `.deb` installs `/usr/bin/v_color_picker`, a desktop launcher, an SVG icon,
and AppStream metadata so graphical software managers can show an Open button.

Linux helper install:

```sh
sudo versions/v_color_picker_vVERSION/v_color_picker_vVERSION_linux_x86_64_install.sh
```

macOS package:

```sh
sudo installer -pkg versions/v_color_picker_vVERSION/v_color_picker_vVERSION_macos_arm64.pkg -target /
```

macOS helper install:

```sh
sudo versions/v_color_picker_vVERSION/v_color_picker_vVERSION_macos_arm64_install.sh
```

BSD/Unix-like helper install:

```sh
sudo versions/v_color_picker_vVERSION/v_color_picker_vVERSION_unix_x86_64_install.sh
```

Windows:

```powershell
.\versions\v_color_picker_vVERSION\v_color_picker_vVERSION_windows_x86_64.exe
msiexec /i .\versions\v_color_picker_vVERSION\v_color_picker_vVERSION_windows_x86_64.msi
```

Both Windows installer paths install for all users, add the app directory to
PATH, and create Start Menu launchers that run through PowerShell.

## Uninstall

Linux:

```sh
sudo versions/v_color_picker_vVERSION/v_color_picker_vVERSION_linux_x86_64_uninstall.sh
```

macOS:

```sh
sudo versions/v_color_picker_vVERSION/v_color_picker_vVERSION_macos_arm64_uninstall.sh
```

BSD/Unix-like:

```sh
sudo versions/v_color_picker_vVERSION/v_color_picker_vVERSION_unix_x86_64_uninstall.sh
```

Windows:

```powershell
.\versions\v_color_picker_vVERSION\v_color_picker_vVERSION_windows_x86_64_uninstall.ps1
```

The uninstall scripts remove the app binary, launchers, metadata, and known app
state/cache directories for all users.

## Notes

- Linux screen capture support depends on the active display server and desktop
  permissions. X11 and Wayland portal capture are supported by the vendored
  capture backend.
- The Linux build intentionally avoids compile-time `pkg-config`,
  PipeWire/SPA, EGL/GBM/DRM, and XCB development header requirements. X11
  capture still needs the runtime `libxcb.so.1` library.
- macOS requires Screen Recording permission for desktop capture.
- BSD/Unix-like capture depends on the active display server support exposed by
  the capture backend.
- Windows `.msi` packaging requires WiX. Windows `.exe` installer packaging
  requires Inno Setup.

## Credits

Powered by The Velasquez.

## License

The `v_color_picker` library is distributed under either of

 * [Apache License, Version 2.0][LICENSE-APACHE]
 * [MIT license][LICENSE-MIT]

at your convenience.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[Release_Badge]: https://github.com/juanchoraf/v_color_picker/actions/workflows/release.yml/badge.svg
[Release_Url]: https://github.com/juanchoraf/v_color_picker/actions/workflows/release.yml
[Build_Badge]: https://github.com/juanchoraf/v_color_picker/actions/workflows/rust.yml/badge.svg?branch=main
[Build_Url]: https://github.com/juanchoraf/v_color_picker/actions?query=branch:main
[Dependencies_Badge]: https://deps.rs/repo/github/juanchoraf/v_color_picker/status.svg
[Dependencies_Url]: https://deps.rs/repo/github/juanchoraf/v_color_picker

[//]: # (licenses)

[LICENSE-APACHE]: https://github.com/juanchoraf/v_color_picker/blob/main/LICENSE-APACHE
[LICENSE-MIT]: https://github.com/juanchoraf/v_color_picker/blob/main/LICENSE-MIT
