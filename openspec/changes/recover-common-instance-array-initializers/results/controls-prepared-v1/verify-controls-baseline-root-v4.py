#!/usr/bin/env python3
"""Verify the closed evidence for the frozen pre-instance-initializer CLI baseline.

This reads existing evidence and current pinned inputs only. It does not invoke Git,
the CLI, Java, Cargo, or any other process.
"""
import hashlib
import json
from pathlib import Path
import re


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
EVIDENCE = HERE.parent / "controls-baseline-root-v1"
ACCEPTANCE = HERE.parent / "controls-baseline-root-acceptance-v4.json"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
EXPECTED_CLI = Path("/private/tmp/jarde-nonfinal-static-cli-v1")
EXPECTED_CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
EXPECTED_METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
EXPECTED_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
PRODUCT_PINS = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
CLASSES = (
    "CommonDirectSuperByteArray", "ThisDelegatingByteArray", "DifferentRhsByteArray",
    "FinalLiteralTwoArrays", "MissingWriteByteArray", "DuplicateWriteByteArray",
    "InterveningEffectByteArray", "ParameterRhsByteArray", "ReverseFieldOrderByteArray",
    "HandlerArrayByteArray",
)
RUNNERS = ("InstanceFieldInitRunner", "ControlsRunner")
CLASS_SET = set(CLASSES) | set(RUNNERS)
NEGATIVE_ARRAY_FIELDS = {
    "DifferentRhsByteArray", "MissingWriteByteArray", "DuplicateWriteByteArray",
    "InterveningEffectByteArray", "ParameterRhsByteArray", "ReverseFieldOrderByteArray",
    "HandlerArrayByteArray",
}


def read(path):
    return Path(path).read_bytes()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify_file_record(base, record):
    path = Path(record["path"])
    if not path.is_absolute():
        path = base / path
    data = read(path)
    assert len(data) == record["bytes"] and sha(data) == record["sha256"], str(path)
    return data


def verify_stream(record, stream):
    return verify_file_record(EVIDENCE, record[stream])


def verify_closed_inventory():
    inventory_path = EVIDENCE / "file-inventory.json"
    rows = json.loads(read(inventory_path))
    listed = set()
    for row in rows:
        relative = row["path"]
        assert relative not in listed and relative != "file-inventory.json", relative
        listed.add(relative)
        path = EVIDENCE / relative
        assert path.is_file(), relative
        data = read(path)
        assert len(data) == row["bytes"] and sha(data) == row["sha256"], relative
    actual = {path.relative_to(EVIDENCE).as_posix() for path in EVIDENCE.rglob("*")
              if path.is_file() and path != inventory_path}
    assert actual == listed, {"unlisted": sorted(actual - listed), "missing": sorted(listed - actual)}
    return {"path": str(inventory_path.relative_to(ROOT)), "sha256": sha(read(inventory_path)),
            "file_count": len(rows), "closed": True}


def verify_frozen_inputs(manifest, preflight):
    cli = manifest["baseline_cli"]
    metadata_path = Path(cli["metadata_path"])
    metadata_bytes = read(metadata_path)
    cli_bytes = read(cli["path"])
    assert Path(cli["path"]) == EXPECTED_CLI
    assert cli["sha256"] == EXPECTED_CLI_SHA256 == sha(cli_bytes)
    assert metadata_path == EXPECTED_METADATA
    assert cli["metadata_sha256"] == EXPECTED_METADATA_SHA256 == sha(metadata_bytes)
    metadata = json.loads(metadata_bytes)
    assert metadata["cli_path"] == str(EXPECTED_CLI)
    assert metadata["cli_sha256"] == EXPECTED_CLI_SHA256
    pins = metadata["candidate_sources"]
    assert set(pins) == PRODUCT_PINS
    current = {}
    for relative, expected in pins.items():
        data = read(ROOT / relative)
        assert sha(data) == expected, relative
        current[relative] = {"sha256": expected, "current_matches_pin": True}

    assert manifest["jdk_manifest"]["path"] == str(JDK_MANIFEST)
    assert manifest["jdk_manifest"]["sha256"] == JDK_MANIFEST_SHA256
    jdk_bytes = read(JDK_MANIFEST)
    assert sha(jdk_bytes) == JDK_MANIFEST_SHA256
    jdk = json.loads(jdk_bytes)
    tools = {}
    for leg in jdk["legs"]:
        assert leg["leg"] in ("javac8", "javac23")
        for name in ("java", "javac", "javap"):
            fact = leg["jdk_tools"][name]
            data = read(fact["path"])
            assert sha(data) == fact["sha256"], f'{leg["leg"]}:{name}'
            tools[(leg["leg"], name)] = fact
    assert set(name for name, _ in tools) == {"javac8", "javac23"}
    assert all(item.get("ok") is True for item in preflight), "all recorded preflight pins passed"
    prep = manifest["preparation_script"]
    prep_path = Path(prep["path"])
    prep_bytes = read(prep_path)
    assert len(prep_bytes) == prep["bytes"] and sha(prep_bytes) == prep["sha256"]
    return {
        "cli": {"path": str(EXPECTED_CLI), "sha256": sha(cli_bytes), "bytes": len(cli_bytes)},
        "metadata": {"path": str(metadata_path), "sha256": sha(metadata_bytes)},
        "product_source_count": len(current), "product_sources": current,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": sha(jdk_bytes)},
        "jdk_tools_verified": len(tools),
        "preparation_script": {"path": str(prep_path), "sha256": sha(prep_bytes),
                               "bytes": len(prep_bytes)},
    }, {key: fact["path"] for key, fact in tools.items()}


