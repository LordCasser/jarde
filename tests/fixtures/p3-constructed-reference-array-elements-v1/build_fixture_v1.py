#!/usr/bin/env python3
"""Compile and freeze the complete constructed-reference-array fixture on both JDKs."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import sys
import time

FIXTURE = pathlib.Path(__file__).resolve().parent
REPO = FIXTURE.parents[2]
RESULTS = REPO / "openspec/changes/compose-constructed-reference-array-elements/results/fixture-v1"
SOURCES = [
    "Main.java",
    "Base.java",
    "Mid.java",
    "DirectA.java",
    "DirectB.java",
    "TwoHop.java",
    "LocalInterface.java",
]
METHODS = ["sequence", "collections", "failures", "ownDirect", "ownTwoHop", "ownInterface"]
EXPECTED_OPCODE_COUNTS = {
    "sequence": {"new": 2, "dup": 4, "invokespecial": 2, "aastore": 2},
    "collections": {"new": 2, "dup": 6, "invokespecial": 2, "aastore": 4},
    "failures": {"new": 2, "dup": 4, "invokespecial": 2, "aastore": 2},
    "ownDirect": {"new": 2, "dup": 4, "invokespecial": 2, "aastore": 2},
    "ownTwoHop": {"new": 2, "dup": 4, "invokespecial": 2, "aastore": 2},
    "ownInterface": {"new": 2, "dup": 4, "invokespecial": 2, "aastore": 2},
}
JDKS = {
    "javac8": {
        "home": pathlib.Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"),
        "flags": ["-source", "8", "-target", "8"],
    },
    "javac23": {
        "home": pathlib.Path("/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"),
        "flags": ["--release", "8"],
    },
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: pathlib.Path) -> str:
    return sha256(path.read_bytes())


def write_bytes(path: pathlib.Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def record_run(label: str, argv: list[pathlib.Path | str], cwd: pathlib.Path, run_root: pathlib.Path,
               commands: list[dict]) -> subprocess.CompletedProcess:
    actual_argv = [str(value) for value in argv]
    result = subprocess.run(actual_argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout_path = run_root / "logs" / f"{label}.stdout"
    stderr_path = run_root / "logs" / f"{label}.stderr"
    write_bytes(stdout_path, result.stdout)
    write_bytes(stderr_path, result.stderr)
    commands.append({
        "label": label,
        "argv": actual_argv,
        "cwd": str(cwd),
        "exit_code": result.returncode,
        "stdout_path": str(stdout_path.relative_to(run_root)),
        "stdout_bytes": len(result.stdout),
        "stdout_sha256": sha256(result.stdout),
        "stderr_path": str(stderr_path.relative_to(run_root)),
        "stderr_bytes": len(result.stderr),
        "stderr_sha256": sha256(result.stderr),
    })
    return result


def parse_main_javap(text: str) -> dict[str, dict[str, list[int]]]:
    instructions: dict[str, dict[str, list[int]]] = {}
    current: str | None = None
    method_pattern = re.compile(r"^\s*(?:public|private|protected)\s+.*\s([A-Za-z_$][\w$]*)\([^)]*\);$")
    opcode_pattern = re.compile(r"^\s*(\d+):\s+(new|dup|invokespecial|aastore)\b")
    for line in text.splitlines():
        method = method_pattern.match(line)
        if method:
            current = method.group(1)
            if current in METHODS:
                instructions.setdefault(current, {opcode: [] for opcode in ("new", "dup", "invokespecial", "aastore")})
            else:
                current = None
            continue
        if current is None:
            continue
        opcode = opcode_pattern.match(line)
        if opcode:
            instructions[current][opcode.group(2)].append(int(opcode.group(1)))
    return instructions


def main() -> int:
    run_id = sys.argv[1] if len(sys.argv) > 1 else "run-001"
    if not re.fullmatch(r"run-[0-9]{3}", run_id):
        raise SystemExit("run id must match run-NNN")
    run_root = RESULTS / run_id
    manifest_path = run_root / "manifest.json"
    if run_root.exists():
        raise SystemExit(f"refusing to overwrite frozen or partial run: {run_root}")
    run_root.mkdir(parents=True)
    empty_path = run_root / "empty-classpath-sourcepath"
    empty_path.mkdir()
    commands: list[dict] = []
    legs: dict[str, dict] = {}
    sources = [FIXTURE / name for name in SOURCES]
    source_records = [
        {"path": path.name, "bytes": path.stat().st_size, "sha256": file_sha(path)}
        for path in sources
    ]

    for leg_name, spec in JDKS.items():
        home: pathlib.Path = spec["home"]
        javac, java, javap = home / "bin/javac", home / "bin/java", home / "bin/javap"
        for tool in (javac, java, javap):
            if not tool.is_file():
                raise SystemExit(f"required tool is missing: {tool}")
        work = run_root / leg_name
        classes = work / "original-classes"
        classes.mkdir(parents=True)
        javac_version = record_run(f"{leg_name}-javac-version", [javac, "-version"], FIXTURE, run_root, commands)
        java_version = record_run(f"{leg_name}-java-version", [java, "-version"], FIXTURE, run_root, commands)
        javap_version = record_run(f"{leg_name}-javap-version", [javap, "-version"], FIXTURE, run_root, commands)
        compile_argv = [javac, *spec["flags"], "-g:none", "-classpath", empty_path,
                        "-sourcepath", empty_path, "-d", classes, *sources]
        compile_result = record_run(f"{leg_name}-compile", compile_argv, FIXTURE, run_root, commands)
        class_files = sorted(classes.rglob("*.class"))
        runtime_result = None
        javap_result = None
        method_instructions = {}
        if compile_result.returncode == 0:
            runtime_result = record_run(
                f"{leg_name}-original-main",
                [java, "-Xverify:all", "-cp", classes, "Main"],
                FIXTURE,
                run_root,
                commands,
            )
            javap_result = record_run(
                f"{leg_name}-javap-main",
                [javap, "-classpath", classes, "-c", "-p", "Main"],
                FIXTURE,
                run_root,
                commands,
            )
            method_instructions = parse_main_javap(javap_result.stdout.decode("utf-8", errors="replace"))
        legs[leg_name] = {
            "jdk_home": str(home),
            "tool_sha256": {tool.name: file_sha(tool) for tool in (javac, java, javap)},
            "version_stdout": {
                "javac": javac_version.stdout.decode("utf-8", errors="replace"),
                "java": java_version.stdout.decode("utf-8", errors="replace"),
                "javap": javap_version.stdout.decode("utf-8", errors="replace"),
            },
            "version_stderr": {
                "javac": javac_version.stderr.decode("utf-8", errors="replace"),
                "java": java_version.stderr.decode("utf-8", errors="replace"),
                "javap": javap_version.stderr.decode("utf-8", errors="replace"),
            },
            "compiler_flags": spec["flags"],
            "empty_classpath_sourcepath": str(empty_path),
            "compile_exit_code": compile_result.returncode,
            "runtime_exit_code": runtime_result.returncode if runtime_result else None,
            "javap_exit_code": javap_result.returncode if javap_result else None,
            "class_files": [
                {"path": str(path.relative_to(classes)), "bytes": path.stat().st_size, "sha256": file_sha(path)}
                for path in class_files
            ],
            "original_stdout_sha256": sha256(runtime_result.stdout) if runtime_result else None,
            "original_stderr_sha256": sha256(runtime_result.stderr) if runtime_result else None,
            "javap_stdout_sha256": sha256(javap_result.stdout) if javap_result else None,
            "javap_stderr_sha256": sha256(javap_result.stderr) if javap_result else None,
            "target_method_opcodes_bci": method_instructions,
        }

    successful = all(
        leg["compile_exit_code"] == 0
        and leg["runtime_exit_code"] == 0
        and leg["javap_exit_code"] == 0
        and len(leg["class_files"]) == len(SOURCES)
        and set(leg["target_method_opcodes_bci"]) == set(METHODS)
        and all(
            all(
                len(leg["target_method_opcodes_bci"][method][opcode]) == count
                for opcode, count in expected.items()
            )
            for method in METHODS
            for expected in [EXPECTED_OPCODE_COUNTS[method]]
        )
        for leg in legs.values()
    )
    streams_match = (
        legs["javac8"]["original_stdout_sha256"] == legs["javac23"]["original_stdout_sha256"]
        and legs["javac8"]["original_stderr_sha256"] == legs["javac23"]["original_stderr_sha256"]
    )
    manifest = {
        "schema": "constructed-reference-array-elements-fixture-v1",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "source_plan": str((FIXTURE / "source-plan-v1.md").relative_to(REPO)),
        "source_plan_sha256": file_sha(FIXTURE / "source-plan-v1.md"),
        "runner": str(pathlib.Path(__file__).resolve().relative_to(REPO)),
        "runner_sha256": file_sha(pathlib.Path(__file__).resolve()),
        "run_id": run_id,
        "fixture_sources": source_records,
        "expected_target_method_opcode_counts": EXPECTED_OPCODE_COUNTS,
        "jdk_legs": legs,
        "commands": commands,
        "checks": {
            "all_classes_compiled": all(leg["compile_exit_code"] == 0 and len(leg["class_files"]) == len(SOURCES) for leg in legs.values()),
            "all_originals_verified_and_ran": all(leg["runtime_exit_code"] == 0 for leg in legs.values()),
            "all_target_method_opcode_shapes_match_recorded_counts": successful,
            "original_stdout_stderr_match_across_jdks": streams_match,
        },
    }
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({
        "manifest": str(manifest_path),
        "manifest_sha256": file_sha(manifest_path),
        "runner_sha256": manifest["runner_sha256"],
        "checks": manifest["checks"],
        "legs": {
            name: {
                "compile_exit": leg["compile_exit_code"],
                "run_exit": leg["runtime_exit_code"],
                "classes": len(leg["class_files"]),
                "stdout_sha256": leg["original_stdout_sha256"],
                "stderr_sha256": leg["original_stderr_sha256"],
            }
            for name, leg in legs.items()
        },
    }, indent=2))
    return 0 if successful and streams_match else 1


if __name__ == "__main__":
    raise SystemExit(main())
