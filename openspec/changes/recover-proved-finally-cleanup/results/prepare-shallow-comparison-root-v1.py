#!/usr/bin/env python3
"""Compare the existing depth-02 finally/loop input with its saved Jarde text and fresh JADX."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
INPUT = RESULTS / "depth-boundary-root-v1/run"
INPUT_RESULT = INPUT / "result.json"
OUTPUT = RESULTS / "shallow-comparison-root-v1"
CLI_META = ROOT / "openspec/changes/recover-returned-int-array-compound-updates/results/candidate-cli-v2.json"
JADX = Path("/opt/homebrew/bin/jadx")
RESULT_SHA256 = "bb7c73bf82a51d4a31f3ebf70005db2c991f27e7fc7e38b8e1fef707c1d359cc"
CLI_META_SHA256 = "ddfbe127890d95ced5c4ed2a89d67d42deb7eba19e6c1e4ce12707221bd8f769"
CLI_SHA256 = "71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110"
SOURCE_SHA256 = "da229c6bc912141e0de22606c5e3425ab2ece3e93848ac0d74b2d89e2fdc417e"
RUNNER_SHA256 = "e6e8faa21f6cb8fdfa9e978e20521f1e3a06978b8f813b78da62d611420c6021"
CLI_TEXT_SHA256 = "3870102f7ad25490d504ed282c7544741b9445cbefef97b59beb4c678422b7dd"
EXPECTED_STDOUT = b"run(0)=0 trace=1\nrun(40)=1 trace=2\n"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def record(path: Path, base: Path | None = None) -> dict:
    data = path.read_bytes()
    try:
        shown = path.relative_to(base).as_posix() if base is not None else str(path)
    except ValueError:
        shown = str(path)
    return {"path": shown, "bytes": len(data), "sha256": sha(data)}


def row_path(row: dict, base: Path) -> Path:
    path = Path(row["path"])
    return path if path.is_absolute() else base / path


def verify_row(row: dict, base: Path, label: str) -> Path:
    path = row_path(row, base)
    if not path.is_file():
        raise RuntimeError(f"{label} is missing: {path}")
    actual = record(path)
    if actual["bytes"] != row["bytes"] or actual["sha256"] != row["sha256"]:
        raise RuntimeError(f"{label} differs from its recorded identity: {actual}")
    return path


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def clean_env(home: Path) -> dict[str, str]:
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    env["JAVA_HOME"] = str(home)
    env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    return env


def run(label: str, argv: list[object], cwd: Path, home: Path) -> tuple[subprocess.CompletedProcess, dict]:
    result = subprocess.run([str(value) for value in argv], cwd=cwd, env=clean_env(home),
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    raw = OUTPUT / "raw"
    raw.mkdir(parents=True, exist_ok=True)
    streams = {}
    for name, contents in (("stdout", result.stdout), ("stderr", result.stderr)):
        path = raw / f"{label}.{name}"
        path.write_bytes(contents)
        streams[name] = record(path, OUTPUT)
    command = {"label": label, "argv": [str(value) for value in argv], "cwd": str(cwd),
               "exit": result.returncode, **streams}
    return result, command


def package_of(source: Path) -> str | None:
    match = re.search(r"^package ([\w.]+);", source.read_text(encoding="utf-8"), re.M)
    return match.group(1) if match else None


def compile_candidate(label: str, source_files: list[Path], runner_name: str,
                      tools: dict[str, Path], home: Path) -> tuple[dict, subprocess.CompletedProcess | None]:
    case = OUTPUT / "cases" / label
    empty = case / "empty-classpath-sourcepath"
    classes = case / "classes"
    empty.mkdir()
    classes.mkdir()
    compile_result, compile_command = run(
        label + "-compile",
        [tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
         "-classpath", empty, "-sourcepath", empty, "-d", classes, *source_files],
        case, home)
    run_result, run_command = None, None
    if compile_result.returncode == 0:
        run_result, run_command = run(label + "-verify-run",
            [tools["java"], "-Xverify:all", "-cp", classes, runner_name], case, home)
    return ({"compile": compile_command, "runtime": run_command,
             "sources": [record(path, OUTPUT) for path in source_files],
             "classes": [record(path, OUTPUT) for path in sorted(classes.rglob("*.class"))],
             "compile_success": compile_result.returncode == 0,
             "runtime_exit": run_result.returncode if run_result is not None else None}, run_result)


def raw_matches(candidate: subprocess.CompletedProcess | None, original: dict[str, bytes | int]) -> bool:
    return bool(candidate is not None
        and candidate.returncode == original["exit"]
        and candidate.stdout == original["stdout"]
        and candidate.stderr == original["stderr"])


def main() -> int:
    if OUTPUT.exists():
        raise SystemExit(f"refusing to overwrite existing comparison: {OUTPUT}")
    if sha(INPUT_RESULT.read_bytes()) != RESULT_SHA256:
        raise SystemExit("depth-boundary result.json identity changed")
    if sha(CLI_META.read_bytes()) != CLI_META_SHA256:
        raise SystemExit("frozen CLI metadata identity changed")
    result = json.loads(INPUT_RESULT.read_text(encoding="utf-8"))
    if result.get("schema") != "cf16-depth-boundary-root-v1" or result.get("status") != "prepared-observations-complete":
        raise SystemExit("unexpected depth-boundary result schema or status")
    cli_meta = json.loads(CLI_META.read_text(encoding="utf-8"))
    if result["cli"]["path"] != cli_meta["cli_path"] or result["cli"]["sha256"] != CLI_SHA256:
        raise SystemExit("result.json and frozen CLI metadata disagree")
    cli_path = Path(cli_meta["cli_path"])
    if not cli_path.is_file() or sha(cli_path.read_bytes()) != CLI_SHA256:
        raise SystemExit("frozen CLI binary identity changed")
    for relative, expected in cli_meta["candidate_sources"].items():
        path = ROOT / relative
        if not path.is_file() or sha(path.read_bytes()) != expected:
            raise SystemExit(f"frozen CLI source identity changed: {relative}")

    jdk_tools = result["jdk_tools"]
    if set(jdk_tools) != {"javac8", "javac23"}:
        raise SystemExit("result.json must contain javac8 and javac23")
    tools_by_leg = {}
    for leg, tool_rows in jdk_tools.items():
        tools = {}
        for name in ("java", "javac", "javap"):
            row = tool_rows[name]
            path = Path(row["path"])
            if not path.is_file() or path.stat().st_size != row["bytes"] or sha(path.read_bytes()) != row["sha256"]:
                raise SystemExit(f"{leg} {name} binary identity changed: {path}")
            tools[name] = path
        tools_by_leg[leg] = tools

    original_records = {entry["leg"]: entry for entry in result["classes"] if entry["depth"] == 2}
    if set(original_records) != {"javac8", "javac23"}:
        raise SystemExit("result.json does not contain both depth-02 inputs")
    input_rows = {}
    runner_sources = {}
    original_sources = {}
    cli_sources = {}
    cli_documents = {}
    original_commands = {}
    originals = {}
    for leg in ("javac8", "javac23"):
        entry = original_records[leg]
        class_path = verify_row(entry["class_file"], INPUT, leg + " original class")
        runner_class_path = verify_row(entry["runner_class"], INPUT, leg + " Runner class")
        source_path = class_path.parent.parent / "source/DeepFinally.java"
        runner_path = class_path.parent.parent / "source/Runner.java"
        if sha(source_path.read_bytes()) != SOURCE_SHA256 or sha(runner_path.read_bytes()) != RUNNER_SHA256:
            raise SystemExit(f"{leg} depth-02 source or fixed Runner changed")
        source_row = entry["cli"]["class_source_text"]
        generated_path = verify_row(source_row, INPUT, leg + " saved Jarde class source")
        document_path = verify_row(entry["cli"]["document_json"], INPUT, leg + " saved Jarde document JSON")
        if source_row["sha256"] != CLI_TEXT_SHA256 or entry["cli"]["cli_text_executed_or_compiled"]:
            raise SystemExit(f"{leg} saved Jarde text identity/scope changed")
        if entry["oracle_stdout"]["sha256"] != "1258e7775af6157810f812a5fcd4611868948d545c7c1257b72b34a32a55f194":
            raise SystemExit(f"{leg} depth-02 original stdout record changed")
        run_label = f"{leg}-depth-02-original-verify-run"
        compile_label = f"{leg}-depth-02-original-compile"
        run_command = next((item for item in result["commands"] if item["label"] == run_label), None)
        compile_command = next((item for item in result["commands"] if item["label"] == compile_label), None)
        if run_command is None or compile_command is None or run_command["exit"] != 0 or compile_command["exit"] != 0:
            raise SystemExit(f"{leg} original compile/run record is missing or unsuccessful")
        raw = {}
        for command in (compile_command, run_command):
            command_raw = {}
            for stream in ("stdout", "stderr"):
                raw_row = command[stream]
                path = verify_row(raw_row, INPUT, command["label"] + " " + stream)
                command_raw[stream] = path.read_bytes()
            raw[command["label"]] = command_raw
        stdout_path = verify_row(run_command["stdout"], INPUT, run_label + " stdout")
        stderr_path = verify_row(run_command["stderr"], INPUT, run_label + " stderr")
        stdout, stderr = stdout_path.read_bytes(), stderr_path.read_bytes()
        if stdout != EXPECTED_STDOUT:
            raise SystemExit(f"{leg} original raw stdout differs from the required depth-02 oracle")
        originals[leg] = {"exit": run_command["exit"], "stdout": stdout, "stderr": stderr}
        original_sources[leg] = source_path
        runner_sources[leg] = runner_path
        cli_sources[leg] = generated_path
        cli_documents[leg] = document_path
        original_commands[leg] = {"compile": compile_command, "run": run_command, "raw": raw}
        input_rows[leg] = {"class": class_path, "runner_class": runner_class_path}

    if not OUTPUT.parent.is_dir():
        raise SystemExit(f"results directory is missing: {OUTPUT.parent}")
    OUTPUT.mkdir()
    (OUTPUT / "raw").mkdir()
    (OUTPUT / "inputs").mkdir()
    (OUTPUT / "cases").mkdir()
    jadx_home = tools_by_leg["javac23"]["java"].parent.parent
    version_result, jadx_version_command = run(
        "jadx-version", [JADX, "--version"], OUTPUT, jadx_home)
    jadx_version_text = (version_result.stdout + version_result.stderr).decode("utf-8", errors="replace")
    jadx_version_record = {"command": jadx_version_command, "reported": jadx_version_text.strip()}
    if version_result.returncode != 0 or version_result.stdout.strip() != b"1.5.6":
        raise SystemExit(f"expected JADX 1.5.6; recorded version output: {jadx_version_record}")
    source_inputs = {}
    oracle_rows = {}
    for leg in ("javac8", "javac23"):
        base = OUTPUT / "inputs" / leg
        (base / "classes").mkdir(parents=True)
        (base / "sources").mkdir()
        (base / "jarde").mkdir()
        copied_class = base / "classes/DeepFinally.class"
        copied_runner_class = base / "classes/Runner.class"
        shutil.copyfile(input_rows[leg]["class"], copied_class)
        shutil.copyfile(input_rows[leg]["runner_class"], copied_runner_class)
        copied_source = base / "sources/DeepFinally.java"
        copied_runner = base / "sources/Runner.java"
        shutil.copyfile(original_sources[leg], copied_source)
        shutil.copyfile(runner_sources[leg], copied_runner)
        copied_jarde = base / "jarde/DeepFinally.java"
        shutil.copyfile(cli_sources[leg], copied_jarde)
        copied_document = base / "jarde/document.json"
        shutil.copyfile(cli_documents[leg], copied_document)
        source_inputs[leg] = {
            "class": record(copied_class, OUTPUT), "runner_class": record(copied_runner_class, OUTPUT),
            "source": record(copied_source, OUTPUT), "runner": record(copied_runner, OUTPUT),
            "jarde_source": record(copied_jarde, OUTPUT),
            "jarde_document_json": record(copied_document, OUTPUT),
        }
        oracle_rows[leg] = {}
        for label, command in original_commands[leg].items():
            if label == "raw":
                continue
            oracle_rows[leg][label] = {key: value for key, value in command.items() if key != "_raw"}
        for command_label, streams in original_commands[leg]["raw"].items():
            stem = command_label.replace(f"{leg}-depth-02-", "")
            for stream, contents in streams.items():
                path = OUTPUT / "raw" / "oracle" / leg / f"{stem}.{stream}"
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(contents)
                oracle_rows[leg].setdefault("copied_raw", []).append(record(path, OUTPUT))

    commands = [jadx_version_command]
    cases = []
    for leg in ("javac8", "javac23"):
        tools = tools_by_leg[leg]
        home = tools["java"].parent.parent
        for kind in ("jarde", "jadx-default", "jadx-none"):
            label = leg + "-" + kind
            case_dir = OUTPUT / "cases" / label
            case_dir.mkdir()
            case = {"label": label, "leg": leg, "kind": kind, "success": False}
            if kind == "jarde":
                candidate_source = OUTPUT / "inputs" / leg / "jarde/DeepFinally.java"
                candidate_runner = OUTPUT / "inputs" / leg / "sources/Runner.java"
                compile_record, runtime = compile_candidate(label, [candidate_source, candidate_runner], "Runner", tools, home)
                case.update(compile_record)
                if runtime is not None:
                    case["success"] = raw_matches(runtime, originals[leg])
                    case["runtime_matches_original"] = case["success"]
                commands.append(case["compile"])
                if case["runtime"] is not None:
                    commands.append(case["runtime"])
                case["source_kind"] = "saved complete class-source.txt copied verbatim"
            else:
                original_class = input_rows[leg]["class"]
                dest = case_dir / "jadx"
                argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
                if kind == "jadx-none":
                    argv += ["--rename-flags", "none"]
                decompiled, decompile_command = run(label + "-decompile", [*argv, "-d", dest, original_class], case_dir,
                                                    tools_by_leg["javac23"]["java"].parent.parent)
                commands.append(decompile_command)
                case["decompile"] = decompile_command
                generated = sorted(dest.rglob("*.java")) if dest.exists() else []
                case["generated_sources"] = [record(path, OUTPUT) for path in generated]
                if decompiled.returncode == 0:
                    packages = {package_of(path) for path in generated}
                    if len(packages) <= 1:
                        package = next(iter(packages), None)
                        runner_text = runner_sources[leg].read_text(encoding="utf-8")
                        copied_runner = case_dir / "Runner.java"
                        copied_runner.write_text((f"package {package};\n\n" if package else "") + runner_text,
                                                  encoding="utf-8")
                        runner_name = (package + "." if package else "") + "Runner"
                        compile_record, runtime = compile_candidate(label, [*generated, copied_runner], runner_name,
                                                                   tools, home)
                        case.update(compile_record)
                        case.update({"kind": kind, "leg": leg, "decompile": decompile_command,
                                     "generated_sources": [record(path, OUTPUT) for path in generated],
                                     "runner": record(copied_runner, OUTPUT), "runner_package": package})
                        commands.append(case["compile"])
                        if case["runtime"] is not None:
                            commands.append(case["runtime"])
                        if runtime is not None:
                            case["success"] = raw_matches(runtime, originals[leg])
                            case["runtime_matches_original"] = case["success"]
                    else:
                        case["source_package_error"] = "generated Java sources have multiple package declarations"
                cases.append(case)
                continue
            cases.append(case)

    counts = {kind: sum(case["kind"] == kind for case in cases)
              for kind in ("jarde", "jadx-default", "jadx-none")}
    expected_counts = {"jarde": 2, "jadx-default": 2, "jadx-none": 2}
    complete = counts == expected_counts
    all_candidates_match = complete and all(case["success"] for case in cases)
    status = ("comparison-complete" if all_candidates_match else
              "comparison-complete-with-candidate-failures" if complete else "comparison-incomplete")
    result_out = {
        "schema": "cf16-shallow-comparison-root-v1",
        "status": status,
        "scope": "depth-02 only; original, saved Jarde source, and fresh JADX default/none source; no method is deleted",
        "input_result": {"path": str(INPUT_RESULT), "sha256": RESULT_SHA256},
        "frozen_cli": {"path": str(cli_path), "sha256": CLI_SHA256,
                       "metadata": str(CLI_META), "metadata_sha256": CLI_META_SHA256,
                       "candidate_sources": cli_meta["candidate_sources"]},
        "script": record(Path(__file__)),
        "jdk_tools": {leg: {name: record(path) for name, path in tools_by_leg[leg].items()}
                      for leg in ("javac8", "javac23")},
        "jadx": str(JADX),
        "jadx_version": jadx_version_record,
        "original_oracles": {leg: {"exit": originals[leg]["exit"],
            "stdout": record(OUTPUT / "raw/oracle" / leg / "original-verify-run.stdout", OUTPUT),
            "stderr": record(OUTPUT / "raw/oracle" / leg / "original-verify-run.stderr", OUTPUT)}
            for leg in ("javac8", "javac23")},
        "input_inventory": source_inputs,
        "original_commands_and_raw": oracle_rows,
        "expected_counts": expected_counts,
        "actual_counts": counts,
        "candidate_success_counts": {kind: sum(case["kind"] == kind and case["success"] for case in cases)
                                      for kind in expected_counts},
        "commands": commands,
        "cases": cases,
        "inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                      "excludes": ["file-inventory.json"]},
    }
    write_json(OUTPUT / "manifest.json", result_out)
    files = [record(path, OUTPUT) for path in sorted(OUTPUT.rglob("*"))
             if path.is_file() and path != OUTPUT / "file-inventory.json"]
    write_json(OUTPUT / "file-inventory.json", files)
    print(json.dumps({"status": status, "counts": counts,
                      "candidate_success_counts": result_out["candidate_success_counts"]}, indent=2))
    return 0 if complete else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"shallow comparison preparation failed: {error}", file=sys.stderr)
        raise
