#!/usr/bin/env python3
"""Run the Java pattern target under the narrower-dependency local disk guard."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
DISK_STOP_BYTES = 2 * 1024 ** 3
TARGET_STOP_BYTES = 1024 ** 3
POLL_SECONDS = 2


def identity(path):
    path = Path(path)
    return {"path": str(path), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def disk_free():
    return shutil.disk_usage(ROOT).free


def target_bytes():
    total = 0
    for directory, _, files in os.walk(ROOT / "target"):
        for name in files:
            try:
                total += (Path(directory) / name).stat().st_size
            except FileNotFoundError:
                pass  # rustc atomically renames/removes its temporary files.
    return total


def terminate_process_group(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return process.wait()
    try:
        return process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        return process.wait()


def main():
    label, *argv = sys.argv[1:]
    if not argv:
        raise SystemExit("need label and argv")
    if argv[:7] != ["cargo", "test", "-p", "jarde-java", "--test", "p3_patterns", "--locked"]:
        raise SystemExit("bounded local runner accepts only the Java pattern integration target")
    destination = RESULTS / label
    destination.mkdir()
    environment = {
        "CARGO_BUILD_JOBS": "1",
        "CARGO_INCREMENTAL": "0",
        "RUST_TEST_THREADS": "1",
        "CARGO_PROFILE_DEV_DEBUG": "0",
        "CARGO_PROFILE_TEST_DEBUG": "0",
        "CARGO_PROFILE_DEV_STRIP": "symbols",
        "CARGO_PROFILE_TEST_STRIP": "symbols",
    }
    environment.update({key: value for key, value in os.environ.items()
                        if key.startswith(("JARDE_", "PROPTEST_"))})
    record = {
        "schema": "root-command-v3-bounded-local-disk-guard",
        "argv": argv,
        "cwd": str(ROOT),
        "env": environment,
        "profile_settings": {
            "dev_debug": "0",
            "test_debug": "0",
            "dev_strip": "symbols",
            "test_strip": "symbols",
            "opt_level": "Cargo default; not overridden",
            "debug_assertions": "Cargo default remains enabled; not overridden",
        },
        "disk_guard": {"threshold_bytes": DISK_STOP_BYTES, "poll_seconds": POLL_SECONDS, "target_cap_bytes": TARGET_STOP_BYTES,
                       "termination": "SIGTERM the command's own process group; SIGKILL after 10s"},
        "runner": identity(__file__),
        "before": [identity(ROOT / relative) for relative in
                   ["crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
                    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
                    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
                    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
                    "crates/jarde-java/tests/p3_patterns.rs",
                    "tests/recover_lambda_primitive_array_capture.rs",
                    "tests/p3_constructed_reference_array_elements.rs",
                    "tests/recover_boxed_number_widening.rs", ".github/workflows/ci.yml"]],
        "free_before": disk_free(),
        "target_bytes_before": target_bytes(),
    }
    if record["free_before"] < DISK_STOP_BYTES or record["target_bytes_before"] >= TARGET_STOP_BYTES:
        record.update({
            "exit": None,
            "disk_stop_reason": "preflight free space below 2 GiB reserve or target at 1 GiB cap",
            "min_free": record["free_before"],
            "free_after": record["free_before"],
        })
        (destination / "start.json").write_text(json.dumps(record, indent=2) + "\n")
        (destination / "stdout").write_bytes(b"")
        (destination / "stderr").write_bytes(b"")
        record.update({"stdout": identity(destination / "stdout"),
                       "stderr": identity(destination / "stderr")})
        (destination / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        raise SystemExit("below bounded local reserve or target cap; command was not launched")

    (destination / "start.json").write_text(json.dumps(record, indent=2) + "\n")
    started = time.monotonic()
    min_free = record["free_before"]
    max_target = record["target_bytes_before"]
    disk_stop_reason = None
    with (destination / "stdout").open("wb") as stdout, (destination / "stderr").open("wb") as stderr:
        process = subprocess.Popen(
            argv, cwd=ROOT, env={**os.environ, **environment},
            stdout=stdout, stderr=stderr, start_new_session=True,
        )
        while process.poll() is None:
            time.sleep(POLL_SECONDS)
            current_free = disk_free()
            min_free = min(min_free, current_free)
            current_target = target_bytes()
            max_target = max(max_target, current_target)
            if current_free < DISK_STOP_BYTES or current_target >= TARGET_STOP_BYTES:
                disk_stop_reason = {
                    "reason": "bounded local machine reserve or target cap crossed during command",
                    "observed_target_bytes": current_target,
                    "target_cap_bytes": TARGET_STOP_BYTES,
                    "observed_free_bytes": current_free,
                    "threshold_bytes": DISK_STOP_BYTES,
                    "process_group_id": process.pid,
                }
                terminate_process_group(process)
                break
        process_exit = process.wait()

    final_free = disk_free()
    max_target = max(max_target, target_bytes())
    record["max_target_bytes"] = max_target
    min_free = min(min_free, final_free)
    record.update({
        "exit": process_exit,
        "runner_exit": 1 if disk_stop_reason is not None else process_exit,
        "elapsed_seconds": time.monotonic() - started,
        "free_after": final_free,
        "min_free": min_free,
        "disk_stop_reason": disk_stop_reason,
        "stdout": identity(destination / "stdout"),
        "stderr": identity(destination / "stderr"),
    })
    (destination / "result.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({"label": label, "exit": process_exit,
                      "runner_exit": record["runner_exit"],
                      "elapsed_seconds": record["elapsed_seconds"],
                      "min_free": min_free,
                      "disk_stop_reason": disk_stop_reason}), flush=True)
    raise SystemExit(record["runner_exit"])


if __name__ == "__main__":
    main()
