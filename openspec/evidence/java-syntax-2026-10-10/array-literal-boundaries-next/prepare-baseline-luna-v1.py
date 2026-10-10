#!/usr/bin/env python3
"""Prepare immutable full-class evidence for three array-literal boundaries.

The script records real command results without editing reconstructed Java.
Every JADX/Jarde case compiles the full three-class source set and its own
Runner, then runs only the newly emitted classes under strict verification.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = Path(__file__).resolve().parent
OUT = EVIDENCE / "baseline-root-v1"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
CLI_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_VERSION = "1.5.6"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
UPSTREAM_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays")
UPSTREAM = {
    "LongArrayLimits": ("TestArrayFill4.java", "30dc65aa5749b7287f12d9bd1eceecf9055399386cc82040c583562bc99134a4"),
    "ConstantIntArray": ("TestArrayFillConstReplace.java", "0055730150e31b81ad538497dee3f78725f054ef464f300786d5a23a842fd9e3"),
    "DependentArrayStores": ("TestArrayFillNegative.java", "c1a10209342aba4e3a1fda64e1eee30309b80b613c0e6b3485fa169a595b80a1"),
}
SOURCE_NAMES = tuple(UPSTREAM)
RUNNER_NAME = "ArrayLiteralBoundariesRunner"
EXPECTED_PHYSICAL_COUNTS = {
    "LongArrayLimits": {"fields": 1, "methods": 2},
    "ConstantIntArray": {"fields": 1, "methods": 2},
    "DependentArrayStores": {"fields": 0, "methods": 2},
}
SOURCE_DIR = EVIDENCE / "sources"
RUNNER = SOURCE_DIR / f"{RUNNER_NAME}.java"
SOURCE_SHA256 = {
    "LongArrayLimits": "e42608ddd6fcd766d276e421803960ec95a90cf74023155148ec603e0c84e8f7",
    "ConstantIntArray": "db6c3062d7ddee05df6ea1db091b55b691fb01ac96ee10065a3a0e1b39583f08",
    "DependentArrayStores": "d21029dde313ca1a4b5ce02fc96f620a42cac6363cc0a2076bf5ad958c8f49f6",
    RUNNER_NAME: "3690b8528bb7dd0ea785c51973faf72c19aa95f03464949fdb22db9374f7f0ef",
}
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_record(path: Path) -> dict:
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def package_of(path: Path) -> str | None:
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", path.read_text(encoding="utf-8"), re.M)
    return match.group(1) if match else None


def extract_block(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start:index + 1]
    raise ValueError(f"unclosed declaration: {signature}")


def token_lines(fragment: str) -> str:
    return "\n".join(line.strip() for line in fragment.strip().splitlines())


def verify_selected_source(name: str, upstream: Path, adapter: Path) -> list[dict]:
    original = upstream.read_text(encoding="utf-8")
    generated = adapter.read_text(encoding="utf-8")
    rows = []
    if name == "LongArrayLimits":
        members = [
            ("field:ARRAY_SIZE", "private static final int ARRAY_SIZE = 4;"),
            ("method:test", "public long[] test() {"),
        ]
        source_field = next(line.strip() for line in original.splitlines() if "ARRAY_SIZE = 4;" in line)
        target_field = next(line.strip() for line in generated.splitlines() if "ARRAY_SIZE = 4;" in line)
        rows.append({"member": members[0][0], "body_matches_after_indent_strip": source_field == target_field})
        signature = members[1][1]
        rows.append({"member": "method:test", "body_matches_after_indent_strip": token_lines(extract_block(original, signature)) == token_lines(extract_block(generated, signature))})
    elif name == "ConstantIntArray":
        signature = "public static final int CONST_INT = 0xffff;"
        source_field = next(line.strip() for line in original.splitlines() if signature in line)
        target_field = next(line.strip() for line in generated.splitlines() if signature in line)
        rows.append({"member": "field:CONST_INT", "body_matches_after_indent_strip": source_field == target_field})
        signature = "public int[] test() {"
        rows.append({"member": "method:test", "body_matches_after_indent_strip": token_lines(extract_block(original, signature)) == token_lines(extract_block(generated, signature))})
    elif name == "DependentArrayStores":
        signature = "public int[] test() {"
        rows.append({"member": "method:test", "body_matches_after_indent_strip": token_lines(extract_block(original, signature)) == token_lines(extract_block(generated, signature))})
    else:
        raise ValueError(f"unknown fixture {name}")
    return rows


class Recorder:
    def __init__(self) -> None:
        self.commands: list[dict] = []

    def run(self, label: str, argv, home: Path | None = None, cwd: Path = ROOT):
        env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
        if home is not None:
            env["JAVA_HOME"] = str(home)
            env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
        argv_text = [str(value) for value in argv]
        try:
            result = subprocess.run(argv_text, cwd=cwd, env=env, capture_output=True, check=False)
            exit_code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except OSError as error:
            exit_code, stdout, stderr = 127, b"", (type(error).__name__ + ": " + str(error)).encode()
        streams = {}
        for name, content in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{name}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            streams[name] = file_record(path)
        row = {"label": label, "argv": argv_text, "cwd": str(cwd),
               "java_home": str(home) if home else None, "exit": exit_code, **streams}
        self.commands.append(row)
        return exit_code, stdout, stderr, row


def copy_sources(source_paths: list[Path], runner_source: Path, package: str | None, case_dir: Path):
    source_dir = case_dir / "sources"
    source_dir.mkdir(parents=True, exist_ok=True)
    copied = []
    for source in source_paths:
        target = source_dir / source.name
        shutil.copyfile(source, target)
        copied.append(target)
    runner_copy = source_dir / runner_source.name
    runner_package = package_of(runner_source)
    if package != runner_package:
        if runner_package is not None:
            raise RuntimeError("cannot move a packaged runner to a different generated package")
        prefix = f"package {package};\n\n" if package else ""
        runner_copy.write_text(prefix + runner_source.read_text(encoding="utf-8"), encoding="utf-8")
    else:
        shutil.copyfile(runner_source, runner_copy)
    copied.append(runner_copy)
    runner_class = (package + "." if package else "") + runner_source.stem
    return copied, runner_copy, runner_class


def compile_run(recorder: Recorder, label: str, sources: list[Path], leg: dict, runner_class: str):
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True)
    classes.mkdir(parents=True)
    tools, home = leg["tools"], leg["home"]
    code, _, _, compile_command = recorder.run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *sources,
    ], home)
    runtime_command = None
    runtime = None
    if code == 0:
        run_code, run_out, run_err, runtime_command = recorder.run(label + "-run", [
            tools["java"], "-Xverify:all", "-cp", classes, runner_class,
        ], home)
        runtime = {"exit": run_code, "stdout": run_out, "stderr": run_err}
    class_files = sorted(path for path in classes.rglob("*.class") if path.is_file())
    case = {
        "label": label, "compile": compile_command, "runtime": runtime_command,
        "source_files": [file_record(path) for path in sources],
        "classes": [file_record(path) for path in class_files],
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
        "compile_success": code == 0,
        "runtime_success": runtime is not None and runtime["exit"] == 0,
    }
    return case, runtime, class_files


def runtime_matches(actual, expected) -> bool:
    return actual is not None and expected is not None and actual == expected


def store_javap(recorder: Recorder, label: str, leg: dict, classes: dict[str, Path]):
    rows = []
    for class_name in (*SOURCE_NAMES, RUNNER_NAME):
        path = classes.get(class_name)
        if path is None:
            rows.append({"class": class_name, "success": False, "reason": "class file missing"})
            continue
        code, _, _, command = recorder.run(
            f"{label}-javap-{class_name}",
            [leg["tools"]["javap"], "-p", "-c", "-s", "-v", path], leg["home"])
        javap_text = (OUT / command["stdout"]["path"]).read_text(encoding="utf-8", errors="replace")
        declarations = [line for line in javap_text.splitlines()
                        if re.match(r"^\s{2}(?:public|private|protected)\s+.+;\s*$", line)]
        method_count = sum("(" in line for line in declarations)
        field_count = sum("(" not in line for line in declarations)
        expected = EXPECTED_PHYSICAL_COUNTS.get(class_name)
        complete = expected is None or {"fields": field_count, "methods": method_count} == expected
        rows.append({"class": class_name, "class_file": file_record(path),
                     "command": command, "method_count": method_count, "field_count": field_count,
                     "expected_physical_counts": expected, "complete_inventory": complete,
                     "success": code == 0 and complete})
    return rows


def source_paths_for_class_names(class_dir: Path) -> dict[str, Path]:
    return {path.stem: path for path in class_dir.rglob("*.class") if path.is_file()}


def check_class_set(class_dir: Path, class_files: list[Path], package: str | None) -> dict:
    prefix = package.replace(".", "/") + "/" if package else ""
    expected = {prefix + f"{name}.class" for name in (*SOURCE_NAMES, RUNNER_NAME)}
    actual = {path.relative_to(class_dir).as_posix() for path in class_files}
    return {"expected": sorted(expected), "actual": sorted(actual), "complete": actual == expected}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--metadata", type=Path, default=CLI_METADATA)
    args = parser.parse_args()
    cli_path = args.cli.resolve()
    metadata_path = args.metadata.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite evidence: {OUT}")
    for source in [*(SOURCE_DIR / f"{name}.java" for name in SOURCE_NAMES), RUNNER,
                   *(UPSTREAM_ROOT / filename for filename, _ in UPSTREAM.values()),
                   *(EVIDENCE / "upstream" / filename for filename, _ in UPSTREAM.values()),
                   JDK_MANIFEST, metadata_path]:
        if not source.is_file():
            raise SystemExit(f"required input missing: {source}")

    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "original-sources").mkdir()
    recorder = Recorder()
    preflight: list[dict] = []
    failures: list = []
    metadata_bytes = metadata_path.read_bytes()
    metadata = json.loads(metadata_bytes)
    metadata_hash = sha(metadata_bytes)
    preflight.append({"label": "metadata", "path": str(metadata_path),
                      "expected_sha256": CLI_METADATA_SHA256, "actual_sha256": metadata_hash,
                      "ok": metadata_hash == CLI_METADATA_SHA256})
    expected_cli_path = Path(metadata["cli_path"]).resolve()
    cli_hash = sha(cli_path.read_bytes()) if cli_path.is_file() else None
    preflight.append({"label": "cli", "path": str(cli_path), "expected_path": str(expected_cli_path),
                      "expected_sha256": CLI_SHA256, "metadata_sha256": metadata.get("cli_sha256"),
                      "actual_sha256": cli_hash,
                      "ok": cli_path == expected_cli_path and cli_hash == CLI_SHA256 == metadata.get("cli_sha256")})
    candidate_pins = metadata.get("candidate_sources", {})
    test_pins = metadata.get("test_sources", {})
    preflight.append({"label": "product-pin-set", "expected_count": 10,
                      "actual_count": len(candidate_pins), "ok": len(candidate_pins) == 10})
    for category, pins in (("candidate", candidate_pins), ("test", test_pins)):
        for relative, expected in pins.items():
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            preflight.append({"label": f"{category}-source:{relative}", "path": str(path),
                              "expected_sha256": expected, "actual_sha256": actual, "ok": actual == expected})

    upstream_rows = []
    selected_members = []
    for class_name, (filename, expected_hash) in UPSTREAM.items():
        live = UPSTREAM_ROOT / filename
        snapshot = EVIDENCE / "upstream" / filename
        adapter = SOURCE_DIR / f"{class_name}.java"
        live_hash = sha(live.read_bytes())
        snapshot_hash = sha(snapshot.read_bytes())
        upstream_rows.append({"class": class_name, "path": str(live), "snapshot": str(snapshot.relative_to(EVIDENCE)),
                              "expected_sha256": expected_hash, "live_sha256": live_hash,
                              "snapshot_sha256": snapshot_hash,
                              "ok": live_hash == snapshot_hash == expected_hash})
        member_rows = verify_selected_source(class_name, snapshot, adapter)
        for row in member_rows:
            row["class"] = class_name
            row["ok"] = row["body_matches_after_indent_strip"]
        selected_members.extend(member_rows)
    preflight.extend(upstream_rows)
    preflight.append({"label": "selected-method-and-field-identity", "ok": all(row["ok"] for row in selected_members),
                      "members": selected_members})
    source_records = []
    for name in (*SOURCE_NAMES, RUNNER_NAME):
        path = SOURCE_DIR / f"{name}.java"
        actual = sha(path.read_bytes())
        ok = actual == SOURCE_SHA256[name]
        source_records.append({"path": str(path.relative_to(EVIDENCE)), "bytes": len(path.read_bytes()),
                               "sha256": actual, "expected_sha256": SOURCE_SHA256[name], "ok": ok})
        preflight.append({"label": f"prepared-source:{name}", "path": str(path),
                          "expected_sha256": SOURCE_SHA256[name], "actual_sha256": actual, "ok": ok})

    jdk_bytes = JDK_MANIFEST.read_bytes()
    jdk = json.loads(jdk_bytes)
    jdk_hash = sha(jdk_bytes)
    preflight.append({"label": "jdk-controls", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": jdk_hash,
                      "ok": jdk.get("status") == "complete" and jdk_hash == JDK_MANIFEST_SHA256})
    legs = {}
    for frozen in jdk.get("legs", []):
        tools = {}
        hashes_ok = True
        for tool_name, fact in frozen.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact.get("sha256")
            hashes_ok &= ok
            preflight.append({"label": f"{frozen['leg']}:{tool_name}", "path": str(path),
                              "expected_sha256": fact.get("sha256"), "actual_sha256": actual, "ok": ok})
            tools[tool_name] = path
        if all(name in tools for name in ("java", "javac", "javap")):
            legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent,
                                   "hashes_ok": hashes_ok}
    preflight.append({"label": "jdk-legs", "expected": ["javac8", "javac23"],
                      "actual": sorted(legs), "ok": set(legs) == {"javac8", "javac23"}})

    jadx_exec = JADX.resolve()
    jadx_exec_hash = sha(jadx_exec.read_bytes()) if jadx_exec.is_file() else None
    preflight.append({"label": "jadx-binary", "path": str(jadx_exec),
                      "expected_sha256": JADX_SHA256, "actual_sha256": jadx_exec_hash,
                      "ok": jadx_exec_hash == JADX_SHA256})
    if not all(row.get("ok", False) for row in preflight):
        write_json(OUT / "preflight.json", preflight)
        inventory_path = OUT / "file-inventory.json"
        write_json(inventory_path, [file_record(path) for path in sorted(OUT.rglob("*"))
                                    if path.is_file() and path != inventory_path])
        raise SystemExit(f"frozen input identity check failed; see {OUT / 'preflight.json'}")

    version_code, version_out, _, version_command = recorder.run("jadx-version", [JADX, "--version"])
    version_ok = version_code == 0 and version_out.strip().decode(errors="replace") == JADX_VERSION
    preflight.append({"label": "jadx-version", "expected": JADX_VERSION,
                      "actual": version_out.decode(errors="replace").strip(),
                      "command": version_command, "exit": version_code, "ok": version_ok})
    if not version_ok:
        write_json(OUT / "preflight.json", preflight)
        inventory_path = OUT / "file-inventory.json"
        write_json(inventory_path, [file_record(path) for path in sorted(OUT.rglob("*"))
                                    if path.is_file() and path != inventory_path])
        raise SystemExit("JADX version check failed")

    # Freeze copies of the source inputs actually used by every full-class leg.
    for name in (*SOURCE_NAMES, RUNNER_NAME):
        shutil.copyfile(SOURCE_DIR / f"{name}.java", OUT / "original-sources" / f"{name}.java")
    for filename, _ in UPSTREAM.values():
        shutil.copyfile(EVIDENCE / "upstream" / filename, OUT / "original-sources" / filename)

    cases = []
    original_results = {}
    original_class_maps = {}
    original_case_by_jdk = {}
    target_class_names = set(SOURCE_NAMES)
    expected_class_names = target_class_names | {RUNNER_NAME}
    for jdk_name in ("javac8", "javac23"):
        leg = legs[jdk_name]
        label = f"{jdk_name}-original"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        input_dir = case_dir / "input-sources"
        input_dir.mkdir()
        sources = []
        for name in (*SOURCE_NAMES, RUNNER_NAME):
            target = input_dir / f"{name}.java"
            shutil.copyfile(OUT / "original-sources" / f"{name}.java", target)
            sources.append(target)
        case, runtime, class_files = compile_run(recorder, label, sources, leg, RUNNER_NAME)
        class_map = source_paths_for_class_names(case_dir / "classes")
        produced_names = set(class_map)
        class_set = check_class_set(case_dir / "classes", class_files, None)
        case.update({"kind": "original", "jdk_leg": jdk_name,
                     "expected_class_names": sorted(expected_class_names),
                     "produced_class_names": sorted(produced_names),
                     "class_output_set": class_set,
                     "complete_class_set": produced_names == expected_class_names and class_set["complete"]})
        javap_rows = store_javap(recorder, label, leg, class_map) if case["compile_success"] else []
        case["javap"] = javap_rows
        case["success"] = bool(case["compile_success"] and case["runtime_success"]
                               and case["complete_class_set"] and all(row.get("success", False) for row in javap_rows))
        case["runtime_sha256"] = ({"exit": runtime["exit"], "stdout": sha(runtime["stdout"]),
                                   "stderr": sha(runtime["stderr"])} if runtime is not None else None)
        if not case["success"]:
            failures.append(label)
        cases.append(case)
        original_results[jdk_name] = runtime
        original_class_maps[jdk_name] = class_map
        original_case_by_jdk[jdk_name] = case

    # JADX consumes one javac 23 jar containing only the three original target classes.
    jar_path = OUT / "jadx-input" / "ArrayLiteralBoundaryTargets.jar"
    jar_path.parent.mkdir(parents=True)
    jar_members = []
    javac23_map = original_class_maps.get("javac23", {})
    if target_class_names <= set(javac23_map):
        with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_DEFLATED) as jar:
            for class_name in SOURCE_NAMES:
                path = javac23_map[class_name]
                info = zipfile.ZipInfo(f"{class_name}.class", date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                jar.writestr(info, path.read_bytes())
                jar_members.append({"name": f"{class_name}.class", **file_record(path)})
        with zipfile.ZipFile(jar_path) as jar:
            expected_entries = [f"{name}.class" for name in SOURCE_NAMES]
            if jar.namelist() != expected_entries:
                failures.append("JADX jar entry set differs from the three target classes")
            for class_name in SOURCE_NAMES:
                if jar.read(f"{class_name}.class") != javac23_map[class_name].read_bytes():
                    failures.append(f"JADX jar member differs from javac23 original: {class_name}")
    else:
        failures.append("JADX jar unavailable: javac23 original target classes are incomplete")
        jar_path.write_bytes(b"")

    decompilations = {}
    for profile in ("default", "none"):
        output = OUT / "jadx" / profile
        output.parent.mkdir(parents=True, exist_ok=True)
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv.extend(["--rename-flags", "none"])
        code, _, _, command = recorder.run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path], legs["javac23"]["home"])
        generated = sorted(path for path in output.rglob("*.java") if path.is_file())
        packages = {package_of(path) for path in generated}
        package = next(iter(packages)) if len(packages) == 1 else None
        expected_source_names = {f"{name}.java" for name in SOURCE_NAMES}
        source_names = {path.name for path in generated}
        row = {"profile": profile, "decompile": command, "decompile_success": code == 0,
               "generated_source_count": len(generated), "source_name_set_complete": source_names == expected_source_names,
               "package_set_consistent": len(packages) == 1,
               "packages": sorted(packages, key=lambda value: value or ""),
               "generated_sources": [file_record(path) for path in generated],
               "input_jar": file_record(jar_path), "jar_members": jar_members}
        if code != 0 or source_names != expected_source_names or package is None:
            failures.append(f"jadx-{profile}-decompile")
        decompilations[profile] = {"row": row, "sources": generated, "package": package}

    for profile in ("default", "none"):
        decompilation = decompilations[profile]
        for jdk_name in ("javac8", "javac23"):
            label = f"{jdk_name}-jadx-{profile}"
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            generated_dir = case_dir / "generated-sources"
            generated_dir.mkdir()
            generated = []
            for path in decompilation["sources"]:
                target = generated_dir / path.name
                shutil.copyfile(path, target)
                generated.append(target)
            if {path.stem for path in generated} != target_class_names:
                failures.append(f"{label}: generated source set incomplete")
            compile_sources, runner_copy, runner_class = copy_sources(generated, RUNNER, decompilation["package"], case_dir)
            case, runtime, _ = compile_run(recorder, label, compile_sources, legs[jdk_name], runner_class)
            class_files = [OUT / row["path"] for row in case["classes"]]
            class_set = check_class_set(OUT / "cases" / label / "classes", class_files, decompilation["package"])
            case.update({"kind": "jadx", "profile": profile, "jdk_leg": jdk_name,
                         "decompilation": decompilation["row"],
                         "generated_sources": [file_record(path) for path in generated],
                         "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                         "class_output_set": class_set})
            case["runtime_matches_original_raw"] = runtime_matches(runtime, original_results[jdk_name])
            case["success"] = bool(case["compile_success"] and case["runtime_success"]
                                   and case["runtime_matches_original_raw"]
                                   and decompilation["row"]["decompile_success"]
                                   and decompilation["row"]["source_name_set_complete"]
                                   and decompilation["row"]["package_set_consistent"]
                                   and class_set["complete"])
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    for jdk_name in ("javac8", "javac23"):
        label = f"{jdk_name}-jarde"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        rendered = []
        rendered_sources = []
        render_ok = True
        original_map = original_class_maps.get(jdk_name, {})
        for class_name in SOURCE_NAMES:
            class_file = original_map.get(class_name)
            if class_file is None:
                render_ok = False
                rendered.append({"class": class_name, "success": False, "reason": "original class file missing"})
                continue
            code, stdout, _, command = recorder.run(
                f"{label}-render-{class_name}",
                [cli_path, "class-source", "--input", class_file, "--class", class_name,
                 "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all"],
                legs[jdk_name]["home"])
            render_row = {"class": class_name, "original_class": file_record(class_file),
                          "command": command, "success": code == 0}
            if code == 0:
                doc_dir = case_dir / "class-source" / class_name
                doc_dir.mkdir(parents=True)
                json_path = doc_dir / "class-source.json"
                json_path.write_bytes(stdout)
                try:
                    doc = json.loads(stdout)
                    source = doc["text"].encode("utf-8")
                    source_path = doc_dir / f"{class_name}.java"
                    source_path.write_bytes(source)
                    render_row.update({"class_source_json": file_record(json_path),
                                       "generated_source": file_record(source_path),
                                       "member_count": len(doc.get("methods", [])),
                                       "field_count": len(doc.get("fields", [])),
                                       "expected_physical_counts": EXPECTED_PHYSICAL_COUNTS[class_name],
                                       "physical_inventory_matches": {
                                           "methods": len(doc.get("methods", [])) == EXPECTED_PHYSICAL_COUNTS[class_name]["methods"],
                                           "fields": len(doc.get("fields", [])) == EXPECTED_PHYSICAL_COUNTS[class_name]["fields"],
                                       }})
                    render_row["physical_inventory_matches"]["complete"] = all(render_row["physical_inventory_matches"].values())
                    render_ok = render_ok and render_row["physical_inventory_matches"]["complete"]
                    rendered_sources.append(source_path)
                except Exception as error:
                    render_ok = False
                    render_row["document_error"] = type(error).__name__ + ": " + str(error)
            else:
                render_ok = False
            rendered.append(render_row)
        packages = {package_of(path) for path in rendered_sources}
        package = next(iter(packages)) if len(packages) == 1 else None
        case = {"label": label, "kind": "jarde", "jdk_leg": jdk_name,
                "rendered_classes": rendered, "render_success": render_ok and len(rendered_sources) == 3,
                "package_set": sorted(packages, key=lambda value: value or ""),
                "package_set_consistent": len(packages) == 1}
        if len(rendered_sources) == 3 and len(packages) == 1:
            compile_sources, runner_copy, runner_class = copy_sources(rendered_sources, RUNNER, package, case_dir)
            compiled, runtime, _ = compile_run(recorder, label, compile_sources, legs[jdk_name], runner_class)
            class_files = [OUT / row["path"] for row in compiled["classes"]]
            class_set = check_class_set(OUT / "cases" / label / "classes", class_files, package)
            case.update(compiled)
            case.update({"runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                         "rendered_sources": [file_record(path) for path in rendered_sources],
                         "class_output_set": class_set})
            case["runtime_matches_original_raw"] = runtime_matches(runtime, original_results[jdk_name])
            case["success"] = bool(case["render_success"] and case["compile_success"] and case["runtime_success"]
                                   and case["runtime_matches_original_raw"] and class_set["complete"])
        else:
            case.update({"compile_success": False, "runtime_success": False,
                         "runtime_matches_original_raw": False, "success": False})
        if not case["success"]:
            failures.append(label)
        cases.append(case)

    expected_counts = {"original": 2, "jadx": 4, "jarde": 2}
    case_counts = {kind: sum(case.get("kind") == kind for case in cases) for kind in expected_counts}
    success_counts = {kind: sum(case.get("kind") == kind and case.get("success", False) for case in cases)
                      for kind in expected_counts}
    if case_counts != expected_counts:
        failures.append({"expected": expected_counts, "actual": case_counts})
    original_cross_jdk_equal = runtime_matches(original_results.get("javac8"), original_results.get("javac23"))
    if not original_cross_jdk_equal:
        failures.append("original cross-JDK raw runtime triples differ")

    manifest = {
        "schema": "array-literal-boundaries-next-baseline-v1",
        "status": "completed" if not failures else "baseline-with-failures",
        "claim_boundary": "Three focused JADX array-literal fixtures adapted to standalone classes; not a broader suite-completion claim.",
        "oracle": "Fresh original complete four-class runs (three target classes plus Runner) on each JDK; reconstructed paths compare exit/stdout/stderr byte-for-byte to the same-JDK original.",
        "source_identities": upstream_rows,
        "selected_member_identity": selected_members,
        "sources": source_records,
        "cli_metadata": {"path": str(metadata_path), "sha256": metadata_hash},
        "frozen_cli": {"path": str(cli_path), "sha256": cli_hash},
        "candidate_sources": candidate_pins,
        "test_sources": test_pins,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
        "jadx": {"path": str(JADX), "version": JADX_VERSION,
                 "executable_path": str(jadx_exec), "executable_sha256": jadx_exec_hash,
                 "version_command": version_command, "jar": file_record(jar_path),
                 "jar_members": jar_members,
                 "profiles": [decompilations[name]["row"] for name in ("default", "none")]},
        "cases_expected": {"original": 2, "jadx": 4, "jarde": 2},
        "case_counts": case_counts, "success_counts": success_counts,
        "original_cross_jdk_raw_equal": original_cross_jdk_equal,
        "environment_policy": {"removed_for_each_process": list(STRIPPED_ENV),
                                "JDK_HOME_and_PATH_set_for_JDK_processes": True},
        "preflight": preflight, "commands": recorder.commands, "cases": cases,
        "failures": failures,
        "script": file_record(Path(__file__)),
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                           "excludes": ["file-inventory.json"]},
    }
    write_json(OUT / "manifest.json", manifest)
    inventory_path = OUT / "file-inventory.json"
    inventory = [file_record(path) for path in sorted(OUT.rglob("*"))
                 if path.is_file() and path != inventory_path]
    write_json(inventory_path, inventory)
    print(json.dumps({"status": manifest["status"], "case_counts": case_counts,
                      "success_counts": success_counts, "original_cross_jdk_raw_equal": original_cross_jdk_equal,
                      "failures": failures, "manifest": str(OUT / "manifest.json"),
                      "inventory": str(inventory_path)}, ensure_ascii=False, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"baseline preparation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise
