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
| Uses GPU-backed GUI rendering when available, with Windows builds preferring the native WGPU backend.
| Shows HEX, HEXA, RGB, RGBA, HSL, HSLA, HWB, LAB, LCH, OKLAB, OKLCH, SRGB, SRGB linear, XYZ-D65, and packed U32 values.
| Alpha slider, zoom slider, related palette, alternatives, copy buttons, pick history, version, update checks, and update installation.
| Release scripts for 64-bit Linux, macOS, and Windows packaging.

## Run

Start the desktop app during development:

```sh
cargo run --release
```

Installed users launch v_color_picker from the operating system app launcher.

Example flow:

1. Open v_color_picker.
2. Press `Pick`.
3. Click any pixel on any monitor.
4. Select the format you need and copy it.

## Updates

`v_color_picker` is GUI-only. The bottom-right controls show the installed
version and an Update button. Update checks the release source and installs the
latest matching binary from `https://github.com/juanchoraf/v_color_picker` over
the current installation when a newer release is available.

## Notes

- Linux screen capture support depends on the active display server and desktop
  permissions. X11 and Wayland portal capture are supported by the vendored
  capture backend.
- GitHub Linux release packages are built on a Debian Bullseye baseline to keep
  the required `libc6` version broadly compatible. The `.deb` bundles the
  safe runtime libraries used by the app, including `libgcc_s`, `libxcb`,
  `libXau`, `libXdmcp`, `libbsd`, and `libmd`.
- The Linux build intentionally avoids compile-time `pkg-config`,
  PipeWire/SPA, EGL/GBM/DRM, and XCB development header requirements.
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