def verify_archived_sources(manifest):
    assert len(manifest["source_inputs"]) == 12
    result = {}
    for item in manifest["source_inputs"]:
        archived_record = item["archive"]
        archive = EVIDENCE / archived_record["path"]
        data = read(archive)
        assert len(data) == archived_record["bytes"] and sha(data) == archived_record["sha256"]
        canonical = read(item["canonical_path"])
        assert data == canonical, item["canonical_path"]
        result[archive.name] = {"bytes": len(data), "sha256": sha(data),
                                "role": item["role"], "canonical_matches": True}
    assert set(result) == {name + ".java" for name in CLASS_SET}
    return result


def expected_census(census):
    names = census["actual_class_names"]
    return (census["exact_complete_class_set"] is True and set(names) == CLASS_SET
            and len(names) == len(CLASS_SET) and set(census["expected_class_names"]) == CLASS_SET)


def verify_all_commands(manifest, jdk_tools):
    records = manifest["commands"]
    by_label = {}
    stream_paths = set()
    for command in records:
        label = command["label"]
        assert label not in by_label, f"duplicate command label: {label}"
        by_label[label] = command
        assert command["exit"] == 0, (label, command["exit"])
        assert command["cwd"] == str(ROOT), (label, command["cwd"])
        argv = command["argv"]
        assert argv and isinstance(argv[0], str)
        leg = "javac8" if label.startswith("javac8-") else "javac23" if label.startswith("javac23-") else None
        if leg is not None and not label.endswith("-render"):
            tool_name = ("javap" if "-original-javap-" in label else
                         "javac" if label.endswith("-javac-version") or
                         label.endswith("-original-compile") or
                         label.endswith("-candidate-full-source-compile") else
                         "java" if label.endswith("-java-version") or label.endswith("-run") else None)
            assert tool_name is not None, (label, argv)
            expected = jdk_tools[(leg, tool_name)]
            assert argv[0] == expected, (label, argv[0], expected)
            java_home = str(Path(argv[0]).parent.parent)
            assert command["java_home"] == java_home, (label, command["java_home"], java_home)
        else:
            assert argv[0] == str(EXPECTED_CLI), (label, argv[0])
            assert command["java_home"] is None, (label, command["java_home"])
        for stream_name in ("stdout", "stderr"):
            stream = command[stream_name]
            path = stream["path"]
            assert path not in stream_paths, f"duplicate command output path: {path}"
            stream_paths.add(path)
            verify_file_record(EVIDENCE, stream)

    embedded = {}

    def visit(value):
        if isinstance(value, dict):
            if {"label", "argv", "stdout", "stderr", "exit"}.issubset(value):
                label = value["label"]
                assert label in by_label, f"case command missing from command table: {label}"
                assert value == by_label[label], f"nested command differs from command table: {label}"
                embedded[label] = embedded.get(label, 0) + 1
            for child in value.values():
                visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)

    visit(manifest["cases"])
    assert len(records) == 80, len(records)
    assert len(stream_paths) == 160, len(stream_paths)
    non_case_labels = {label for label in by_label if label.endswith("-java-version") or label.endswith("-javac-version")}
    assert len(non_case_labels) == 4
    assert len(embedded) == 76 and set(embedded) == set(by_label) - non_case_labels, (
        sorted((set(by_label) - non_case_labels) - set(embedded)),
        sorted(set(embedded) - (set(by_label) - non_case_labels)))
    assert all(count == 1 for count in embedded.values()), embedded
    return {"command_count": len(records), "unique_labels": len(by_label),
            "raw_stream_count": len(stream_paths), "unique_stream_paths": True,
            "all_stream_hashes_verified": True, "fixed_tool_argv_verified": True,
            "case_records_match_command_table": True}


