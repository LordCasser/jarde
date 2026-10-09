#!/usr/bin/env python3
"""Draft-only dual-javac compile/javap recorder; intentionally not executed in this task."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent
SOURCE_FILES = {
    "nested": [("NestedControls", ROOT / "NestedControls.java")],
    "boundary": [("BoundaryControls", ROOT / "BoundaryControls.java")],
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


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_digest(path: pathlib.Path) -> str:
    return digest(path.read_bytes())


def save(path: pathlib.Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def command(label: str, argv: list[pathlib.Path | str], cwd: pathlib.Path,
            run_dir: pathlib.Path, commands: list[dict]) -> subprocess.CompletedProcess:
    actual_argv = [str(value) for value in argv]
    result = subprocess.run(actual_argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout = run_dir / "logs" / f"{label}.stdout"
    stderr = run_dir / "logs" / f"{label}.stderr"
    save(stdout, result.stdout)
    save(stderr, result.stderr)
    commands.append({
        "label": label,
        "argv": actual_argv,
        "cwd": str(cwd),
        "exit_code": result.returncode,
        "stdout_path": str(stdout.relative_to(run_dir)),
        "stdout_bytes": len(result.stdout),
        "stdout_sha256": digest(result.stdout),
        "stderr_path": str(stderr.relative_to(run_dir)),
        "stderr_bytes": len(result.stderr),
        "stderr_sha256": digest(result.stderr),
    })
    return result


def instruction_sites(text: str) -> dict[str, dict[str, list[int]]]:
    methods: dict[str, dict[str, list[int]]] = {}
    current: str | None = None
    method_re = re.compile(r"^\s*(?:public|private|protected|static).*[\s.]([A-Za-z_$][\w$]*)\([^)]*\);$")
    instruction_re = re.compile(r"^\s*(\d+):\s+(new|dup|invokespecial|aastore)\b")
    for line in text.splitlines():
        found_method = method_re.match(line)
        if found_method:
            current = found_method.group(1)
            methods.setdefault(current, {name: [] for name in ("new", "dup", "invokespecial", "aastore")})
            continue
        if current is None:
            continue
        found_instruction = instruction_re.match(line)
        if found_instruction:
            methods[current][found_instruction.group(2)].append(int(found_instruction.group(1)))
    return methods


def main() -> int:
    run_id = sys.argv[1] if len(sys.argv) > 1 else "run-001"
    if not re.fullmatch(r"run-[0-9]{3}", run_id):
        raise SystemExit("run id must match run-NNN")
    run_dir = ROOT / run_id
    if run_dir.exists():
        raise SystemExit(f"refusing to overwrite existing control record: {run_dir}")
    run_dir.mkdir(parents=True)
    empty = run_dir / "empty-classpath-sourcepath"
    empty.mkdir()
    commands: list[dict] = []
    leg_records: dict[str, dict] = {}

    source_records = {
        family: [{"path": path.name, "bytes": path.stat().st_size, "sha256": file_digest(path)}
                 for _, path in sources]
        for family, sources in SOURCE_FILES.items()
    }

    for leg, spec in JDKS.items():
        home: pathlib.Path = spec["home"]
        javac, javap = home / "bin/javac", home / "bin/javap"
        if not javac.is_file() or not javap.is_file():
            raise SystemExit(f"required tools missing under {home}")
        javac_version = command(f"{leg}-javac-version", [javac, "-version"], ROOT, run_dir, commands)
        javap_version = command(f"{leg}-javap-version", [javap, "-version"], ROOT, run_dir, commands)
        family_records = {}
        for family, sources in SOURCE_FILES.items():
            output = run_dir / leg / family / "classes"
            output.mkdir(parents=True)
            compile_argv = [javac, *spec["flags"], "-g:none", "-classpath", empty,
                            "-sourcepath", empty, "-d", output, *[path for _, path in sources]]
            compile_result = command(f"{leg}-{family}-compile", compile_argv, ROOT, run_dir, commands)
            classes = sorted(output.rglob("*.class"))
            javap_result = None
            parsed = {}
            if compile_result.returncode == 0:
                class_name = sources[0][0]
                javap_result = command(
                    f"{leg}-{family}-javap",
                    [javap, "-classpath", output, "-c", "-p", class_name],
                    ROOT,
                    run_dir,
                    commands,
                )
                parsed = instruction_sites(javap_result.stdout.decode("utf-8", errors="replace"))
            family_records[family] = {
                "sources": source_records[family],
                "compiler_flags": spec["flags"],
                "empty_classpath_sourcepath": str(empty),
                "compile_exit_code": compile_result.returncode,
                "javap_exit_code": javap_result.returncode if javap_result else None,
                "classes": [
                    {"path": str(path.relative_to(output)), "bytes": path.stat().st_size,
                     "sha256": file_digest(path)}
                    for path in classes
                ],
                "javap_instruction_bci": parsed,
            }
        leg_records[leg] = {
            "jdk_home": str(home),
            "tool_sha256": {"javac": file_digest(javac), "javap": file_digest(javap)},
            "version_stdout": {"javac": javac_version.stdout.decode(errors="replace"),
                               "javap": javap_version.stdout.decode(errors="replace")},
            "version_stderr": {"javac": javac_version.stderr.decode(errors="replace"),
                               "javap": javap_version.stderr.decode(errors="replace")},
            "families": family_records,
        }

    manifest = {
        "schema": "constructed-reference-array-control-drafts-v1",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "generated only by running this script; not executed when draft was prepared",
        "runner": str(pathlib.Path(__file__).resolve()),
        "runner_sha256": file_digest(pathlib.Path(__file__).resolve()),
        "jdk_legs": leg_records,
        "commands": commands,
        "scope_note": "nested and boundary sources compile into separate class directories; no Java runtime command is issued",
    }
    manifest_path = run_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({
        "manifest": str(manifest_path),
        "manifest_sha256": file_digest(manifest_path),
        "runner_sha256": manifest["runner_sha256"],
        "compile_exit_codes": {
            leg: {family: data["compile_exit_code"] for family, data in record["families"].items()}
            for leg, record in leg_records.items()
        },
    }, indent=2))
    return 0 if all(
        data["compile_exit_code"] == 0 and data["javap_exit_code"] == 0
        for record in leg_records.values()
        for data in record["families"].values()
    ) else 1


if __name__ == "__main__":
    raise SystemExit(main())
