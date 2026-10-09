#!/usr/bin/env python3
"""Build and record the isolated handler-range fixture with two Java 8 compilers."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[5]
CHANGE = ROOT / "openspec/changes/compose-constructed-reference-array-elements"
RESULTS = CHANGE / "results/handler-fixture-v1"
FIXTURE = ROOT / "tests/fixtures/p3-constructed-reference-array-handler-v1"
SOURCE = FIXTURE / "HandlerControls.java"
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


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_digest(path: pathlib.Path) -> str:
    return digest(path.read_bytes())


def run(label: str, argv: list[pathlib.Path | str], cwd: pathlib.Path,
        commands: list[dict]) -> subprocess.CompletedProcess[bytes]:
    actual_argv = [str(value) for value in argv]
    result = subprocess.run(actual_argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout_rel = pathlib.Path("logs") / f"{label}.stdout"
    stderr_rel = pathlib.Path("logs") / f"{label}.stderr"
    (RESULTS / stdout_rel).write_bytes(result.stdout)
    (RESULTS / stderr_rel).write_bytes(result.stderr)
    commands.append({
        "label": label,
        "argv": actual_argv,
        "cwd": str(cwd),
        "exit_code": result.returncode,
        "stdout_path": str(stdout_rel),
        "stdout_bytes": len(result.stdout),
        "stdout_sha256": digest(result.stdout),
        "stderr_path": str(stderr_rel),
        "stderr_bytes": len(result.stderr),
        "stderr_sha256": digest(result.stderr),
    })
    return result


def method_instructions(text: str, name: str) -> list[dict]:
    lines = text.splitlines()
    start = None
    for index, line in enumerate(lines):
        if line.strip() == f"static java.lang.Object[] {name}();":
            start = index
            break
    if start is None:
        raise RuntimeError(f"javap output has no {name}() method")
    code = next((i for i in range(start, len(lines)) if lines[i].strip() == "Code:"), None)
    if code is None:
        raise RuntimeError(f"javap output has no Code section for {name}()")
    instructions = []
    handlers = []
    instruction_pattern = re.compile(r"^\s*(\d+):\s+([a-z][a-z0-9_]*)\b(.*)$")
    handler_pattern = re.compile(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|Class\s+\S+)\s*$")
    in_exception_table = False
    for line in lines[code + 1:]:
        stripped = line.strip()
        if stripped == "LineNumberTable:" or stripped == "LocalVariableTable:" or stripped == "StackMapTable:":
            break
        if stripped == "Exception table:":
            in_exception_table = True
            continue
        if in_exception_table:
            if stripped.startswith("from"):
                continue
            match = handler_pattern.match(line)
            if match:
                handlers.append({
                    "start_pc": int(match.group(1)),
                    "end_pc": int(match.group(2)),
                    "handler_pc": int(match.group(3)),
                    "catch_type": match.group(4),
                })
                continue
            if stripped:
                in_exception_table = False
        match = instruction_pattern.match(line)
        if match:
            instructions.append({
                "bci": int(match.group(1)),
                "opcode": match.group(2),
                "operands": match.group(3).strip(),
            })
    if not instructions or not handlers:
        raise RuntimeError(f"javap did not expose handled() instructions and exception table: {instructions}, {handlers}")
    return {"instructions": instructions, "exception_table": handlers}


def main() -> int:
    if not SOURCE.is_file():
        raise SystemExit(f"fixture source is missing: {SOURCE}")
    allowed_empty = {pathlib.Path(__file__).name, "empty-classpath-sourcepath"}
    existing = [path for path in RESULTS.iterdir() if path.name not in allowed_empty] \
        if RESULTS.exists() else []
    if existing:
        raise SystemExit(f"refusing to overwrite existing result artifacts: {existing}")
    RESULTS.mkdir(parents=True, exist_ok=True)
    empty = RESULTS / "empty-classpath-sourcepath"
    empty.mkdir(exist_ok=True)
    (RESULTS / "logs").mkdir(exist_ok=True)
    source_before = file_digest(SOURCE)
    commands: list[dict] = []
    legs: dict[str, dict] = {}

    for leg, spec in JDKS.items():
        home: pathlib.Path = spec["home"]
        javac, javap = home / "bin/javac", home / "bin/javap"
        if not javac.is_file() or not javap.is_file():
            raise SystemExit(f"required compiler tools are missing under {home}")
        javac_version = run(f"{leg}-javac-version", [javac, "-version"], ROOT, commands)
        javap_version = run(f"{leg}-javap-version", [javap, "-version"], ROOT, commands)
        output = RESULTS / leg / "classes"
        output.mkdir(parents=True)
        compile_result = run(
            f"{leg}-compile",
            [javac, *spec["flags"], "-g:none", "-classpath", empty,
             "-sourcepath", empty, "-d", output, SOURCE],
            ROOT,
            commands,
        )
        classes = sorted(output.rglob("*.class"))
        if compile_result.returncode != 0 or [path.name for path in classes] != ["HandlerControls.class"]:
            raise RuntimeError(f"{leg} did not produce exactly one HandlerControls.class")
        javap_result = run(
            f"{leg}-javap",
            [javap, "-classpath", output, "-c", "-p", "-v", "HandlerControls"],
            ROOT,
            commands,
        )
        if javap_result.returncode != 0:
            raise RuntimeError(f"{leg} javap failed with exit {javap_result.returncode}")
        parsed = method_instructions(javap_result.stdout.decode("utf-8", errors="replace"), "handled")
        class_copy = FIXTURE / leg / "HandlerControls.class"
        class_copy.parent.mkdir(parents=True, exist_ok=True)
        if class_copy.exists():
            raise SystemExit(f"refusing to overwrite frozen class copy: {class_copy}")
        class_copy.write_bytes(classes[0].read_bytes())
        legs[leg] = {
            "jdk_home": str(home),
            "flags": spec["flags"],
            "javac_sha256": file_digest(javac),
            "javap_sha256": file_digest(javap),
            "javac_version_stdout": javac_version.stdout.decode("utf-8", errors="replace"),
            "javac_version_stderr": javac_version.stderr.decode("utf-8", errors="replace"),
            "javap_version_stdout": javap_version.stdout.decode("utf-8", errors="replace"),
            "javap_version_stderr": javap_version.stderr.decode("utf-8", errors="replace"),
            "compile_exit_code": compile_result.returncode,
            "javap_exit_code": javap_result.returncode,
            "class_path": str(classes[0].relative_to(RESULTS)),
            "class_bytes": classes[0].stat().st_size,
            "class_sha256": file_digest(classes[0]),
            "fixture_copy_path": str(class_copy.relative_to(ROOT)),
            "fixture_copy_sha256": file_digest(class_copy),
            "handled_code_and_handlers": parsed,
        }

    manifest = {
        "schema": "constructed-reference-array-handler-fixture-v1",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "runner_path": str(pathlib.Path(__file__).resolve()),
        "runner_sha256": file_digest(pathlib.Path(__file__).resolve()),
        "source_path": str(SOURCE.relative_to(ROOT)),
        "source_bytes": SOURCE.stat().st_size,
        "source_sha256_before": source_before,
        "source_sha256_after": file_digest(SOURCE),
        "source_contract": "one source class; uses only java.lang types; no main; handled() has one RuntimeException handler; no runtime command is issued",
        "empty_classpath_sourcepath": str(empty),
        "legs": legs,
        "commands": commands,
        "scope_note": "only javac/javap were executed; no java runtime, Cargo, Git, or formatter was run",
    }
    manifest_path = RESULTS / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({
        "manifest": str(manifest_path.relative_to(ROOT)),
        "manifest_sha256": file_digest(manifest_path),
        "runner_sha256": manifest["runner_sha256"],
        "source_sha256": manifest["source_sha256_after"],
        "legs": {name: {
            "compile_exit_code": leg["compile_exit_code"],
            "javap_exit_code": leg["javap_exit_code"],
            "class_sha256": leg["class_sha256"],
            "handled": leg["handled_code_and_handlers"],
        } for name, leg in legs.items()},
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