ACCESS_BITS = {
    "ACC_PUBLIC": 0x0001, "ACC_PRIVATE": 0x0002, "ACC_PROTECTED": 0x0004,
    "ACC_STATIC": 0x0008, "ACC_FINAL": 0x0010,
    "ACC_SUPER": 0x0020, "ACC_SYNCHRONIZED": 0x0020,
    "ACC_VOLATILE": 0x0040, "ACC_BRIDGE": 0x0040,
    "ACC_TRANSIENT": 0x0080, "ACC_VARARGS": 0x0080,
    "ACC_NATIVE": 0x0100, "ACC_INTERFACE": 0x0200, "ACC_ABSTRACT": 0x0400,
    "ACC_STRICT": 0x0800, "ACC_SYNTHETIC": 0x1000, "ACC_ANNOTATION": 0x2000,
    "ACC_ENUM": 0x4000, "ACC_MODULE": 0x8000, "ACC_MANDATED": 0x8000,
}


def parse_javap_members(raw, class_name):
    text = raw.decode("utf-8", errors="replace")
    lines = text.splitlines()
    try:
        body_start = lines.index("{") + 1
        body_end = len(lines) - 1 - lines[::-1].index("}")
    except ValueError as error:
        raise AssertionError(f"javap body delimiters missing for {class_name}") from error
    body = lines[body_start:body_end]
    header_positions = [index for index, line in enumerate(body)
                        if line.startswith("  ") and not line.startswith("   ")
                        and line.rstrip().endswith(";")]
    fields, methods, bci_by_method, instructions_by_method = set(), set(), {}, {}
    for position_index, start in enumerate(header_positions):
        end = header_positions[position_index + 1] if position_index + 1 < len(header_positions) else len(body)
        header = body[start].strip()[:-1]
        segment = body[start + 1:end]
        descriptor_line = next((line.strip() for line in segment if line.strip().startswith("descriptor:")), None)
        flags_line = next((line.strip() for line in segment if line.strip().startswith("flags:")), None)
        assert descriptor_line is not None and flags_line is not None, (class_name, header)
        descriptor = descriptor_line.split("descriptor:", 1)[1].strip()
        flag_names = re.findall(r"ACC_[A-Z_]+", flags_line)
        flags = 0
        for flag in flag_names:
            assert flag in ACCESS_BITS, (class_name, header, flag)
            flags |= ACCESS_BITS[flag]
        if header == "static {}":
            name = "<clinit>"
            methods.add((name, descriptor, flags))
            instructions = {int(match.group(1)): line.strip() for line in segment
                            if (match := re.match(r"\s*(\d+):", line))}
            bcis = set(instructions)
            bci_by_method[(name, descriptor)] = bcis
            opcodes_by_method[(name, descriptor)] = {int(m.group(1)): m.group(2) for line in segment if (m := re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line))}
            instructions_by_method[(name, descriptor)] = instructions
        elif "(" in header:
            prefix = header.split("(", 1)[0]
            name = prefix.split()[-1]
            if name == class_name:
                name = "<init>"
            methods.add((name, descriptor, flags))
            instructions = {int(match.group(1)): line.strip() for line in segment
                            if (match := re.match(r"\s*(\d+):", line))}
            bcis = set(instructions)
            bci_by_method[(name, descriptor)] = bcis
            opcodes_by_method[(name, descriptor)] = {int(m.group(1)): m.group(2) for line in segment if (m := re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line))}
            instructions_by_method[(name, descriptor)] = instructions
        else:
            name = header.split()[-1]
            fields.add((name, descriptor, flags))
    assert methods, class_name
    assert all(bci_by_method[key] for key in bci_by_method), (class_name, bci_by_method)
    return {"fields": fields, "methods": methods, "bci_by_method": bci_by_method,
            "instructions_by_method": instructions_by_method}


