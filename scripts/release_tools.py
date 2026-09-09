#!/usr/bin/env python3
"""Validate release identity and stage verified CI packages (Python 3.11+).

This does not build packages or validate desktop behavior. CI verifies package
contents; docs/SMOKE_TESTS.md is the separate, mandatory human release gate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import sys
import tomllib

TAG_PATTERN = re.compile(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-[0-9A-Za-z.-]+)?")
ASSET_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._+-]*")
EXTENSIONS = (".exe", ".deb", ".AppImage")


def validate_tag(root: Path, ref_type: str, tag: str) -> str:
    """Require a tag matching both application version sources exactly."""
    if ref_type != "tag":
        raise ValueError("Release requires a tag event, not a branch event")
    if not TAG_PATTERN.fullmatch(tag):
        raise ValueError(f"Invalid version tag: {tag!r}")
    tauri = json.loads((root / "nyrva/tauri.conf.json").read_text(encoding="utf-8"))
    with (root / "nyrva/Cargo.toml").open("rb") as file:
        cargo = tomllib.load(file)
    tauri_version = tauri["version"]
    cargo_version = cargo["package"]["version"]
    if tauri_version != cargo_version:
        raise ValueError(f"Tauri/Cargo versions differ: {tauri_version!r} != {cargo_version!r}")
    if tag != f"v{tauri_version}":
        raise ValueError(f"Tag {tag!r} does not match application version {tauri_version!r}")
    return tag


def prepare_assets(source: Path, output: Path) -> list[Path]:
    """Flatten the three package formats and emit deterministic SHA-256 sums.

    Refuse missing/duplicate/empty packages, unsafe names and existing output.
    No existing file or published release is overwritten by this command.
    """
    if not source.is_dir():
        raise ValueError(f"Artifact directory does not exist: {source}")
    if output.exists():
        raise ValueError(f"Output already exists; refusing to overwrite: {output}")
    selected = []
    for extension in EXTENSIONS:
        matches = sorted(source.rglob(f"*{extension}"))
        if len(matches) != 1:
            raise ValueError(f"Expected exactly one {extension} package, found {len(matches)}")
        package = matches[0]
        if package.is_symlink():
            raise ValueError(f"Package must not be a symlink: {package.name}")
        if not package.is_file() or package.stat().st_size == 0:
            raise ValueError(f"Package is not a regular file or is empty: {package.name}")
        if not ASSET_PATTERN.fullmatch(package.name):
            raise ValueError(f"Unsafe asset filename: {package.name!r}")
        selected.append(package)

    output.mkdir(parents=True, exist_ok=False)
    staged = []
    checksums = []
    for package in sorted(selected, key=lambda path: path.name):
        destination = output / package.name
        shutil.copy2(package, destination)
        with destination.open("rb") as file:
            digest = hashlib.file_digest(file, "sha256").hexdigest()
        checksums.append(f"{digest}  {destination.name}\n")
        staged.append(destination)
    (output / "SHA256SUMS").write_text("".join(checksums), encoding="utf-8")
    return staged


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    tag = commands.add_parser("validate-tag")
    tag.add_argument("--root", type=Path, default=Path("."))
    tag.add_argument("--ref-type", required=True)
    tag.add_argument("--tag", required=True)
    assets = commands.add_parser("prepare-assets")
    assets.add_argument("--source", type=Path, required=True)
    assets.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    try:
        if arguments.command == "validate-tag":
            result = validate_tag(arguments.root, arguments.ref_type, arguments.tag)
            print(f"Validated release tag: {result}")
        else:
            result = prepare_assets(arguments.source, arguments.output)
            print(f"Prepared {len(result)} packages and SHA256SUMS in {arguments.output}")
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Release validation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
