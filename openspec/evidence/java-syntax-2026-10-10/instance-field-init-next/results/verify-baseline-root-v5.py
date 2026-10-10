#!/usr/bin/env python3
"""Independently verify the v3 three-class instance-field initializer baseline.

This records a successful original/Jarde replay alongside four intentional
JADX semantic counterexamples. It does not recast the baseline as 8/8 success.
"""
import hashlib
import json
from pathlib import Path
import re
import zipfile


HERE = Path(__file__).resolve().parent


def locate_root():
    for parent in HERE.parents:
        if (parent / "Cargo.toml").is_file() and (parent / "openspec/changes").is_dir():
            return parent
    raise RuntimeError("cannot locate repository root from verifier location")


ROOT = locate_root()
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/instance-field-init-next"
BASELINE = EVIDENCE / "baseline-root-v3"
METADATA_PATH = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
JDK_MANIFEST_PATH = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
OUTPUT = HERE / "baseline-root-verification-v5.json"
METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_PATH = "/private/tmp/jarde-nonfinal-static-cli-v1"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
BASELINE_MANIFEST_SHA256 = "da14af7be88c14054641f50e0e2065b087894cf72b7e31d0e9621773b33c8b18"
BASELINE_INVENTORY_SHA256 = "6651b423344c475082f77f0b3f4fd71df01b7a2f5a4d7851907a20c67732eab0"
JADX_PATH = "/opt/homebrew/bin/jadx"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
PRODUCT_PINS = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
FIXTURES = ("CommonDirectSuperByteArray", "ThisDelegatingByteArray", "DifferentRhsByteArray")
RUNNER = "InstanceFieldInitRunner"
LEGS = ("javac8", "javac23")
EXPECTED_COUNTS = {"original": 2, "jadx": 4, "jarde": 2}
EXPECTED_FAILURES = {
    "javac8-jadx-default", "javac23-jadx-default", "javac8-jadx-none", "javac23-jadx-none",
}
EXPECTED_METHODS = {
    "mark": ("(I)B", 0x0008), "<init>()V": ("()V", 0x0001),
    "<init>(I)V": ("(I)V", 0x0001), "<clinit>()V": ("()V", 0x0008),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return Path(path).read_bytes()


def record(path):
    data = read(path)
    try:
        rel = Path(path).relative_to(BASELINE).as_posix()
    except ValueError:
        rel = str(path)
    return {"path": rel, "bytes": len(data), "sha256": sha(data)}


def method_name(method):
    return bytes(method["item"]["name"]["raw"]).decode("utf-8")


def method_desc(method):
    return bytes(method["item"]["descriptor"]["raw"]).decode("utf-8")


def identity_name(identity):
    return bytes(identity["name"]).decode("utf-8")


def identity_desc(identity):
    return bytes(identity["descriptor"]).decode("utf-8")


def check_inventory():
    manifest_path = BASELINE / "manifest.json"
    inventory_path = BASELINE / "file-inventory.json"
    manifest_bytes, inventory_bytes = read(manifest_path), read(inventory_path)
    assert sha(manifest_bytes) == BASELINE_MANIFEST_SHA256
    assert sha(inventory_bytes) == BASELINE_INVENTORY_SHA256
    inventory = json.loads(inventory_bytes)
    seen = set()
    for item in inventory:
        relative = item["path"]
        assert relative not in seen and relative != "file-inventory.json"
        seen.add(relative)
        path = BASELINE / relative
        data = read(path)
        assert len(data) == item["bytes"] and sha(data) == item["sha256"], relative
    actual = {p.relative_to(BASELINE).as_posix() for p in BASELINE.rglob("*")
              if p.is_file() and p != inventory_path}
    assert seen == actual
    manifest = json.loads(manifest_bytes)
    assert manifest["file_inventory"] == {
        "path": "file-inventory.json", "includes": ["manifest.json"], "excludes": ["file-inventory.json"]
    }
    return manifest, {"manifest_sha256": sha(manifest_bytes), "inventory_sha256": sha(inventory_bytes),
                      "file_count": len(inventory), "closed": True}


def verify_tools(manifest):
    metadata_bytes = read(METADATA_PATH)
    assert sha(metadata_bytes) == METADATA_SHA256
    metadata = json.loads(metadata_bytes)
    assert metadata["cli_path"] == CLI_PATH and metadata["cli_sha256"] == CLI_SHA256
    assert set(metadata["candidate_sources"]) == PRODUCT_PINS
    assert manifest["candidate_sources"] == metadata["candidate_sources"]
    assert manifest["cli_metadata"] == {"path": str(METADATA_PATH), "sha256": METADATA_SHA256}
    assert manifest["frozen_cli"] == {"path": CLI_PATH, "sha256": CLI_SHA256}
    cli_hash = sha(read(CLI_PATH))
    assert cli_hash == CLI_SHA256
    product_sources = {}
    for relative, expected in metadata["candidate_sources"].items():
        actual = sha(read(ROOT / relative))
        assert actual == expected, relative
        product_sources[relative] = {"sha256": expected, "current_matches_pin": True}

    jdk_bytes = read(JDK_MANIFEST_PATH)
    assert sha(jdk_bytes) == JDK_MANIFEST_SHA256
    jdk_manifest = json.loads(jdk_bytes)
    assert jdk_manifest["status"] == "complete"
    tools = {}
    for leg in jdk_manifest["legs"]:
        by_name = {}
        for name, item in leg["jdk_tools"].items():
            path = Path(item["path"])
            digest = sha(read(path))
            assert digest == item["sha256"]
            by_name[name] = {"path": str(path), "sha256": digest}
        assert set(by_name) >= {"java", "javac", "javap"}
        tools[leg["leg"]] = by_name
    assert set(tools) == set(LEGS)
    jadx_executable = manifest["jadx_executable"]
    assert jadx_executable["path"] == "/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx"
    assert sha(read(jadx_executable["path"])) == JADX_SHA256
    assert manifest["jadx"]["path"] == JADX_PATH and manifest["jadx"]["expected_version"] == JADX_VERSION
    version_cmd = manifest["jadx"]["version_command"]
    verify_command_raw(version_cmd)
    assert version_cmd["argv"] == [JADX_PATH, "--version"] and version_cmd["exit"] == 0
    assert read(BASELINE / version_cmd["stdout"]["path"]).strip() == JADX_VERSION.encode()
    return {"metadata_sha256": METADATA_SHA256, "cli_path": CLI_PATH, "cli_sha256": cli_hash,
            "product_sources": product_sources, "jdk_manifest_sha256": JDK_MANIFEST_SHA256,
            "jdk_tools": tools, "jadx_executable_sha256": JADX_SHA256, "jadx_version": JADX_VERSION}


def verify_command_raw(command):
    for stream in ("stdout", "stderr"):
        item = command[stream]
        data = read(BASELINE / item["path"])
        assert len(data) == item["bytes"] and sha(data) == item["sha256"], (command["label"], stream)


def source_record_ok(item):
    data = read(BASELINE / item["path"])
    return len(data) == item["bytes"] and sha(data) == item["sha256"]


def verify_javap(class_name, class_hash, case, manifest):
    row = next(item for item in case["javap"] if item["class"] == class_name)
    assert row["success"]
    class_file = row["class_file"]
    assert class_file["sha256"] == class_hash
    command = row["command"]
    verify_command_raw(command)
    assert command["exit"] == 0
    argv = command["argv"]
    assert argv[1:5] == ["-p", "-c", "-s", "-v"]
    javap_out = read(BASELINE / command["stdout"]["path"]).decode("utf-8", errors="replace")
    assert class_name in javap_out
    assert re.search(r"(?m)^\s*byte\[\] bytes;\n\s*descriptor: \[B\n\s*flags:\s*(?:\(0x0000\))?\s*\n", javap_out)
    assert re.search(r"(?m)^\s*static java\.lang\.String trace;\n\s*descriptor: Ljava/lang/String;\n\s*flags: (?:\(0x0008\) )?ACC_STATIC", javap_out)
    method_blocks = parse_javap_methods(javap_out, class_name)
    assert set(method_blocks) == {"mark", "<init>()V", "<init>(I)V", "<clinit>()V"}
    for name, (descriptor, flags) in EXPECTED_METHODS.items():
        block = method_blocks[name]
        assert f"descriptor: {descriptor}" in block
        flag_line = next(line.strip() for line in block.splitlines() if line.strip().startswith("flags:"))
        hex_match = re.search(r"\(0x([0-9a-fA-F]{4})\)", flag_line)
        assert (int(hex_match.group(1), 16) if hex_match else flags) == flags
        expected_flag = "ACC_PUBLIC" if flags == 1 else "ACC_STATIC"
        assert flag_line.split(":", 1)[1].replace(hex_match.group(0) if hex_match else "", "").strip() == expected_flag
    for ctor in ("<init>()V", "<init>(I)V"):
        block = method_blocks[ctor]
        instructions = instruction_rows(block)
        expected_markers = {
            "CommonDirectSuperByteArray": 2,
            "ThisDelegatingByteArray": 0 if ctor == "<init>()V" else 2,
            "DifferentRhsByteArray": 1,
        }[class_name]
        assert sum("invokestatic" in op and "Method mark:(I)B" in op for _, op in instructions) == expected_markers
        assert sum("newarray" in op and "byte" in op for _, op in instructions) == (0 if expected_markers == 0 else 1)
        assert sum("bastore" in op for _, op in instructions) == expected_markers
        assert sum("putfield" in op and "Field bytes:[B" in op for _, op in instructions) == (0 if expected_markers == 0 else 1)
        if class_name == "ThisDelegatingByteArray" and ctor == "<init>()V":
            assert any('Method "<init>":(I)V' in op for _, op in instructions)
            assert not any('Method java/lang/Object."<init>":()V' in op for _, op in instructions)
        else:
            assert any('Method java/lang/Object."<init>":()V' in op for _, op in instructions)
    if class_name == "DifferentRhsByteArray":
        noarg = method_blocks["<init>()V"]
        int_ctor = method_blocks["<init>(I)V"]
        assert re.search(r"bipush\s+31", noarg)
        assert re.search(r"bipush\s+32", int_ctor)
    return {"class": class_name, "flags_and_descriptors": True,
            "constructor_array_bcis": {
                name: [{"bci": bci, "instruction": op.strip()} for bci, op in instruction_rows(method_blocks[name])
                       if any(opcode in op for opcode in ("newarray", "bastore", "putfield", "invokestatic"))]
                for name in ("<init>()V", "<init>(I)V")
            }, "javap_stdout_sha256": command["stdout"]["sha256"]}


def parse_javap_methods(text, class_name):
    lines = text.splitlines()
    header_re = re.compile(r"^  (?:(?:public|protected|private|static) .+\([^)]*\);|static \{\};)$")
    headers = [(i, line.strip()) for i, line in enumerate(lines) if header_re.match(line)]
    result = {}
    for index, (start, header) in enumerate(headers):
        end = headers[index + 1][0] if index + 1 < len(headers) else len(lines)
        block = "\n".join(lines[start:end])
        if header == "static byte mark(int);":
            key = "mark"
        elif header == f"public {class_name}();":
            key = "<init>()V"
        elif header == f"public {class_name}(int);":
            key = "<init>(I)V"
        elif header == "static {};":
            key = "<clinit>()V"
        else:
            continue
        assert key not in result
        result[key] = block
    return result


def instruction_rows(block):
    rows = []
    for line in block.splitlines():
        match = re.match(r"\s*(\d+):\s+(.*)$", line)
        if match:
            rows.append((int(match.group(1)), match.group(2)))
    return rows


def verify_document(class_name, leg, case, original_class_hash, javap_info, manifest_for_identity):
    rendered = next(item for item in case["rendered_classes"] if item["class"] == class_name)
    assert rendered["success"] and rendered["member_count"] == 4 and rendered["field_count"] == 2
    json_path = BASELINE / rendered["json"]["path"]
    json_bytes = read(json_path)
    assert len(json_bytes) == rendered["json"]["bytes"] and sha(json_bytes) == rendered["json"]["sha256"]
    document = json.loads(json_bytes)
    text = document["text"]
    source = read(BASELINE / rendered["source"]["path"])
    text_copy = read(BASELINE / rendered["text"]["path"])
    assert source == text_copy == text.encode()
    fields = document["fields"]
    # Jarde class_bytes digest is BLAKE3, not the archival SHA-256. Bind the
    # physical owner consistently; input SHA-256 is independently checked at argv.
    expected_owner = fields[0]["item"]["identity"]["owner"]
    input_class = next(item for item in manifest_for_identity["classes"] if Path(item["path"]).name == class_name + ".class")
    assert expected_owner["class_bytes"]["length"] == input_class["bytes"]
    assert re.fullmatch(r"[0-9a-f]{64}", expected_owner["class_bytes"]["digest"])
    field_names = [bytes(field["item"]["name"]["raw"]).decode() for field in fields]
    assert field_names == ["trace", "bytes"]
    assert [field["item"]["access_flags"] for field in fields] == [8, 0]
    assert [bytes(field["item"]["descriptor"]["raw"]).decode() for field in fields] == ["Ljava/lang/String;", "[B"]
    for field, name, descriptor in zip(fields, field_names, ("Ljava/lang/String;", "[B")):
        identity = field["item"]["identity"]["member"]
        assert identity_name(identity) == name and identity_desc(identity) == descriptor
        assert field["item"]["identity"]["owner"] == expected_owner
    assert re.search(r"(?m)^\s*byte\[\]\s+bytes\s*;", text)
    assert not re.search(r"(?m)^\s*byte\[\]\s+bytes\s*=", text)

    methods = document["methods"]
    physical = {(method_name(method), method_desc(method)): method for method in methods}
    assert len(methods) == 4 and set(physical) == {
        ("mark", "(I)B"), ("<init>", "()V"), ("<init>", "(I)V"), ("<clinit>", "()V")
    }
    class_digest = original_class_hash
    method_summaries = []
    for key, method in physical.items():
        name, desc = key
        identity = method["item"]["identity"]["member"]
        assert identity_name(identity) == name and identity_desc(identity) == desc
        assert method["item"]["access_flags"] == EXPECTED_METHODS[
            "mark" if name == "mark" else ("<init>()V" if desc == "()V" and name == "<init>"
                                             else "<init>(I)V" if name == "<init>" else "<clinit>()V")
        ][1]
        owner_hash = method["item"]["identity"]["owner"]["class_bytes"]["digest"]
        assert method["item"]["identity"]["owner"] == expected_owner
        outcome = method["outcome"]
        report = outcome["report"]
        assert outcome["kind"] == "recovered"
        assert report["outcome"] == "produced" and report["quality"] == "structured"
        assert report["representation"] == "java" and report["source_map"]["segments"]
        source_map_bcis = set()
        for segment in report["source_map"]["segments"]:
            origin = segment["origin"]
            for fact in ([origin["primary"]] if origin.get("primary") else []) + origin.get("derived", []):
                source_map_bcis.add(fact["bci"])
                origin_method = fact["method"]
                assert bytes(origin_method["name"]).decode() == name
                assert bytes(origin_method["descriptor"]).decode() == desc
                assert origin_method == method["item"]["identity"]
        method_summaries.append({"name": name, "descriptor": desc,
                                 "access_flags": method["item"]["access_flags"],
                                 "structured": True, "source_map_segments": len(report["source_map"]["segments"]),
                                 "source_map_bcis": sorted(source_map_bcis)})
        if name == "<init>":
            javap_block = javap_info[class_name]["method_blocks"][
                "<init>()V" if desc == "()V" else "<init>(I)V"]
            required_bcis = {bci for bci, opcode in instruction_rows(javap_block)
                             if any(token in opcode for token in
                                    ("newarray", "bastore", "putfield", "Method mark:(I)B"))}
            assert required_bcis <= source_map_bcis, (class_name, desc, required_bcis - source_map_bcis)

    class_text = text
    noarg = physical[("<init>", "()V")]["outcome"]["report"]["text"]
    int_ctor = physical[("<init>", "(I)V")]["outcome"]["report"]["text"]
    expected_rhs = {
        "CommonDirectSuperByteArray": "new byte[]{mark(10), mark(20)}",
        "ThisDelegatingByteArray": "new byte[]{mark(21), mark(22)}",
        "DifferentRhsByteArray": None,
    }[class_name]
    if class_name == "CommonDirectSuperByteArray":
        assert noarg.count(expected_rhs) == 1 and int_ctor.count(expected_rhs) == 1
        assert "body:noarg;" in noarg and "body:int:" in int_ctor
    elif class_name == "ThisDelegatingByteArray":
        assert "this(7);" in noarg and "this.bytes =" not in noarg
        assert int_ctor.count(expected_rhs) == 1
        assert "body:target:" in int_ctor and "body:delegate;" in noarg
    else:
        assert "new byte[]{mark(31)}" in noarg and "new byte[]{mark(32)}" in int_ctor
        assert "this.bytes = new byte[]{mark(31)}" in class_text
        assert "this.bytes = new byte[]{mark(32)}" in class_text
    return {"class": class_name, "json_sha256": sha(json_bytes),
            "physical_methods": method_summaries, "physical_fields": field_names,
            "instance_bytes_stays_in_constructor": True}


def verify_case_commands(manifest):
    commands = manifest["commands"]
    assert len(commands) == 31
    labels = set()
    raw_streams = []
    for command in commands:
        assert command["label"] not in labels
        labels.add(command["label"])
        verify_command_raw(command)
        for stream in ("stdout", "stderr"):
            raw_streams.append({"label": command["label"], "stream": stream,
                                "path": command[stream]["path"], "bytes": command[stream]["bytes"],
                                "sha256": command[stream]["sha256"]})
    assert len(raw_streams) == 62
    return commands, raw_streams


def verify_case_compile_and_runtime(case, kind, jdk_tools, baseline):
    assert case["compile_success"] is True
    compile_row = case["compile"]
    compile_cmd = next(cmd for cmd in baseline["commands"] if cmd["label"] == compile_row["label"])
    argv = compile_cmd["argv"]
    assert argv[0] == jdk_tools["javac"]["path"]
    for flag in ("-source", "8", "-target", "8", "-g:none", "-Xlint:-options", "-classpath", "-sourcepath", "-d"):
        assert flag in argv
    assert compile_cmd["exit"] == 0
    assert len(case["source_files"]) == 4
    assert {Path(item["path"]).name for item in case["source_files"]} == {name + ".java" for name in (*FIXTURES, RUNNER)}
    for item in case["source_files"]:
        assert source_record_ok(item)
    if kind in ("original", "jarde"):
        for item in case["source_files"]:
            name = Path(item["path"]).name
            expected = (BASELINE / "original-sources" / name) if kind == "original" or name == RUNNER + ".java" else (BASELINE / next(row["source"]["path"] for row in case["rendered_classes"] if row["class"] + ".java" == name))
            assert read(BASELINE / item["path"]) == read(expected)
    assert len(case["classes"]) == 4
    for item in case["classes"]:
        data = read(BASELINE / item["path"])
        assert len(data) == item["bytes"] and sha(data) == item["sha256"]
    runtime = case["runtime"]
    runtime_cmd = next(cmd for cmd in baseline["commands"] if cmd["label"] == runtime["label"])
    assert runtime_cmd["argv"][0] == jdk_tools["java"]["path"]
    assert "-Xverify:all" in runtime_cmd["argv"]
    class_dirs = {str((BASELINE / item["path"]).parent) for item in case["classes"]}
    assert len(class_dirs) == 1
    assert runtime_cmd["argv"][runtime_cmd["argv"].index("-cp") + 1] == class_dirs.pop()
    assert case["empty_classpath_sourcepath"]
    cp = str(BASELINE / case["empty_classpath_sourcepath"])
    assert argv[argv.index("-classpath") + 1] == cp
    assert argv[argv.index("-sourcepath") + 1] == cp
    return compile_cmd, runtime_cmd


def verify_all():
    manifest, evidence_identity = check_inventory()
    assert manifest["status"] == "baseline-with-failures"
    assert manifest["case_counts"] == EXPECTED_COUNTS
    assert manifest["success_counts"] == {"original": 2, "jadx": 0, "jarde": 2}
    assert set(manifest["failures"]) == EXPECTED_FAILURES

    tools_identity = verify_tools(manifest)
    # The prepared sources/Runner remain the exact inputs copied into baseline-root-v3.
    fixture_identity = {}
    for class_name, entry in manifest["fixtures"].items():
        assert class_name in FIXTURES
        source_path = EVIDENCE / (class_name + ".java")
        archived = BASELINE / "original-sources" / source_path.name
        assert sha(read(source_path)) == entry["sha256"]
        assert sha(read(archived)) == entry["sha256"] and read(source_path) == read(archived)
        fixture_identity[class_name] = {"bytes": entry["bytes"], "sha256": entry["sha256"]}
    runner_path = EVIDENCE / (RUNNER + ".java")
    archived_runner = BASELINE / "original-sources" / runner_path.name
    assert sha(read(runner_path)) == manifest["runner"]["sha256"]
    assert read(runner_path) == read(archived_runner)
    assert len(manifest["preflight"]) > 0 and all(item.get("ok") for item in manifest["preflight"])
    commands, raw_streams = verify_case_commands(manifest)
    case_map = {case["label"]: case for case in manifest["cases"]}
    assert set(case_map) == {f"{leg}-original" for leg in LEGS} | {
        f"{leg}-jadx-{profile}" for leg in LEGS for profile in ("default", "none")
    } | {f"{leg}-jarde" for leg in LEGS}

    jdk_manifest = json.loads(read(JDK_MANIFEST_PATH))
    leg_tool_map = {leg["leg"]: {name: fact["path"] for name, fact in leg["jdk_tools"].items()}
                    for leg in jdk_manifest["legs"]}
    javap_info = {class_name: {} for class_name in FIXTURES}
    original_outputs = {}
    compile_runtime_rows = []
    for leg in LEGS:
        case = case_map[f"{leg}-original"]
        assert case["success"] and case["runtime_success"] and case["runtime"]["exit"] == 0
        compile_cmd, runtime_cmd = verify_case_compile_and_runtime(case, "original", tools_identity["jdk_tools"][leg], manifest)
        compile_runtime_rows.extend([compile_cmd["label"], runtime_cmd["label"]])
        assert case["complete_class_set"] == sorted({f"{name}.class" for name in FIXTURES} | {f"{RUNNER}.class"})
        assert case["produced_class_names"] == case["complete_class_set"]
        stdout = read(BASELINE / case["runtime"]["stdout"]["path"])
        stderr = read(BASELINE / case["runtime"]["stderr"]["path"])
        original_outputs[leg] = (case["runtime"]["exit"], stdout, stderr)
        assert case["runtime_matches_original_raw"] is True
        for class_name in FIXTURES:
            class_item = next(item for item in case["classes"] if Path(item["path"]).name == f"{class_name}.class")
            javap_info[class_name][leg] = verify_javap(class_name, class_item["sha256"], case, manifest)
            command = next(row["command"] for row in case["javap"] if row["class"] == class_name)
            text = read(BASELINE / command["stdout"]["path"]).decode("utf-8", errors="replace")
            javap_info[class_name][leg]["method_blocks"] = parse_javap_methods(text, class_name)

    assert original_outputs["javac8"] == original_outputs["javac23"]
    original_text = original_outputs["javac8"][1].decode()
    assert "common=" in original_text and "delegated=" in original_text and "different=" in original_text
    assert "distinct=true" in original_text
    assert original_text.count("eval:21;eval:22;body:target:7;body:delegate;") == 1

    jarde_summaries = []
    for leg in LEGS:
        case = case_map[f"{leg}-jarde"]
        assert case["success"] and case["compile_success"] and case["runtime_success"]
        compile_cmd, runtime_cmd = verify_case_compile_and_runtime(case, "jarde", tools_identity["jdk_tools"][leg], manifest)
        compile_runtime_rows.extend([compile_cmd["label"], runtime_cmd["label"]])
        assert case["all_three_sources_rendered"] and case["runtime_matches_original_raw"]
        assert len(case["rendered_classes"]) == 3
        for rendered in case["rendered_classes"]:
            render_cmd = next(cmd for cmd in commands if cmd["label"] == rendered["command"]["label"])
            assert render_cmd["exit"] == 0 and render_cmd["argv"][0] == CLI_PATH
            assert render_cmd["argv"][render_cmd["argv"].index("--input") + 1].endswith(f"{rendered['class']}.class")
            raw_json = read(BASELINE / render_cmd["stdout"]["path"])
            assert sha(raw_json) == rendered["json"]["sha256"]
            original_class = next(item for item in case_map[f"{leg}-original"]["classes"]
                                  if Path(item["path"]).name == f"{rendered['class']}.class")
            doc = json.loads(raw_json)
            assert render_cmd["argv"][render_cmd["argv"].index("--input") + 1].endswith(original_class["path"])
            jarde_summaries.append(verify_document(rendered["class"], leg, case,
                                                    original_class["sha256"],
                                                    {name: {**javap_info[name][leg],
                                                            "method_blocks": javap_info[name][leg]["method_blocks"]}
                                                     for name in FIXTURES}, case_map[f"{leg}-original"]))
        assert read(BASELINE / case["runtime"]["stdout"]["path"]) == original_outputs[leg][1]
        assert read(BASELINE / case["runtime"]["stderr"]["path"]) == original_outputs[leg][2]

    jadx_rows = []
    jadx_info = manifest["jadx"]
    jar_record = jadx_info["input_jar"]
    jar_path = BASELINE / jar_record["path"]
    assert len(read(jar_path)) == jar_record["bytes"] and sha(read(jar_path)) == jar_record["sha256"]
    expected_jar_members = {f"{name}.class" for name in FIXTURES}
    with zipfile.ZipFile(jar_path) as jar:
        assert set(jar.namelist()) == expected_jar_members
        for member in jadx_info["jar_members"]:
            payload = jar.read(member["name"])
            assert len(payload) == member["bytes"] and sha(payload) == member["sha256"]
            assert member["name"] in expected_jar_members
    assert len(jadx_info["jar_members"]) == 3
    javac23_original_classes = {
        Path(item["path"]).name: item
        for item in case_map["javac23-original"]["classes"]
        if Path(item["path"]).name in expected_jar_members
    }
    assert set(javac23_original_classes) == expected_jar_members
    assert {
        member["name"]: member["sha256"] for member in jadx_info["jar_members"]
    } == {
        name: item["sha256"] for name, item in javac23_original_classes.items()
    }
    for profile in ("default", "none"):
        profile_row = next(row for row in manifest["jadx_profiles"] if row["profile"] == profile)
        assert profile_row["generated_source_count"] == 3
        assert set(Path(item["path"]).name for item in profile_row["generated_sources"]) == {
            f"{name}.java" for name in FIXTURES
        }
        decompile = profile_row["decompile"]
        verify_command_raw(decompile)
        assert decompile["exit"] == 0
        assert decompile["argv"][-1] == str(jar_path)
        if profile == "none":
            assert "--rename-flags" in decompile["argv"] and decompile["argv"][decompile["argv"].index("--rename-flags") + 1] == "none"
        for leg in LEGS:
            case = case_map[f"{leg}-jadx-{profile}"]
            assert case["compile_success"] is True
            assert case["runtime_success"] is False and case["success"] is False
            compile_cmd, runtime_cmd = verify_case_compile_and_runtime(case, "jadx", tools_identity["jdk_tools"][leg], manifest)
            compile_runtime_rows.extend([compile_cmd["label"], runtime_cmd["label"]])
            assert len(case["generated_sources"]) == 3 and all(case["all_generated_sources_compiled"].values())
            assert case["runner_class_name"] == ("defpackage." if profile == "default" else "") + RUNNER
            for source_item in case["generated_sources"]:
                assert source_record_ok(source_item)
                counterpart = next(item for item in profile_row["generated_sources"]
                                   if Path(item["path"]).name == Path(source_item["path"]).name)
                assert source_item["sha256"] == counterpart["sha256"]
            runner_source = BASELINE / next(item["path"] for item in case["source_files"]
                                            if Path(item["path"]).name == f"{RUNNER}.java")
            runner_bytes = read(runner_source)
            original_runner_bytes = read(BASELINE / f"original-sources/{RUNNER}.java")
            expected_runner = (b"package defpackage;\n\n" + original_runner_bytes) if profile == "default" else original_runner_bytes
            assert runner_bytes == expected_runner
            different_source = BASELINE / next(item["path"] for item in case["generated_sources"]
                                               if Path(item["path"]).name == "DifferentRhsByteArray.java")
            different_text = read(different_source).decode()
            assert re.search(r"byte\[\]\s+bytes\s*=\s*\{\s*mark\(31\)\s*\}", different_text)
            assert "mark(32)" not in different_text and "this.bytes" not in different_text
            stdout = read(BASELINE / case["runtime"]["stdout"]["path"])
            stderr = read(BASELINE / case["runtime"]["stderr"]["path"])
            assert case["runtime"]["exit"] == 1
            assert stdout.splitlines() == original_outputs[leg][1].splitlines()[:2]
            assert b"AssertionError: array: [31]" in stderr
            assert case["runtime"]["exit"] != original_outputs[leg][0]
            jadx_rows.append({"label": case["label"], "compile_exit": compile_cmd["exit"],
                              "runtime_exit": case["runtime"]["exit"], "known_semantic_negative": True,
                              "stdout_sha256": case["runtime"]["stdout"]["sha256"],
                              "stderr_sha256": case["runtime"]["stderr"]["sha256"]})

    assert len(jarde_summaries) == 6
    assert len(jadx_rows) == 4
    return {
        "status": "verified-with-known-jadx-semantic-negatives",
        "baseline_manifest": evidence_identity,
        "tools": tools_identity,
        "fixtures": fixture_identity,
        "runner": record(archived_runner),
        "case_counts": manifest["case_counts"],
        "success_counts": manifest["success_counts"],
        "recorded_failures": sorted(manifest["failures"]),
        "original_cross_jdk_raw_equal": manifest["original_cross_jdk_equal"],
        "original_and_jarde_observations": {leg: {"exit": row[0], "stdout_sha256": sha(row[1]), "stderr_sha256": sha(row[2])}
                                           for leg, row in original_outputs.items()},
        "jarde_structured_class_reports": jarde_summaries,
        "jadx_negative_runs": jadx_rows,
        "raw_command_streams": raw_streams,
        "command_count": len(commands),
        "closed_inventory": True,
        "claim_boundary": "Two original and two Jarde full-class runs match raw output. Four JADX full-source runs compile but preserve the observed DifferentRhsByteArray field-promotion failure; this is not an 8/8 semantic pass and does not claim instance-field recovery.",
    }


def main():
    if OUTPUT.exists():
        raise SystemExit(f"refusing to overwrite {OUTPUT}")
    result = verify_all()
    OUTPUT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "case_counts": result["case_counts"],
                      "success_counts": result["success_counts"], "known_jadx_failures": len(result["jadx_negative_runs"]),
                      "manifest_sha256": result["baseline_manifest"]["manifest_sha256"],
                      "inventory_sha256": result["baseline_manifest"]["inventory_sha256"]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
