#!/usr/bin/env python3
"""Replay the two frozen array-field baselines against a candidate CLI.

Preparation only: root supplies the frozen candidate CLI and metadata when it
is ready to run this script. Generated source is compiled exactly as rendered.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
OUT = Path(__file__).resolve().parent / "candidate-luna-v1"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
PRODUCT_PINS = {
    "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs",
    "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs",
    "src/facade.rs",
    "src/class_source.rs",
    "Cargo.lock",
}
FAMILIES = {
    "literal": {
        "directory": "array-field-initializers-literal",
        "class": "ArrayFieldLiteral",
        "static_fields": ["a"],
        "required_source": ["static byte[] a = new byte[]{10, 20, 30};"],
        "runner": "Runner",
        "baseline_manifest_sha256": "a021eedcde975dcb7bf7b7821d5097141b93292181015a45415752b21ddbafe1",
        "source_sha256": "184d4d068f785c75d5db8a30579284ed42817eaf91b012e55681581ab145a67b",
        "runner_sha256": "fd69838bf2a372d1bceb27b83fffc3b24b23a2ba1967bb51038e4473b1d4e163",
        "class_sha256": {
            "javac8": "6ead8bb0f3c466ba4dc10958a4cb349699f587bc9d5115f0656d0e2df60aa891",
            "javac23": "dc388f1ed9c0a44c9b87f9d61057b7a3f6c09fa1387b50d8f992a049e8544827",
        },
    },
    "ordered": {
        "directory": "array-field-initializers",
        "class": "ArrayFieldInitializers",
        "static_fields": ["trace", "before", "a", "after"],
        "required_source": [
            "static int trace = 0;", "static byte before = mark(1);",
            "static byte[] a = new byte[]{mark(2), mark(3), mark(4)};",
            "static byte after = mark(5);",
        ],
        "runner": "Runner",
        "baseline_manifest_sha256": "93f14c0b125a738378c18a7a1a1f63dc6fb8c86c8c349c879b8e0e567c7c89d5",
        "source_sha256": "8ed268bad1dff3025b1d02e3c95a26b06889c02a1c14f52cc953f8e944e61650",
        "runner_sha256": "b6541a755e1e9b6c2c4b7d601b14fcc2a27f820650ec968fe0f2d150f79a28cd",
        "class_sha256": {
            "javac8": "5bd4b86f40a268640dd391df54a050f44056918928ae240232ebed6671fa24ee",
            "javac23": "3660a28f19cb7304ddc9c952a89f1b43378834faf0251f0910770ba9f89a6789",
        },
    },
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_record(path):
    data = path.read_bytes()
    try:
        name = str(path.relative_to(OUT))
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def read_json(path):
    return json.loads(path.read_bytes())


def record_stream(label, name, content):
    path = OUT / "streams" / f"{label}.{name}"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(content)
    return file_record(path)


def run(label, argv, home=None):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    result = subprocess.run([str(value) for value in argv], cwd=ROOT, env=env,
                            capture_output=True, check=False)
    command = {
        "label": label, "argv": [str(value) for value in argv],
        "cwd": str(ROOT), "java_home": str(home) if home else None,
        "exit": result.returncode,
        "stdout": record_stream(label, "stdout", result.stdout),
        "stderr": record_stream(label, "stderr", result.stderr),
    }
    COMMANDS.append(command)
    return result, command


def generated_sources(folder):
    return sorted(path for path in folder.rglob("*.java") if path.is_file())


def package_of(path):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", path.read_text(), re.M)
    return match.group(1) if match else None


def compile_run(label, sources, tools, home, runner_name):
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True)
    classes.mkdir()
    compiled, compile_record = run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *sources,
    ], home)
    runtime_record = None
    runtime = None
    if compiled.returncode == 0:
        runtime, runtime_record = run(label + "-run", [
            tools["java"], "-Xverify:all", "-cp", classes, runner_name,
        ], home)
    class_files = sorted(classes.rglob("*.class"))
    return {
        "compile": compile_record, "runtime": runtime_record,
        "source_files": [file_record(path) for path in sources],
        "classes": [file_record(path) for path in class_files],
        "compile_success": compiled.returncode == 0,
        "runtime_success": runtime is not None and runtime.returncode == 0,
    }, runtime, class_files


def runtime_matches(actual, baseline_root, baseline_case):
    if actual is None:
        return False
    runtime = baseline_case.get("runtime")
    if not runtime:
        return False
    expected_stdout = (baseline_root / runtime["stdout"]["path"]).read_bytes()
    expected_stderr = (baseline_root / runtime["stderr"]["path"]).read_bytes()
    return (actual.returncode == runtime["exit"]
            and actual.stdout == expected_stdout and actual.stderr == expected_stderr)


def method_name(method):
    return bytes(method.get("item", {}).get("name", {}).get("raw", [])).decode("utf-8", "replace")


def method_descriptor(method):
    return bytes(method.get("item", {}).get("descriptor", {}).get("raw", [])).decode("utf-8", "replace")


def proof_checks(document, family):
    """Check the intended narrow static-field projection and retained clinit evidence."""
    expected = FAMILIES[family]
    checks = {}
    proof = document.get("initializer_proof", {})
    fields = document.get("fields", [])
    expected_static = expected["static_fields"]
    proof_fields = proof.get("fields", [])
    field_names = [
        bytes(item.get("item", {}).get("name", {}).get("raw", [])).decode("utf-8", "replace")
        for item in fields
    ]
    proof_names = [
        field_names[item["field_index"]]
        for item in proof_fields
        if isinstance(item.get("field_index"), int) and 0 <= item["field_index"] < len(field_names)
    ]
    checks["initializer_proof_proved"] = proof.get("kind") == "proved"
    checks["proved_static_fields_in_order"] = proof_names == expected_static
    checks["proof_field_count"] = len(proof_fields) == len(expected_static)
    checks["proved_fields_are_static"] = all(
        isinstance(item.get("field_index"), int)
        and 0 <= item["field_index"] < len(fields)
        and (fields[item["field_index"]].get("item", {}).get("access_flags", 0) & 0x0008) != 0
        for item in proof_fields
    )
    write_orders = [item.get("write_order") for item in proof_fields]
    checks["proof_write_order_is_physical_order"] = write_orders == list(range(len(expected_static)))

    source = document.get("text", "")
    source_positions = [source.find(fragment) for fragment in expected["required_source"]]
    checks["static_initializer_declaration_order"] = all(pos >= 0 for pos in source_positions) and source_positions == sorted(source_positions)
    checks["no_root_static_block"] = re.search(r"^\s*static\s*\{", source, re.M) is None
    checks["instance_b_remains_uninitialized_field"] = re.search(r"(?m)^\s*byte\[\]\s+b\s*;", source) is not None

    methods = document.get("methods", [])
    clinit = next((m for m in methods if method_name(m) == "<clinit>" and method_descriptor(m) == "()V"), None)
    checks["physical_clinit_retained"] = clinit is not None
    checks["all_methods_have_physical_identity"] = all(
        bool(method.get("item", {}).get("identity")) and bool(method.get("item", {}).get("descriptor", {}).get("raw"))
        for method in methods
    )
    clinit_report = clinit.get("outcome", {}).get("report", {}) if clinit else {}
    checks["clinit_report_produced_structured"] = (
        clinit_report.get("outcome") == "produced" and clinit_report.get("quality") == "structured"
    )
    checks["clinit_source_map_retained"] = bool(clinit_report.get("source_map", {}).get("segments"))

    constructor = next((m for m in methods if method_name(m) == "<init>" and method_descriptor(m) == "()V"), None)
    constructor_text = constructor.get("outcome", {}).get("report", {}).get("text", "") if constructor else ""
    checks["instance_b_assigned_in_constructor"] = "this.b = new byte[]" in constructor_text
    if family == "ordered":
        checks["ordered_proof_has_four_fields"] = len(proof_fields) == 4
    else:
        checks["literal_proof_only_static_a"] = proof_names == ["a"]
    return checks


def inventory_record(out_dir, path):
    data = path.read_bytes()
    return {"path": str(path.relative_to(out_dir)), "bytes": len(data), "sha256": sha(data)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path, help="path to frozen candidate CLI")
    parser.add_argument("--metadata", required=True, type=Path, help="candidate-cli metadata JSON")
    args = parser.parse_args()
    cli = args.cli.resolve()
    metadata_path = args.metadata.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite candidate evidence: {OUT}")
    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "streams").mkdir()

    global COMMANDS
    COMMANDS = []
    preflight = []
    failures = []
    metadata = read_json(metadata_path)
    if metadata.get("cli_path") and Path(metadata["cli_path"]).resolve() != cli:
        preflight.append({"label": "metadata-cli-path", "ok": False,
                          "expected": str(metadata["cli_path"]), "actual": str(cli)})
    else:
        preflight.append({"label": "metadata-cli-path", "ok": True, "actual": str(cli)})
    actual_cli_hash = sha(cli.read_bytes()) if cli.is_file() else None
    cli_hash_ok = actual_cli_hash == metadata.get("cli_sha256")
    preflight.append({"label": "candidate-cli", "path": str(cli),
                      "expected_sha256": metadata.get("cli_sha256"),
                      "actual_sha256": actual_cli_hash, "ok": cli_hash_ok})
    candidate_sources = metadata.get("candidate_sources", {})
    source_keys_ok = set(candidate_sources) == PRODUCT_PINS
    preflight.append({"label": "candidate-source-pin-set", "expected": sorted(PRODUCT_PINS),
                      "actual": sorted(candidate_sources), "ok": source_keys_ok})
    for relative, expected_hash in candidate_sources.items():
        path = ROOT / relative
        actual_hash = sha(path.read_bytes()) if path.is_file() else None
        preflight.append({"label": "candidate-source:" + relative, "path": str(path),
                          "expected_sha256": expected_hash, "actual_sha256": actual_hash,
                          "ok": actual_hash == expected_hash})

    jdk_manifest = read_json(JDK_MANIFEST)
    preflight.append({"label": "jdk-controls-manifest", "path": str(JDK_MANIFEST),
                      "sha256": sha(JDK_MANIFEST.read_bytes()),
                      "expected_sha256": JDK_MANIFEST_SHA256,
                      "ok": jdk_manifest.get("status") == "complete"
                            and sha(JDK_MANIFEST.read_bytes()) == JDK_MANIFEST_SHA256})
    legs = {}
    for leg in jdk_manifest.get("legs", []):
        tools = {}
        leg_ok = True
        for name, fact in leg.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual_hash = sha(path.read_bytes()) if path.is_file() else None
            ok = actual_hash == fact.get("sha256")
            leg_ok &= ok
            preflight.append({"label": f"{leg['leg']}:{name}", "path": str(path),
                              "expected_sha256": fact.get("sha256"), "actual_sha256": actual_hash,
                              "ok": ok})
            tools[name] = path
        if all(name in tools for name in ("java", "javac", "javap")):
            legs[leg["leg"]] = {"tools": tools, "home": tools["java"].parent.parent, "hashes_ok": leg_ok}

    baseline_roots = {}
    baseline_cases = {}
    for family, spec in FAMILIES.items():
        baseline_root = EVIDENCE / spec["directory"] / "baseline-root-v1"
        manifest_path = baseline_root / "manifest.json"
        inventory_path = baseline_root / "file-inventory.json"
        manifest = read_json(manifest_path)
        inventory = read_json(inventory_path)
        manifest_hash = sha(manifest_path.read_bytes())
        inventory_ok = True
        for item in inventory:
            path = baseline_root / item["path"]
            actual_hash = sha(path.read_bytes()) if path.is_file() else None
            if actual_hash != item["sha256"] or (path.is_file() and path.stat().st_size != item["bytes"]):
                inventory_ok = False
        complete_set = {str(path.relative_to(baseline_root)) for path in baseline_root.rglob("*")
                        if path.is_file() and path != inventory_path}
        inventory_ok &= complete_set == {item["path"] for item in inventory}
        source_record = manifest.get("source", {})
        runner_record = manifest.get("runner", {})
        source_path = baseline_root / source_record.get("path", "")
        runner_path = baseline_root / runner_record.get("path", "")
        baseline_facts_ok = (
            manifest_hash == spec["baseline_manifest_sha256"]
            and source_record.get("sha256") == spec["source_sha256"]
            and runner_record.get("sha256") == spec["runner_sha256"]
            and source_path.is_file() and sha(source_path.read_bytes()) == spec["source_sha256"]
            and runner_path.is_file() and sha(runner_path.read_bytes()) == spec["runner_sha256"]
        )
        preflight.append({"label": family + "-baseline-manifest", "path": str(manifest_path),
                          "sha256": manifest_hash, "expected_sha256": spec["baseline_manifest_sha256"],
                          "source_sha256": sha(source_path.read_bytes()) if source_path.is_file() else None,
                          "runner_sha256": sha(runner_path.read_bytes()) if runner_path.is_file() else None,
                          "ok": manifest.get("status") == "completed" and inventory_ok and baseline_facts_ok})
        baseline_roots[family] = baseline_root
        for leg_name in ("javac8", "javac23"):
            original_case = next((case for case in manifest["cases"]
                                  if case.get("kind") == "original" and case.get("jdk_leg") == leg_name), None)
            jarde_case = next((case for case in manifest["cases"]
                               if case.get("kind") == "jarde" and case.get("jdk_leg") == leg_name), None)
            if (not original_case or not jarde_case or not original_case.get("success")
                    or not jarde_case.get("success")):
                preflight.append({"label": f"{family}-{leg_name}-baseline-cases", "ok": False})
                continue
            class_record = next((item for item in manifest.get("inputs", []) if item["leg"] == leg_name), None)
            class_path = baseline_root / class_record["path"] if class_record else None
            actual_class_hash = sha(class_path.read_bytes()) if class_path and class_path.is_file() else None
            class_ok = bool(class_record and actual_class_hash == class_record["sha256"]
                            and actual_class_hash == spec["class_sha256"][leg_name])
            preflight.append({"label": f"{family}-{leg_name}-original-class", "path": str(class_path) if class_path else None,
                              "expected_sha256": class_record.get("sha256") if class_record else None,
                              "actual_sha256": actual_class_hash,
                              "baseline_expected_sha256": spec["class_sha256"][leg_name], "ok": class_ok})
            baseline_cases[(family, leg_name)] = {"original": original_case, "jarde": jarde_case,
                                                  "class_path": class_path, "manifest": manifest}

    required_ok = all(item.get("ok") for item in preflight)
    if not required_ok:
        (OUT / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
        raise SystemExit(f"candidate or baseline identity check failed; see {OUT / 'preflight.json'}")

    cases = []
    for family, spec in FAMILIES.items():
        baseline_root = baseline_roots[family]
        for leg_name in ("javac8", "javac23"):
            label = f"{family}-{leg_name}-candidate"
            leg = legs[leg_name]
            tools, home = leg["tools"], leg["home"]
            baseline = baseline_cases[(family, leg_name)]
            case_dir = OUT / "cases" / label
            case_dir.mkdir(parents=True)
            item = {"label": label, "family": family, "jdk_leg": leg_name,
                    "original_class": file_record(baseline["class_path"]), "success": False}

            rendered, item["render"] = run(label + "-render", [
                cli, "class-source", "--input", baseline["class_path"], "--class", spec["class"],
                "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all",
            ], home)
            document_path = case_dir / "class-source.json"
            if rendered.returncode != 0:
                item["render_success"] = False
                cases.append(item)
                continue
            document_path.write_bytes(rendered.stdout)
            item["document"] = file_record(document_path)
            item["render_success"] = True
            try:
                document = json.loads(rendered.stdout)
                generated = case_dir / (spec["class"] + ".java")
                generated.write_text(document["text"])
                runner_text = (baseline_root / "original-sources" / "Runner.java").read_text()
                generated_package = package_of(generated)
                runner = case_dir / "Runner.java"
                runner.write_text((f"package {generated_package};\n\n" if generated_package else "") + runner_text)
                runner_name = (generated_package + "." if generated_package else "") + spec["runner"]
                item["runner_package_adaptation"] = generated_package
                item["proof_checks"] = proof_checks(document, family)
                item["member_count"] = len(document.get("methods", []))
                item["field_count"] = len(document.get("fields", []))
                compiled, runtime, class_files = compile_run(label, [generated, runner], tools, home, runner_name)
                item.update(compiled)
                item["runtime_matches_original_raw"] = runtime_matches(runtime, baseline_root, baseline["original"])
                item["runtime_matches_frozen_jarde_raw"] = runtime_matches(runtime, baseline_root, baseline["jarde"])
                if class_files:
                    target_class = next((path for path in class_files if path.name == spec["class"] + ".class"), None)
                    if target_class:
                        _, item["javap"] = run(label + "-javap", [
                            tools["javap"], "-p", "-c", "-s", "-v", target_class,
                        ], home)
                        item["javap_class"] = file_record(target_class)
                item["success"] = bool(
                    item["compile_success"] and item["runtime_success"]
                    and item["runtime_matches_original_raw"] and item["runtime_matches_frozen_jarde_raw"]
                    and all(item["proof_checks"].values())
                )
            except Exception as error:
                item["candidate_document_error"] = type(error).__name__ + ": " + str(error)
            cases.append(item)

    counts = {"candidate": len(cases)}
    successes = {"candidate": sum(case.get("success", False) for case in cases)}
    failed_cases = [case["label"] for case in cases if not case.get("success", False)]
    if len(cases) != 4:
        failed_cases.append({"expected_candidate_legs": 4, "actual": len(cases)})
    manifest = {
        "schema": "preserve-nonfinal-static-initializer-candidate-luna-v1",
        "scope": "two frozen complete Java 8 array-field classes across javac8 and javac23; candidate source is compiled unchanged",
        "status": "completed" if not failed_cases else "candidate-with-failures",
        "claim_boundary": "Checks static initializer projection only. Instance byte[] b remains a declaration plus constructor assignment; instance-field restoration is not claimed.",
        "oracle": "The original class and previously accepted baseline Jarde run stdout, stderr, and exit per JDK leg; expected observations are read from baseline raw files, not hardcoded.",
        "arguments": {"cli": str(cli), "metadata": str(metadata_path)},
        "metadata": file_record(metadata_path),
        "candidate_cli": {"path": str(cli), "sha256": actual_cli_hash},
        "candidate_sources": candidate_sources,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": sha(JDK_MANIFEST.read_bytes())},
        "baseline_manifests": {
            family: {"path": str(EVIDENCE / spec["directory"] / "baseline-root-v1/manifest.json"),
                     "sha256": sha((EVIDENCE / spec["directory"] / "baseline-root-v1/manifest.json").read_bytes())}
            for family, spec in FAMILIES.items()
        },
        "preflight": preflight, "case_counts": counts, "success_counts": successes,
        "commands": COMMANDS, "cases": cases, "failures": failed_cases,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                           "excludes": ["file-inventory.json"]},
    }
    manifest_path = OUT / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    inventory_path = OUT / "file-inventory.json"
    inventory = [inventory_record(OUT, path) for path in sorted(OUT.rglob("*"))
                 if path.is_file() and path != inventory_path]
    inventory_path.write_text(json.dumps(inventory, indent=2) + "\n")
    if failed_cases:
        print(json.dumps({"status": manifest["status"], "failures": failed_cases}, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps({"status": manifest["status"], "case_counts": counts,
                      "success_counts": successes, "output": str(OUT)}, indent=2))


if __name__ == "__main__":
    main()
