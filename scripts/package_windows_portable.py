#!/usr/bin/env python3
"""Build a portable ZIP and optional cargo-packager NSIS installer."""
import argparse
import hashlib
import json
import re
import shutil
import struct
import subprocess
import tempfile
import tomllib
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

parser = argparse.ArgumentParser()
parser.add_argument("--picotool", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--app", type=Path, default=Path("target/release/picoforge.exe"))
parser.add_argument("--runtime-dir", type=Path, required=True,
                    help="Matching Microsoft VC redistributable CRT directory")
parser.add_argument("--installer-output", type=Path)
parser.add_argument("--packager", default="cargo-packager")
args = parser.parse_args()

files = [(args.app, "picoforge.exe"), (args.picotool, "picotool.exe")]
files += [(Path(name), name) for name in ("LICENSE", "README.md", "README.zh.md")]
files += [(path, path.name) for path in Path("third_party").glob("*-LICENSE.TXT")]
files += [(path, path.name) for path in args.picotool.parent.glob("*.dll")]
files.append((args.runtime_dir / "vcruntime140.dll", "vcruntime140.dll"))
if (args.runtime_dir / "vcruntime140_1.dll").is_file():
    files.append((args.runtime_dir / "vcruntime140_1.dll", "vcruntime140_1.dll"))

def machine(path):
    with path.open("rb") as source:
        if source.read(2) != b"MZ":
            raise ValueError(f"Not a Windows executable: {path}")
        source.seek(0x3c)
        source.seek(struct.unpack("<I", source.read(4))[0])
        if source.read(4) != b"PE\0\0":
            raise ValueError(f"Invalid PE header: {path}")
        return struct.unpack("<H", source.read(2))[0]

for path, _ in files:
    if not path.is_file():
        parser.error(f"Missing package input: {path}")
architecture = machine(args.app)
target = {0x14c: "i686-pc-windows-msvc", 0x8664: "x86_64-pc-windows-msvc"}.get(architecture)
if not target:
    parser.error("Only x86 and x64 Windows packages are supported")
for path, _ in files:
    if path.suffix.lower() in (".exe", ".dll") and machine(path) != architecture:
        parser.error(f"Architecture mismatch: {path}")

version = subprocess.run([str(args.picotool.resolve()), "version"],
                         capture_output=True, text=True, check=True, timeout=10).stdout
match = re.search(r"picotool v(\d+)\.(\d+)\.(\d+)", version)
if match is None or tuple(map(int, match.groups())) < (2, 3, 1):
    parser.error("picotool 2.3.1 or newer is required")

args.output.parent.mkdir(parents=True, exist_ok=True)
with ZipFile(args.output, "w", ZIP_DEFLATED, compresslevel=6) as archive:
    for source, name in files:
        archive.write(source, name)
    archive.writestr("portable.txt", "")
with ZipFile(args.output) as archive:
    if bad := archive.testzip():
        raise SystemExit(f"Corrupt ZIP entry: {bad}")
print(args.output)
print(version.strip())
print("sha256:" + hashlib.sha256(args.output.read_bytes()).hexdigest())

if args.installer_output:
    args.installer_output.parent.mkdir(parents=True, exist_ok=True)
    # A fresh staging directory keeps portable.txt and local settings out of installers.
    with tempfile.TemporaryDirectory(prefix="installer-", dir=args.output.parent) as temporary:
        stage = Path(temporary).resolve()
        for source, name in files:
            shutil.copy2(source, stage / name)
        package = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["package"]
        config = {
            "name": "pico-forge-all", "productName": "PicoForge All",
            "version": package["version"], "identifier": "org.xiaonetwork.pico-forge-all",
            "publisher": "XiaoNetwork-Astral", "description": package["description"],
            "licenseFile": str(Path("LICENSE").resolve()),
            "icons": [str(Path("static/appIcons/icon.ico").resolve())],
            "binaries": [{"path": "picoforge", "main": True}],
            "binariesDir": str(stage), "outDir": str(stage / "output"),
            "targetTriple": target, "formats": ["nsis"],
            "resources": [{"src": str(stage / name), "target": name}
                          for _, name in files if name != "picoforge.exe"],
            "nsis": {"installMode": "currentUser", "languages": ["English", "SimpChinese"]},
        }
        config_file = stage / "packager.json"
        config_file.write_text(json.dumps(config, indent=2), encoding="utf-8")
        subprocess.run([args.packager, "--config", str(config_file)], check=True)
        installers = list((stage / "output").glob("*.exe"))
        if len(installers) != 1:
            raise SystemExit(f"Expected one installer, found {installers}")
        shutil.copy2(installers[0], args.installer_output)
    print(args.installer_output)
    print("sha256:" + hashlib.sha256(args.installer_output.read_bytes()).hexdigest())
