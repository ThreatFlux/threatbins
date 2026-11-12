#!/usr/bin/env python3
"""Plan immutable package publication and verify its actual source archive."""

import argparse
import hashlib
import json
import re
import tarfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SHA = re.compile(r"[a-f0-9]{40}")
VERSION = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+")


def reuse_existing(source: str, published: str | None, changed: bool) -> bool:
    """A config-only commit may reuse an unchanged, immutable package version."""
    if not SHA.fullmatch(source) or published is not None and not SHA.fullmatch(published):
        raise ValueError("expected full source commit SHAs")
    if published is None or source == published:
        return False
    if changed:
        raise ValueError("office-sdk inputs changed: bump the package version before publishing")
    return True


def plan(source: str, published: str | None, changed: bool) -> dict:
    """Use the complete package-tree comparison made by the read-only workflow."""
    if not SHA.fullmatch(source) or published is not None and not SHA.fullmatch(published):
        raise ValueError("expected full source commit SHAs")
    version = tomllib.loads((ROOT / "office-sdk/Cargo.toml").read_text())["package"]["version"]
    if not VERSION.fullmatch(version):
        raise ValueError("expected a stable package version")
    return {"version": version, "tag": f"office-sdk-v{version}", "reuse": reuse_existing(source, published, changed), "published_source": published or source}


def verify(directory: Path, version: str, source: str) -> str:
    """Check the downloaded bytes, checksum filename, VCS identity and version."""
    if not VERSION.fullmatch(version) or not SHA.fullmatch(source):
        raise ValueError("invalid version or source SHA")
    name = f"office-sdk-{version}.crate"
    archive = directory / name
    receipt = (directory / (name + ".sha256")).read_text(encoding="utf-8")
    match = re.fullmatch(r"([a-f0-9]{64}) [ *]" + re.escape(name) + r"\n?", receipt)
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if match is None or match[1] != actual:
        raise ValueError("package checksum or receipt filename does not match")
    prefix = f"office-sdk-{version}/"
    with tarfile.open(archive, "r:gz") as package:
        vcs = json.load(package.extractfile(prefix + ".cargo_vcs_info.json"))
        manifest = tomllib.loads(package.extractfile(prefix + "Cargo.toml").read().decode())
    if vcs["git"]["sha1"] != source or vcs["git"].get("dirty", False):
        raise ValueError("package VCS identity does not match the release source")
    if manifest["package"]["name"] != "office-sdk" or manifest["package"]["version"] != version:
        raise ValueError("package manifest does not match the release version")
    return actual


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    planning = commands.add_parser("plan")
    planning.add_argument("--source", required=True)
    planning.add_argument("--published-source")
    planning.add_argument("--changed", action="store_true")
    checking = commands.add_parser("verify")
    checking.add_argument("--source", required=True)
    checking.add_argument("--version", required=True)
    checking.add_argument("--directory", required=True, type=Path)
    args = parser.parse_args()
    if args.command == "plan":
        print(json.dumps(plan(args.source, args.published_source, args.changed)))
    else:
        print(verify(args.directory, args.version, args.source))


if __name__ == "__main__":
    main()
