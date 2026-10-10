#!/usr/bin/env python3
"""Prepare a deterministic complete-class one-armed-loop comparison; root runs it after review."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import zipfile
from datetime import datetime, timezone

from blake3 import blake3


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
INPUTS = HERE / "inputs-prepared-luna-v1"
SOURCE = INPUTS / "PlainOneArmLoops.java"
RUNNER = INPUTS / "Runner.java"
OUT = HERE / "baseline-root-v1"

JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
EXPECTED_JDK_LEGS = ("javac8", "javac23")
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
CLI = Path("/private/tmp/jarde-field-multiply-cli-v2")
CLI_SHA256 = "b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59"
METADATA = ROOT / "openspec/changes/recover-int-field-multiply-updates/results/candidate-cli-v2.json"
METADATA_SHA256 = "f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8"
SOURCE_SHA256 = "8f5ccc668017113caf93002d3e07853206412ca581a23dfb98950ad7393805b4"
RUNNER_SHA256 = "240b28968c2a0f466660e2b191b6c08cc801e7024362c0dccbe2f47427bc2f9b"
CLASS_NAME = "PlainOneArmLoops"
METHOD_KEYS = (
    ("<init>", "()V"),
    ("prefixWhile", "(ZI)I"),
    ("noPrefix", "(ZI)I"),
    ("loopAndTail", "(ZI)I"),
    ("takenArm", "(ZI)I"),
)
METHOD_FLAGS = {"<init>()V": 0x0001, "prefixWhile(ZI)I": 0x0009,
                "noPrefix(ZI)I": 0x0009, "loopAndTail(ZI)I": 0x0009,
                "takenArm(ZI)I": 0x0009}
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def b3(data: bytes) -> str:
    return blake3(data).hexdigest()


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def file_record(path: Path) -> dict:
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def inventory_rows() -> list[dict]:
    return [file_record(path) for path in sorted(OUT.rglob("*"))
            if path.is_file() and path != OUT / "file-inventory.json"]


def package_of_text(text: str) -> str | None:
    match = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", text)
    return match.group(1) if match else None


def adapt_runner(package: str | None, destination: Path) -> Path:
    original = RUNNER.read_text(encoding="utf-8")
    package_match = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", original)
    original_package = package_match.group(1) if package_match else None
    if original_package not in (None, package):
        raise RuntimeError("Runner has an incompatible existing package")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if package != original_package:
        original = f"package {package};\n\n" + original if package else original
    destination.write_text(original, encoding="utf-8")
    return destination


class Recorder:
    def __init__(self) -> None:
        self.commands: list[dict] = []

    def run(self, label: str, argv, home: Path | None = None):
        argv = [str(value) for value in argv]
        env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
        if home is not None:
            env["JAVA_HOME"] = str(home)
            env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
        started = datetime.now(timezone.utc).isoformat()
        tick = time.monotonic()
        try:
            result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, check=False)
            exit_code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except OSError as error:
            exit_code, stdout = 127, b""
            stderr = f"{type(error).__name__}: {error}".encode("utf-8", errors="replace")
        streams = {}
        for stream, payload in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{stream}.raw"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            streams[stream] = file_record(path)
        row = {"label": label, "argv": argv, "cwd": str(ROOT),
               "java_home": str(home) if home else None,
               "started_at": started, "duration_seconds": time.monotonic() - tick,
               "exit": exit_code, "streams": streams}
        if any(command["label"] == label for command in self.commands):
            raise RuntimeError(f"duplicate command label: {label}")
        self.commands.append(row)
        return exit_code, stdout, stderr, row


def class_files(directory: Path) -> list[Path]:
    return sorted(path for path in directory.rglob("*.class") if path.is_file())


def compile_run(recorder: Recorder, label: str, sources: list[Path], leg: dict,
                runner_class: str) -> tuple[dict, dict | None, list[Path]]:
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True, exist_ok=True)
    classes.mkdir(parents=True, exist_ok=True)
    tools, home = leg["tools"], leg["home"]
    code, _, _, compile_command = recorder.run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *sources,
    ], home)
    runtime_command, runtime = None, None
    if code == 0:
        run_code, stdout, stderr, runtime_command = recorder.run(
            label + "-run", [tools["java"], "-Xverify:all", "-cp", classes, runner_class], home)
        runtime = {"exit": run_code, "stdout": stdout, "stderr": stderr}
    outputs = class_files(classes)
    package = runner_class.rpartition(".")[0]
    prefix = package.replace(".", "/") + "/" if package else ""
    expected = {prefix + CLASS_NAME + ".class", prefix + "Runner.class"}
    actual = {path.relative_to(classes).as_posix() for path in outputs}
    row = {
        "label": label, "compile": compile_command, "runtime": runtime_command,
        "source_files": [file_record(path) for path in sources],
        "classes": [file_record(path) for path in outputs],
        "expected_class_paths": sorted(expected), "actual_class_paths": sorted(actual),
        "class_set_exact": actual == expected,
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
        "compile_success": code == 0,
        "runtime_success": runtime is not None and runtime["exit"] == 0,
    }
    return row, runtime, outputs


def parse_javap(text: str) -> dict:
    flag_bits = {
        "ACC_PUBLIC": 0x0001, "ACC_PRIVATE": 0x0002, "ACC_PROTECTED": 0x0004,
        "ACC_STATIC": 0x0008, "ACC_FINAL": 0x0010, "ACC_SYNCHRONIZED": 0x0020,
        "ACC_VOLATILE": 0x0040, "ACC_TRANSIENT": 0x0080, "ACC_NATIVE": 0x0100,
        "ACC_INTERFACE": 0x0200, "ACC_ABSTRACT": 0x0400, "ACC_STRICT": 0x0800,
        "ACC_SYNTHETIC": 0x1000, "ACC_ANNOTATION": 0x2000, "ACC_ENUM": 0x4000,
    }
    fields, methods = [], []
    current = None
    in_body = False

    def finish(row):
        if row is None:
            return
        if row["descriptor"] is None:
            raise RuntimeError(f"javap member has no descriptor: {row['declaration']}")
        (methods if row["kind"] == "method" else fields).append(row)

    for line in text.splitlines():
        if not in_body:
            if line == "{":
                in_body = True
            continue
        if line == "}":
            finish(current)
            current = None
            break
        declaration = re.fullmatch(r"  (\S.*);", line)
        if declaration:
            finish(current)
            value = declaration.group(1)
            is_method = "(" in value
            head = value.split("(", 1)[0] if is_method else value
            name = head.split()[-1]
            if is_method and name.rsplit(".", 1)[-1] == CLASS_NAME:
                name = "<init>"
            current = {"kind": "method" if is_method else "field", "declaration": value,
                       "name": name, "descriptor": None, "flags": 0, "bcis": []}
            continue
        if current is None:
            continue
        descriptor = re.fullmatch(r"\s+descriptor: (\S+)\s*", line)
        if descriptor:
            current["descriptor"] = descriptor.group(1)
            continue
        flag_match = re.fullmatch(r"\s+flags:\s*(.*?)\s*", line)
        if flag_match:
            flags = re.findall(r"ACC_[A-Z_]+", flag_match.group(1))
            current["flags"] = sum(flag_bits.get(flag, 0) for flag in flags)
            continue
        if line.strip() == "Code:":
            current["in_code"] = True
            continue
        if current.get("in_code"):
            bci = re.match(r"\s*(\d+):\s+", line)
            if bci:
                current["bcis"].append(int(bci.group(1)))
    result = {}
    for method in methods:
        key = (method["name"], method["descriptor"])
        if key in result:
            raise RuntimeError(f"duplicate javap method: {key}")
        result[key] = {"flags": method["flags"], "bcis": method["bcis"],
                       "declaration": method["declaration"]}
    return {"fields": fields, "methods": result}


def method_key(item: dict) -> tuple[str, str]:
    identity = item["identity"]["member"] if "member" in item["identity"] else item["identity"]
    return (bytes(identity["name"]).decode("utf-8"), bytes(identity["descriptor"]).decode("ascii"))


def verify_report_map(document: dict, original_bytes: bytes, javap: dict, mode: str) -> dict:
    owner = document["class"]
    original_b3 = b3(original_bytes)
    assert owner["class_bytes"] == {"digest": original_b3, "length": len(original_bytes)}
    assert owner["location"]["kind"] == "standalone_root"
    assert owner["location"]["snapshot"] == original_b3
    assert owner["variant"] == {"kind": "base"}
    assert document["execution"]["status"] == "complete"
    assert document.get("fields", []) == []
    methods = document.get("methods", [])
    assert len(methods) == len(METHOD_KEYS)
    seen, rows = set(), []
    for expected_index, method in enumerate(methods):
        item = method["item"]
        key = method_key(item)
        assert key in METHOD_KEYS and key not in seen
        seen.add(key)
        assert item["index"] == expected_index
        expected_physical = javap["methods"][key]
        assert item["access_flags"] == expected_physical["flags"] == METHOD_FLAGS[key[0] + key[1]]
        identity = item["identity"]
        assert identity["owner"] == owner
        assert set(identity) == {"owner", "name", "descriptor"}
        assert identity["name"] == list(key[0].encode("utf-8"))
        assert identity["descriptor"] == list(key[1].encode("ascii"))
        outcome = method["outcome"]
        method_fact = {"method": list(key), "method_index": item["index"],
                       "access_flags": item["access_flags"],
                       "declaration": expected_physical["declaration"],
                       "expected_javap_bcis": expected_physical["bcis"],
                       "outcome_kind": outcome["kind"], "report_present": False,
                       "quality": None, "representation": None, "content": None,
                       "fallbacks": None, "text_sha256": None,
                       "source_map_state": None, "source_map_segments": 0,
                       "source_map_origins": 0, "mapped_bcis": [],
                       "bci_coverage_complete": False,
                       "origin_bindings_and_bcis_valid": False,
                       "validation_errors": []}
        if outcome["kind"] != "recovered":
            method_fact["validation_errors"].append("method outcome is not recovered")
            rows.append(method_fact)
            continue
        report = outcome.get("report")
        if not isinstance(report, dict):
            method_fact["validation_errors"].append("recovered outcome has no report object")
            rows.append(method_fact)
            continue
        method_fact.update({"report_present": True, "quality": report.get("quality"),
                            "representation": report.get("representation"),
                            "content": report.get("content"),
                            "fallbacks": report.get("fallbacks"),
                            "text_sha256": sha(report.get("text", "").encode("utf-8"))})
        if report.get("outcome") != "produced":
            method_fact["validation_errors"].append("report outcome is not produced")
        if report.get("quality") != "structured":
            method_fact["validation_errors"].append("quality is not structured")
        if report.get("representation") != "java":
            method_fact["validation_errors"].append("representation is not java")
        if report.get("content") != "contains_statements":
            method_fact["validation_errors"].append("content is not contains_statements")
        if report.get("fallbacks") != []:
            method_fact["validation_errors"].append("fallback list is nonempty or absent")
        if report.get("execution", {}).get("status") != "complete":
            method_fact["validation_errors"].append("method execution status is not complete")

        categories = report.get("evidence", {}).get("categories", [])
        map_category = next((row for row in categories if row.get("kind") == "source_map"), None)
        map_state = map_category.get("state", {}).get("state") if map_category else None
        method_fact["source_map_state"] = map_state
        if map_state != "complete":
            method_fact["validation_errors"].append("source_map evidence state is not complete")
        segments = report.get("source_map", {}).get("segments", [])
        raw_text = report.get("text", "").encode("utf-8")
        mapped_bcis, origin_count, origins_valid = set(), 0, True
        for segment in segments:
            if not (0 <= segment.get("start", -1) < segment.get("end", -1) <= len(raw_text)):
                origins_valid = False
                method_fact["validation_errors"].append("source-map segment is outside report text")
            origin = segment.get("origin", {})
            points = ([origin["primary"]] if origin.get("primary") is not None else [])
            points.extend(origin.get("derived", []))
            for point in points:
                origin_count += 1
                origin_method = point.get("method", {})
                bci = point.get("bci")
                if (origin_method.get("owner") != owner
                        or bytes(origin_method.get("name", [])).decode("utf-8", errors="replace") != key[0]
                        or bytes(origin_method.get("descriptor", [])).decode("ascii", errors="replace") != key[1]
                        or bci not in expected_physical["bcis"]):
                    origins_valid = False
                    method_fact["validation_errors"].append("source-map origin does not bind to method owner/name/descriptor/javap BCI")
                if isinstance(bci, int):
                    mapped_bcis.add(bci)
        complete = mapped_bcis == set(expected_physical["bcis"])
        if not mapped_bcis:
            origins_valid = False
            method_fact["validation_errors"].append("source-map contains no mapped BCI origins")
        if not complete:
            method_fact["validation_errors"].append("mapped BCI set does not cover every javap instruction BCI")
        method_fact.update({"source_map_segments": len(segments), "source_map_origins": origin_count,
                            "mapped_bcis": sorted(mapped_bcis), "bci_coverage_complete": complete,
                            "origin_bindings_and_bcis_valid": origins_valid})
        rows.append(method_fact)
    assert seen == set(METHOD_KEYS)
    all_structured = all(row["quality"] == "structured" and row["representation"] == "java"
                         and row["content"] == "contains_statements" and row["fallbacks"] == []
                         and row["outcome_kind"] == "recovered" for row in rows)
    all_bci_covered = all(row["bci_coverage_complete"] for row in rows)
    all_origins_valid = all(row["origin_bindings_and_bcis_valid"] for row in rows)
    all_maps_complete = all(row["source_map_state"] == "complete" for row in rows)
    return {"input_owner_exact_b3_length_location_variant": True,
            "physical_field_count": len(javap["fields"]),
            "physical_method_count": len(javap["methods"]),
            "physical_methods": {name + desc: {"flags": facts["flags"], "bcis": facts["bcis"]}
                                 for (name, desc), facts in javap["methods"].items()},
            "source_map_facts": rows, "method_presentation_facts": rows,
            "all_methods_structured_java_statements_without_fallback": all_structured,
            "all_methods_bci_coverage_complete": all_bci_covered,
            "all_method_origins_bind_to_physical_bcis": all_origins_valid,
            "all_method_source_map_categories_complete": all_maps_complete,
            "all_profile_requires_complete_bci_coverage": mode == "all"}


def runtime_summary(runtime: dict | None) -> dict | None:
    if runtime is None:
        return None
    return {"exit": runtime["exit"], "stdout_sha256": sha(runtime["stdout"]),
            "stderr_sha256": sha(runtime["stderr"])}


if OUT.exists():
    raise SystemExit(f"refusing to overwrite evidence: {OUT}")
if not SOURCE.is_file() or not RUNNER.is_file():
    raise SystemExit("prepared source or Runner is missing")

OUT.mkdir(parents=True)
for dirname in ("cases", "streams", "original-sources", "jadx-input", "jadx-output", "inputs"):
    (OUT / dirname).mkdir()
shutil.copyfile(SOURCE, OUT / "original-sources/PlainOneArmLoops.java")
shutil.copyfile(RUNNER, OUT / "original-sources/Runner.java")
recorder = Recorder()
failures: list[str] = []
preflight = []

for label, path, expected in (("fixture-source", SOURCE, SOURCE_SHA256),
                              ("runner-source", RUNNER, RUNNER_SHA256)):
    actual = sha(path.read_bytes())
    preflight.append({"label": label, "path": str(path), "expected_sha256": expected,
                      "actual_sha256": actual, "ok": actual == expected})

jdk_raw = JDK_MANIFEST.read_bytes()
jdk_manifest = json.loads(jdk_raw)
preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                  "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": sha(jdk_raw),
                  "ok": sha(jdk_raw) == JDK_MANIFEST_SHA256 and jdk_manifest.get("status") == "complete"})
legs = {}
for frozen in jdk_manifest.get("legs", []):
    tools, ok = {}, True
    for name, fact in frozen.get("jdk_tools", {}).items():
        path = Path(fact["path"])
        actual = sha(path.read_bytes()) if path.is_file() else None
        good = actual == fact["sha256"]
        ok &= good
        preflight.append({"label": f"{frozen['leg']}:{name}", "path": str(path),
                          "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": good})
        tools[name] = path
    if all(name in tools for name in ("java", "javac", "javap")):
        legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent, "tools_ok": ok}
preflight.append({"label": "jdk-leg-set", "expected": list(EXPECTED_JDK_LEGS),
                  "actual": sorted(legs), "ok": set(legs) == set(EXPECTED_JDK_LEGS)})

metadata_bytes = METADATA.read_bytes() if METADATA.is_file() else b""
metadata = json.loads(metadata_bytes) if metadata_bytes else {}
cli_actual = sha(CLI.read_bytes()) if CLI.is_file() else None
metadata_ok = (sha(metadata_bytes) == METADATA_SHA256
               and metadata.get("cli_path") == str(CLI)
               and metadata.get("cli_sha256") == CLI_SHA256)
preflight.append({"label": "frozen-jarde-cli-v2", "path": str(CLI),
                  "expected_sha256": CLI_SHA256, "actual_sha256": cli_actual,
                  "metadata_path": str(METADATA), "expected_metadata_sha256": METADATA_SHA256,
                  "actual_metadata_sha256": sha(metadata_bytes) if metadata_bytes else None,
                  "metadata_cli_matches": metadata_ok,
                  "ok": cli_actual == CLI_SHA256 and metadata_ok})
jadx_resolved = JADX.resolve()
jadx_actual = sha(jadx_resolved.read_bytes()) if jadx_resolved.is_file() else None
preflight.append({"label": "fixed-jadx-launcher", "path": str(JADX),
                  "resolved_path": str(jadx_resolved), "expected_sha256": JADX_SHA256,
                  "actual_sha256": jadx_actual, "ok": jadx_actual == JADX_SHA256})
if any(not row["ok"] for row in preflight):
    failures.extend(row["label"] for row in preflight if not row["ok"])
    write_json(OUT / "preflight.json", preflight)
    write_json(OUT / "file-inventory.json", inventory_rows())
    raise SystemExit("fixed fixture/tool preflight failed; no compile, decompile, or candidate render was attempted")

cases, oracles, original_classes, javap_by_leg = [], {}, {}, {}
for leg_name in EXPECTED_JDK_LEGS:
    leg = legs[leg_name]
    for tool in ("java", "javac", "javap"):
        code, _, _, command = recorder.run(f"{leg_name}-{tool}-version", [leg["tools"][tool], "-version"], leg["home"])
        if code != 0:
            failures.append(command["label"])
    label = f"{leg_name}-original"
    case_dir = OUT / "cases" / label
    case_dir.mkdir()
    source_copy = case_dir / "PlainOneArmLoops.java"
    runner_copy = case_dir / "Runner.java"
    shutil.copyfile(SOURCE, source_copy)
    shutil.copyfile(RUNNER, runner_copy)
    case, runtime, outputs = compile_run(recorder, label, [source_copy, runner_copy], leg, "Runner")
    target = next((path for path in outputs if path.name == CLASS_NAME + ".class"), None)
    complete = {path.name for path in outputs} == {CLASS_NAME + ".class", "Runner.class"}
    case.update({"kind": "original", "jdk_leg": leg_name, "complete_class_set": complete})
    if target is not None and case["compile_success"]:
        original_classes[leg_name] = target
        code, stdout, _, javap_command = recorder.run(
            f"{label}-javap", [leg["tools"]["javap"], "-p", "-c", "-s", "-v", target], leg["home"])
        javap_text = stdout.decode("utf-8", errors="replace")
        parsed = parse_javap(javap_text)
        javap_path = case_dir / "javap.txt"
        javap_path.write_bytes(stdout)
        census_exact = (not parsed["fields"] and list(parsed["methods"]) == list(METHOD_KEYS)
                        and {key[0] + key[1]: value["flags"] for key, value in parsed["methods"].items()}
                        == METHOD_FLAGS and all(value["bcis"] for value in parsed["methods"].values()))
        case["javap"] = {"command": javap_command, "text": file_record(javap_path),
                         "physical_field_count": len(parsed["fields"]),
                         "physical_methods": {name + desc: {"flags": facts["flags"], "bcis": facts["bcis"],
                                                              "declaration": facts["declaration"]}
                                              for (name, desc), facts in parsed["methods"].items()},
                         "physical_members_exact": census_exact}
        case["actual_class"] = {**file_record(target), "blake3": b3(target.read_bytes())}
        javap_by_leg[leg_name] = parsed
        case["success"] = bool(case["compile_success"] and case["runtime_success"] and complete
                               and census_exact and code == 0)
    else:
        case["success"] = False
    if runtime is not None:
        if runtime["exit"] != 0 or runtime["stderr"]:
            failures.append(f"{label}: original Runner did not complete cleanly")
        oracles[leg_name] = runtime
        case["runtime_raw"] = runtime_summary(runtime)
    if not case["success"]:
        failures.append(label)
    cases.append(case)

# JADX consumes exactly the javac23 target class, without Runner or helper classes.
source_class = original_classes.get("javac23")
jar_path = OUT / "jadx-input/PlainOneArmLoops.class.jar"
jar_members = []
if source_class is not None:
    payload = source_class.read_bytes()
    with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as jar:
        info = zipfile.ZipInfo(CLASS_NAME + ".class", date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_STORED
        jar.writestr(info, payload)
    with zipfile.ZipFile(jar_path) as jar:
        jar_exact = (jar.namelist() == [CLASS_NAME + ".class"]
                     and jar.read(CLASS_NAME + ".class") == payload)
    jar_members.append({"name": CLASS_NAME + ".class", "bytes": len(payload),
                        "sha256": sha(payload), "blake3": b3(payload)})
    if not jar_exact:
        failures.append("JADX input JAR is not exactly the javac23 target class")
else:
    jar_exact = False
    failures.append("JADX input class unavailable")
    jar_path.write_bytes(b"")

jadx_version_code, jadx_version_out, _, jadx_version_command = recorder.run(
    "jadx-version", [JADX, "--version"], legs.get("javac23", {}).get("home"))
jadx_version_ok = jadx_version_code == 0 and jadx_version_out.strip() == JADX_VERSION.encode()
if not jadx_version_ok:
    failures.append("jadx-version")

decompiled = {}
for profile in ("default", "none"):
    output = OUT / "jadx-output" / profile
    argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
    if profile == "none":
        argv.extend(["--rename-flags", "none"])
    code, _, _, command = recorder.run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path],
                                        legs.get("javac23", {}).get("home"))
    sources = sorted(path for path in output.rglob("*.java") if path.is_file()) if output.exists() else []
    names = {path.name for path in sources}
    packages = {package_of_text(path.read_text(encoding="utf-8")) for path in sources}
    row = {"profile": profile, "command": command, "decompile_success": code == 0,
           "generated_sources": [file_record(path) for path in sources],
           "source_name_set_exact": names == {CLASS_NAME + ".java"},
           "package_set": sorted(packages, key=lambda value: value or ""),
           "package_set_single": len(packages) == 1, "input_jar_exact": jar_exact}
    if code != 0 or names != {CLASS_NAME + ".java"} or len(packages) != 1:
        failures.append(f"jadx-{profile}-decompile")
    decompiled[profile] = {"sources": sources,
                           "package": next(iter(packages)) if len(packages) == 1 else None,
                           "row": row}

for profile in ("default", "none"):
    decomp = decompiled[profile]
    for leg_name in EXPECTED_JDK_LEGS:
        label = f"{leg_name}-jadx-{profile}"
        if len(decomp["sources"]) != 1:
            cases.append({"label": label, "kind": "jadx", "profile": profile,
                          "jdk_leg": leg_name, "success": False,
                          "blocked": "generated source set incomplete"})
            failures.append(label)
            continue
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        source_copy = case_dir / (CLASS_NAME + ".java")
        shutil.copyfile(decomp["sources"][0], source_copy)
        runner_copy = adapt_runner(decomp["package"], case_dir / "Runner.java")
        runner_class = (decomp["package"] + "." if decomp["package"] else "") + "Runner"
        case, runtime, outputs = compile_run(recorder, label, [source_copy, runner_copy], legs[leg_name], runner_class)
        prefix = decomp["package"].replace(".", "/") + "/" if decomp["package"] else ""
        expected = {prefix + CLASS_NAME + ".class", prefix + "Runner.class"}
        exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == expected
        same = (runtime is not None and oracles.get(leg_name) is not None
                and runtime["exit"] == oracles[leg_name]["exit"]
                and runtime["stdout"] == oracles[leg_name]["stdout"]
                and runtime["stderr"] == oracles[leg_name]["stderr"])
        case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                     "generated_source": file_record(decomp["sources"][0]),
                     "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                     "decompilation": decomp["row"], "complete_class_set": exact,
                     "runtime_matches_same_jdk_original_raw": same,
                     "runtime_raw": runtime_summary(runtime),
                     "success": case["compile_success"] and case["runtime_success"] and exact and same})
        if not case["success"]:
            failures.append(label)
        cases.append(case)

# Render default and all from each original class; only Runner package adaptation is allowed.
render_rows = []
for leg_name in EXPECTED_JDK_LEGS:
    class_file = original_classes.get(leg_name)
    render_label = f"{leg_name}-jarde-render"
    render_dir = OUT / "cases" / render_label
    render_dir.mkdir()
    profile_rows, texts, documents = {}, {}, {}
    for mode in ("default", "all"):
        argv = [CLI, "class-source", "--input", class_file, "--class", CLASS_NAME,
                "--policy", "single-class", "--release", "8", "--format", "json"]
        if mode == "all":
            argv.extend(["--evidence", "all"])
        code, stdout, _, command = recorder.run(f"{render_label}-{mode}", argv, legs[leg_name]["home"])
        row = {"mode": mode, "command": command, "success": code == 0}
        if code == 0:
            document_path = render_dir / f"class-source-{mode}.json"
            document_path.write_bytes(stdout)
            try:
                document = json.loads(stdout)
                text = document["text"]
                text_path = render_dir / f"{CLASS_NAME}-{mode}.java"
                text_path.write_text(text, encoding="utf-8")
                map_facts = verify_report_map(document, class_file.read_bytes(), javap_by_leg[leg_name], mode)
                documents[mode], texts[mode] = document, text
                row.update({"document": file_record(document_path), "generated_text": file_record(text_path),
                            "physical_field_count": map_facts["physical_field_count"],
                            "physical_method_count": map_facts["physical_method_count"],
                            "physical_methods": map_facts["physical_methods"],
                            "input_owner_exact_b3_length_location_variant": map_facts["input_owner_exact_b3_length_location_variant"],
                            "source_map_facts": map_facts["source_map_facts"],
                            "method_presentation_facts": map_facts["method_presentation_facts"],
                            "all_methods_structured_java_statements_without_fallback":
                                map_facts["all_methods_structured_java_statements_without_fallback"],
                            "all_methods_bci_coverage_complete": map_facts["all_methods_bci_coverage_complete"],
                            "all_method_origins_bind_to_physical_bcis":
                                map_facts["all_method_origins_bind_to_physical_bcis"],
                            "all_method_source_map_categories_complete":
                                map_facts["all_method_source_map_categories_complete"],
                            "all_profile_requires_complete_bci_coverage":
                                map_facts["all_profile_requires_complete_bci_coverage"]})
            except Exception as error:
                row["success"] = False
                row["document_error"] = f"{type(error).__name__}: {error}"
        profile_rows[mode] = row
    text_equal = "default" in texts and "all" in texts and texts["default"] == texts["all"]
    render_rows.append({"label": render_label, "profiles": profile_rows,
                        "default_all_text_equal": text_equal})
    for mode in ("default", "all"):
        label = f"{leg_name}-jarde-{mode}"
        document = documents.get(mode)
        if document is None:
            profile = profile_rows[mode]
            cases.append({"label": label, "kind": "jarde", "jdk_leg": leg_name,
                          "evidence_mode": mode, "rendered_profile": profile,
                          "success": False, "blocked": "candidate source unavailable",
                          "candidate_observation": {
                              "source_rendered": profile["success"],
                              "method_facts_available": "method_presentation_facts" in profile,
                              "default_all_text_equal": text_equal,
                              "compile_attempted": False,
                              "runtime_attempted": False,
                              "interpretation": "Candidate render observation; original and JADX baseline status is independent."}})
            continue
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        generated = case_dir / (CLASS_NAME + ".java")
        generated.write_text(document["text"], encoding="utf-8")
        package = package_of_text(document["text"])
        runner_copy = adapt_runner(package, case_dir / "Runner.java")
        runner_class = (package + "." if package else "") + "Runner"
        compiled, runtime, outputs = compile_run(recorder, label, [generated, runner_copy],
                                                 legs[leg_name], runner_class)
        prefix = package.replace(".", "/") + "/" if package else ""
        expected = {prefix + CLASS_NAME + ".class", prefix + "Runner.class"}
        exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == expected
        same = (runtime is not None and oracles.get(leg_name) is not None
                and runtime["exit"] == oracles[leg_name]["exit"]
                and runtime["stdout"] == oracles[leg_name]["stdout"]
                and runtime["stderr"] == oracles[leg_name]["stderr"])
        case = {**compiled, "kind": "jarde", "jdk_leg": leg_name, "evidence_mode": mode,
                "rendered_profile": profile_rows[mode], "default_all_text_equal": text_equal,
                "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                "runtime_raw": runtime_summary(runtime),
                "success": profile_rows[mode]["success"] and text_equal and compiled["compile_success"]
                           and compiled["runtime_success"] and exact and same}
        # Jarde is the candidate under observation. A failed source render, compile,
        # or runtime is product evidence, not a failure of the collector/oracles.
        case["candidate_observation"] = {
            "source_rendered": profile_rows[mode]["success"],
            "all_methods_structured_java_statements_without_fallback":
                profile_rows[mode].get("all_methods_structured_java_statements_without_fallback"),
            "all_methods_bci_coverage_complete":
                profile_rows[mode].get("all_methods_bci_coverage_complete"),
            "all_method_origins_bind_to_physical_bcis":
                profile_rows[mode].get("all_method_origins_bind_to_physical_bcis"),
            "all_method_source_map_categories_complete":
                profile_rows[mode].get("all_method_source_map_categories_complete"),
            "method_results_by_name": {
                tuple(fact["method"])[0]: fact
                for fact in profile_rows[mode].get("method_presentation_facts", [])},
            "default_all_text_equal": text_equal,
            "compile_success": compiled["compile_success"],
            "compile_exit": compiled["compile"]["exit"],
            "runtime_attempted": runtime is not None,
            "runtime_matches_same_jdk_original_raw": same,
            "complete_class_set": exact,
            "interpretation": "Candidate behavior/presentation observation; does not change original or JADX baseline status."}
        cases.append(case)

case_counts = {kind: sum(row.get("kind") == kind for row in cases)
               for kind in ("original", "jadx", "jarde")}
success_counts = {kind: sum(row.get("kind") == kind and row.get("success", False) for row in cases)
                  for kind in ("original", "jadx", "jarde")}
expected_counts = {"original": 2, "jadx": 4, "jarde": 4}
if case_counts != expected_counts:
    failures.append(f"case counts differ: {case_counts}")
if any(success_counts[kind] != expected_counts[kind] for kind in ("original", "jadx")):
    failures.append(f"original/JADX baseline success counts differ: {success_counts}")
original_cross_jdk_raw_equal = (all(leg in oracles for leg in EXPECTED_JDK_LEGS)
                                and all(oracles["javac8"][key] == oracles["javac23"][key]
                                        for key in ("exit", "stdout", "stderr")))
if not original_cross_jdk_raw_equal:
    failures.append("original JDK runtime triples differ")

manifest = {
    "schema": "em23-one-arm-loop-controls-baseline-luna-v1",
    "status": "completed" if not failures else "baseline-with-failures",
    "claim_boundary": "One complete public class with four one-armed-if/while arrangements. Original and JADX are baseline controls; Jarde is a candidate observation. Jarde method presentation/BCI and complete-source compile/runtime outcomes are recorded separately, so a candidate compile failure is not treated as a passed candidate or as a failure of the original/JADX controls.",
    "oracle": "Fresh original complete class plus the same Runner on each fixed JDK; each reconstructed source leg is compared byte-for-byte against its same-JDK original exit/stdout/stderr.",
    "source": file_record(OUT / "original-sources/PlainOneArmLoops.java"),
    "runner": file_record(OUT / "original-sources/Runner.java"),
    "prepared_input_sha256": {"source": SOURCE_SHA256, "runner": RUNNER_SHA256},
    "original_stdout_javac23_raw": oracles.get("javac23", {}).get("stdout", b"").decode("utf-8", errors="replace"),
    "oracle_policy": "No output bytes are hard-coded; same-JDK original raw runtime triples are the oracle.",
    "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
    "jdk_legs": {name: {"home": str(legs[name]["home"]),
                        "tools": {tool: {"path": str(path), "sha256": sha(path.read_bytes())}
                                  for tool, path in legs[name]["tools"].items()}}
                 for name in legs},
    "frozen_jarde_cli": {"path": str(CLI), "sha256": cli_actual,
                         "metadata_path": str(METADATA), "metadata_sha256": METADATA_SHA256,
                         "metadata_not_copied_to_output": True},
    "jadx": {"launcher": str(JADX), "resolved_launcher": str(jadx_resolved),
             "sha256": jadx_actual, "expected_version": JADX_VERSION,
             "version_command": jadx_version_command,
             "input_jar": file_record(jar_path) if jar_path.is_file() else None,
             "jar_members": jar_members, "input_jar_exact": jar_exact,
             "profiles": [decompiled[profile]["row"] for profile in ("default", "none")]},
    "original_physical_facts": [row.get("javap") for row in cases if row.get("kind") == "original"],
    "render_profiles": render_rows,
    "case_counts": case_counts, "success_counts": success_counts,
    "candidate_observations": [row["candidate_observation"] | {"label": row["label"],
                                  "jdk_leg": row["jdk_leg"], "evidence_mode": row["evidence_mode"]}
                               for row in cases if row.get("kind") == "jarde"],
    "expected_case_counts": expected_counts,
    "original_cross_jdk_raw_equal": original_cross_jdk_raw_equal,
    "preflight": preflight, "commands": recorder.commands, "cases": cases,
    "failures": failures,
    "execution_policy": {"removed_environment": list(STRIPPED_ENV),
                         "complete_class_compile": ["-source", "8", "-target", "8", "-g:none"],
                         "empty_classpath_and_sourcepath": True, "runtime_verifier": "-Xverify:all",
                         "jadx_input_is_only_javac23_target_class": True,
                         "generated_target_sources_compiled_unmodified": True,
                         "only_runner_package_adaptation_allowed": True,
                         "jarde_default_and_all_texts_compiled_per_jdk": True,
                         "method_presentation_and_bci_facts_recorded_per_method": True,
                         "candidate_compile_or_runtime_failure_is_product_observation": True,
                         "all_profile_bci_coverage_required_for_candidate_acceptance": True},
    "prepared_script": file_record(Path(__file__)),
    "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
                        "excludes": ["file-inventory.json"]},
}
write_json(OUT / "manifest.json", manifest)
summary = {"schema": "em23-one-arm-loop-controls-summary-luna-v1",
           "status": manifest["status"], "case_counts": case_counts,
           "success_counts": success_counts,
           "original_cross_jdk_raw_equal": original_cross_jdk_raw_equal,
           "failures": failures}
write_json(OUT / "summary.json", summary)
write_json(OUT / "file-inventory.json", inventory_rows())
print(json.dumps(summary, ensure_ascii=False, indent=2))
if failures:
    raise SystemExit(1)
