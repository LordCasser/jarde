#!/usr/bin/env python3
"""Prepare complete-class no-clinit/super-argument runtime evidence; never edits source."""

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
OUT = EVIDENCE / "baseline-root-v2"
SOURCE_NAMES = ("ArrayFieldInitBase", "CommonNoClinitArrayInit")
RUNNER_NAME = "Runner"
SOURCE_DIR = EVIDENCE
RUNNER = EVIDENCE / "Runner.java"
SOURCE_SHA256 = {
    "ArrayFieldInitBase": "16c212caacb1e4fcfb67e78670b4fdd3418cf6e3d5e3977cfd80535403b3d1a9",
    "CommonNoClinitArrayInit": "617ff8efc82edec37b50b5d87a3ef43e74fe64374c10caad95ac24211b3521d1",
    "Runner": "51336925efdb1ef4abd6270bc63bbb9c752e8a7ea2b98e96b6c2c93bc33fad4d",
}
EXPECTED_PHYSICAL = {
    "ArrayFieldInitBase": {
        "fields": {"received:I"},
        "methods": {"<init>(I)V"},
    },
    "CommonNoClinitArrayInit": {
        "fields": {"trace:I", "first:[B", "second:[B"},
        "methods": {"<init>()V", "<init>(I)V", "mark(I)B", "run(I)B"},
    },
}

JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
CLI_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_VERSION = "1.5.6"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
EXPECTED_PRODUCT_PINS = {
    "Cargo.lock",
    "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/build.rs",
    "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/field.rs",
    "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs",
    "src/class_source.rs",
    "src/facade.rs",
}
EXPECTED_TEST_PINS = {
    "tests/class_static_initializer_projection.rs",
    "tests/interface_initializer_proof.rs",
    "crates/jarde-java/tests/p3_patterns.rs",
    ".github/workflows/ci.yml",
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
            exit_code, stdout = 127, b""
            stderr = (type(error).__name__ + ": " + str(error)).encode()
        streams = {}
        for stream_name, content in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{stream_name}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            streams[stream_name] = file_record(path)
        row = {"label": label, "argv": argv_text, "cwd": str(cwd),
               "java_home": str(home) if home else None, "exit": exit_code, **streams}
        self.commands.append(row)
        return exit_code, stdout, stderr, row


def copy_sources(sources: list[Path], runner_source: Path, package: str | None, case_dir: Path):
    source_dir = case_dir / "sources"
    source_dir.mkdir(parents=True, exist_ok=True)
    copied = []
    for source in sources:
        target = source_dir / source.name
        shutil.copyfile(source, target)
        copied.append(target)
    runner_copy = source_dir / runner_source.name
    runner_package = package_of(runner_source)
    if package != runner_package:
        if runner_package is not None:
            raise RuntimeError("cannot move a packaged Runner to a different generated package")
        prefix = f"package {package};\n\n" if package else ""
        runner_copy.write_text(prefix + runner_source.read_text(encoding="utf-8"), encoding="utf-8")
    else:
        shutil.copyfile(runner_source, runner_copy)
    return copied + [runner_copy], runner_copy, (package + "." if package else "") + RUNNER_NAME


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


def class_map(classes_dir: Path) -> dict[str, Path]:
    return {path.stem: path for path in classes_dir.rglob("*.class") if path.is_file()}


def class_set(classes_dir: Path, class_files: list[Path], package: str | None):
    prefix = package.replace(".", "/") + "/" if package else ""
    expected = {prefix + f"{name}.class" for name in (*SOURCE_NAMES, RUNNER_NAME)}
    actual = {path.relative_to(classes_dir).as_posix() for path in class_files}
    return {"expected": sorted(expected), "actual": sorted(actual), "complete": actual == expected}


def method_signature(method: dict) -> str:
    item = method["item"]
    return item["name"]["escaped"] + item["descriptor"]["escaped"]


def field_signature(field: dict) -> str:
    item = field["item"]
    return item["name"]["escaped"] + ":" + item["descriptor"]["escaped"]


def inspect_class_source(doc: dict, class_name: str, require_source_maps: bool) -> dict:
    fields = doc.get("fields", [])
    methods = doc.get("methods", [])
    actual_fields = {field_signature(field) for field in fields}
    actual_methods = {method_signature(method) for method in methods}
    expected = EXPECTED_PHYSICAL[class_name]
    method_maps = []
    all_maps_complete = True
    all_methods_qualified = True
    for method in methods:
        report = method.get("outcome", {}).get("report", {})
        source_map = report.get("source_map")
        evidence = report.get("evidence", {}).get("categories", [])
        source_category = next((item for item in evidence if item.get("kind") == "source_map"), None)
        complete = (
            source_map is not None
            and isinstance(source_map.get("segments"), list)
            and source_category is not None
            and source_category.get("state", {}).get("state") == "complete"
        )
        all_maps_complete &= complete
        execution = report.get("execution", {})
        quality_ok = (
            report.get("quality") == "structured"
            and report.get("outcome") == "produced"
            and execution.get("status") == "complete"
            and report.get("fallbacks", []) == []
        )
        all_methods_qualified &= quality_ok
        method_maps.append({
            "index": method["item"].get("index"),
            "signature": method_signature(method),
            "identity": method["item"].get("identity"),
            "access_flags": method["item"].get("access_flags"),
            "quality_and_execution_complete": quality_ok,
            "source_map_complete": complete,
            "source_map": source_map,
        })
    physical_ok = actual_fields == expected["fields"] and actual_methods == expected["methods"]
    no_clinit = not any(signature.startswith("<clinit>") for signature in actual_methods)
    execution_complete = doc.get("execution", {}).get("status") == "complete"
    source_maps_ok = all_maps_complete if require_source_maps else True
    return {
        "class": class_name,
        "physical_inventory_matches": physical_ok,
        "expected_fields": sorted(expected["fields"]),
        "actual_fields": sorted(actual_fields),
        "fields": [
            {"index": field["item"].get("index"), "signature": field_signature(field),
             "identity": field["item"].get("identity"),
             "access_flags": field["item"].get("access_flags")}
            for field in fields
        ],
        "expected_methods": sorted(expected["methods"]),
        "actual_methods": sorted(actual_methods),
        "method_source_maps": method_maps,
        "source_maps_complete": all_maps_complete,
        "source_maps_required": require_source_maps,
        "method_quality_and_execution_complete": all_methods_qualified,
        "execution_complete": execution_complete,
        "no_clinit": no_clinit,
        "success": physical_ok and no_clinit and execution_complete and all_methods_qualified and source_maps_ok,
    }


def source_set_check(classes_dir: Path, paths: list[Path], package: str | None = None) -> dict:
    return class_set(classes_dir, paths, package)


def add_inventory(OUT_PATH: Path, inventory_path: Path) -> None:
    rows = [file_record(path) for path in sorted(OUT_PATH.rglob("*"))
            if path.is_file() and path != inventory_path]
    inventory_path.write_text(json.dumps(rows, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--metadata", type=Path, default=CLI_METADATA)
    args = parser.parse_args()
    cli_path, metadata_path = args.cli.resolve(), args.metadata.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite evidence: {OUT}")
    if metadata_path != CLI_METADATA.resolve():
        raise SystemExit(f"metadata path is frozen: {CLI_METADATA}")
    for source in (EVIDENCE / "ArrayFieldInitBase.java", EVIDENCE / "CommonNoClinitArrayInit.java", RUNNER,
                   JDK_MANIFEST, metadata_path):
        if not source.is_file():
            raise SystemExit(f"required input missing: {source}")

    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "original-sources").mkdir()
    (OUT / "jadx-input").mkdir()
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
    canonical_pins = metadata.get("canonical_files", {})
    preflight.append({"label": "product-pin-set", "expected_count": 10,
                      "expected_paths": sorted(EXPECTED_PRODUCT_PINS), "actual_count": len(candidate_pins),
                      "actual_paths": sorted(candidate_pins), "ok": len(candidate_pins) == 10
                      and set(candidate_pins) == EXPECTED_PRODUCT_PINS})
    preflight.append({"label": "test-pin-set", "expected_count": 4,
                      "expected_paths": sorted(EXPECTED_TEST_PINS), "actual_count": len(test_pins),
                      "actual_paths": sorted(test_pins), "ok": len(test_pins) == 4
                      and set(test_pins) == EXPECTED_TEST_PINS})
    preflight.append({"label": "canonical-pin-set", "expected_count": 16,
                      "actual_count": len(canonical_pins), "ok": len(canonical_pins) == 16})
    for category, pins in (("candidate", candidate_pins), ("test", test_pins),
                           ("canonical", canonical_pins)):
        for relative, expected in pins.items():
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            preflight.append({"label": f"{category}-pin:{relative}", "path": str(path),
                              "expected_sha256": expected, "actual_sha256": actual, "ok": actual == expected})

    source_records = []
    for name in (*SOURCE_NAMES, RUNNER_NAME):
        path = EVIDENCE / f"{name}.java"
        data = path.read_bytes()
        actual = sha(data)
        ok = actual == SOURCE_SHA256[name]
        source_records.append({"path": str(path.relative_to(EVIDENCE)), "bytes": len(data),
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
        tools, hashes_ok = {}, True
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
    jadx_hash = sha(jadx_exec.read_bytes()) if jadx_exec.is_file() else None
    preflight.append({"label": "jadx-binary", "path": str(jadx_exec), "expected_sha256": JADX_SHA256,
                      "actual_sha256": jadx_hash, "ok": jadx_hash == JADX_SHA256})
    if not all(row.get("ok", False) for row in preflight):
        write_json(OUT / "preflight.json", preflight)
        add_inventory(OUT, OUT / "file-inventory.json")
        raise SystemExit(f"frozen input identity check failed; see {OUT / 'preflight.json'}")

    version_code, version_out, _, version_command = recorder.run(
        "jadx-version", [JADX, "--version"], legs["javac23"]["home"])
    version_actual = version_out.decode(errors="replace").strip()
    version_ok = version_code == 0 and version_actual == JADX_VERSION
    preflight.append({"label": "jadx-version", "expected": JADX_VERSION, "actual": version_actual,
                      "command": version_command, "exit": version_code, "ok": version_ok})
    if not version_ok:
        write_json(OUT / "preflight.json", preflight)
        add_inventory(OUT, OUT / "file-inventory.json")
        raise SystemExit("JADX version check failed")

    for name in (*SOURCE_NAMES, RUNNER_NAME):
        shutil.copyfile(EVIDENCE / f"{name}.java", OUT / "original-sources" / f"{name}.java")

    cases: list[dict] = []
    originals: dict[str, dict | None] = {}
    original_classes: dict[str, dict[str, Path]] = {}
    expected_names = set((*SOURCE_NAMES, RUNNER_NAME))
    for jdk_name in ("javac8", "javac23"):
        leg = legs[jdk_name]
        label = f"{jdk_name}-original"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        input_dir = case_dir / "input-sources"
        input_dir.mkdir()
        sources = []
        for name in (*SOURCE_NAMES, RUNNER_NAME):
            copied = input_dir / f"{name}.java"
            shutil.copyfile(OUT / "original-sources" / f"{name}.java", copied)
            sources.append(copied)
        case, runtime, class_files = compile_run(recorder, label, sources, leg, RUNNER_NAME)
        produced = class_map(case_dir / "classes")
        set_row = source_set_check(case_dir / "classes", class_files)
        case.update({"kind": "original", "jdk_leg": jdk_name,
                     "expected_class_names": sorted(expected_names),
                     "produced_class_names": sorted(produced), "class_output_set": set_row,
                     "complete_class_set": set(produced) == expected_names and set_row["complete"]})
        javap_rows = []
        if case["compile_success"]:
            for class_name in (*SOURCE_NAMES, RUNNER_NAME):
                class_file = produced.get(class_name)
                if class_file is None:
                    javap_rows.append({"class": class_name, "success": False, "reason": "class file missing"})
                    continue
                code, stdout, _, command = recorder.run(
                    f"{label}-javap-{class_name}",
                    [leg["tools"]["javap"], "-p", "-c", "-s", "-v", class_file], leg["home"])
                raw_path = OUT / command["stdout"]["path"]
                body = stdout.decode("utf-8", errors="replace")
                physical_ok = True
                if class_name in EXPECTED_PHYSICAL:
                    expected = EXPECTED_PHYSICAL[class_name]
                    counts = re.search(
                        r"(?m)^\s*interfaces:\s*(\d+),\s*fields:\s*(\d+),\s*methods:\s*(\d+)\b",
                        body,
                    )
                    has_clinit = bool(re.search(r"(?m)^\s*static\s*\{\};?\s*$", body)) or "<clinit>" in body
                    physical_ok = (
                        counts is not None
                        and int(counts.group(2)) == len(expected["fields"])
                        and int(counts.group(3)) == len(expected["methods"])
                        and not has_clinit
                    )
                javap_rows.append({"class": class_name, "class_file": file_record(class_file),
                                   "command": command, "raw_javap": file_record(raw_path),
                                   "physical_counts_and_no_clinit": physical_ok,
                                   "success": code == 0 and physical_ok})
        case["javap"] = javap_rows
        case["success"] = bool(case["compile_success"] and case["runtime_success"]
                               and case["complete_class_set"] and all(row.get("success", False) for row in javap_rows))
        if not case["success"]:
            failures.append(label)
        case["runtime_sha256"] = ({"exit": runtime["exit"], "stdout": sha(runtime["stdout"]),
                                   "stderr": sha(runtime["stderr"])} if runtime is not None else None)
        cases.append(case)
        originals[jdk_name] = runtime
        original_classes[jdk_name] = produced

    # JADX gets exactly the two original target class files, compiled together with their superclass.
    jar_path = OUT / "jadx-input" / "NoClinitSuperArgumentTargets.jar"
    jar_members = []
    javac23_map = original_classes.get("javac23", {})
    if set(SOURCE_NAMES) <= set(javac23_map):
        with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_DEFLATED) as jar:
            for class_name in SOURCE_NAMES:
                class_file = javac23_map[class_name]
                info = zipfile.ZipInfo(f"{class_name}.class", date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                jar.writestr(info, class_file.read_bytes())
                jar_members.append({"name": f"{class_name}.class", **file_record(class_file)})
        with zipfile.ZipFile(jar_path) as jar:
            expected_jar_entries = [f"{name}.class" for name in SOURCE_NAMES]
            if jar.namelist() != expected_jar_entries:
                failures.append("JADX jar does not contain exactly the two original target classes")
            for class_name in SOURCE_NAMES:
                if jar.read(f"{class_name}.class") != javac23_map[class_name].read_bytes():
                    failures.append(f"JADX jar member differs from javac23 original: {class_name}")
    else:
        failures.append("JADX jar unavailable: javac23 original target classes are incomplete")
        jar_path.write_bytes(b"")

    decompilations = {}
    for profile in ("default", "none"):
        output = OUT / "jadx" / profile
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv.extend(["--rename-flags", "none"])
        code, _, _, command = recorder.run(
            f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path], legs["javac23"]["home"])
        generated = sorted(path for path in output.rglob("*.java") if path.is_file()) if output.exists() else []
        packages = {package_of(path) for path in generated}
        package = next(iter(packages)) if len(packages) == 1 else None
        expected_sources = {f"{name}.java" for name in SOURCE_NAMES}
        source_set_ok = {path.name for path in generated} == expected_sources
        row = {"profile": profile, "decompile": command, "decompile_success": code == 0,
               "generated_source_count": len(generated), "source_name_set_complete": source_set_ok,
               "package_set_consistent": len(packages) == 1,
               "packages": sorted(packages, key=lambda value: value or ""),
               "generated_sources": [file_record(path) for path in generated],
               "input_jar": file_record(jar_path), "jar_members": jar_members}
        if code != 0 or not source_set_ok or len(packages) != 1:
            failures.append(f"jadx-{profile}-decompile")
        decompilations[profile] = {"row": row, "sources": generated, "package": package}

    for profile in ("default", "none"):
        decomp = decompilations[profile]
        for jdk_name in ("javac8", "javac23"):
            label = f"{jdk_name}-jadx-{profile}"
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            generated_dir = case_dir / "generated-sources"
            generated_dir.mkdir()
            generated = []
            for path in decomp["sources"]:
                target = generated_dir / path.name
                shutil.copyfile(path, target)
                generated.append(target)
            complete_generated = {path.stem for path in generated} == set(SOURCE_NAMES)
            compile_sources, runner_copy, runner_class = copy_sources(generated, RUNNER, decomp["package"], case_dir)
            case, runtime, class_files = compile_run(recorder, label, compile_sources, legs[jdk_name], runner_class)
            set_row = source_set_check(case_dir / "classes", class_files, decomp["package"])
            case.update({"kind": "jadx", "profile": profile, "jdk_leg": jdk_name,
                         "decompilation": decomp["row"], "generated_sources": [file_record(path) for path in generated],
                         "generated_source_set_complete": complete_generated,
                         "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                         "class_output_set": set_row,
                         "complete_class_set": complete_generated and set_row["complete"]})
            actual_runtime = ({"exit": runtime["exit"], "stdout": runtime["stdout"], "stderr": runtime["stderr"]}
                              if runtime is not None else None)
            case["runtime_matches_same_jdk_original_raw"] = actual_runtime is not None and actual_runtime == originals[jdk_name]
            case["success"] = bool(case["compile_success"] and case["runtime_success"]
                                   and case["runtime_matches_same_jdk_original_raw"]
                                   and decomp["row"]["decompile_success"]
                                   and decomp["row"]["source_name_set_complete"]
                                   and decomp["row"]["package_set_consistent"] and set_row["complete"])
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    # Each Jarde target is rendered twice per JDK; --evidence all JSON retains every physical
    # field/method identity and each method's complete source-map segments, not just the source text.
    for jdk_name in ("javac8", "javac23"):
        label = f"{jdk_name}-jarde"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        class_rows = []
        all_sources = []
        render_success = True
        for class_name in SOURCE_NAMES:
            class_file = original_classes.get(jdk_name, {}).get(class_name)
            profiles = {}
            profile_text = {}
            inventory_rows = {}
            if class_file is None:
                render_success = False
                class_rows.append({"class": class_name, "success": False, "reason": "original class file missing"})
                continue
            for profile in ("default", "all"):
                argv = [cli_path, "class-source", "--input", class_file, "--class", class_name,
                        "--policy", "single-class", "--release", "8", "--format", "json"]
                if profile == "all":
                    argv.extend(["--evidence", "all"])
                command_label = f"{label}-render-{class_name}-{profile}"
                code, stdout, _, command = recorder.run(command_label, argv)
                profile_row = {"profile": profile, "command": command, "exit": code,
                               "success": code == 0}
                if code == 0:
                    doc_dir = case_dir / "class-source" / class_name / profile
                    doc_dir.mkdir(parents=True)
                    json_path = doc_dir / "class-source.json"
                    json_path.write_bytes(stdout)
                    try:
                        doc = json.loads(stdout)
                        source_bytes = doc["text"].encode("utf-8")
                        source_path = doc_dir / f"{class_name}.java"
                        source_path.write_bytes(source_bytes)
                        profile_row["class_source_json"] = file_record(json_path)
                        profile_row["generated_source"] = file_record(source_path)
                        profile_row["source_text_sha256"] = sha(source_bytes)
                        profile_text[profile] = source_bytes
                        inventory = inspect_class_source(doc, class_name, require_source_maps=(profile == "all"))
                        inventory_rows[profile] = inventory
                        profile_row["physical_inventory"] = {
                            "fields": inventory["fields"],
                            "methods": [
                                {key: value for key, value in method.items() if key != "source_map"}
                                for method in inventory["method_source_maps"]
                            ],
                            "physical_inventory_matches": inventory["physical_inventory_matches"],
                            "no_clinit": inventory["no_clinit"],
                            "source_maps_complete": inventory["source_maps_complete"],
                            "source_maps_required": inventory["source_maps_required"],
                            "method_quality_and_execution_complete": inventory["method_quality_and_execution_complete"],
                            "execution_complete": inventory["execution_complete"],
                        }
                        if profile == "all":
                            # Keep the full extracted map separately in the manifest and the raw JSON above.
                            profile_row["method_source_maps"] = inventory["method_source_maps"]
                            all_sources.append(source_path)
                        profile_row["inventory_success"] = inventory["success"]
                        profile_row["success"] = profile_row["success"] and inventory["success"]
                    except Exception as error:
                        profile_row["success"] = False
                        profile_row["document_error"] = type(error).__name__ + ": " + str(error)
                profiles[profile] = profile_row
                render_success &= profile_row["success"]
            text_equal = "default" in profile_text and "all" in profile_text and profile_text["default"] == profile_text["all"]
            inventory_equal = all(inventory_rows.get(profile, {}).get("physical_inventory_matches", False)
                                  for profile in ("default", "all"))
            if not text_equal:
                failures.append(f"{label}: {class_name} default/all source text differs")
            if not inventory_equal:
                failures.append(f"{label}: {class_name} default/all physical inventory differs")
            class_rows.append({"class": class_name, "original_class": file_record(class_file),
                               "profiles": [profiles.get(profile, {"profile": profile, "success": False})
                                            for profile in ("default", "all")],
                               "default_all_text_equal": text_equal,
                               "physical_inventory_matches": inventory_equal,
                               "success": all(profiles.get(profile, {}).get("success", False)
                                              for profile in ("default", "all")) and text_equal})
        packages = {package_of(path) for path in all_sources}
        package = next(iter(packages)) if len(packages) == 1 else None
        case = {"label": label, "kind": "jarde", "jdk_leg": jdk_name,
                "rendered_classes": class_rows, "render_success": render_success and len(all_sources) == 2,
                "package_set": sorted(packages, key=lambda value: value or ""),
                "package_set_consistent": len(packages) == 1,
                "cli_render_count": 4, "cli_render_count_expected": 4}
        if len(all_sources) == 2 and len(packages) == 1:
            compile_sources, runner_copy, runner_class = copy_sources(all_sources, RUNNER, package, case_dir)
            compiled, runtime, class_files = compile_run(recorder, label, compile_sources, legs[jdk_name], runner_class)
            set_row = source_set_check(case_dir / "classes", class_files, package)
            case.update(compiled)
            case.update({"runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                         "rendered_sources": [file_record(path) for path in all_sources],
                         "class_output_set": set_row})
            actual_runtime = ({"exit": runtime["exit"], "stdout": runtime["stdout"], "stderr": runtime["stderr"]}
                              if runtime is not None else None)
            case["runtime_matches_same_jdk_original_raw"] = actual_runtime is not None and actual_runtime == originals[jdk_name]
            case["success"] = bool(case["render_success"] and case["compile_success"] and case["runtime_success"]
                                   and case["runtime_matches_same_jdk_original_raw"] and set_row["complete"])
        else:
            case.update({"compile_success": False, "runtime_success": False,
                         "runtime_matches_same_jdk_original_raw": False, "success": False})
        if not case["success"]:
            failures.append(label)
        cases.append(case)

    expected_counts = {"original": 2, "jadx": 4, "jarde": 2}
    case_counts = {kind: sum(case.get("kind") == kind for case in cases) for kind in expected_counts}
    success_counts = {kind: sum(case.get("kind") == kind and case.get("success", False) for case in cases)
                      for kind in expected_counts}
    if case_counts != expected_counts:
        failures.append({"expected_case_counts": expected_counts, "actual_case_counts": case_counts})
    original_cross_jdk_equal = originals.get("javac8") is not None and originals.get("javac23") is not None and originals["javac8"] == originals["javac23"]
    if not original_cross_jdk_equal:
        failures.append("original cross-JDK raw runtime triples differ")
    cli_render_count = sum(len(command["argv"]) > 1 and command["argv"][1] == "class-source"
                           for command in recorder.commands)
    if cli_render_count != 8:
        failures.append({"expected_jarde_cli_render_count": 8, "actual_jarde_cli_render_count": cli_render_count})

    manifest = {
        "schema": "no-clinit-super-args-baseline-v1",
        "status": "completed" if not failures else "baseline-with-failures",
        "claim_boundary": "Two focused complete Java classes with two distinct direct-super constructor paths; not a broader initializer-suite claim.",
        "oracle": "Fresh original complete three-class runs (two targets plus Runner) on each JDK; JADX and Jarde replays compare exit/stdout/stderr byte-for-byte to the same-JDK original.",
        "source_files": source_records,
        "source_inventory_expected": {
            name: {"fields": sorted(facts["fields"]), "methods": sorted(facts["methods"])}
            for name, facts in EXPECTED_PHYSICAL.items()
        },
        "cli_metadata": {"path": str(metadata_path), "sha256": metadata_hash,
                         "candidate_pin_count": len(candidate_pins), "test_pin_count": len(test_pins),
                         "canonical_pin_count": len(canonical_pins)},
        "frozen_cli": {"path": str(cli_path), "sha256": cli_hash},
        "candidate_sources": candidate_pins,
        "test_sources": test_pins,
        "canonical_files": canonical_pins,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
        "jadx": {"path": str(JADX.resolve()), "version": JADX_VERSION, "sha256": jadx_hash,
                 "version_command": version_command, "jar": file_record(jar_path), "jar_members": jar_members,
                 "profiles": [decompilations[profile]["row"] for profile in ("default", "none")]},
        "cases_expected": expected_counts,
        "case_counts": case_counts,
        "success_counts": success_counts,
        "jarde_cli_render_count": cli_render_count,
        "jarde_cli_render_count_expected": 8,
        "jarde_cli_evidence_policy": "For each target and JDK, save default and --evidence all JSON; require identical source text and complete physical method source maps from all mode.",
        "original_cross_jdk_raw_equal": original_cross_jdk_equal,
        "environment_policy": {"removed_for_each_process": list(STRIPPED_ENV),
                                "JDK_HOME_and_PATH_set_for_JDK_processes": True,
                                "compile_classpath_and_sourcepath": "empty per leg",
                                "runtime": "new classes only with -Xverify:all"},
        "preflight": preflight,
        "commands": recorder.commands,
        "cases": cases,
        "failures": failures,
        "script": file_record(Path(__file__)),
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                           "excludes": ["file-inventory.json"]},
    }
    write_json(OUT / "manifest.json", manifest)
    add_inventory(OUT, OUT / "file-inventory.json")
    print(json.dumps({"status": manifest["status"], "case_counts": case_counts,
                      "success_counts": success_counts, "original_cross_jdk_raw_equal": original_cross_jdk_equal,
                      "jarde_cli_render_count": cli_render_count, "failures": failures,
                      "manifest": str(OUT / "manifest.json"), "inventory": str(OUT / "file-inventory.json")},
                     ensure_ascii=False, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"baseline preparation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise
