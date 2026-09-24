# Building PicoForge All

Use current stable Rust. Clone this repository, install the dependencies below, then run `cargo build --release --locked`.

## Windows

Install Visual Studio Build Tools with **Desktop development with C++**, the Windows SDK, and the Rust MSVC toolchain. PC/SC is provided by Windows.

Windows releases provide x86 (`i686-pc-windows-msvc`) and x64 (`x86_64-pc-windows-msvc`) builds. Install the target with `rustup target add`, then pass it to `cargo build --release --locked --target`.

Packaging uses Python 3.11+ and `cargo-packager 0.11.8` with NSIS, as upstream does. From a matching MSVC developer shell, `scripts/prepare_windows_picotool.ps1 -Arch x86 -WorkDir target/picotool-x86` prepares the bundled tool (x86 also needs `pip install cmake==3.31.10 ninja`; use `-Arch x64` for x64). Run `scripts/package_windows_portable.py --help` for the ZIP and installer arguments; `--runtime-dir` selects the matching `Microsoft.VC*.CRT` directory under `VCToolsRedistDir`.

The ZIP contains `portable.txt`; the installer does not. The release workflow builds and packages both architectures.

## macOS

Install Xcode Command Line Tools and Rust. PC/SC is provided by macOS.

## Linux

On Ubuntu/Debian:

```sh
sudo apt install build-essential pkg-config libpcsclite-dev pcscd libccid libudev-dev libvulkan-dev libwayland-dev wayland-protocols libxkbcommon-dev libxcb1-dev libxkbcommon-x11-dev libfontconfig1-dev libasound2-dev libdbus-1-dev libx11-dev libxcb-shape0-dev libxcb-xfixes0-dev libusb-1.0-0-dev
cargo build --release --locked
```

A graphical session and access to the security key are required to run the application. The repository also includes Nix development files.

## Tests

`cargo test --locked` runs unit tests. Tests marked `ignored` require hardware, external tools or explicit destructive-operation authorization; read each test before opting in.
