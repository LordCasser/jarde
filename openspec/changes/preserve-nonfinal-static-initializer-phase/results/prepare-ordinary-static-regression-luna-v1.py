#!/usr/bin/env python3
"""Prepare a fresh ordinary non-final static-initializer regression replay.

This file is preparation only. Root should review it before running it; it
creates a new ordinary-static-regression-root-v1 directory and refuses overwrite.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
OUT = HERE / "ordinary-static-regression-root-v1"
SOURCE_FILE = ROOT / "src/enum_constants.rs"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_METADATA = HERE / "candidate-cli-v1.json"
CLI_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_PATH = Path("/private/tmp/jarde-nonfinal-static-cli-v1")
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
CLASS_NAME = "OrdinaryInit"
RUNNER_NAME = "Runner"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")

EXPECTED_SOURCE = '''class OrdinaryInit {
  static int first;
  static int second;

  static { first = 4; second = twice(); }

  static int twice() { return first * 2; }
}
'''

RUNNER_SOURCE = '''public class Runner {
  public static void main(String[] args) {
    System.out.println("first=" + OrdinaryInit.first);
    System.out.println("second=" + OrdinaryInit.second);
    System.out.println("twice=" + OrdinaryInit.twice());
    System.out.println("first=" + OrdinaryInit.first);
    System.out.println("second=" + OrdinaryInit.second);
    System.out.println("twice=" + OrdinaryInit.twice());
  }
}
'''

COMMANDS = []


def sha(data):
    return hashlib.sha256(data).hexdigest()


def record(path):
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def run(label, argv, home=None):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home is not None:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    result = subprocess.run([str(value) for value in argv], cwd=ROOT, env=env,
                            capture_output=True, check=False)
    streams = {}
    for name, content in (("stdout", result.stdout), ("stderr", result.stderr)):
        path = OUT / "streams" / f"{label}.{name}"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        streams[name] = record(path)
    item = {
        "label": label,
        "argv": [str(value) for value in argv],
        "cwd": str(ROOT),
        "java_home": str(home) if home is not None else None,
        "exit": result.returncode,
        **streams,
    }
    COMMANDS.append(item)
    return result, item


def extract_frozen_source():
    text = SOURCE_FILE.read_text()
    test_start = text.index("fn ordinary_class_fields_and_static_initializer_keep_the_existing_projection()")
    source_start = text.index('const SOURCE: &str = r#"', test_start) + len('const SOURCE: &str = r#"')
    source_end = text.index('"#;', source_start)
    source = text[source_start:source_end]
    if source != EXPECTED_SOURCE:
        raise ValueError("OrdinaryInit SOURCE no longer matches the frozen expected text")
    return source


def decode_raw(value):
    return bytes(value).decode("utf-8", "replace")


def method_key(method):
    item = method.get("item", {})
    return decode_raw(item.get("name", {}).get("raw", [])), decode_raw(item.get("descriptor", {}).get("raw", []))


def javap_clinit_putstatic(javap_text):
    lines = javap_text.splitlines()
    code_indices = [index for index, line in enumerate(lines) if line.strip() == "Code:"]
    for ordinal, code_index in enumerate(code_indices):
        prior = code_indices[ordinal - 1] if ordinal else -1
        descriptor_index = next((index for index in range(code_index - 1, prior, -1)
                                 if re.match(r"^\s*descriptor:\s*\S+\s*$", lines[index])), None)
        if descriptor_index is None or not re.search(r"^\s*descriptor:\s*\(\)V\s*$", lines[descriptor_index]):
            continue
        header = lines[descriptor_index - 1].strip() if descriptor_index else ""
        if not re.search(r"\bstatic\s*\{\};$", header):
            continue
        end = code_indices[ordinal + 1] if ordinal + 1 < len(code_indices) else len(lines)
        writes = []
        for line in lines[code_index + 1:end]:
            match = re.match(r"^\s*(\d+):\s+putstatic\b.*//\s*Field\s+([^:]+):(\S+)\s*$", line)
            if match:
                writes.append({"bci": int(match.group(1)), "owner_field": match.group(2), "descriptor": match.group(3)})
        return writes
    return None


def source_map_bcis(clinit_report, expected_identity):
    bcis = set()
    for segment in clinit_report.get("source_map", {}).get("segments", []):
        origin = segment.get("origin", {})
        for item in [origin.get("primary", {}), *origin.get("derived", [])]:
            method = item.get("method", {})
            if (method == expected_identity
                    and decode_raw(method.get("name", {}).get("raw", [])) == "<clinit>"
                    and decode_raw(method.get("descriptor", {}).get("raw", [])) == "()V"):
                bci = item.get("bci")
                if isinstance(bci, int):
                    bcis.add(bci)
    return bcis


def package_of(text):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", text, re.M)
    return match.group(1) if match else None


def check_document(document, javap_text):
    checks = {}
    fields = document.get("fields", [])
    names = [decode_raw(field.get("item", {}).get("name", {}).get("raw", [])) for field in fields]
    proof = document.get("initializer_proof", {})
    proof_fields = proof.get("fields", [])
    proof_names = [names[item["field_index"]] for item in proof_fields
                   if isinstance(item.get("field_index"), int)
                   and 0 <= item["field_index"] < len(names)]
    checks["initializer_proof_is_proved"] = proof.get("kind") == "proved"
    checks["initializer_proves_exactly_first_second"] = len(proof_fields) == 2 and proof_names == ["first", "second"]
    checks["proved_fields_are_nonfinal_static"] = len(proof_fields) == 2 and all(
        isinstance(item.get("field_index"), int)
        and 0 <= item["field_index"] < len(fields)
        and (fields[item["field_index"]].get("item", {}).get("access_flags", 0) & 0x0008) != 0
        and (fields[item["field_index"]].get("item", {}).get("access_flags", 0) & 0x0010) == 0
        for item in proof_fields
    )
    source = document.get("text", "")
    checks["root_has_no_static_block"] = re.search(r"^\s*static\s*\{", source, re.M) is None
    checks["text_keeps_ordered_nonfinal_declarations"] = (
        source.find("static int first = 4;") >= 0
        and source.find("static int second = twice();") > source.find("static int first = 4;")
    )

    methods = document.get("methods", [])
    clinit = next((method for method in methods if method_key(method) == ("<clinit>", "()V")), None)
    checks["physical_clinit_member_retained"] = clinit is not None
    clinit_report = clinit.get("outcome", {}).get("report", {}) if clinit else {}
    writes = javap_clinit_putstatic(javap_text)
    checks["physical_clinit_has_exactly_two_target_writes"] = (
        writes is not None
        and [(item["owner_field"], item["descriptor"]) for item in writes]
        == [("OrdinaryInit.first", "I"), ("OrdinaryInit.second", "I")]
    )
    clinit_identity = clinit.get("item", {}).get("identity") if clinit else None
    mapped = source_map_bcis(clinit_report, clinit_identity)
    checks["both_physical_clinit_writes_have_source_map_origins"] = (
        writes is not None and len(writes) == 2 and all(item["bci"] in mapped for item in writes)
    )
    checks["clinit_report_is_structured_with_source_map"] = (
        clinit_report.get("quality") == "structured"
        and clinit_report.get("representation") == "java"
        and bool(clinit_report.get("source_map", {}).get("segments"))
    )
    return checks


def compile_and_run(label, source_paths, tools, home, class_name):
    case_dir = OUT / "cases" / label
    case_dir.mkdir(parents=True, exist_ok=True)
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True)
    classes.mkdir()
    compiled, compile_record = run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *source_paths,
    ], home)
    runtime = None
    runtime_record = None
    if compiled.returncode == 0:
        runtime, runtime_record = run(label + "-run", [tools["java"], "-Xverify:all", "-cp", classes, class_name], home)
    class_files = sorted(classes.rglob("*.class"))
    return {
        "compile": compile_record,
        "runtime": runtime_record,
        "source_files": [record(path) for path in source_paths],
        "classes": [record(path) for path in class_files],
        "compile_success": compiled.returncode == 0,
        "runtime_success": runtime is not None and runtime.returncode == 0,
    }, runtime, class_files


def same_raw(actual, oracle):
    return (actual is not None and oracle is not None
            and actual.returncode == oracle.returncode
            and actual.stdout == oracle.stdout and actual.stderr == oracle.stderr)


def write_closed_evidence(manifest):
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    inventory_path = OUT / "file-inventory.json"
    rows = [record(path) for path in sorted(OUT.rglob("*"))
            if path.is_file() and path != inventory_path]
    inventory_path.write_text(json.dumps(rows, indent=2) + "\n")


def main():
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite ordinary static regression evidence: {OUT}")
    source = extract_frozen_source()
    OUT.mkdir(parents=True)
    (OUT / "original-sources").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "cases").mkdir()

    source_path = OUT / "original-sources/OrdinaryInit.java"
    runner_path = OUT / "original-sources/Runner.java"
    source_path.write_text(source)
    runner_path.write_text(RUNNER_SOURCE)

    preflight = []
    metadata_bytes = CLI_METADATA.read_bytes() if CLI_METADATA.is_file() else b""
    metadata = json.loads(metadata_bytes) if metadata_bytes else {}
    actual_metadata_sha = sha(metadata_bytes) if metadata_bytes else None
    preflight.append({"label": "frozen-cli-metadata", "path": str(CLI_METADATA),
                      "expected_sha256": CLI_METADATA_SHA256, "actual_sha256": actual_metadata_sha,
                      "ok": actual_metadata_sha == CLI_METADATA_SHA256})
    actual_cli_sha = sha(CLI_PATH.read_bytes()) if CLI_PATH.is_file() else None
    preflight.append({"label": "frozen-cli", "path": str(CLI_PATH),
                      "expected_sha256": CLI_SHA256, "actual_sha256": actual_cli_sha,
                      "ok": actual_cli_sha == CLI_SHA256 and metadata.get("cli_path") == str(CLI_PATH)
                      and metadata.get("cli_sha256") == CLI_SHA256})
    manifest_bytes = JDK_MANIFEST.read_bytes() if JDK_MANIFEST.is_file() else b""
    actual_manifest_sha = sha(manifest_bytes) if manifest_bytes else None
    preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": actual_manifest_sha,
                      "ok": actual_manifest_sha == JDK_MANIFEST_SHA256})
    jdk_manifest = json.loads(manifest_bytes) if manifest_bytes else {"legs": []}

    legs = []
    for frozen in jdk_manifest.get("legs", []):
        tools = {}
        for name, fact in frozen.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact["sha256"]
            preflight.append({"label": frozen["leg"] + ":" + name, "path": str(path),
                              "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": ok})
            tools[name] = path
        if {"java", "javac", "javap"} <= set(tools):
            legs.append({"name": frozen["leg"], "home": tools["java"].parent.parent, "tools": tools})
    preflight.append({"label": "two-required-jdk-legs",
                      "ok": {leg["name"] for leg in legs} == {"javac8", "javac23"}})

    (OUT / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
    failures = [item["label"] for item in preflight if not item.get("ok")]
    cases = []
    oracles = {}
    original_classes = {}
    rendered_texts = {}
    if not failures:
        for leg in legs:
            name, home, tools = leg["name"], leg["home"], leg["tools"]
            for tool in ("java", "javac", "javap"):
                run(name + "-" + tool + "-version", [tools[tool], "-version"], home)
            item = {"label": name + "-original", "kind": "original", "jdk_leg": name}
            result, runtime, class_files = compile_and_run(item["label"], [source_path, runner_path], tools, home, RUNNER_NAME)
            item.update(result)
            item["success"] = bool(item["compile_success"] and item["runtime_success"])
            class_path = next((path for path in class_files if path.name == CLASS_NAME + ".class"), None)
            if class_path is not None:
                original_classes[name] = class_path
                javap, item["javap"] = run(name + "-original-javap", [tools["javap"], "-p", "-c", "-s", class_path], home)
                item["javap_success"] = javap.returncode == 0
                item["javap_text"] = javap.stdout.decode("utf-8", "replace")
            if runtime is not None:
                oracles[name] = runtime
            cases.append(item)
            if not item["success"] or not item.get("javap_success", False):
                failures.append(item["label"])

        for leg in legs:
            name, home, tools = leg["name"], leg["home"], leg["tools"]
            source_class = original_classes.get(name)
            original_javap = next((case.get("javap_text") for case in cases
                                   if case.get("label") == name + "-original"), None)
            if source_class is None or original_javap is None:
                failures.append(name + "-render-blocked-original-class")
                continue
            for profile in ("default", "all"):
                label = name + "-" + profile
                case_dir = OUT / "cases" / label
                case_dir.mkdir()
                argv = [CLI_PATH, "class-source", "--input", source_class,
                        "--class", CLASS_NAME, "--policy", "single-class", "--release", "8", "--format", "json"]
                if profile == "all":
                    argv += ["--evidence", "all"]
                rendered, render_record = run(label + "-render", argv)
                item = {"label": label, "kind": "rendered", "profile": profile, "jdk_leg": name,
                        "render": render_record, "render_success": rendered.returncode == 0, "success": False}
                if rendered.returncode == 0:
                    document_path = case_dir / "class-source.json"
                    document_path.write_bytes(rendered.stdout)
                    item["document"] = record(document_path)
                    try:
                        document = json.loads(rendered.stdout)
                        item["text_sha256"] = sha(document.get("text", "").encode("utf-8"))
                        rendered_texts[(name, profile)] = document.get("text", "")
                        item["checks"] = check_document(document, original_javap)
                        generated_path = case_dir / "OrdinaryInit.java"
                        generated_path.write_bytes(document["text"].encode("utf-8"))
                        package = package_of(document["text"])
                        generated_runner = case_dir / "Runner.java"
                        generated_runner.write_text((f"package {package};\n\n" if package else "") + RUNNER_SOURCE)
                        item["generated_text_unchanged"] = generated_path.read_bytes() == document["text"].encode("utf-8")
                        item["runner_package"] = package
                        runner_class = (package + "." if package else "") + RUNNER_NAME
                        compiled, runtime, class_files = compile_and_run(label, [generated_path, generated_runner], tools, home, runner_class)
                        item.update(compiled)
                        item["runtime_matches_original_raw"] = same_raw(runtime, oracles.get(name))
                        item["success"] = bool(
                            item["render_success"] and item["generated_text_unchanged"]
                            and item["compile_success"] and item["runtime_success"]
                            and item["runtime_matches_original_raw"] and all(item["checks"].values())
                        )
                    except Exception as error:
                        item["document_error"] = type(error).__name__ + ": " + str(error)
                cases.append(item)
                if not item["success"]:
                    failures.append(label)

    text_values = list(rendered_texts.values())
    check_text_equal = len(rendered_texts) == 4 and len(set(text_values)) == 1
    if not check_text_equal:
        failures.append("all-four-rendered-texts-identical")

    manifest = {
        "schema": "ordinary-static-regression-root-v1",
        "status": "completed" if not failures else "completed-with-failures",
        "scope": "OrdinaryInit from src/enum_constants.rs, source/target 8, JDK 8 and 23; default and evidence-all class-source renders",
        "claim_boundary": "Public rendering and full generated-source runtime comparison only; no internal rollback claim.",
        "preparation_script": record(Path(__file__).resolve()),
        "source": record(source_path),
        "runner": record(runner_path),
        "cli": {"path": str(CLI_PATH), "sha256": actual_cli_sha,
                "metadata_path": str(CLI_METADATA), "metadata_sha256": actual_metadata_sha},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": actual_manifest_sha},
        "preflight": preflight,
        "commands": COMMANDS,
        "cases": cases,
        "cross_profile_text_equal": check_text_equal,
        "failures": failures,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "preflight.json"],
                           "excludes": ["file-inventory.json"]},
    }
    write_closed_evidence(manifest)
    if failures:
        print(json.dumps({"status": manifest["status"], "failures": failures, "output": str(OUT)}, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps({"status": manifest["status"], "case_count": len(cases), "output": str(OUT)}, indent=2))


if __name__ == "__main__":
    main()