def verify_source_maps_and_reports(document, class_name, profile, javap_members=None):
    assert document["outcome"] == "performed"
    assert document["declaration"]["name"] == class_name
    coverage = document["coverage"]["artifact_structural"]
    assert coverage["state"] == "complete_within_schema"
    scans = {item["label"]: item for item in coverage["scanned"]}
    fields = document["fields"]
    methods = document["methods"]
    assert scans["class_fields"]["end"] == len(fields)
    assert scans["class_methods"]["end"] == len(methods)
    assert [field["item"]["index"] for field in fields] == list(range(len(fields)))
    method_ids = []
    source_map_segments = 0
    unmapped_bcis = []
    for index, method in enumerate(methods):
        item = method["item"]
        assert item["index"] == index
        name = bytes(item["identity"]["name"]).decode("utf-8")
        descriptor = bytes(item["identity"]["descriptor"]).decode("ascii")
        flags = item["access_flags"]
        assert item["identity"]["owner"] == document["class"]
        method_ids.append((name, descriptor, flags))
        outcome = method["outcome"]
        assert outcome["kind"] == "recovered", (class_name, name, outcome.get("kind"))
        body = outcome["report"]
        assert body["outcome"] == "produced", (class_name, name)
        if javap_members is not None:
            assert (name, descriptor, flags) in javap_members["methods"], (class_name, name, descriptor, flags)
        if profile == "all":
            source_map = body["source_map"]
            category = next(item for item in body["evidence"]["categories"]
                            if item["kind"] == "source_map")
            assert category["state"]["state"] == "complete"
            segments = source_map["segments"]
            assert isinstance(segments, list) and segments
            body_text_len = len(body["text"].encode("utf-8"))
            mapped_bcis = set()
            for segment in segments:
                assert 0 <= segment["start"] < segment["end"] <= body_text_len
                origin = segment["origin"]
                origins = ([origin["primary"]] if origin.get("primary") else []) + origin.get("derived", [])
                assert origins
                for source in origins:
                    assert source["method"]["name"] == item["identity"]["name"]
                    assert source["method"]["descriptor"] == item["identity"]["descriptor"]
                    assert source["method"]["owner"] == document["class"]
                    mapped_bcis.add(source["bci"])
            physical_bcis = javap_members["bci_by_method"][(name, descriptor)]
            assert not mapped_bcis - physical_bcis, (class_name, name, "nonphysical origin")
            missing = physical_bcis - mapped_bcis
            if missing:
                assert class_name == "HandlerArrayByteArray" and name == "<init>" and descriptor in ("()V", "(I)V") and missing == {19}, (class_name, name, descriptor, missing)
                assert javap_members["opcodes_by_method"][(name, descriptor)][19] == "goto"
                unmapped_bcis.append({"method": name, "descriptor": descriptor, "bcis": [19], "opcode": "goto"})
            source_map_segments += len(segments)
    assert len(set(method_ids)) == len(method_ids), f"duplicate physical method IDs in {class_name}"
    if javap_members is not None:
        assert set(method_ids) == javap_members["methods"]

    field_ids = []
    for field in fields:
        item = field["item"]
        identity = item["identity"]["member"]
        field_name = bytes(identity["name"]).decode("utf-8")
        descriptor = bytes(identity["descriptor"]).decode("ascii")
        flags = item["access_flags"]
        assert item["identity"]["owner"] == document["class"]
        field_ids.append((field_name, descriptor, flags))
        if javap_members is not None:
            assert (field_name, descriptor, flags) in javap_members["fields"], (class_name, field_name, descriptor, flags)
    assert len(set(field_ids)) == len(field_ids), f"duplicate physical field IDs in {class_name}"
    if javap_members is not None:
        assert set(field_ids) == javap_members["fields"]

    proof = document["initializer_proof"]
    assert isinstance(proof, dict) and isinstance(proof.get("kind"), str)
    clinit_bcis = javap_members["bci_by_method"].get(("<clinit>", "()V"), set()) \
        if javap_members is not None else set()
    clinit_instructions = javap_members["instructions_by_method"].get(("<clinit>", "()V"), {}) \
        if javap_members is not None else {}
    for write in proof.get("fields", []):
        assert 0 <= write["field_index"] < len(fields)
        assert write["write_bci"] >= 0 and write["write_order"] >= 0
        if javap_members is not None:
            assert write["write_bci"] in clinit_bcis
            field_name = bytes(fields[write["field_index"]]["item"]["identity"]["member"]["name"]).decode("utf-8")
            field_descriptor = bytes(fields[write["field_index"]]["item"]["identity"]["member"]["descriptor"]).decode("ascii")
            instruction = clinit_instructions[write["write_bci"]]
            assert "putstatic" in instruction and f"Field {field_name}:{field_descriptor}" in instruction
    return {"field_count": len(fields), "method_count": len(methods),
            "methods": [{"name": name, "descriptor": desc, "access_flags": flags}
                        for name, desc, flags in method_ids],
            "fields": [{"name": name, "descriptor": desc, "access_flags": flags}
                       for name, desc, flags in field_ids],
            "source_map_segments": source_map_segments,
            "unmapped_physical_bcis": unmapped_bcis,
            "initializer_proof_kind": proof["kind"],
            "initializer_proof_write_count": len(proof.get("fields", []))}


