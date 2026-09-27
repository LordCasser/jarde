#!/usr/bin/env python3
"""Replay nested-enum and enum-interface DT-13 shapes with complete source sets."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
JARDE_CLI = os.environ.get("JARDE_CLI")
SOURCES = [HERE / name for name in ("NestedShape.java", "InterfaceShape.java", "PlainShape.java", "PlainImpl.java", "Dollar$Shape.java")]
RUNNER = HERE / "Runner.java"
PACKAGE = "dt13"
EXPECTED_JADX = "2fb1b16386941660fda07e9017285aec40fcb37f"


def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), text=True, capture_output=True, **kwargs)


def checked(args, **kwargs):
    result = run(args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-4000:]}")
    return result.stdout


def compile_sources(source_paths, classes):
    classes.mkdir()
    return run(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *source_paths])


def normalize(text, work):
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def normalize_javap(text, work):
    text = normalize(text, work)
    return re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                  r"  Last modified <NORMALIZED>; size \1 bytes", text)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def compile_and_run(source_root, runner_source, work, label):
    runner = work / f"{label}-runner-src" / PACKAGE / "Runner.java"
    runner.parent.mkdir(parents=True)
    runner.write_text(runner_source)
    sources = sorted(source_root.rglob("*.java")) + [runner]
    classes = work / f"{label}-classes"
    compile = compile_sources(sources, classes)
    result = {"compile_exit": compile.returncode}
    if compile.returncode:
        result["compile_stdout"] = normalize(compile.stdout, work)
        result["compile_stderr"] = normalize(compile.stderr, work)
        return result, None
    execution = run(["java", "-Xverify:all", "-cp", classes, f"{PACKAGE}.Runner"])
    result.update({"run_exit": execution.returncode, "run_stdout": execution.stdout,
                   "run_stderr": normalize(execution.stderr, work)})
    return result, execution.stdout if execution.returncode == 0 else None


def declaration_name(source):
    match = re.search(r"(?m)^\s*(?:(?:public|protected|private|abstract|final|static|strictfp)\s+)*(?:@interface|class|enum|interface)\s+([A-Za-z_$][A-Za-z0-9_$]*)", source)
    if not match:
        raise RuntimeError("could not find the top-level declaration in Jarde class-source text")
    return match.group(1)


def class_inner_table(data):
    """Return the InnerClasses payload span, decoded names and raw eight-byte rows."""
    cursor = 8
    count = int.from_bytes(data[cursor:cursor + 2], "big")
    cursor += 2
    utf8 = {}
    classes = {}
    index = 1
    while index < count:
        tag = data[cursor]
        cursor += 1
        if tag == 1:
            length = int.from_bytes(data[cursor:cursor + 2], "big")
            cursor += 2
            utf8[index] = data[cursor:cursor + length].decode("utf-8")
            cursor += length
        elif tag in (3, 4):
            cursor += 4
        elif tag in (5, 6):
            cursor += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            classes[index] = int.from_bytes(data[cursor:cursor + 2], "big") if tag == 7 else None
            cursor += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cursor += 4
        elif tag == 15:
            cursor += 3
        else:
            raise RuntimeError(f"unknown constant-pool tag {tag}")
        index += 1
    class_flags_offset = cursor
    cursor += 6
    interfaces = int.from_bytes(data[cursor:cursor + 2], "big")
    cursor += 2 + interfaces * 2
    for _ in range(2):
        members = int.from_bytes(data[cursor:cursor + 2], "big")
        cursor += 2
        for _ in range(members):
            cursor += 6
            attributes = int.from_bytes(data[cursor:cursor + 2], "big")
            cursor += 2
            for _ in range(attributes):
                length = int.from_bytes(data[cursor + 2:cursor + 6], "big")
                cursor += 6 + length
    attributes = int.from_bytes(data[cursor:cursor + 2], "big")
    cursor += 2
    for _ in range(attributes):
        name_index = int.from_bytes(data[cursor:cursor + 2], "big")
        length = int.from_bytes(data[cursor + 2:cursor + 6], "big")
        payload = cursor + 6
        if utf8.get(name_index) == "InnerClasses":
            row_count = int.from_bytes(data[payload:payload + 2], "big")
            rows = []
            for row_index in range(row_count):
                start = payload + 2 + row_index * 8
                raw = data[start:start + 8]
                class_index, outer_index, name_index, flags = (
                    int.from_bytes(raw[0:2], "big"), int.from_bytes(raw[2:4], "big"),
                    int.from_bytes(raw[4:6], "big"), int.from_bytes(raw[6:8], "big"))
                class_name = utf8[classes[class_index]]
                outer_name = utf8[classes[outer_index]] if outer_index else None
                inner_name = utf8.get(name_index) if name_index else None
                rows.append((class_name, outer_name, inner_name, flags, raw))
            return (cursor + 2, payload, payload + length, rows, class_flags_offset)
        cursor = payload + length
    return None


def mutate_inner_rows(data, transform):
    table = class_inner_table(data)
    if table is None:
        raise RuntimeError("requested fixture class has no InnerClasses attribute")
    header, start, end, rows, _ = table
    changed = transform(rows)
    payload = len(changed).to_bytes(2, "big") + b"".join(row[4] for row in changed)
    return data[:header] + len(payload).to_bytes(4, "big") + payload + data[end:]


def rewrite_jar(source, target, replacements, duplicate=None):
    with zipfile.ZipFile(source) as original, zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as output:
        for info in original.infolist():
            payload = replacements.get(info.filename, original.read(info.filename))
            output.writestr(info.filename, payload)
        if duplicate is not None:
            output.writestr(duplicate, replacements.get(duplicate, original.read(duplicate)))


with tempfile.TemporaryDirectory(prefix="jarde-dt13-enums-") as temporary:
    work = Path(temporary)
    original_classes = work / "original-classes"
    original_compile = compile_sources(SOURCES + [RUNNER], original_classes)
    if original_compile.returncode:
        raise RuntimeError(original_compile.stderr)
    original_run = run(["java", "-Xverify:all", "-cp", original_classes, f"{PACKAGE}.Runner"])
    if original_run.returncode:
        raise RuntimeError(original_run.stderr)
    original_output = original_run.stdout
    bytecodes = {}
    for class_file in sorted((original_classes / PACKAGE).glob("*.class")):
        internal = f"{PACKAGE}/{class_file.stem}"
        bytecodes[internal] = checked(["javap", "-v", "-p", "-classpath", original_classes, internal])
    javap_text = "\n\n".join(normalize_javap(output, work) for output in bytecodes.values())
    (HERE / "original-javap.txt").write_text(javap_text)

    jar_path = work / "input.jar"
    with zipfile.ZipFile(jar_path, "w", zipfile.ZIP_DEFLATED) as archive:
        for class_file in sorted((original_classes / PACKAGE).glob("*.class")):
            if class_file.name != "Runner.class":
                archive.write(class_file, f"{PACKAGE}/{class_file.name}")

    jadx_commit = checked(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT).strip()
    if jadx_commit != EXPECTED_JADX:
        raise RuntimeError(f"expected fixed JADX commit {EXPECTED_JADX}, got {jadx_commit}")
    jadx_dir = work / "jadx"
    checked([JADX, "-d", jadx_dir, jar_path])
    jadx_source_root = work / "jadx-source"
    jadx_source_root.mkdir()
    for path in sorted(jadx_dir.rglob("*.java")):
        target = jadx_source_root / path.relative_to(jadx_dir)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(path.read_text())
    jadx_result, jadx_output = compile_and_run(jadx_source_root, RUNNER.read_text(), work, "jadx")

    if JARDE_CLI is None:
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        env["CARGO_INCREMENTAL"] = "0"
        env["CARGO_BUILD_JOBS"] = "2"
        build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
        if build.returncode:
            raise RuntimeError(build.stderr[-5000:])
        cli = work / "cargo-target/debug/jarde-cli"
    else:
        cli = Path(JARDE_CLI)

    jarde_source_root = work / "jarde-source"
    jarde_source_root.mkdir()
    jarde_physical_source_root = work / "jarde-physical-source"
    jarde_physical_source_root.mkdir()
    jarde_outputs = {}
    for class_file in sorted((original_classes / PACKAGE).glob("*.class")):
        if class_file.name == "Runner.class":
            continue
        internal = f"{PACKAGE}/{class_file.stem}"
        result = run([cli, "class-source", "--input", jar_path, "--class", internal,
                      "--policy", "plain-jar", "--release", "8", "--format", "text"])
        jarde_outputs[internal] = {"exit": result.returncode}
        if result.returncode:
            raise RuntimeError(f"Jarde class-source failed for {internal}: {normalize(result.stderr, work)}")
        source = result.stdout
        name = declaration_name(source)
        physical_target = jarde_physical_source_root / PACKAGE / f"{name}.java"
        physical_target.parent.mkdir(parents=True, exist_ok=True)
        if physical_target.exists():
            raise RuntimeError(f"two class files produced the same Jarde physical declaration {name}")
        physical_target.write_text(source)
        # Root requests provide the compilable source set. Child reports remain available as
        # standalone physical class sources and are not duplicated into that source set.
        if "$" not in class_file.stem or class_file.stem == "Dollar$Shape":
            target = jarde_source_root / PACKAGE / f"{name}.java"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(source)
        safe_internal = internal.replace("/", "_").replace("$", "_")
        (HERE / f"jarde-{safe_internal}.java.txt").write_text(source)

    root_report = run([cli, "class-source", "--input", jar_path, "--class", "dt13/NestedShape",
                       "--policy", "plain-jar", "--release", "8", "--format", "json"])
    require(root_report.returncode == 0, "Jarde root class-source JSON query failed")
    root_document = json.loads(root_report.stdout)
    nested_report = root_document["nested_enum_family"]
    require(nested_report["state"] == "prepared"
            and nested_report["projection"]["state"] == "projected",
            f"root report did not publish the proved nested enum projection: {nested_report!r}")
    require("child" not in nested_report,
            "root nested-enum summary duplicated the physical child report")
    derived = nested_report["projection"]["derived"]
    declarations = [entry for entry in derived if entry["kind"] == "nested_enum_declaration"]
    references = [entry for entry in derived if entry["kind"] == "nested_enum_type_reference"]
    require(len(declarations) == 2 and len(references) == 2,
            "root report did not publish both nested declarations and proved source references")
    derived_text = "\n".join(root_document["text"][entry["start"]:entry["end"]]
                             for entry in declarations)
    require("public enum Major {" in derived_text and "public enum Minor {" in derived_text,
            "derived range did not contain the complete two-level enum declaration")
    require('return "dt13.NestedShape$Major";' in root_document["text"],
            "root projection rewrote a nested binary name inside a string literal")
    for declaration in declarations:
        anchors = declaration["anchors"]
        require(sum(anchor["kind"] == "class_definition" for anchor in anchors) == 2
                and sum(anchor["kind"] == "enum_constant_group" for anchor in anchors) == 1,
                "derived enum declaration did not retain owner/child and group anchors")
    require(all(any(anchor["kind"] == "method_point" for anchor in entry["anchors"])
                and any(anchor["kind"] == "field" for anchor in entry["anchors"])
                for entry in references),
            "derived source references do not retain method and enum-field anchors")
    child_major = (HERE / "jarde-dt13_NestedShape_Major.java.txt").read_text()
    require("public enum NestedShape$Major {" in child_major,
            "independent child query no longer reports its physical class identity")

    nested_class = "dt13/NestedShape$Major.class"
    owner_class = "dt13/NestedShape.class"
    major_bytes = (original_classes / "dt13/NestedShape$Major.class").read_bytes()
    owner_bytes = (original_classes / "dt13/NestedShape.class").read_bytes()

    def rows_without_major(rows):
        return [row for row in rows if not (row[0] == "dt13/NestedShape$Major" and row[1] == "dt13/NestedShape")]

    missing_child_row = mutate_inner_rows(major_bytes, rows_without_major)

    def duplicate_major(rows):
        matching = [row for row in rows if row[0] == "dt13/NestedShape$Major" and row[1] == "dt13/NestedShape"]
        require(len(matching) == 1, "fixture owner must carry one direct Major row")
        return rows + matching

    duplicate_root_row = mutate_inner_rows(owner_bytes, duplicate_major)

    def non_enum_conflict(rows):
        matching = [row for row in rows if row[0] == "dt13/NestedShape$Major" and row[1] == "dt13/NestedShape"]
        require(len(matching) == 1, "fixture owner must carry one direct Major row")
        row = matching[0]
        flags = row[3] & ~0x4000
        raw = row[4][:6] + flags.to_bytes(2, "big")
        return rows + [(*row[:3], flags, raw)]

    non_enum_conflict_owner = mutate_inner_rows(owner_bytes, non_enum_conflict)

    def conflict_child_flags(rows):
        changed = []
        for row in rows:
            if row[0] == "dt13/NestedShape$Major" and row[1] == "dt13/NestedShape":
                raw = row[4][:6] + (row[3] ^ 0x0001).to_bytes(2, "big")
                row = (*row[:3], row[3] ^ 0x0001, raw)
            changed.append(row)
        return changed

    conflicting_child_row = mutate_inner_rows(major_bytes, conflict_child_flags)
    wrong_enum_flags = bytearray(major_bytes)
    _, _, _, _, class_flags_offset = class_inner_table(major_bytes)
    wrong_enum_flags[class_flags_offset:class_flags_offset + 2] = (
        int.from_bytes(wrong_enum_flags[class_flags_offset:class_flags_offset + 2], "big") & ~0x4000
    ).to_bytes(2, "big")
    negative_variants = {
        "missing_child_row": {nested_class: missing_child_row},
        "duplicate_owner_row": {owner_class: duplicate_root_row},
        "non_enum_conflicting_owner_row": {owner_class: non_enum_conflict_owner},
        "conflicting_child_flags": {nested_class: conflicting_child_row},
        "wrong_enum_flags": {nested_class: bytes(wrong_enum_flags)},
    }
    negative_results = {}
    for label, replacements in negative_variants.items():
        variant_jar = work / f"{label}.jar"
        rewrite_jar(jar_path, variant_jar, replacements)
        result = run([cli, "class-source", "--input", variant_jar, "--class", "dt13/NestedShape",
                      "--policy", "plain-jar", "--release", "8", "--format", "json"])
        require(result.returncode == 0, f"Jarde refused {label} before publishing its class-source report")
        document = json.loads(result.stdout)
        family = document["nested_enum_family"]
        require(family["state"] == "refused", f"Jarde did not refuse nested projection for {label}")
        require("public enum Major {" not in document["text"],
                f"Jarde published part of the nested enum subtree for {label}")
        require(any(diagnostic["code"] == "nested_enum_source_refused"
                    for diagnostic in document["diagnostics"]),
                f"Jarde did not expose the {label} refusal in report diagnostics")
        negative_results[label] = family["reason"]

    duplicate_jar = work / "duplicate-child-definition.jar"
    rewrite_jar(jar_path, duplicate_jar, {}, nested_class)
    duplicate_result = run([cli, "class-source", "--input", duplicate_jar, "--class", "dt13/NestedShape",
                            "--policy", "plain-jar", "--release", "8", "--format", "json"])
    duplicate_document = json.loads(duplicate_result.stdout)
    require("public enum Major {" not in duplicate_document.get("text", ""),
            "ambiguous selected child definitions produced a partial nested enum projection")
    negative_results["duplicate_child_definition"] = duplicate_document.get("nested_enum_family", {}).get("state", "no_report")

    for path in sorted(jadx_source_root.rglob("*.java")):
        safe = path.relative_to(jadx_source_root).as_posix().replace("/", "_")
        (HERE / f"jadx-{safe}").write_text(path.read_text())
    jarde_sources_only = work / "jarde-sources-only-classes"
    jarde_sources_compile = compile_sources(sorted(jarde_source_root.rglob("*.java")), jarde_sources_only)
    if jarde_sources_compile.returncode:
        jarde_sources_result = {"compile_exit": jarde_sources_compile.returncode,
                                "compile_stderr": normalize(jarde_sources_compile.stderr, work)}
    else:
        jarde_sources_result = {"compile_exit": 0}
    jarde_api_result, jarde_api_output = compile_and_run(jarde_source_root, RUNNER.read_text(), work, "jarde-api")
    physical_child_compiles = {}
    for source in sorted(jarde_physical_source_root.rglob("*.java")):
        if "$" not in source.stem:
            continue
        classes = work / f"physical-{source.stem}-classes"
        result = compile_sources([source], classes)
        physical_child_compiles[source.stem] = result.returncode

    expected_output = "nested=FIRST:LEFT:FIRST:LEFT\ninterface=FIRST:true\nplain=FIRST:false\nclass=true\n"
    require(original_compile.returncode == 0, "original Java 8 fixture did not compile")
    require(original_run.returncode == 0 and original_output == expected_output,
            "original -Xverify:all runtime did not match the frozen expected output")
    require(jadx_result["compile_exit"] == 0 and jadx_result.get("run_exit") == 0,
            "complete JADX sources did not compile and run with -Xverify:all")
    require(jadx_output == original_output, "JADX complete-source runtime differed from original")
    require(all(item["exit"] == 0 for item in jarde_outputs.values()),
            "one or more Jarde physical class-source requests failed")
    require(jarde_sources_result["compile_exit"] == 0,
            "complete set of Jarde root-level sources did not compile")
    require(jarde_api_result["compile_exit"] == 0 and jarde_api_result.get("run_exit") == 0,
            "Jarde root-level nested sources did not compile and run with the original API consumer")
    require(jarde_api_output == original_output,
            "Jarde nested API runtime differed from original")
    require(all(code == 0 for code in physical_child_compiles.values()),
            "one or more independently queried physical nested-enum reports did not compile")
    require("public enum InterfaceShape implements dt13.Marker" in
            (jarde_source_root / PACKAGE / "InterfaceShape.java").read_text(),
            "Jarde enum-interface positive did not retain implements Marker")
    require("plain=FIRST:false" in original_output and "interface=FIRST:true" in original_output,
            "enum implements I positive or plain enum negative assertion did not hold")
    dollar_top = (jarde_source_root / PACKAGE / "Dollar$Shape.java").read_text()
    require("public enum Dollar$Shape {" in dollar_top and "public enum Shape" not in dollar_top,
            "a dollar-named top-level enum was guessed into a nested source declaration")
    (HERE / "original-run.txt").write_text(original_output)
    results = {
        "jadx_commit": jadx_commit,
        "javac": checked(["javac", "-version"]).strip(),
        "java_version": run(["java", "-version"]).stderr.strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in SOURCES + [RUNNER, HERE / "JardeFlatRunner.java"]},
        "class_sha256": {name: hashlib.sha256((original_classes / f"{name}.class").read_bytes()).hexdigest()
                         for name in bytecodes},
        "negative_boundary": "PlainShape must remain a non-implementing enum; PlainImpl remains a class implementing Marker",
        "nested_enum_negative_results": negative_results,
        "nested_api_boundary": "original Runner uses NestedShape.Major and NestedShape.Major.Minor; root Jarde source projects nested enums and physically proved enum constant references while child class-source reports stay independently available",
        "jarde_class_source": jarde_outputs,
        "original": {"compile_exit": 0, "run_exit": original_run.returncode,
                     "run_stdout": original_output},
        "jadx": jadx_result,
        "jarde_sources_only": jarde_sources_result,
        "jarde_original_api_runner": jarde_api_result,
        "jarde_physical_child_compile_exits": physical_child_compiles,
        "runtime_equal": {"original_jadx": original_output == jadx_output,
                          "original_jarde_original_api": None if jarde_api_output is None else original_output == jarde_api_output},
        "assertions": {
            "original_java8_compile_and_verified_run": True,
            "jadx_complete_java8_compile_and_verified_run_matches_original": True,
            "jarde_root_level_source_set_java8_compiles": True,
            "jarde_independent_nested_enum_reports_java8_compile": True,
            "jarde_original_nested_api_consumer_verified_run_matches_original": True,
            "nested_binary_name_string_literal_preserved": True,
            "jarde_enum_implements_marker_header": True,
            "plain_enum_negative_boundary": True,
            "dollar_named_top_level_enum_stays_top_level": True,
        },
    }
    (HERE / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))
