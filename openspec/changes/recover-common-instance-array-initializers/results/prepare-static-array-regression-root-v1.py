#!/usr/bin/env python3
"""Replay accepted static-array controls against an instance-array candidate CLI.

This script is preparation-only in this checkout. Root supplies the frozen CLI
and its metadata and runs it when ready. Candidate-generated Java is compiled
unchanged; the only extra Java source is the original matching Runner.
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
BASELINES = {
    "literal": {
        "directory": "array-field-initializers-literal",
        "class": "ArrayFieldLiteral",
        "static_fields": ["a"],
        "source_sha256": "184d4d068f785c75d5db8a30579284ed42817eaf91b012e55681581ab145a67b",
        "runner_sha256": "fd69838bf2a372d1bceb27b83fffc3b24b23a2ba1967bb51038e4473b1d4e163",
        "manifest_sha256": "a021eedcde975dcb7bf7b7821d5097141b93292181015a45415752b21ddbafe1",
        "inventory_sha256": "1a47eb7179fd7539ee07d03467b54968acf2564e90e6d7d133c32ba50c1de762",
        "class_sha256": {
            "javac8": "6ead8bb0f3c466ba4dc10958a4cb349699f587bc9d5115f0656d0e2df60aa891",
            "javac23": "dc388f1ed9c0a44c9b87f9d61057b7a3f6c09fa1387b50d8f992a049e8544827",
        },
    },
    "ordered": {
        "directory": "array-field-initializers",
        "class": "ArrayFieldInitializers",
        "static_fields": ["trace", "before", "a", "after"],
        "source_sha256": "8ed268bad1dff3025b1d02e3c95a26b06889c02a1c14f52cc953f8e944e61650",
        "runner_sha256": "b6541a755e1e9b6c2c4b7d601b14fcc2a27f820650ec968fe0f2d150f79a28cd",
        "manifest_sha256": "93f14c0b125a738378c18a7a1a1f63dc6fb8c86c8c349c879b8e0e567c7c89d5",
        "inventory_sha256": "4c6999337f865c44f1bf43c309d364bab87981158eb0e356fee9f70de3aedf6e",
        "class_sha256": {
            "javac8": "5bd4b86f40a268640dd391df54a050f44056918928ae240232ebed6671fa24ee",
            "javac23": "3660a28f19cb7304ddc9c952a89f1b43378834faf0251f0910770ba9f89a6789",
        },
    },
}
STATIC_CANDIDATE = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-luna-v1"
STATIC_ACCEPTANCE = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-root-verification-v1.json"
STATIC_ACCEPTANCE_SHA256 = "b70e779c0ee718083d53cff8a310b3807e89f04f7dbbd1d9043d7ee6272e09b4"
OUT = Path(__file__).resolve().parent / "static-array-regression-root-v1"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
PROFILES = ("default", "all")
LEGS = ("javac8", "javac23")
COMMANDS = []


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def record(path):
    data = path.read_bytes()
    try:
        name = str(path.relative_to(OUT))
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def stream(label, suffix, content):
    path = OUT / "streams" / f"{label}.{suffix}"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(content)
    return record(path)


def run(label, argv, home=None):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    result = subprocess.run([str(value) for value in argv], cwd=ROOT, env=env,
                            capture_output=True, check=False)
    item = {"label": label, "argv": [str(value) for value in argv], "cwd": str(ROOT),
            "java_home": str(home) if home else None, "exit": result.returncode,
            "stdout": stream(label, "stdout", result.stdout),
            "stderr": stream(label, "stderr", result.stderr)}
    COMMANDS.append(item)
    return result, item


def closed_inventory(root, inventory_path):
    inventory = read_json(inventory_path)
    ok = True
    listed = set()
    for item in inventory:
        path = root / item["path"]
        listed.add(item["path"])
        ok &= path.is_file() and path.stat().st_size == item["bytes"] and sha(path.read_bytes()) == item["sha256"]
    actual = {str(path.relative_to(root)) for path in root.rglob("*")
              if path.is_file() and path != inventory_path}
    return bool(ok and actual == listed), inventory


def package_of(text):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", text, re.M)
    return match.group(1) if match else None


def physical_facts(document):
    """Physical members are kept separate from assembled class source text."""
    return {"fields": document.get("fields", []), "methods": document.get("methods", [])}


def document_checks(document, family, accepted_document):
    spec = BASELINES[family]
    proof = document.get("initializer_proof", {})
    fields = document.get("fields", [])
    proof_fields = proof.get("fields", [])
    field_names = [bytes(field.get("item", {}).get("name", {}).get("raw", [])).decode("utf-8", "replace")
                   for field in fields]
    proof_names = [field_names[item["field_index"]] for item in proof_fields
                   if isinstance(item.get("field_index"), int)
                   and 0 <= item["field_index"] < len(field_names)]
    text = document.get("text", "")
    checks = {
        "static_initializer_proof_remains_proved": proof.get("kind") == "proved",
        "static_group_names_and_order_preserved": proof_names == spec["static_fields"],
        "static_write_order_preserved": [item.get("write_order") for item in proof_fields]
            == list(range(len(spec["static_fields"]))),
        "static_group_members_are_static": all(
            isinstance(item.get("field_index"), int)
            and 0 <= item["field_index"] < len(fields)
            and fields[item["field_index"]].get("item", {}).get("access_flags", 0) & 0x0008
            for item in proof_fields),
        "instance_b_has_source_initializer": re.search(
            r"(?m)^\s*byte\[\]\s+b\s*=\s*(?:new\s+byte\s*\[\]\s*)?\{[^;]*\};", text) is not None,
        "physical_fields_equal_accepted_static_document": document.get("fields", []) == accepted_document.get("fields", []),
        "physical_methods_and_maps_equal_accepted_static_document": document.get("methods", []) == accepted_document.get("methods", []),
    }
    current_identities = [item.get("item", {}).get("identity") for item in fields]
    accepted_identities = [item.get("item", {}).get("identity") for item in accepted_document.get("fields", [])]
    checks["field_owner_B3_identity_equal"] = current_identities == accepted_identities
    current_method_ids = [item.get("item", {}).get("identity") for item in document.get("methods", [])]
    accepted_method_ids = [item.get("item", {}).get("identity") for item in accepted_document.get("methods", [])]
    checks["method_owner_B3_identity_equal"] = current_method_ids == accepted_method_ids
    return checks


def javac_run(label, target_source, runner_source, target_class, runner_name, tools, home):
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True)
    classes.mkdir(parents=True)
    compiled, compile_record = run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes,
        target_source, runner_source,
    ], home)
    runtime = runtime_record = None
    if compiled.returncode == 0:
        runtime, runtime_record = run(label + "-run", [
            tools["java"], "-Xverify:all", "-cp", classes, runner_name,
        ], home)
    class_files = sorted(classes.rglob("*.class"))
    expected = {target_class + ".class", "Runner.class"}
    actual = {path.name for path in class_files}
    return {
        "compile": compile_record, "runtime": runtime_record,
        "source_files": [record(target_source), record(runner_source)],
        "classes": [record(path) for path in class_files],
        "exact_class_set": actual == expected,
        "compile_success": compiled.returncode == 0,
        "runtime_success": runtime is not None and runtime.returncode == 0,
    }, runtime


def runtime_matches(runtime, baseline_root, original_case):
    if runtime is None:
        return False
    raw = original_case.get("runtime")
    if not raw:
        return False
    expected_stdout = (baseline_root / raw["stdout"]["path"]).read_bytes()
    expected_stderr = (baseline_root / raw["stderr"]["path"]).read_bytes()
    return runtime.returncode == raw["exit"] and runtime.stdout == expected_stdout and runtime.stderr == expected_stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--metadata", required=True, type=Path)
    parser.add_argument("--metadata-sha256", required=True)
    args = parser.parse_args()
    cli, metadata_path = args.cli.resolve(), args.metadata.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite evidence directory: {OUT}")
    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "oracles").mkdir()
    (OUT / "streams").mkdir()

    preflight = []
    metadata = read_json(metadata_path)
    cli_hash = sha(cli.read_bytes()) if cli.is_file() else None
    metadata_hash = sha(metadata_path.read_bytes()) if metadata_path.is_file() else None
    preflight.append({"label": "cli-identity", "path": str(cli), "expected_sha256": args.cli_sha256,
                      "actual_sha256": cli_hash, "ok": cli_hash == args.cli_sha256 == metadata.get("cli_sha256")
                      and metadata.get("cli_path") == str(cli)})
    preflight.append({"label": "metadata-identity", "path": str(metadata_path),
                      "expected_sha256": args.metadata_sha256, "actual_sha256": metadata_hash,
                      "ok": metadata_hash == args.metadata_sha256})
    source_groups = (("candidate_sources", 10), ("test_sources", 4), ("canonical_files", 16))
    for key, count in source_groups:
        sources = metadata.get(key, {})
        preflight.append({"label": "metadata-" + key, "count": len(sources), "expected_count": count,
                          "ok": len(sources) == count})
        for relative, expected in sources.items():
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            preflight.append({"label": key + ":" + relative, "expected_sha256": expected,
                              "actual_sha256": actual, "ok": actual == expected})

    jdk_manifest = read_json(JDK_MANIFEST)
    jdk_ok = sha(JDK_MANIFEST.read_bytes()) == JDK_MANIFEST_SHA256 and jdk_manifest.get("status") == "complete"
    preflight.append({"label": "jdk-manifest", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": sha(JDK_MANIFEST.read_bytes()),
                      "ok": jdk_ok})
    legs = {}
    for leg in jdk_manifest.get("legs", []):
        name = leg.get("leg")
        tools, hashes_ok = {}, True
        for tool_name, fact in leg.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact.get("sha256")
            hashes_ok &= ok
            preflight.append({"label": f"{name}:{tool_name}", "path": str(path),
                              "expected_sha256": fact.get("sha256"), "actual_sha256": actual, "ok": ok})
            tools[tool_name] = path
        if all(key in tools for key in ("java", "javac")):
            legs[name] = {"tools": tools, "home": tools["java"].parent.parent, "hashes_ok": hashes_ok}

    baselines, original_cases, accepted_docs = {}, {}, {}
    for family, spec in BASELINES.items():
        root = EVIDENCE / spec["directory"] / "baseline-root-v1"
        manifest_path, inventory_path = root / "manifest.json", root / "file-inventory.json"
        manifest = read_json(manifest_path)
        inventory_ok, _ = closed_inventory(root, inventory_path)
        source = root / "original-sources" / (spec["class"] + ".java")
        runner = root / "original-sources" / "Runner.java"
        baseline_ok = (manifest.get("status") == "completed"
                       and sha(manifest_path.read_bytes()) == spec["manifest_sha256"]
                       and sha(inventory_path.read_bytes()) == spec["inventory_sha256"]
                       and sha(source.read_bytes()) == spec["source_sha256"]
                       and sha(runner.read_bytes()) == spec["runner_sha256"] and inventory_ok)
        preflight.append({"label": family + "-frozen-original", "manifest_sha256": sha(manifest_path.read_bytes()),
                          "source_sha256": sha(source.read_bytes()), "runner_sha256": sha(runner.read_bytes()),
                          "inventory_closed": inventory_ok, "ok": baseline_ok})
        baselines[family] = {"root": root, "manifest": manifest, "source": source, "runner": runner}
        for leg in LEGS:
            original_case = next((case for case in manifest.get("cases", [])
                                  if case.get("kind") == "original" and case.get("jdk_leg") == leg), None)
            class_path = root / "cases" / (leg + "-original") / "classes" / (spec["class"] + ".class")
            class_hash = sha(class_path.read_bytes()) if class_path.is_file() else None
            ok = bool(original_case and original_case.get("success") and class_hash == spec["class_sha256"][leg])
            preflight.append({"label": f"{family}-{leg}-class-oracle", "path": str(class_path),
                              "actual_sha256": class_hash, "expected_sha256": spec["class_sha256"][leg], "ok": ok})
            original_cases[(family, leg)] = {"case": original_case, "class": class_path}

            accepted_case = STATIC_CANDIDATE / "cases" / f"{family}-{leg}-candidate"
            accepted_doc_path = accepted_case / "class-source.json"
            accepted_doc_hash = sha(accepted_doc_path.read_bytes()) if accepted_doc_path.is_file() else None
            accepted_doc = read_json(accepted_doc_path) if accepted_doc_path.is_file() else {}
            accepted_docs[(family, leg)] = accepted_doc
            accepted_declaration = accepted_doc.get("declaration", {}).get("declaration", "")
            preflight.append({"label": f"{family}-{leg}-accepted-static-document",
                              "path": str(accepted_doc_path), "sha256": accepted_doc_hash,
                              "ok": bool(accepted_doc_hash and accepted_doc.get("text")
                                         and re.search(r"\bclass\s+" + re.escape(spec["class"]) + r"\b", accepted_declaration))})

    acceptance = read_json(STATIC_ACCEPTANCE)
    static_manifest = STATIC_CANDIDATE / "manifest.json"
    static_inventory = STATIC_CANDIDATE / "file-inventory.json"
    static_inv_ok, _ = closed_inventory(STATIC_CANDIDATE, static_inventory)
    acceptance_ok = (sha(STATIC_ACCEPTANCE.read_bytes()) == STATIC_ACCEPTANCE_SHA256
                     and acceptance.get("status") == "accepted"
                     and sha(static_manifest.read_bytes()) == acceptance.get("candidate_manifest_sha256")
                     and sha(static_inventory.read_bytes()) == acceptance.get("candidate_inventory_sha256")
                     and static_inv_ok)
    preflight.append({"label": "accepted-static-candidate-evidence", "path": str(STATIC_ACCEPTANCE),
                      "sha256": sha(STATIC_ACCEPTANCE.read_bytes()), "inventory_closed": static_inv_ok,
                      "ok": acceptance_ok})

    if not all(item.get("ok") for item in preflight):
        (OUT / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
        raise SystemExit(f"preflight failed; details at {OUT / 'preflight.json'}")

    oracle_records = {}
    for family, spec in BASELINES.items():
        root = baselines[family]["root"]
        for leg in LEGS:
            original_case = original_cases[(family, leg)]["case"]
            runtime_record = original_case["runtime"]
            oracle_dir = OUT / "oracles" / f"{family}-{leg}"
            oracle_dir.mkdir(parents=True)
            raw = {}
            for stream_name in ("stdout", "stderr"):
                source_path = root / runtime_record[stream_name]["path"]
                target_path = oracle_dir / stream_name
                target_path.write_bytes(source_path.read_bytes())
                raw[stream_name] = record(target_path)
            oracle_records[(family, leg)] = {"exit": runtime_record["exit"], **raw}

    cases = []
    docs = {}
    for family, spec in BASELINES.items():
        for leg_name in LEGS:
            leg = legs[leg_name]
            baseline = baselines[family]
            input_class = original_cases[(family, leg_name)]["class"]
            for profile in PROFILES:
                label = f"{family}-{leg_name}-{profile}"
                case_dir = OUT / "cases" / label
                case_dir.mkdir(parents=True)
                argv = [cli, "class-source", "--input", input_class, "--class", spec["class"],
                        "--policy", "single-class", "--release", "8", "--format", "json"]
                if profile == "all":
                    argv.extend(["--evidence", "all"])
                rendered, render_record = run(label + "-render", argv, leg["home"])
                item = {"label": label, "family": family, "jdk_leg": leg_name, "profile": profile,
                        "original_class": record(input_class), "oracle_raw": oracle_records[(family, leg_name)],
                        "render": render_record,
                        "success": False}
                if rendered.returncode != 0:
                    item["render_success"] = False
                    cases.append(item)
                    docs[(family, leg_name, profile)] = None
                    continue
                doc_path = case_dir / "class-source.json"
                doc_path.write_bytes(rendered.stdout)
                item["document"] = record(doc_path)
                try:
                    document = json.loads(rendered.stdout)
                    docs[(family, leg_name, profile)] = document
                    target_source = case_dir / (spec["class"] + ".java")
                    target_source.write_bytes(document["text"].encode("utf-8"))
                    runner_text = baseline["runner"].read_text()
                    package = package_of(document["text"])
                    runner_source = case_dir / "Runner.java"
                    runner_source.write_text((f"package {package};\n\n" if package else "") + runner_text)
                    item["runner_package_adaptation"] = package
                    item["source_text_sha256"] = sha(target_source.read_bytes())
                    item["proof_checks"] = document_checks(
                        document, family, accepted_docs[(family, leg_name)])
                    compiled, runtime = javac_run(
                        label, target_source, runner_source, spec["class"],
                        (package + "." if package else "") + "Runner", leg["tools"], leg["home"])
                    item.update(compiled)
                    item["runtime_matches_original_raw"] = runtime_matches(
                        runtime, baseline["root"], original_cases[(family, leg_name)]["case"])
                    item["success"] = bool(item["compile_success"] and item["runtime_success"]
                                            and item["exact_class_set"]
                                            and item["runtime_matches_original_raw"]
                                            and all(item["proof_checks"].values()))
                except Exception as error:
                    item["candidate_error"] = type(error).__name__ + ": " + str(error)
                    docs[(family, leg_name, profile)] = None
                cases.append(item)

    profile_checks = {}
    for family in BASELINES:
        for leg in LEGS:
            default = docs.get((family, leg, "default"))
            all_doc = docs.get((family, leg, "all"))
            profile_checks[f"{family}-{leg}-default-all-full-text-identical"] = bool(
                default and all_doc and default.get("text") == all_doc.get("text"))
    failures = [case["label"] for case in cases if not case.get("success")]
    failures.extend(name for name, ok in profile_checks.items() if not ok)
    if len(cases) != 8:
        failures.append(f"expected-8-candidate-legs-got-{len(cases)}")
    manifest = {
        "schema": "common-instance-array-static-regression-root-v1",
        "status": "completed" if not failures else "candidate-with-failures",
        "scope": "two previously accepted complete static-array class controls, fresh candidate default/all rendering and unchanged full-source compilation on javac8 and javac23",
        "claim_boundary": "Regression evidence for prior literal/ordered static-array behavior while checking instance b promotion. It does not independently establish source-symbol provenance.",
        "oracle": "Per-leg frozen original class stdout/stderr/exit; accepted static candidate physical method/field reports and source maps are byte-structure compared.",
        "arguments": {"cli": str(cli), "cli_sha256": args.cli_sha256,
                      "metadata": str(metadata_path), "metadata_sha256": args.metadata_sha256},
        "candidate_cli": {"path": str(cli), "sha256": cli_hash},
        "metadata": record(metadata_path), "preflight": preflight,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": sha(JDK_MANIFEST.read_bytes())},
        "frozen_baselines": {family: {"manifest": str(baselines[family]["root"] / "manifest.json"),
                                       "manifest_sha256": BASELINES[family]["manifest_sha256"],
                                       "inventory_sha256": BASELINES[family]["inventory_sha256"]}
                             for family in BASELINES},
        "accepted_static_candidate": {"manifest_sha256": sha(static_manifest.read_bytes()),
                                       "inventory_sha256": sha(static_inventory.read_bytes()),
                                       "acceptance_sha256": sha(STATIC_ACCEPTANCE.read_bytes())},
        "case_counts": {"candidate_legs": len(cases), "expected_candidate_legs": 8},
        "profile_checks": profile_checks,
        "success_counts": {"candidate_legs": sum(case.get("success", False) for case in cases)},
        "commands": COMMANDS, "cases": cases, "failures": failures,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                           "excludes": ["file-inventory.json"]},
    }
    manifest_path = OUT / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    inventory_path = OUT / "file-inventory.json"
    inventory = [record(path) for path in sorted(OUT.rglob("*"))
                 if path.is_file() and path != inventory_path]
    inventory_path.write_text(json.dumps(inventory, indent=2) + "\n")
    if failures:
        print(json.dumps({"status": manifest["status"], "failures": failures}, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps({"status": manifest["status"], "case_counts": manifest["case_counts"],
                      "success_counts": manifest["success_counts"], "output": str(OUT)}, indent=2))


if __name__ == "__main__":
    main()
