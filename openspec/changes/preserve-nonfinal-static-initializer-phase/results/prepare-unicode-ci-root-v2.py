#!/usr/bin/env python3
"""Prepare a fresh, immutable-input Unicode class-source CI replay.

Preparation only: root reviews this script before running it. It uses the
already-frozen candidate CLI and JDK manifest, and refuses to overwrite its
versioned evidence directory.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
OUT = HERE / "unicode-ci-root-v2"
FIXTURE = ROOT / "tests/fixtures/recover-unicode-identifiers/ut"
SOURCE = FIXTURE / "UT.java"
FROZEN_CLASSES = {
    "UT": FIXTURE / "UT.class",
    "UT$内部类": FIXTURE / "UT$内部类.class",
}
FROZEN_SHA256 = {
    "UT.java": "2b122dce339956497a1e3ccef1b6152977cb6f44bf34fb59afa0076f13010d85",
    "UT.class": "00557f2a5007ad47a9d60839339c94de481e3cb5ee47da6e6255c92823a49d5c",
    "UT$内部类.class": "cda8bae084dac3852d7ce1ede1cde654041e452f4567a7047809f876a0b13de3",
}
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_METADATA = HERE / "candidate-cli-v1.json"
CLI_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_PATH = Path("/private/tmp/jarde-nonfinal-static-cli-v1")
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
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


def run(label, argv, home=None, cwd=ROOT):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home is not None:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    result = subprocess.run([str(value) for value in argv], cwd=cwd, env=env,
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
        "cwd": str(cwd),
        "java_home": str(home) if home is not None else None,
        "exit": result.returncode,
        **streams,
    }
    COMMANDS.append(item)
    return result, item


def decode_raw(value):
    return bytes(value).decode("utf-8", "replace")


def member_name(member):
    return decode_raw(member.get("item", {}).get("name", {}).get("raw", []))


def member_descriptor(member):
    item = member.get("item", {})
    descriptor = item.get("descriptor", {})
    if isinstance(descriptor, dict):
        return decode_raw(descriptor.get("raw", []))
    return ""


def declaration_line(text, name):
    pattern = re.compile(
        r"^\s*(?:(?:public|protected|private|static|final|transient|volatile)\s+)*"
        r"[\w.$<>\[\]]+\s+" + re.escape(name) + r"\b[^;]*;\s*$",
        re.M,
    )
    return next((line for line in text.splitlines() if pattern.match(line)), None)


def source_package(text):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", text, re.M)
    return match.group(1) if match else None


def source_top_level_name(text):
    # The rendered class may contain comments before its declaration. Keep this
    # check narrow: source is written unchanged; this name only selects filename.
    match = re.search(r"^\s*(?:(?:public|abstract|final|strictfp)\s+)*(?:class|interface|enum|record)\s+([\w$]+)", text, re.M)
    return match.group(1) if match else None


def inspect_document(document, class_name):
    text = document.get("text", "")
    fields = []
    for field in document.get("fields", []):
        fields.append({
            "name": member_name(field),
            "descriptor": member_descriptor(field),
            "identity": field.get("item", {}).get("identity"),
            "access_flags": field.get("item", {}).get("access_flags"),
            "rendered_declaration": declaration_line(text, member_name(field)),
            "text_has_initializer": bool(declaration_line(text, member_name(field))
                                          and "=" in declaration_line(text, member_name(field))),
        })
    methods = []
    for index, method in enumerate(document.get("methods", [])):
        outcome = method.get("outcome", {})
        report = outcome.get("report", {})
        source_map = report.get("source_map", {})
        segments = source_map.get("segments", [])
        methods.append({
            "index": index,
            "name": member_name(method),
            "descriptor": member_descriptor(method),
            "identity": method.get("item", {}).get("identity"),
            "outcome_kind": outcome.get("kind"),
            "quality": report.get("quality"),
            "representation": report.get("representation"),
            "source_map_segment_count": len(segments),
            "source_map_origin_count": sum(
                1 for segment in segments if segment.get("origin") is not None),
        })
    proof = document.get("initializer_proof")
    expected_names = {"UT": {"变量", "描述"}, "UT$内部类": {"名字"}}[class_name]
    declarations = [field["name"] for field in fields]
    expected_text = {
        "UT": (
            "static int 变量 = 1;",
            'static java.lang.String 描述 = new java.lang.StringBuilder().append("变量=").append(UT.变量).toString();',
            "static int 方法(int arg0)",
            "static class 内部类 extends java.lang.Object",
            "java.lang.String 名字;",
            "UT.描述",
            "方法(21)",
            "local1.名字",
        ),
        "UT$内部类": ("java.lang.String 名字;",),
    }[class_name]
    checks = {
        "source_class_name_matches_request": source_top_level_name(text) in (class_name, class_name.split("$")[-1]),
        "all_expected_fields_present": expected_names <= set(declarations),
        "text_contains_unicode_declarations": all(name in text for name in expected_names),
        "exact_expected_unicode_and_identity_strings": all(value in text for value in expected_text),
        "no_unicode_alias_marker": "__" not in text,
        "no_identifier_rejection_marker": "is not a Java identifier" not in text,
        "no_fallback_marker": "not recovered: the recovery run" not in text,
    }
    return {
        "class_source_text_sha256": sha(text.encode("utf-8")),
        "package": source_package(text),
        "field_projection": fields,
        "field_projection_count": len(fields),
        "initializer_proof": proof,
        "methods": methods,
        "full_method_count": len(methods),
        "checks": checks,
    }


def javap_physical_clinit(text):
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if line.strip() != "static {};":
            continue
        descriptor_index = next((cursor for cursor in range(index + 1, min(index + 8, len(lines)))
                                 if re.match(r"^\s*descriptor:\s*\S+\s*$", lines[cursor])), None)
        descriptor = (lines[descriptor_index].strip().split(":", 1)[1].strip()
                      if descriptor_index is not None else None)
        code_index = (next((cursor for cursor in range(descriptor_index + 1,
                                                       min(descriptor_index + 12, len(lines)))
                            if lines[cursor].strip() == "Code:"), None)
                      if descriptor_index is not None else None)
        return {"present": True, "descriptor": descriptor,
                "has_code": descriptor == "()V" and code_index is not None}
    return {"present": False, "descriptor": None, "has_code": False}


def write_class_family_jar(path, class_paths):
    """Write exactly two stored class entries with fixed ZIP metadata."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED, strict_timestamps=True) as archive:
        for entry_name, class_path in class_paths:
            info = zipfile.ZipInfo(entry_name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            info.flag_bits = 0x800 if any(ord(char) > 127 for char in entry_name) else 0
            archive.writestr(info, class_path.read_bytes())
    with zipfile.ZipFile(path, "r") as archive:
        entries = []
        for info in archive.infolist():
            data = archive.read(info.filename)
            entries.append({"name": info.filename, "bytes": len(data), "sha256": sha(data),
                            "compression": info.compress_type, "date_time": list(info.date_time)})
    return {"file": record(path), "entries": entries,
            "exact_entry_names": [row["name"] for row in entries]
            == ["UT.class", "UT$内部类.class"]}


def compile_sources(label, source_paths, tools, home, classes, class_name):
    case_dir = OUT / "cases" / label
    case_dir.mkdir(parents=True, exist_ok=True)
    empty = case_dir / "empty-classpath-sourcepath"
    empty.mkdir()
    classes.mkdir(parents=True, exist_ok=True)
    compiled, compile_record = run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-encoding", "UTF-8",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *source_paths,
    ], home)
    runtime = None
    runtime_record = None
    if compiled.returncode == 0:
        runtime, runtime_record = run(label + "-run", [
            tools["java"], "-Xverify:all", "-Dfile.encoding=UTF-8", "-cp", classes, class_name,
        ], home)
    class_files = sorted(classes.rglob("*.class"))
    return {
        "compile": compile_record,
        "runtime": runtime_record,
        "source_files": [record(path) for path in source_paths],
        "classes": [record(path) for path in class_files],
        "compile_success": compiled.returncode == 0,
        "runtime_success": runtime is not None and runtime.returncode == 0,
    }, runtime


