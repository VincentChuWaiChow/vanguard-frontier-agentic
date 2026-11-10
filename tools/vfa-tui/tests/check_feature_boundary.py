#!/usr/bin/env python3
"""Check the engine's normal dependency closure, never the whole desktop graph.

Python 3.11+ and Cargo are required. --self-test alone is offline and does not
invoke Cargo; with --manifest-path it also checks that selected package live.
An explicit consumer is checked with its defaults, under its own lock graph.
"""

import argparse
import copy
import json
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path


ADAPTERS = {"ratatui", "crossterm", "terminal-light", "clap", "tracing-subscriber"}
TREE_ROW = re.compile(r"^(\d+)([A-Za-z0-9_-]+) v([^ ]+)(?: \((.+)\))?$")


class BoundaryError(Exception):
    """A missing prerequisite or a violated boundary, both fatal to this gate."""


def read_manifest(path):
    try:
        path = Path(path).resolve(strict=True)
        with path.open("rb") as handle:
            return path, tomllib.load(handle)
    except (OSError, ValueError, tomllib.TOMLDecodeError) as error:
        raise BoundaryError(f"cannot read manifest {path}: {error}") from error


def validate_manifest(manifest):
    if manifest.get("package", {}).get("name") != "vfa-tui":
        raise BoundaryError("selected engine manifest must identify package vfa-tui")
    features = manifest.get("features", {})
    if "terminal" not in features.get("default", []):
        raise BoundaryError("default features must explicitly enable terminal")
    terminal = features.get("terminal", [])
    if not isinstance(terminal, list):
        raise BoundaryError("terminal must be a feature list")
    dependencies = manifest.get("dependencies", {})
    for name in sorted(ADAPTERS):
        dependency = dependencies.get(name)
        if not isinstance(dependency, dict) or dependency.get("optional") is not True:
            raise BoundaryError(f"adapter dependency {name} must be optional")
        if f"dep:{name}" not in terminal:
            raise BoundaryError(f"terminal must enable dep:{name}")
    binaries = manifest.get("bin", [])
    if not binaries or not any(binary.get("name") == "vfa-tui" for binary in binaries):
        raise BoundaryError("missing explicit vfa-tui binary declaration")
    if any("terminal" not in binary.get("required-features", []) for binary in binaries):
        raise BoundaryError("every engine binary must require terminal")


def run_cargo(arguments):
    result = subprocess.run(
        ["cargo", *arguments], capture_output=True, text=True, check=False
    )
    if result.returncode:
        raise BoundaryError(
            f"cargo {' '.join(arguments)} failed ({result.returncode}):\n{result.stderr}"
        )
    return result.stdout


def validate_binary_targets(targets):
    if not isinstance(targets, list):
        raise BoundaryError("Cargo metadata must include discovered targets")
    binaries = [target for target in targets if "bin" in target.get("kind", [])]
    if not binaries or not any(target.get("name") == "vfa-tui" for target in binaries):
        raise BoundaryError("Cargo metadata is missing the engine binary")
    for target in binaries:
        if "terminal" not in target.get("required-features", []):
            raise BoundaryError(
                f"discovered binary {target.get('name')} must require terminal"
            )


def package_identity(manifest_path, check_binaries=False):
    try:
        metadata = json.loads(run_cargo([
            "metadata", "--manifest-path", str(manifest_path),
            "--format-version", "1", "--no-deps", "--locked",
        ]))
        matches = [package for package in metadata["packages"]
                   if Path(package["manifest_path"]).resolve() == manifest_path]
        if len(matches) != 1:
            raise BoundaryError("manifest must identify exactly one Cargo package")
        package = matches[0]
        if check_binaries:
            validate_binary_targets(package.get("targets"))
        return package["name"], package["version"], manifest_path.parent
    except (KeyError, TypeError, ValueError) as error:
        raise BoundaryError(f"malformed Cargo package metadata: {error}") from error


def parse_tree(output):
    rows = []
    previous_depth = -1
    for line in output.splitlines():
        if not line.strip():
            continue
        match = TREE_ROW.fullmatch(line)
        if match is None:
            raise BoundaryError(f"malformed cargo tree row: {line!r}")
        depth, name, version, source = match.groups()
        depth = int(depth)
        if depth > previous_depth + 1:
            raise BoundaryError("cargo tree has missing parent/root rows")
        rows.append((depth, name, version, source))
        previous_depth = depth
    if not rows:
        raise BoundaryError("cargo tree is empty")
    return rows


def same_package(row, identity):
    _, name, version, source = row
    expected_name, expected_version, directory = identity
    return (name == expected_name and version == expected_version
            and source is not None and Path(source).resolve() == directory)


def forbidden(name):
    return (name in ADAPTERS or name.startswith("ratatui-")
            or name == "gpui" or name.startswith(("gpui-", "gpui_")))


