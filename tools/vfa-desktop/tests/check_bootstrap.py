#!/usr/bin/env python3
"""Grade bootstrap pinning/normal features, not compilation or native behavior."""

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tomllib


LAYERS = {"gpui-kit", "gpui-base", "gpui-component", "gpui-component-macros", "gpui-kit-assets"}
FEATURES = {"assets", "component"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def toolkit_packages(lock):
    return [
        {key: package[key] for key in ("name", "version", "source", "checksum")}
        for package in lock["package"]
        if package["name"] in LAYERS or package["name"].startswith("gpui-pre")
    ]


def validate(manifest, lock_bytes, provenance, pin, files):
    require(manifest["package"].get("publish") is False, "Desktop must not publish")
    require("workspace" not in manifest, "Do not introduce a Cargo workspace")
    normal = manifest["dependencies"]["gpui-kit"]
    development = manifest["dev-dependencies"]["gpui-kit"]
    require(normal["version"] == "=0.9.1", "Kit must select the verified exact release")
    require(normal.get("default-features") is False, "Kit defaults must be explicit")
    require(set(normal["features"]) == FEATURES, "Unexpected normal Kit features")
    require(development["version"] == normal["version"], "Test source/version differs")
    require(development.get("default-features") is False, "Test defaults must be explicit")
    require(set(development["features"]) == FEATURES | {"test-support"}, "Incorrect dev test features")
    engine = manifest["dependencies"]["vfa-tui"]
    require(engine.get("default-features") is False, "Engine must exclude terminal")
    require(not engine.get("features"), "Unexpected engine feature")
    require(engine.get("path") == "../vfa-tui", "Engine must reuse the adjacent crate")
    forbidden = {"gpui", "gpui-pre", "gpui-fast", "gpui-webview", "gpui-shell"}
    require(not forbidden.intersection(manifest["dependencies"]), "Extra direct GPUI layer")
    native = [target for target in manifest.get("test", []) if target["name"] == "native_smoke"]
    require(len(native) == 1, "Missing native smoke target")
    require(native[0].get("test") is False and native[0].get("harness") is False,
            "Native smoke must be explicit and main-thread")
    require(native[0].get("path", "tests/native_smoke.rs") == "tests/native_smoke.rs",
            "Native target points to another source")
    require(all(files.values()), "Required bootstrap/test artifact is missing")
    require(provenance["schema_version"] == 1, "Unknown provenance schema")
    require(provenance["kit"]["commit"] == "39ab849a93b5f15507b969164d49891db8d5b470",
            "Unverified Kit source commit")
    require(provenance["kit"]["version"] == "0.9.1", "Provenance Kit version drift")
    require(provenance["kit"]["immutable_facade_source_matches_archive"] is True,
            "Immutable facade agreement was not established")
    require(provenance["gpui_snapshot"]["version"] == "0.3.12", "Snapshot drift")
    require(provenance["gpui_snapshot"]["upstream_commit"] == "d2a4fb6ded7c470be25d1541281da3882495b65b",
            "GPUI upstream commit differs from the acquired snapshot")
    require(provenance["normal_kit_features"] == sorted(FEATURES), "Normal feature provenance drift")
    require(provenance["dev_only_kit_features"] == ["test-support"], "Dev feature provenance drift")
    require(provenance["lockfile_sha256"] == hashlib.sha256(lock_bytes).hexdigest(), "Lock hash drift")
    lock = tomllib.loads(lock_bytes.decode())
    require(provenance["packages"] == toolkit_packages(lock), "Locked toolkit identities/checksums drift")
    for package in provenance["packages"]:
        expected = "0.9.1" if package["name"] in LAYERS else (
            "0.12.15" if package["name"] == "gpui-pre-reqwest" else "0.3.12"
        )
        require(package["version"] == expected, f"Mixed snapshot/layer: {package['name']}")
        require(package["source"] == "registry+https://github.com/rust-lang/crates.io-index",
                "Unexpected registry/source")
    kit = next(p for p in provenance["packages"] if p["name"] == "gpui-kit")
    require(provenance["kit"]["archive_sha256"] == kit["checksum"], "Kit archive checksum differs")
    require(pin["toolchain"]["channel"] == provenance["compiler"]["release"], "Compiler pin drift")


def self_test(context):
    changes = [
        ("publish", lambda c: c[0]["package"].update(publish=True)),
        ("caret Kit", lambda c: c[0]["dependencies"]["gpui-kit"].update(version="0.9.1")),
        ("normal test support", lambda c: c[0]["dependencies"]["gpui-kit"]["features"].append("test-support")),
        ("normal Fast", lambda c: c[0]["dependencies"]["gpui-kit"]["features"].append("gpui-fast")),
        ("default features", lambda c: c[0]["dependencies"]["gpui-kit"].pop("default-features")),
        ("test version", lambda c: c[0]["dev-dependencies"]["gpui-kit"].update(version="=0.9.0")),
        ("terminal engine", lambda c: c[0]["dependencies"]["vfa-tui"].update({"default-features": True})),
        ("automatic native test", lambda c: c[0]["test"][0].update(test=True)),
        ("threaded native harness", lambda c: c[0]["test"][0].update(harness=True)),
        ("redirected native source", lambda c: c[0]["test"][0].update(path="tests/ui.rs")),
        ("missing UI target", lambda c: c[4].update({"tests/ui.rs": False})),
        ("Kit commit", lambda c: c[2]["kit"].update(commit="0" * 40)),
        ("unverified facade", lambda c: c[2]["kit"].update(immutable_facade_source_matches_archive=False)),
        ("GPUI upstream commit", lambda c: c[2]["gpui_snapshot"].update(upstream_commit="0" * 40)),
        ("snapshot", lambda c: c[2]["gpui_snapshot"].update(version="0.3.8")),
        ("checksum", lambda c: c[2]["packages"][0].update(checksum="0" * 64)),
        ("lock hash", lambda c: c[2].update(lockfile_sha256="0" * 64)),
        ("compiler pin", lambda c: c[3]["toolchain"].update(channel="stable")),
    ]
    for name, mutate in changes:
        candidate = copy.deepcopy(context)
        mutate(candidate)
        require(candidate != context, f"Inactive mutation: {name}")
        try:
            validate(*candidate)
        except ValueError:
            continue
        raise ValueError(f"Accepted negative mutation: {name}")
    print(f"PASS: {len(changes)} deterministic pin/feature/target mutants rejected")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", required=True, type=Path)
    parser.add_argument("--toolchain", help="Explicit installed alias; compiler bits must match numeric pin")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--documents-only", action="store_true", help="Do not claim live compiler/archive/graph checks")
    args = parser.parse_args()
    root = args.manifest_path.resolve().parent
    load = lambda name: tomllib.loads((root / name).read_text())
    provenance = json.loads((root / "provenance/toolkit.json").read_text())
    files = {name: (root / name).is_file() for name in [
        "src/main.rs", "src/lib.rs", "tests/ui.rs", "tests/native_smoke.rs",
    ]}
    context = (load("Cargo.toml"), (root / "Cargo.lock").read_bytes(), provenance,
               load("rust-toolchain.toml"), files)
    validate(*context)
    if args.self_test:
        self_test(context)
    print("PASS: declared pinning/provenance/feature/test-target contract")
    if args.documents_only:
        print("UNVERIFIED: live compiler/archive/graph checks not selected")
        return
    command = lambda name: [name] + (["+" + args.toolchain] if args.toolchain else [])
    run = lambda argv: subprocess.check_output(argv, text=True)
    compiler = dict(line.split(": ", 1) for line in run(command("rustc") + ["-Vv"]).splitlines()[1:] if ": " in line)
    for key, value in provenance["compiler"].items():
        require(compiler.get(key) == value, f"Actual compiler differs: {key}")
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    archive_name = "gpui-kit-0.9.1.crate"
    archives = list((cargo_home / "registry/cache").glob("*/" + archive_name))
    require(len(archives) == 1, "Select an unambiguous downloaded Kit archive")
    require(hashlib.sha256(archives[0].read_bytes()).hexdigest() == provenance["kit"]["archive_sha256"],
            "Downloaded Kit archive differs")
    sources = list((cargo_home / "registry/src").glob("*/gpui-kit-0.9.1/.cargo_vcs_info.json"))
    require(len(sources) == 1, "Missing/unresolved Kit source VCS record")
    require(json.loads(sources[0].read_text())["git"]["sha1"] == provenance["kit"]["commit"],
            "Downloaded source commit differs")
    with tarfile.open(archives[0]) as archive:
        facade = archive.extractfile("gpui-kit-0.9.1/src/lib.rs").read()
        require(hashlib.sha256(facade).hexdigest() == provenance["kit"]["facade_sha256"],
                "Archived facade differs from the captured immutable facade")
        require(facade == (sources[0].parent / "src/lib.rs").read_bytes(),
                "Extracted facade differs from the verified archive")
    snapshot_sources = list((cargo_home / "registry/src").glob("*/gpui-pre-0.3.12/Cargo.toml"))
    require(len(snapshot_sources) == 1, "Missing/ambiguous GPUI source metadata")
    snapshot = tomllib.loads(snapshot_sources[0].read_text())
    require(snapshot["package"]["metadata"]["gpui-pre"]["zed-rev"] == provenance["gpui_snapshot"]["upstream_commit"],
            "Snapshot source metadata differs")
    tree = run(command("cargo") + ["tree", "--manifest-path", str(args.manifest_path.resolve()),
        "--locked", "--offline", "--target", compiler["host"], "--edges", "normal,build",
        "--prefix", "none", "--format", "{p}|{f}"])
    observed_features = None
    for line in tree.splitlines():
        package, features = line.split("|", 1)
        name = package.split()[0]
        require(not name.startswith(("gpui-fast", "gpui-shell", "gpui-webview", "quickjs")),
                f"Forbidden normal layer: {name}")
        require("test-support" not in features, f"Dev support leaked: {name}")
        if name == "gpui-kit":
            observed_features = set(features.replace(" (*)", "").split(","))
    require(observed_features == FEATURES, "Actual normal Kit features differ")
    print("PASS: actual compiler, cached archive/VCS and normal feature graph")
    print("UNVERIFIED here: compilation, UI callback behavior and native launch require their own graders")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
