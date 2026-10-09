#!/usr/bin/env python3
"""单个 root 命令的原始输出、argv、退出码和文件身份；目录不可覆盖。"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent


def identity(path):
    path = Path(path)
    return {"path": str(path), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def main():
    label, *argv = sys.argv[1:]
    if not argv:
        raise SystemExit("need label and argv")
    destination = RESULTS / label
    destination.mkdir()
    environment = {"CARGO_BUILD_JOBS": "1", "CARGO_INCREMENTAL": "0",
                   "RUST_TEST_THREADS": "1", "CARGO_PROFILE_DEV_DEBUG": "0",
                   "CARGO_PROFILE_TEST_DEBUG": "0"}
    environment.update({key: value for key, value in os.environ.items()
                        if key.startswith(("JARDE_", "PROPTEST_"))})
    record = {"schema": "root-command-v1", "argv": argv, "cwd": str(ROOT),
              "env": environment, "runner": identity(__file__),
              "before": [identity(ROOT / relative) for relative in
                         ["crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
                          "tests/p3_constructor_primitive_conversion_arguments.rs"]],
              "free_before": shutil.disk_usage(ROOT).free}
    if record["free_before"] < 20 * 1024 ** 3:
        raise SystemExit("below 20 GiB build stop line")
    (destination / "start.json").write_text(json.dumps(record, indent=2) + "\n")
    started = time.monotonic()
    with (destination / "stdout").open("wb") as stdout, (destination / "stderr").open("wb") as stderr:
        process = subprocess.run(argv, cwd=ROOT, env={**os.environ, **environment},
                                 stdout=stdout, stderr=stderr)
    record.update({"exit": process.returncode, "elapsed_seconds": time.monotonic() - started,
                   "free_after": shutil.disk_usage(ROOT).free,
                   "stdout": identity(destination / "stdout"),
                   "stderr": identity(destination / "stderr")})
    (destination / "result.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({"label": label, "exit": process.returncode,
                      "elapsed_seconds": record["elapsed_seconds"]}), flush=True)
    raise SystemExit(process.returncode)


if __name__ == "__main__":
    main()