def validate_tree(output, engine, root):
    rows = parse_tree(output)
    roots = [row for row in rows if row[0] == 0]
    if len(roots) != 1 or not same_package(roots[0], root):
        raise BoundaryError("tree root does not match selected Cargo manifest package")
    engines = 0
    engine_depth = None
    for row in rows:
        depth, name, _, _ = row
        if engine_depth is not None and depth <= engine_depth:
            engine_depth = None
        if same_package(row, engine):
            engines += 1
            engine_depth = depth
        elif name == engine[0]:
            raise BoundaryError("engine name found with mismatched version or source path")
        if engine_depth is not None and forbidden(name):
            raise BoundaryError(f"presentation dependency in engine closure: {name}")
    if not engines:
        raise BoundaryError("selected engine is absent from consumer dependency graph")
    return engines


def check_live(engine_manifest, consumer_manifest):
    engine_path, manifest = read_manifest(engine_manifest)
    validate_manifest(manifest)
    engine = package_identity(engine_path, check_binaries=True)
    selected_path = engine_path
    root = engine
    if consumer_manifest is not None:
        selected_path, _ = read_manifest(consumer_manifest)
        root = package_identity(selected_path)
    arguments = [
        "tree", "--manifest-path", str(selected_path), "--locked", "--edges", "normal",
        "--prefix", "depth", "--format", "{p}", "--no-dedupe",
    ]
    if consumer_manifest is None:
        arguments.append("--no-default-features")
    occurrences = validate_tree(run_cargo(arguments), engine, root)
    print(f"PASS: {occurrences} engine subtree(s), no normal presentation dependencies")


def self_test():
    checks = 0

    def rejects(action):
        nonlocal checks
        try:
            action()
        except BoundaryError:
            checks += 1
        else:
            raise AssertionError("negative fixture was accepted")

    manifest = {
        "package": {"name": "vfa-tui"},
        "features": {"default": ["terminal"],
                     "terminal": [f"dep:{name}" for name in sorted(ADAPTERS)]},
        "dependencies": {name: {"optional": True} for name in ADAPTERS},
        "bin": [{"name": "vfa-tui", "required-features": ["terminal"]}],
    }
    validate_manifest(manifest)
    checks += 1
    targets = [
        {"name": "vfa_tui", "kind": ["lib"]},
        {"name": "vfa-tui", "kind": ["bin"], "required-features": ["terminal"]},
    ]
    validate_binary_targets(targets)
    checks += 1
    rejects(lambda: validate_binary_targets(
        targets + [{"name": "auto-probe", "kind": ["bin"]}]
    ))
    rejects(lambda: validate_binary_targets(None))
    rejects(lambda: validate_binary_targets([]))
    for mutation in ("default", "bin", "optional", "terminal", "identity"):
        bad = copy.deepcopy(manifest)
        if mutation == "default":
            bad["features"]["default"] = []
        elif mutation == "bin":
            bad["bin"][0]["required-features"] = []
        elif mutation == "optional":
            bad["dependencies"]["clap"]["optional"] = False
        elif mutation == "terminal":
            bad["features"]["terminal"].remove("dep:clap")
        else:
            bad["package"]["name"] = "not-the-engine"
        rejects(lambda: validate_manifest(bad))
    with tempfile.TemporaryDirectory(prefix="boundary café ") as temporary:
        directory = Path(temporary).resolve()
        engine = ("vfa-tui", "0.1.0", directory / "engine")
        consumer = ("desktop", "0.1.0", directory / "desktop")
        standalone = f"0vfa-tui v0.1.0 ({engine[2]})\n1serde v1.0.0\n"
        desktop = (f"0desktop v0.1.0 ({consumer[2]})\n"
                   f"1vfa-tui v0.1.0 ({engine[2]})\n2serde v1.0.0\n"
                   "1gpui-kit v0.9.1\n2gpui v0.3.12\n")
        validate_tree(standalone, engine, engine)
        validate_tree(desktop, engine, consumer)
        checks += 2
        for name in sorted(ADAPTERS | {"ratatui-core", "gpui", "gpui-kit", "gpui_platform"}):
            rejects(lambda: validate_tree(standalone + f"2{name} v1.0.0\n", engine, engine))
        for bad in ("", "not a tree", "1serde v1.0.0\n", standalone + "3serde v1.0.0\n",
                    "0serde v1.0.0\n", standalone.replace(str(engine[2]), str(directory / "wrong"))):
            rejects(lambda: validate_tree(bad, engine, engine))
        rejects(lambda: validate_tree(desktop.replace(
            f"1vfa-tui v0.1.0 ({engine[2]})\n2serde v1.0.0\n", ""), engine, consumer))
        rejects(lambda: validate_tree(desktop.replace(
            "2serde v1.0.0", "2gpui v0.3.12"), engine, consumer))
        rejects(lambda: read_manifest(directory / "missing.toml"))
        invalid = directory / "invalid.toml"
        invalid.write_text("[not valid TOML", encoding="utf-8")
        rejects(lambda: read_manifest(invalid))
    print(f"PASS: {checks} deterministic feature-boundary self-tests")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", type=Path)
    parser.add_argument("--consumer-manifest", type=Path)
    parser.add_argument("--self-test", action="store_true")
    arguments = parser.parse_args()
    if not arguments.manifest_path and (not arguments.self_test or arguments.consumer_manifest):
        parser.error("--manifest-path is required for live or consumer checks")
    try:
        if arguments.self_test:
            self_test()
        if arguments.manifest_path:
            check_live(arguments.manifest_path, arguments.consumer_manifest)
    except (BoundaryError, OSError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