def verify_javap_and_original(manifest, commands, jdk_tools):
    originals = {case["jdk_leg"]: case for case in manifest["cases"] if case["kind"] == "original"}
    assert set(originals) == {"javac8", "javac23"}
    verified = {}
    javap_members = {}
    for leg, case in originals.items():
        assert case["success"] and case["compile_success"] and case["runtime_success"]
        assert expected_census(case["class_census"])
        compile_record = case["compile"]
        compile_argv = compile_record["argv"]
        assert compile_record["exit"] == 0 and compile_argv[0] == jdk_tools[(leg, "javac")]
        assert compile_argv[1:6] == ["-source", "8", "-target", "8", "-g"]
        original_classpath = Path(compile_argv[compile_argv.index("-classpath") + 1])
        original_sourcepath = Path(compile_argv[compile_argv.index("-sourcepath") + 1])
        assert original_classpath.is_dir() and not list(original_classpath.iterdir())
        assert original_sourcepath.is_dir() and not list(original_sourcepath.iterdir())
        assert len(case["source_files"]) == 12
        assert compile_argv[-12:] == [str(EVIDENCE / item["path"]) for item in case["source_files"]]
        original_classes = {Path(item["path"]).stem: str(EVIDENCE / item["path"])
                            for item in case["classes"]}
        assert set(original_classes) == CLASS_SET
        assert len(case["javap"]) == 12
        names = set()
        for entry in case["javap"]:
            class_name = entry["class"]
            assert class_name in CLASS_SET and class_name not in names
            names.add(class_name)
            command = entry["command"]
            assert entry["success"] and command["exit"] == 0
            argv = command["argv"]
            assert argv[1:5] == ["-p", "-c", "-s", "-v"]
            assert argv[0] == jdk_tools[(leg, "javap")] and argv[-1] == original_classes[class_name]
            stdout = verify_stream(command, "stdout")
            stderr = verify_stream(command, "stderr")
            assert stdout and not stderr
            javap_members[(leg, class_name)] = parse_javap_members(stdout, class_name)
            commands[command["label"]] = command
        assert names == CLASS_SET
        for runner in RUNNERS:
            record = case["runtimes"][runner]
            assert record["exit"] == 0
            argv = record["argv"]
            assert argv[0] == jdk_tools[(leg, "java")]
            assert "-Xverify:all" in argv and argv[-1] == runner
            verify_stream(record, "stdout")
            verify_stream(record, "stderr")
        verified[leg] = {"class_count": 12, "javap_count": len(names),
                         "runners": sorted(case["runtimes"]), "original_compile_and_runners_passed": True}
    return originals, verified, javap_members


