#!/usr/bin/env python3
"""Build a private CLI and capture method-aware arm-loop diagnostics on two frozen classes."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import sys


ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
OUT = RESULTS / "arm-loop-diagnostic-luna-v1"
CLI_PATH = Path("/private/tmp/jarde-arm-loop-probe-cli-v1")
PATCH = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/arm-loop-diagnostic-luna-v1.patch"
BASELINES = (
    ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/baseline-root-v1",
    ROOT / "openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/baseline-root-v1",
)
ROOT_RUNNER = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
BUILD_ARGV = ["cargo", "build", "-p", "jarde-cli", "--locked"]
PROBE_MARKER = "JRE_ARM_LOOP_PROBE"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_guarded_runner():
    spec = importlib.util.spec_from_file_location("loop_latch_root_v9_guarded_runner", ROOT_RUNNER)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load guarded runner: {ROOT_RUNNER}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def original_javac23_class(baseline: Path) -> dict:
    manifest_path = baseline / "manifest.json"
    raw = manifest_path.read_bytes()
    manifest = json.loads(raw)
    cases = [case for case in manifest.get("cases", [])
             if case.get("label") == "javac23-original" and case.get("kind") == "original"
             and case.get("jdk_leg") == "javac23"]
    if len(cases) != 1:
        raise RuntimeError(f"expected one javac23 original case in {manifest_path}")
    case = cases[0]
    if not case.get("compile_success") or not case.get("runtime_success"):
        raise RuntimeError(f"javac23 original class did not compile and run successfully: {manifest_path}")
    class_record = case["actual_class"]
    class_path = (baseline / class_record["path"]).resolve(strict=True)
    class_bytes = class_path.read_bytes()
    if len(class_bytes) != class_record["bytes"] or sha(class_bytes) != class_record["sha256"]:
        raise RuntimeError(f"original class does not match its manifest: {class_path}")
    class_name = class_path.stem
    return {
        "manifest": {"path": str(manifest_path.relative_to(ROOT)), "sha256": sha(raw)},
        "schema": manifest["schema"],
        "class": {"path": str(class_path), "name": class_name,
                  "bytes": len(class_bytes), "sha256": sha(class_bytes)},
    }


def main() -> int:
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite diagnostic evidence: {OUT}")
    if CLI_PATH.exists():
        raise SystemExit(f"refusing to overwrite frozen diagnostic CLI: {CLI_PATH}")
    if not PATCH.is_file():
        raise SystemExit(f"temporary diagnostic patch is missing: {PATCH}")
    patch_text = PATCH.read_text(encoding="utf-8")
    if PROBE_MARKER not in patch_text:
        raise SystemExit("temporary patch does not contain the diagnostic marker")

    report_source = ROOT / "crates/jarde-java/src/report.rs"
    region_source = ROOT / "crates/jarde-java/src/region.rs"
    for source in (report_source, region_source):
        if PROBE_MARKER not in source.read_text(encoding="utf-8"):
            raise SystemExit(f"temporary diagnostic marker is not applied in {source}")

    classes = [original_javac23_class(path) for path in BASELINES]
    runner = load_guarded_runner()
    runner.OUT = OUT
    OUT.mkdir(parents=True)
    build_out = OUT / "build"
    build_out.mkdir()
    runner.OUT = build_out
    env = os.environ.copy()
    stripped = []
    for key in runner.STRIPPED_ENV_KEYS:
        if key in env:
            env.pop(key)
            stripped.append(key)
    env.update(runner.ENV_VALUES)
    preflight = {
        "patch": {"path": str(PATCH.relative_to(ROOT)), "sha256": sha(PATCH.read_bytes()),
                  "marker": PROBE_MARKER},
        "baseline_classes": classes,
        "stripped_environment_keys": stripped,
        "source_pins_before": runner.source_pins(runner.PRODUCT_PATHS),
        "source_pins_after": None,
        "guards": {"minimum_free_bytes": runner.FREE_LIMIT,
                   "maximum_target_bytes": runner.TARGET_LIMIT},
    }
    rows = []
    execution = {"schema": "recover-prefixed-one-arm-loops-arm-loop-diagnostic-luna-v1",
                 "status": "running", "validation_runner": str(Path(__file__).resolve()),
                 "preflight": preflight, "commands": rows, "frozen_cli": None}
    write_json(OUT / "execution.json", execution)

    try:
        preflight["jdk23"] = runner.configure_jdk23(env)
        result = runner.run_command(0, BUILD_ARGV, env)
        rows.append(result)
        execution["commands"] = rows
        write_json(OUT / "execution.json", execution)
        if result["exit_code"] != 0 or result["guard_stop"] is not None:
            raise RuntimeError("guarded cargo build failed or hit a disk guard")

        built_cli = ROOT / "target/debug/jarde-cli"
        info = built_cli.stat()
        if not stat.S_ISREG(info.st_mode) or not info.st_mode & 0o111:
            raise RuntimeError(f"built CLI is not a regular executable: {built_cli}")
        shutil.copy2(built_cli, CLI_PATH)
        CLI_PATH.chmod(0o555)
        cli_digest = sha(CLI_PATH.read_bytes())
        execution["frozen_cli"] = {"path": str(CLI_PATH), "sha256": cli_digest,
                                   "bytes": CLI_PATH.stat().st_size, "mode": "0555"}

        diagnostic_out = OUT / "commands"
        diagnostic_out.mkdir()
        runner.OUT = diagnostic_out
        for class_index, item in enumerate(classes):
            class_info = item["class"]
            argv = [str(CLI_PATH), "class-source", "--input", class_info["path"],
                    "--class", class_info["name"], "--policy", "single-class",
                    "--release", "8", "--format", "json"]
            row = runner.run_command(100 + len(rows), argv, env)
            row["input_class"] = class_info
            row["diagnostic_mode"] = "default"
            row["extra_env_overrides"] = {}
            rows.append(row)
            execution["commands"] = rows
            write_json(OUT / "execution.json", execution)
            if row["exit_code"] != 0 or row["guard_stop"] is not None:
                raise RuntimeError(f"default class-source failed for {class_info['name']}")

            probe_env = dict(env)
            probe_env[PROBE_MARKER] = "1"
            probe_row = runner.run_command(100 + len(rows), argv, probe_env)
            probe_row["input_class"] = class_info
            probe_row["diagnostic_mode"] = "JRE_ARM_LOOP_PROBE=1"
            probe_row["extra_env_overrides"] = {PROBE_MARKER: "1"}
            rows.append(probe_row)
            execution["commands"] = rows
            write_json(OUT / "execution.json", execution)
            if probe_row["exit_code"] != 0 or probe_row["guard_stop"] is not None:
                raise RuntimeError(f"probe class-source failed for {class_info['name']}")

        preflight["source_pins_after"] = runner.source_pins(runner.PRODUCT_PATHS)
        if preflight["source_pins_after"] != preflight["source_pins_before"]:
            raise RuntimeError("product source pins changed during diagnostic build and replay")
        execution["status"] = "diagnostic-captured"
        write_json(OUT / "execution.json", execution)
        return 0
    except (KeyError, OSError, RuntimeError, TypeError, ValueError) as error:
        preflight["source_pins_after"] = runner.source_pins(runner.PRODUCT_PATHS)
        execution["status"] = "failed"
        execution["failure"] = f"{type(error).__name__}: {error}"
        write_json(OUT / "execution.json", execution)
        print(execution["failure"], file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
