#!/usr/bin/env python3
"""Build a self-contained Windows ZIP from the release binary and picotool."""
import argparse
import hashlib
import re
import subprocess
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

parser = argparse.ArgumentParser()
parser.add_argument("--picotool", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--app", type=Path, default=Path("target/release/picoforge.exe"))
args = parser.parse_args()

for path in (args.app, args.picotool, Path("LICENSE"), Path("README.md"),
             Path("README.zh.md"), Path("third_party/picotool-LICENSE.TXT")):
    if not path.is_file():
        parser.error(f"Missing package input: {path}")

version = subprocess.run([str(args.picotool.resolve()), "version"],
                         capture_output=True, text=True, check=True, timeout=10).stdout
match = re.search(r"picotool v(\d+)\.(\d+)\.(\d+)", version)
if match is None or tuple(map(int, match.groups())) < (2, 3, 1):
    parser.error("picotool 2.3.1 or newer is required")

args.output.parent.mkdir(parents=True, exist_ok=True)
with ZipFile(args.output, "w", ZIP_DEFLATED, compresslevel=6) as archive:
    for source, name in ((args.app, "picoforge.exe"),
                         (args.picotool, "picotool.exe"),
                         (Path("LICENSE"), "LICENSE"),
                         (Path("README.md"), "README.md"),
                         (Path("README.zh.md"), "README.zh.md"),
                         (Path("third_party/picotool-LICENSE.TXT"), "picotool-LICENSE.TXT")):
        archive.write(source, name)
with ZipFile(args.output) as archive:
    if bad := archive.testzip():
        raise SystemExit(f"Corrupt ZIP entry: {bad}")
print(args.output)
print(version.strip())
print("sha256:" + hashlib.sha256(args.output.read_bytes()).hexdigest())