def verify_render_cases(manifest, originals, commands, javap_members):
    renders = [case for case in manifest["cases"] if case["kind"] == "render"]
    assert len(renders) == 40
    texts, reports = {}, {}
    for case in renders:
        leg, class_name, profile = case["jdk_leg"], case["class"], case["profile"]
        assert leg in originals and class_name in CLASSES and profile in ("default", "all")
        assert case["render_success"] and case["success"]
        command = case["render"]
        assert command["exit"] == 0
        argv = command["argv"]
        assert argv[1:3] == ["class-source", "--input"]
        original_classes = {Path(item["path"]).stem: str(EVIDENCE / item["path"])
                            for item in originals[leg]["classes"]}
        assert argv[3] == original_classes[class_name]
        assert argv[4:10] == ["--class", class_name, "--policy", "single-class", "--release", "8"]
        assert argv[10:12] == ["--format", "json"]
        if profile == "default":
            assert "--evidence" not in argv
        else:
            assert argv[-2:] == ["--evidence", "all"]
        stdout = verify_stream(command, "stdout")
        stderr = verify_stream(command, "stderr")
        assert stdout and not stderr
        document_record = case["document"]
        document_bytes = verify_file_record(EVIDENCE, document_record)
        assert document_bytes == stdout
        document = json.loads(document_bytes)
        text = document["text"]
        assert text and sha(text.encode("utf-8")) == case["text_sha256"]
        census = verify_source_maps_and_reports(
            document, class_name, profile, javap_members[(leg, class_name)])
        texts[(leg, class_name, profile)] = text
        reports[(leg, class_name, profile)] = census
        commands[command["label"]] = command

    assert len(texts) == 40
    for leg in originals:
        for class_name in CLASSES:
            assert texts[(leg, class_name, "default")] == texts[(leg, class_name, "all")]
    for class_name in CLASSES:
        for profile in ("default", "all"):
            assert texts[("javac8", class_name, profile)] == texts[("javac23", class_name, profile)]

    for class_name in NEGATIVE_ARRAY_FIELDS:
        document = json.loads(read(EVIDENCE / next(
            case["document"]["path"] for case in renders
            if case["class"] == class_name and case["jdk_leg"] == "javac8" and case["profile"] == "all")))
        array_fields = [field for field in document["fields"]
                        if field["item"]["descriptor"]["escaped"] == "[B"]
        assert array_fields, class_name
        assert all("=" not in field["declaration"] for field in array_fields), class_name
    return {"render_count": len(renders), "classes_per_leg": 10,
            "default_all_text_equal": True, "cross_jdk_text_equal": True,
            "full_method_census_count": sum(len(item["methods"]) for item in reports.values()),
            "source_map_segment_count": sum(item["source_map_segments"] for item in reports.values()),
            "initializer_proof_reports": len(reports),
            "unmapped_handler_gotos": [{"jdk": key[0], "class": key[1], "profile": key[2], **gap} for key, report in reports.items() for gap in report["unmapped_physical_bcis"]],
            "source_map_claim_boundary": "All reported origins are physical; four handler constructor goto@19 anchors are absent. No full physical BCI coverage claim.",
            "negative_array_fields_not_promoted": sorted(NEGATIVE_ARRAY_FIELDS)}, texts


def verify_candidate_cases(manifest, originals, texts, commands, jdk_tools):
    candidates = {case["jdk_leg"]: case for case in manifest["cases"] if case["kind"] == "candidate"}
    assert set(candidates) == {"javac8", "javac23"}
    verified = {}
    for leg, case in candidates.items():
        assert case["success"] and case["compile_success"] and case["runtime_success"]
        assert case["source_text_is_cli_text"] and expected_census(case["class_census"])
        source_entries = case["generated_class_sources"]
        assert len(source_entries) == 10
        generated = {}
        for item in source_entries:
            path = EVIDENCE / item["path"]
            data = verify_file_record(EVIDENCE, item)
            class_name = path.stem
            assert data.decode("utf-8") == texts[(leg, class_name, "all")]
            generated[class_name] = data
        assert set(generated) == set(CLASSES)
        runner_entries = case["runner_sources"]
        assert len(runner_entries) == 2
        for item in runner_entries:
            data = verify_file_record(EVIDENCE, item)
            runner_name = Path(item["path"]).stem
            original = read(EVIDENCE / "original-sources" / (runner_name + ".java"))
            assert data == original, runner_name
        assert case["runner_package_adaptation"] is None

        compile_record = case["compile"]
        argv = compile_record["argv"]
        assert compile_record["exit"] == 0 and argv[0] == jdk_tools[(leg, "javac")]
        assert argv[1:6] == ["-source", "8", "-target", "8", "-g"]
        assert "-classpath" in argv and "-sourcepath" in argv and "-d" in argv
        classpath = Path(argv[argv.index("-classpath") + 1])
        sourcepath = Path(argv[argv.index("-sourcepath") + 1])
        classes_dir = Path(argv[argv.index("-d") + 1])
        assert classpath.is_dir() and not list(classpath.iterdir())
        assert sourcepath.is_dir() and not list(sourcepath.iterdir())
        assert classes_dir.is_dir()
        assert argv[-12:] == [str(EVIDENCE / entry["path"]) for entry in case["source_files"]]
        assert len(case["source_files"]) == 12
        for runner in RUNNERS:
            actual = case["runtimes"][runner]
            original = originals[leg]["runtimes"][runner]
            actual_stdout = verify_stream(actual, "stdout")
            actual_stderr = verify_stream(actual, "stderr")
            original_stdout = verify_stream(original, "stdout")
            original_stderr = verify_stream(original, "stderr")
            assert actual["exit"] == original["exit"] == 0
            assert actual_stdout == original_stdout and actual_stderr == original_stderr
            run_argv = actual["argv"]
            assert run_argv[0] == jdk_tools[(leg, "java")]
            assert "-Xverify:all" in run_argv
            assert run_argv[-1] == runner
            assert Path(run_argv[run_argv.index("-cp") + 1]) == classes_dir
            commands[actual["label"]] = actual
        verified[leg] = {"class_count": 12, "class_set_complete": True,
                         "runner_sources_byte_equal_to_original": True,
                         "empty_classpath_sourcepath": True, "verified_runner_count": 2,
                         "raw_runtimes_match_original": True}
    return verified


