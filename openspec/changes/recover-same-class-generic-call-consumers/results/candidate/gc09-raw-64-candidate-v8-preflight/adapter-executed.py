#!/usr/bin/env python3
"""Replay the frozen raw-receiver runner with complete subprocess evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import time


HERE = Path(__file__).resolve().parent
RUNNER = (
    HERE.parents[2]
    / "evidence"
    / "raw-receiver-field-selection-2026-10-09"
    / "replay.py"
).resolve()
DEFAULT_JADX = Path(
    "/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx"
)
ACTIVE_PREFLIGHT: Path | None = None
ACTIVE_OUTPUT: Path | None = None
ACTIVE_RUN_METADATA: dict[str, object] | None = None


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def write_result_hashes(output: Path) -> None:
    index = output / "result-file-hashes.tsv"
    with index.open("w") as stream:
        stream.write("path\tsha256\n")
        for path in sorted(output.rglob("*")):
            if path.is_file() and path != index:
                stream.write(f"{path.relative_to(output)}\t{sha256_file(path)}\n")


def capture(
    argv: list[str],
    *,
    cwd: str,
    log_root: Path,
    index: int,
    scope: str,
) -> tuple[subprocess.CompletedProcess[bytes] | None, dict[str, object]]:
    """Run one process and preserve both streams, including empty streams."""
    stem = f"{index:04d}-{scope.replace('/', '_').replace(' ', '_')}"
    stdout_path = log_root / f"{stem}.stdout"
    stderr_path = log_root / f"{stem}.stderr"
    record: dict[str, object] = {
        "index": index,
        "scope": scope,
        "argv": argv,
        "cwd": cwd,
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
    }
    try:
        completed = subprocess.run(
            argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=cwd, check=False
        )
    except BaseException as error:
        record["exception"] = f"{type(error).__name__}: {error}"
        record["exit_code"] = None
        log_root.mkdir(parents=True, exist_ok=True)
        stdout_path.write_bytes(b"")
        stderr_path.write_bytes((record["exception"] + "\n").encode())
        record["stdout_sha256"] = sha256_bytes(b"")
        record["stderr_sha256"] = sha256_file(stderr_path)
        return None, record
    log_root.mkdir(parents=True, exist_ok=True)
    stdout_path.write_bytes(completed.stdout)
    stderr_path.write_bytes(completed.stderr)
    record["exit_code"] = completed.returncode
    record["stdout_sha256"] = sha256_bytes(completed.stdout)
    record["stderr_sha256"] = sha256_bytes(completed.stderr)
    return completed, record


def fixture_hashes(root: Path) -> dict[str, object]:
    source_root = root / "source"
    repo = root.parents[2]
    fixture_root = repo / "tests" / "fixtures" / "raw-receiver-field-selection"
    runner_sources = sorted(source_root.glob("*.java"))
    fixture_sources = sorted(fixture_root.glob("*.java"))
    input_jars = sorted(root.glob("jdk*/*/input/*.jar"))
    if len(runner_sources) != 18 or len(fixture_sources) != 23 or len(input_jars) != 64:
        raise RuntimeError(
            "raw64 frozen inventory mismatch: "
            f"runner_sources={len(runner_sources)}, "
            f"fixture_sources={len(fixture_sources)}, input_jars={len(input_jars)}"
        )
    paths = runner_sources + fixture_sources + input_jars
    return {
        "counts": {
            "runner_source_files": len(runner_sources),
            "fixture_source_files": len(fixture_sources),
            "frozen_input_jars": len(input_jars),
        },
        "sha256": {str(path): sha256_file(path) for path in paths},
    }


def tool_hashes(cli: Path, jadx: Path, jdk_roots: dict[str, Path]) -> dict[str, object]:
    tools: list[dict[str, str]] = []
    for label, path in [("jarde-cli", cli), ("jadx-cli", jadx)]:
        resolved = path.resolve(strict=True)
        tools.append(
            {"tool": label, "path": str(resolved), "sha256": sha256_file(resolved)}
        )
    lib_dir = jadx.resolve(strict=True).parent.parent / "lib"
    for jar in sorted(lib_dir.glob("*.jar")):
        tools.append(
            {"tool": f"jadx-lib/{jar.name}", "path": str(jar.resolve()), "sha256": sha256_file(jar)}
        )
    jdk_tool_specs: list[tuple[str, str, Path, list[str]]] = []
    jdk_release_facts: list[dict[str, str]] = []
    for jdk_name, jdk_root in jdk_roots.items():
        release_file = jdk_root / "release"
        release_text = release_file.read_text(errors="replace") if release_file.is_file() else ""
        release_values = {}
        for line in release_text.splitlines():
            key, separator, value = line.partition("=")
            if separator:
                release_values[key] = value.strip('"')
        package_file = jdk_root.parent / "Info.plist"
        package_values = (
            plistlib.loads(package_file.read_bytes()) if package_file.is_file() else {}
        )
        jdk_release_facts.append(
            {
                "jdk": jdk_name,
                "root": str(jdk_root.resolve(strict=True)),
                "release_file": str(release_file.resolve()) if release_file.is_file() else "absent",
                "release_file_sha256": sha256_file(release_file) if release_file.is_file() else "absent",
                "package_file": str(package_file.resolve()) if package_file.is_file() else "absent",
                "package_file_sha256": sha256_file(package_file) if package_file.is_file() else "absent",
                "java_version": release_values.get(
                    "JAVA_VERSION", str(package_values.get("JVMVersion", "unknown"))
                ),
                "full_version": release_values.get(
                    "JAVA_RUNTIME_VERSION",
                    str(package_values.get("CFBundleGetInfoString", "unknown")),
                ),
                "release_text": release_text,
                "package_version": str(
                    package_values.get("CFBundleShortVersionString", "unknown")
                ),
            }
        )
        for tool_name, version_args in (
            ("java", ["-version"]),
            ("javac", ["-version"]),
            ("jar", ["-J-version"] if jdk_name == "jdk8" else ["--version"]),
            ("javap", ["-version"]),
        ):
            path = (jdk_root / "bin" / tool_name).resolve(strict=True)
            jdk_tool_specs.append((jdk_name, tool_name, path, version_args))
    jdk_tools = [
        {
            "jdk": jdk_name,
            "tool": tool_name,
            "path": str(path),
            "sha256": sha256_file(path),
            "version_args": version_args,
        }
        for jdk_name, tool_name, path, version_args in jdk_tool_specs
    ]
    index = 1
    probe_root = Path(os.environ.get("GC09_PREFLIGHT_LOGS", "."))
    for fact, (jdk_name, tool_name, path, version_args) in zip(jdk_tools, jdk_tool_specs):
        completed, record = capture(
            [str(path), *version_args],
            cwd=str(Path.cwd()),
            log_root=probe_root,
            index=index,
            scope=f"jdk-preflight/{jdk_name}/{tool_name}",
        )
        index += 1
        fact.update(
            {
                "version_argv": record["argv"],
                "version_exit_code": record["exit_code"],
                "version_stdout": record["stdout"],
                "version_stderr": record["stderr"],
                "version_stdout_sha256": record["stdout_sha256"],
                "version_stderr_sha256": record["stderr_sha256"],
            }
        )
        if completed is None:
            fact["version_exception"] = record.get("exception", "unable to start")
    return {"tools": tools, "jdk_tools": jdk_tools, "jdk_release_facts": jdk_release_facts}


def main() -> int:
    global ACTIVE_OUTPUT, ACTIVE_PREFLIGHT, ACTIVE_RUN_METADATA
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--jadx", default=str(DEFAULT_JADX))
    parser.add_argument("--out", required=True)
    parser.add_argument("--label", required=True)
    args = parser.parse_args()

    output = Path(args.out).resolve()
    preflight = output.with_name(output.name + "-preflight")
    if output.exists() or preflight.exists():
        raise SystemExit("output or preflight path already exists; refusing to overwrite")
    preflight.mkdir(parents=True, exist_ok=False)
    ACTIVE_OUTPUT = output
    ACTIVE_PREFLIGHT = preflight
    ACTIVE_RUN_METADATA = {
        "scope": "GC09 raw-64 candidate-v2",
        "state": "preflight-started",
        "output": str(output),
        "output_was_absent_before_run": True,
        "started_unix_ns": time.time_ns(),
    }
    write_json(preflight / "outer-run-metadata.json", ACTIVE_RUN_METADATA)

    runner_bytes = RUNNER.read_bytes()
    adapter_bytes = Path(__file__).read_bytes()
    runner_sha = sha256_bytes(runner_bytes)
    adapter_sha = sha256_bytes(adapter_bytes)
    (preflight / "replay-executed.py").write_bytes(runner_bytes)
    (preflight / "adapter-executed.py").write_bytes(adapter_bytes)
    cli = Path(args.cli).resolve(strict=True)
    jadx = Path(args.jadx).resolve(strict=True)
    if sha256_file(cli) != args.cli_sha256:
        raise SystemExit("candidate CLI SHA-256 does not match --cli-sha256")

    namespace: dict[str, object] = {
        "__name__": "_gc09_raw64_frozen_runner",
        "__file__": str(RUNNER),
        "__package__": None,
    }
    exec(compile(runner_bytes, str(RUNNER), "exec"), namespace)
    jdk_roots = {
        name: value[0]
        for name, value in namespace["JDKS"].items()  # type: ignore[union-attr]
    }
    source_hashes = fixture_hashes(RUNNER.parent)
    preflight_records: list[dict[str, object]] = []
    os.environ["GC09_PREFLIGHT_LOGS"] = str(preflight / "jdk-version-logs")
    tools = tool_hashes(cli, jadx, jdk_roots)
    del os.environ["GC09_PREFLIGHT_LOGS"]

    preflight_metadata: dict[str, object] = {
        "scope": "GC09 raw-64 candidate-v2",
        "output": str(output),
        "output_was_absent_before_run": True,
        "label": args.label,
        "adapter": {"path": str(Path(__file__).resolve()), "sha256": adapter_sha},
        "runner": {
            "path": str(RUNNER),
            "sha256_before_run": runner_sha,
            "snapshot": str(preflight / "replay-executed.py"),
            "snapshot_sha256": sha256_file(preflight / "replay-executed.py"),
            "execution___file__": str(RUNNER),
        },
        "cli": {"path": str(cli), "sha256": sha256_file(cli)},
        "jadx": {"path": str(jadx), "sha256": sha256_file(jadx)},
        "tools": tools,
        "source_and_input_hashes_before_run": source_hashes,
        "cwd": str(Path.cwd()),
        "started_unix_ns": time.time_ns(),
    }
    write_json(preflight / "outer-preflight-metadata.json", preflight_metadata)
    failed_jdk_probes = [
        tool
        for tool in tools["jdk_tools"]
        if tool["version_exit_code"] != 0
    ]
    if failed_jdk_probes:
        raise RuntimeError(f"JDK version preflight failed: {failed_jdk_probes}")

    invocation = [
        str(RUNNER),
        "--cli",
        str(cli),
        "--cli-sha256",
        args.cli_sha256,
        "--jadx",
        str(jadx),
        "--label",
        args.label,
        "--out",
        str(output),
    ]
    runner_output: dict[str, object] = {
        "argv": invocation,
        "cwd": str(Path.cwd()),
        "started_unix_ns": time.time_ns(),
        "runner_sha256": runner_sha,
        "adapter_sha256": adapter_sha,
        "execution___file__": str(RUNNER),
    }
    ACTIVE_RUN_METADATA = runner_output
    write_json(preflight / "outer-run-metadata.json", runner_output)

    command_records: list[dict[str, object]] = []
    original_record = namespace["record"]
    command_index = 1

    def captured_run(scope: str, command: list[object], stdout_path: object = None):
        nonlocal command_index
        argv = [str(item) for item in command]
        completed, record = capture(
            argv,
            cwd=str(Path.cwd()),
            log_root=output / "subprocess-logs",
            index=command_index,
            scope=scope,
        )
        command_index += 1
        command_records.append(record)
        if completed is None:
            raise RuntimeError(str(record.get("exception", "subprocess failed to start")))
        return original_record(scope, command, completed, stdout_path)

    namespace["run"] = captured_run
    old_argv = sys.argv
    sys.argv = invocation
    status = 0
    try:
        namespace["main"]()
    except BaseException as error:
        status = error.code if isinstance(error, SystemExit) and isinstance(error.code, int) else 1
        runner_output["exception"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        sys.argv = old_argv
        runner_output["exit_code"] = status
        runner_output["finished_unix_ns"] = time.time_ns()
        runner_output["subprocess_count"] = len(command_records)
        runner_output["subprocess_records"] = command_records
        runner_output["source_and_input_hashes_after_run"] = fixture_hashes(RUNNER.parent)
        runner_output["source_and_input_hashes_unchanged"] = (
            runner_output["source_and_input_hashes_after_run"] == source_hashes
        )
        actual_runner_hash = sha256_file(RUNNER)
        runner_output["runner_sha256_after_run"] = actual_runner_hash
        runner_output["runner_source_unchanged"] = actual_runner_hash == runner_sha
        if not runner_output["source_and_input_hashes_unchanged"]:
            status = 1
            runner_output["integrity_failure"] = "frozen source or input jar changed during replay"
        if not runner_output["runner_source_unchanged"]:
            status = 1
            runner_output["integrity_failure"] = "historical runner changed during replay"
        runner_output["exit_code"] = status
        write_json(preflight / "outer-run-metadata.json", runner_output)
        if output.exists():
            shutil.copy2(preflight / "replay-executed.py", output / "replay-executed.py")
            shutil.copy2(preflight / "adapter-executed.py", output / "adapter-executed.py")
            shutil.copy2(
                preflight / "outer-preflight-metadata.json",
                output / "outer-preflight-metadata.json",
            )
            shutil.copy2(preflight / "outer-run-metadata.json", output / "outer-run-metadata.json")
            shutil.copytree(
                preflight / "jdk-version-logs",
                output / "jdk-version-logs",
                dirs_exist_ok=True,
            )
            if command_records:
                with (output / "subprocesses.jsonl").open("w") as stream:
                    for item in command_records:
                        stream.write(json.dumps(item, sort_keys=True) + "\n")
            runner_output["result_file_hashes"] = {
                str(path.relative_to(output)): sha256_file(path)
                for path in sorted(output.rglob("*"))
                if path.is_file()
                and path.name not in {"result-file-hashes.tsv", "outer-run-metadata.json"}
            }
            write_json(output / "outer-run-metadata.json", runner_output)
            write_result_hashes(output)

    if status != 0:
        return status
    return 0


if __name__ == "__main__":
    try:
        exit_code = main()
    except BaseException as error:
        if ACTIVE_PREFLIGHT is not None:
            metadata = ACTIVE_RUN_METADATA or {
                "scope": "GC09 raw-64 candidate-v2",
                "output": str(ACTIVE_OUTPUT),
                "started_unix_ns": time.time_ns(),
            }
            metadata["outer_exception"] = f"{type(error).__name__}: {error}"
            metadata["finished_unix_ns"] = time.time_ns()
            write_json(ACTIVE_PREFLIGHT / "outer-run-metadata.json", metadata)
            if ACTIVE_OUTPUT is not None and ACTIVE_OUTPUT.exists():
                shutil.copy2(
                    ACTIVE_PREFLIGHT / "outer-run-metadata.json",
                    ACTIVE_OUTPUT / "outer-run-metadata.json",
                )
                write_result_hashes(ACTIVE_OUTPUT)
        raise
    raise SystemExit(exit_code)
