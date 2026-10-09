#!/usr/bin/env python3
"""Capture read-only class-source JSON for the frozen controls with the supplied CLI."""
from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import time
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[5]
OUT = pathlib.Path(__file__).resolve().parent
CLI = pathlib.Path("/private/tmp/jarde-em18-composition-cli-v1")
FIXTURE = ROOT / "tests/fixtures/p3-constructed-reference-array-controls-v1"
LEGS = {
    "javac8": FIXTURE / "javac8",
    "javac23": FIXTURE / "javac23",
}
CLASSES = ["NestedControls", "BoundaryControls"]


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_digest(path: pathlib.Path) -> str:
    return digest(path.read_bytes())


def main() -> int:
    if not CLI.is_file():
        raise SystemExit(f"provided frozen CLI is missing: {CLI}")
    commands = []
    cases = []
    for leg, class_dir in LEGS.items():
        for class_name in CLASSES:
            class_file = class_dir / f"{class_name}.class"
            jar = OUT / "inputs" / leg / f"{class_name}.jar"
            jar.parent.mkdir(parents=True, exist_ok=True)
            with zipfile.ZipFile(jar, "w", compression=zipfile.ZIP_STORED) as archive:
                archive.write(class_file, arcname=f"{class_name}.class")
            argv = [
                str(CLI),
                "class-source",
                "--input",
                str(jar),
                "--class",
                class_name,
                "--policy",
                "plain-jar",
                "--format",
                "json",
                "--evidence",
                "all",
                "--release",
                "8",
            ]
            result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            stdout_path = OUT / "reports" / leg / f"{class_name}.json"
            stderr_path = OUT / "reports" / leg / f"{class_name}.stderr"
            stdout_path.parent.mkdir(parents=True, exist_ok=True)
            stdout_path.write_bytes(result.stdout)
            stderr_path.write_bytes(result.stderr)
            command = {
                "leg": leg,
                "class": class_name,
                "argv": argv,
                "cwd": str(ROOT),
                "exit_code": result.returncode,
                "input_class_path": str(class_file),
                "input_class_sha256": file_digest(class_file),
                "input_jar_path": str(jar),
                "input_jar_sha256": file_digest(jar),
                "stdout_path": str(stdout_path.relative_to(OUT)),
                "stdout_bytes": len(result.stdout),
                "stdout_sha256": digest(result.stdout),
                "stderr_path": str(stderr_path.relative_to(OUT)),
                "stderr_bytes": len(result.stderr),
                "stderr_sha256": digest(result.stderr),
            }
            commands.append(command)
            try:
                report = json.loads(result.stdout)
            except json.JSONDecodeError:
                report = None
            cases.append({"leg": leg, "class": class_name, "exit_code": result.returncode,
                          "report_json_parsed": report is not None})

    manifest = {
        "schema": "constructed-reference-array-controls-cli1-agent-v1",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "purpose": "read-only inspection of real class-source reports; no Java runtime or Cargo invoked",
        "cli_path": str(CLI),
        "cli_sha256": file_digest(CLI),
        "fixture_copy_manifest": str((FIXTURE / "copy-root-v1.json").relative_to(ROOT)),
        "commands": commands,
        "cases": cases,
    }
    path = OUT / "manifest.json"
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"manifest": str(path), "manifest_sha256": file_digest(path),
                      "cli_sha256": manifest["cli_sha256"], "cases": cases}, indent=2))
    return 0 if all(case["exit_code"] == 0 and case["report_json_parsed"] for case in cases) else 1


if __name__ == "__main__":
    raise SystemExit(main())