def main():
    if ACCEPTANCE.exists():
        raise SystemExit(f"refusing to overwrite {ACCEPTANCE}")
    manifest_path = EVIDENCE / "manifest.json"
    manifest_bytes = read(manifest_path)
    manifest = json.loads(manifest_bytes)
    assert manifest["schema"] == "recover-common-instance-array-controls-baseline-root-v1"
    assert manifest["status"] == "completed" and manifest["failures"] == []
    assert manifest["claim_boundary"] == (
        "Fresh CLI full-class source replay and full generated-source raw runtime comparison only; "
        "no internal rollback claim."
    )
    assert len(manifest["cases"]) == 44

    inventory = verify_closed_inventory()
    preflight = json.loads(read(EVIDENCE / "preflight.json"))
    frozen, jdk_tools = verify_frozen_inputs(manifest, preflight)
    command_summary = verify_all_commands(manifest, jdk_tools)
    archived_sources = verify_archived_sources(manifest)
    command_records = {record["label"]: record for record in manifest["commands"]}
    originals, original_summary, javap_members = verify_javap_and_original(
        manifest, command_records, jdk_tools)
    render_summary, texts = verify_render_cases(manifest, originals, command_records, javap_members)
    candidate_summary = verify_candidate_cases(manifest, originals, texts, command_records, jdk_tools)

    javap_records = [item for label, item in command_records.items() if "-original-javap-" in label]
    render_records = [item for label, item in command_records.items() if label.endswith("-render")]
    assert len(javap_records) == 24 and len(render_records) == 40
    assert all(command["exit"] == 0 for command in javap_records + render_records)
    result = {
        "schema": "recover-common-instance-array-controls-baseline-root-acceptance-v4",
        "status": "verified-baseline-only",
        "evidence_path": str(manifest_path.relative_to(ROOT)),
        "evidence_manifest_sha256": sha(manifest_bytes),
        "closed_inventory": inventory,
        "frozen_inputs": frozen,
        "command_evidence": command_summary,
        "archived_original_sources": archived_sources,
        "original_classes_and_javap": original_summary,
        "cli_render_replay": render_summary,
        "candidate_full_source_runs": candidate_summary,
        "cli_invocations": len(render_records), "original_javap_invocations": len(javap_records),
        "case_count": len(manifest["cases"]),
        "instance_initializer_implementation_claim": "not_claimed",
        "scope_note": "This accepts only the frozen static-initializer CLI baseline replay and its public source/runtime evidence; it does not claim instance initializer recovery is implemented or complete.",
    }
    ACCEPTANCE.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "output": str(ACCEPTANCE.relative_to(ROOT)),
                      "render_count": len(render_records), "javap_count": len(javap_records),
                      "case_count": len(manifest["cases"])}))


if __name__ == "__main__":
    main()
