# PicoForge All

English | [中文](README.zh.md)

A desktop companion for **[Pico All](https://github.com/XiaoNetwork-Astral/pico-all)** security keys, built with Rust and GPUI.

## Features

- Manage passkeys, OATH accounts, OTP slots, PIV, OpenPGP and SmartCard-HSM
- Search and filter stored keys, objects and security events
- Read and verify Audit logs with calendar timestamps
- Configure status lights, sign firmware locally and install verified updates
- English and Simplified Chinese, with searchable IANA time zones

This is an independent fork of [PicoForge](https://github.com/librekeys/picoforge). Report issues [here](https://github.com/XiaoNetwork-Astral/pico-forge-all/issues). Existing RS-Key and pico-fido support is retained; available features depend on the connected firmware.

## Download and install

Download a **Windows x86 or x64** portable ZIP or installer from [Releases](https://github.com/XiaoNetwork-Astral/pico-forge-all/releases/latest). Extract the ZIP and run `picoforge.exe`, or run the `setup.exe` installer. For Linux and macOS, see [building from source](docs/Building.md).

The portable ZIP includes `portable.txt`; when this file exists beside the executable, settings and application data stay in that directory. Without it, PicoForge All uses standard system directories. The marker's contents do not matter; restart the app after adding or removing it.

The Windows package includes picotool for firmware operations. Source builds use [picotool 2.3.1+](https://github.com/raspberrypi/picotool/releases) from PATH or `PICOTOOL`.

On Windows, PicoForge All offers to restart as administrator when needed to read restricted device information. You can keep using it without elevation.

## Usage

- **Compose** changes device settings; **Software → Settings** controls the interface language and displayed time zone
- **Firmware** inspects, signs and installs UF2 files; select your existing secp256k1 PEM key to sign, and retain the same key for future updates
- **Audit** controls event recording, reads events and verifies the log; the device clock is synchronized on connection or refresh by default
- **HSM Objects** accepts text or files up to **1,800 bytes**; new IDs are assigned automatically

When the light flashes, press and release the device button (BOOTSEL). Matching Pico All firmware provides a default 60-second confirmation window. FIDO has no default PIN; known factory PINs for other applets are filled only when the device reports they are unchanged.

A device with Secure Boot enabled needs firmware signed with its original trusted key. Keep the key backed up offline. Erase/reset operations and permanent hardware changes show their consequences before confirmation.

## Build

Requires current stable Rust and the platform libraries listed in [Building](docs/Building.md).

```sh
git clone https://github.com/XiaoNetwork-Astral/pico-forge-all.git
cd pico-forge-all
cargo build --release --locked
```

Output: `target/release/picoforge.exe` on Windows or `target/release/picoforge` on Linux/macOS. Run `cargo test --locked` for offline unit tests; hardware tests are ignored by default.

## Troubleshooting

See [device detection and firmware setup](docs/Troubleshooting.md). Include the application version, firmware version and relevant console errors when reporting an issue; omit PINs and private keys.

## License and credits

[AGPL-3.0](LICENSE). Based on [PicoForge](https://github.com/librekeys/picoforge) by Suyog Tandel and the PicoForge contributors; Pico All integration and fork maintenance by BlueFunny.