def same_raw(actual, oracle):
    return (actual is not None and oracle is not None
            and actual.returncode == oracle.returncode
            and actual.stdout == oracle.stdout and actual.stderr == oracle.stderr)


def write_closed_evidence(manifest):
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    inventory_path = OUT / "file-inventory.json"
    rows = [record(path) for path in sorted(OUT.rglob("*"))
            if path.is_file() and path != inventory_path]
    inventory_path.write_text(json.dumps(rows, ensure_ascii=False, indent=2) + "\n")


def main():
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite Unicode CI evidence: {OUT}")

    OUT.mkdir(parents=True)
    for subdir in ("original-sources", "frozen-inputs", "input-jars", "streams", "cases"):
        (OUT / subdir).mkdir()
    preflight = []

    fixture_sources = SOURCE.read_bytes() if SOURCE.is_file() else b""
    actual_fixture_pins = {"UT.java": sha(fixture_sources) if fixture_sources else None}
    for name, path in FROZEN_CLASSES.items():
        actual_fixture_pins[path.name] = sha(path.read_bytes()) if path.is_file() else None
    for name, expected in FROZEN_SHA256.items():
        actual = actual_fixture_pins.get(name)
        preflight.append({"label": "unicode-fixture:" + name, "expected_sha256": expected,
                          "actual_sha256": actual, "ok": actual == expected})
    if fixture_sources:
        source_copy = OUT / "original-sources/UT.java"
        source_copy.write_bytes(fixture_sources)
        for name, path in FROZEN_CLASSES.items():
            if path.is_file():
                (OUT / "frozen-inputs" / path.name).write_bytes(path.read_bytes())

    metadata_bytes = CLI_METADATA.read_bytes() if CLI_METADATA.is_file() else b""
    metadata = json.loads(metadata_bytes) if metadata_bytes else {}
    actual_metadata_sha = sha(metadata_bytes) if metadata_bytes else None
    preflight.append({"label": "frozen-cli-metadata", "path": str(CLI_METADATA),
                      "expected_sha256": CLI_METADATA_SHA256, "actual_sha256": actual_metadata_sha,
                      "ok": actual_metadata_sha == CLI_METADATA_SHA256})
    for category in ("candidate_sources", "test_sources", "canonical_files"):
        pins = metadata.get(category, {})
        expected_counts = {"candidate_sources": 10, "test_sources": 4, "canonical_files": 16}
        preflight.append({"label": "metadata-category:" + category,
                          "expected_count": expected_counts[category], "actual_count": len(pins),
                          "ok": len(pins) == expected_counts[category]})
        for relative, expected in pins.items():
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            preflight.append({"label": category + ":" + relative,
                              "expected_sha256": expected, "actual_sha256": actual,
                              "ok": actual == expected})
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
            preflight.append({"label": frozen["leg"] + ":" + name, "path": str(path),
                              "expected_sha256": fact["sha256"], "actual_sha256": actual,
                              "ok": actual == fact["sha256"]})
            tools[name] = path
        if {"javac", "java", "javap"} <= set(tools):
            legs.append({"name": frozen["leg"], "home": tools["java"].parent.parent, "tools": tools})
    preflight.append({"label": "two-required-jdk-legs",
                      "ok": {leg["name"] for leg in legs} == {"javac8", "javac23"}})
    (OUT / "preflight.json").write_text(json.dumps(preflight, ensure_ascii=False, indent=2) + "\n")
    failures = [item["label"] for item in preflight if not item.get("ok")]

    cases = []
    rendered_texts = {}
    original_runtimes = {}
    source_texts = {}
    input_jars = {}
    if not failures:
        for leg in legs:
            name, home, tools = leg["name"], leg["home"], leg["tools"]
            for tool in ("javac", "java", "javap"):
                run(name + "-" + tool + "-version", [tools[tool], "-version"], home)

            original_sources = OUT / "original-sources/UT.java"
            original_classes = OUT / "cases" / (name + "-original") / "classes"
            compiled, runtime = compile_sources(name + "-original", [original_sources], tools, home,
                                                original_classes, "UT")
            item = {"label": name + "-original", "kind": "original", "jdk_leg": name,
                    **compiled, "success": bool(compiled["compile_success"] and compiled["runtime_success"])}
            if runtime is not None:
                original_runtimes[name] = runtime
                item["expected_utf8_stdout"] = runtime.stdout == "变量=1/42/中文\n".encode("utf-8")
            else:
                item["expected_utf8_stdout"] = False
            expected_classes = {"UT.class", "UT$内部类.class"}
            actual_classes = {Path(row["path"]).name for row in compiled["classes"]}
            item["complete_two_class_output"] = actual_classes == expected_classes
            if name == "javac8":
                class_records = {Path(row["path"]).name: row["sha256"] for row in compiled["classes"]}
                item["javac8_frozen_class_identity"] = all(
                    class_records.get(class_name + ".class") == FROZEN_SHA256[class_name + ".class"]
                    for class_name in FROZEN_CLASSES)
            else:
                item["javac8_frozen_class_identity"] = None
            javap_evidence = {}
            for class_name in ("UT", "UT$内部类"):
                class_path = next((path for path in original_classes.rglob("*.class")
                                   if path.name == class_name + ".class"), None)
                if class_path is None:
                    continue
                javap_result, javap_record = run(name + "-original-" + class_name + "-javap",
                                                 [tools["javap"], "-p", "-c", "-s", "-v", class_path], home)
                text = javap_result.stdout.decode("utf-8", "replace")
                javap_evidence[class_name] = {
                    "record": javap_record,
                    "success": javap_result.returncode == 0,
                    "physical_clinit": javap_physical_clinit(text),
                }
            item["javap"] = javap_evidence
            outer_clinit = javap_evidence.get("UT", {}).get("physical_clinit", {})
            item["outer_physical_clinit_contract"] = (
                outer_clinit.get("present") is True
                and outer_clinit.get("descriptor") == "()V"
                and outer_clinit.get("has_code") is True
            )
            class_paths = {
                name: next((path for path in original_classes.rglob("*.class")
                            if path.name == name + ".class"), None)
                for name in ("UT", "UT$内部类")
            }
            if all(class_paths.values()):
                jar_path = OUT / "input-jars" / name / "UT.jar"
                item["plain_jar"] = write_class_family_jar(jar_path, [
                    ("UT.class", class_paths["UT"]),
                    ("UT$内部类.class", class_paths["UT$内部类"]),
                ])
                input_jars[name] = jar_path
            else:
                item["plain_jar"] = None
            item["success"] = bool(item["success"] and item["expected_utf8_stdout"]
                                    and item["complete_two_class_output"]
                                    and item["outer_physical_clinit_contract"]
                                    and item["plain_jar"] is not None
                                    and item["plain_jar"]["exact_entry_names"]
                                    and item["javac8_frozen_class_identity"] is not False
                                    and all(row["success"] for row in javap_evidence.values()))
            cases.append(item)
            if not item["success"]:
                failures.append(item["label"])

        for leg in legs:
            name, home, tools = leg["name"], leg["home"], leg["tools"]
            jar_path = input_jars.get(name)
            if jar_path is None:
                failures.append(name + "-missing-two-class-jar")
                continue
            for class_name in ("UT", "UT$内部类"):
                if not jar_path.is_file():
                    failures.append(name + "-missing-class-family-jar")
                    continue
                for profile in ("default", "all"):
                    label = name + "-" + class_name.replace("$", "-inner-") + "-" + profile
                    case_dir = OUT / "cases" / label
                    case_dir.mkdir()
                    argv = [CLI_PATH, "class-source", "--input", jar_path,
                            "--class", class_name, "--policy", "plain-jar", "--release", "8",
                            "--format", "json"]
                    if profile == "all":
                        argv += ["--evidence", "all"]
                    rendered, render_record = run(label + "-render", argv)
                    item = {"label": label, "kind": "rendered", "profile": profile,
                            "jdk_leg": name, "class": class_name, "render": render_record,
                            "render_success": rendered.returncode == 0, "success": False}
                    if rendered.returncode == 0:
                        document_path = case_dir / "class-source.json"
                        document_path.write_bytes(rendered.stdout)
                        item["document"] = record(document_path)
                        try:
                            document = json.loads(rendered.stdout)
                            text = document["text"]
                            rendered_texts[(name, class_name, profile)] = text
                            item["inspection"] = inspect_document(document, class_name)
                            generated_name = "UT.java" if class_name == "UT" else "UT$内部类.java"
                            generated_path = case_dir / generated_name
                            generated_path.write_bytes(text.encode("utf-8"))
                            source_texts[(name, profile, class_name)] = text
                            item["generated_source"] = record(generated_path)
                            item["generated_source_bytes_unchanged"] = (
                                generated_path.read_bytes() == text.encode("utf-8"))
                            item["source_map_inventory"] = [
                                {"method_index": method["index"], "name": method["name"],
                                 "descriptor": method["descriptor"],
                                 "quality": method["quality"],
                                 "representation": method["representation"],
                                 "segment_count": method["source_map_segment_count"],
                                 "origin_count": method["source_map_origin_count"]}
                                for method in item["inspection"]["methods"]]

                            inspection_ok = (all(item["inspection"]["checks"].values())
                                             if class_name == "UT" else True)
                            item["inspection_checks_are_authoritative"] = class_name == "UT"
                            item["success"] = bool(item["render_success"]
                                                    and item["generated_source_bytes_unchanged"]
                                                    and inspection_ok)
                        except Exception as error:
                            item["document_error"] = type(error).__name__ + ": " + str(error)
                    cases.append(item)
                    if not item["success"]:
                        failures.append(label)

            for profile in ("default", "all"):
                outer_text = source_texts.get((name, profile, "UT"))
                if outer_text is None:
                    failures.append(name + "-" + profile + "-missing-outer-source")
                    continue
                label = name + "-" + profile + "-two-class-family"
                source_path = OUT / "cases" / label / "UT.java"
                source_path.parent.mkdir(parents=True, exist_ok=True)
                source_path.write_bytes(outer_text.encode("utf-8"))
                classes_dir = OUT / "cases" / label / "classes"
                compiled_pair, runtime_pair = compile_sources(
                    label, [source_path], tools, home, classes_dir, "UT")
                output_class_names = [Path(row["path"]).name for row in compiled_pair["classes"]]
                pair = {
                    "label": label,
                    "kind": "outer-source-complete-class-family",
                    "profile": profile,
                    "jdk_leg": name,
                    "generated_source_is_exact_outer_report": source_path.read_bytes() == outer_text.encode("utf-8"),
                    "generated_compile": compiled_pair,
                    "complete_two_class_output": sorted(output_class_names) == ["UT$内部类.class", "UT.class"],
                    "runtime_matches_original_raw": same_raw(runtime_pair, original_runtimes.get(name)),
                    "expected_utf8_stdout": runtime_pair is not None
                    and runtime_pair.stdout == "变量=1/42/中文\n".encode("utf-8"),
                }
                pair["success"] = bool(compiled_pair["compile_success"]
                                       and compiled_pair["runtime_success"]
                                       and pair["generated_source_is_exact_outer_report"]
                                       and pair["complete_two_class_output"]
                                       and pair["runtime_matches_original_raw"]
                                       and pair["expected_utf8_stdout"])
                cases.append(pair)
                if not pair["success"]:
                    failures.append(label)

    keys = [(leg["name"], class_name) for leg in legs for class_name in ("UT", "UT$内部类")]
    equal_profile_text = {}
    for name, class_name in keys:
        default_text = rendered_texts.get((name, class_name, "default"))
        all_text = rendered_texts.get((name, class_name, "all"))
        equal_profile_text[f"{name}:{class_name}"] = (
            default_text is not None and default_text == all_text)
        if class_name == "UT" and not equal_profile_text[f"{name}:{class_name}"]:
            failures.append("default-all-text:" + name + ":" + class_name)

    cross_jdk_text_equal = {}
    for class_name in ("UT", "UT$内部类"):
        for profile in ("default", "all"):
            text8 = rendered_texts.get(("javac8", class_name, profile))
            text23 = rendered_texts.get(("javac23", class_name, profile))
            cross_jdk_text_equal[f"{class_name}:{profile}"] = text8 is not None and text8 == text23

    manifest = {
        "schema": "unicode-ci-root-v2",
        "status": "completed" if not failures else "completed-with-failures",
        "scope": "Frozen UT Unicode fixture, exact two-class PlainJar family; javac 8/23 source-target 8 UTF-8; class-source default/all",
        "claim_boundary": "Public source rendering, compile checks, class-family runtime raw comparison; no internal rollback claim.",
        "preparation_script": record(Path(__file__).resolve()),
        "fixture_source": record(OUT / "original-sources/UT.java") if (OUT / "original-sources/UT.java").exists() else None,
        "frozen_fixture_classes": [record(path) for path in sorted((OUT / "frozen-inputs").glob("*.class"))],
        "cli": {"path": str(CLI_PATH), "sha256": actual_cli_sha,
                "metadata_path": str(CLI_METADATA), "metadata_sha256": actual_metadata_sha},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": actual_manifest_sha},
        "preflight": preflight,
        "commands": COMMANDS,
        "cases": cases,
        "expected_utf8_stdout": "变量=1/42/中文\n",
        "raw_oracle_is_authoritative": True,
        "default_all_text_equal": equal_profile_text,
        "cross_jdk_text_equal_observed": cross_jdk_text_equal,
        "failures": failures,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "preflight.json"],
                           "excludes": ["file-inventory.json"]},
    }
    write_closed_evidence(manifest)
    if failures:
        print(json.dumps({"status": manifest["status"], "failures": failures,
                          "output": str(OUT)}, ensure_ascii=False, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps({"status": manifest["status"], "case_count": len(cases),
                      "output": str(OUT)}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
