#!/usr/bin/env python3
"""Independently verify the completed Unicode CI replay (never executes tools)."""
import hashlib
from importlib.metadata import version as package_version
import json
from pathlib import Path
import re
import subprocess
import zipfile
from blake3 import blake3


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
OUT = HERE / "unicode-ci-root-v2"
EXECUTION = HERE / "root-unicode-ci-execution-v2"
PREPARE = HERE / "prepare-unicode-ci-root-v2.py"
CLI_METADATA = HERE / "candidate-cli-v1.json"
CLI_PATH = Path("/private/tmp/jarde-nonfinal-static-cli-v1")
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
RESULT = HERE / "unicode-ci-acceptance-luna-v3.json"
PINNED_COMMIT = "6fd51a18dd83d980f2f060246c209fb6fb0afba1"
EXPECTED = {
    "cli_metadata_sha256": "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a",
    "cli_sha256": "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e",
    "jdk_manifest_sha256": "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec",
    "prepare_sha256": "26257e6ca1b41823d69faeed58b6a31ed0677d7f6cd2243310e0cba9c86829c5",
    "unicode_test_source_sha256": "9776f2f52eb7324fd83e5f5e389181c0577098b419d8773a7b0212e62dfb0890",
    "unicode_test_patch_sha256": "daec9c976dc66b9fefcf8d360ea5782338b3660ac12ebb788a0c69cbb251b3fa",
    "fixture": {
        "UT.java": "2b122dce339956497a1e3ccef1b6152977cb6f44bf34fb59afa0076f13010d85",
        "UT.class": "00557f2a5007ad47a9d60839339c94de481e3cb5ee47da6e6255c92823a49d5c",
        "UT$内部类.class": "cda8bae084dac3852d7ce1ede1cde654041e452f4567a7047809f876a0b13de3",
    },
    "metadata_counts": {"candidate_sources": 10, "test_sources": 4, "canonical_files": 16},
    "stdout": "变量=1/42/中文\n",
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def b3(data):
    return blake3(data).hexdigest()


def read_record(row):
    path = Path(row["path"])
    payload = path.read_bytes() if path.is_absolute() else (OUT / path).read_bytes()
    assert len(payload) == row["bytes"], row["path"]
    assert sha(payload) == row["sha256"], row["path"]
    return payload


def raw_record(row):
    return (row["exit"], read_record(row["stdout"]), read_record(row["stderr"]))


def raw_command(row):
    return (row["exit"], read_record(row["stdout"]), read_record(row["stderr"]))


def bind_command_record(record, commands_by_label):
    label = record["label"]
    assert label in commands_by_label, f"nested command has no top-level command: {label}"
    command = commands_by_label[label]
    for key in ("argv", "cwd", "java_home", "exit", "stdout", "stderr"):
        assert record.get(key) == command.get(key), f"nested command mismatch: {label}:{key}"
    return command


def raw_name(member):
    return bytes(member.get("item", {}).get("name", {}).get("raw", [])).decode("utf-8")


def raw_descriptor(member):
    return bytes(member.get("item", {}).get("descriptor", {}).get("raw", [])).decode("ascii")


def method_key(member):
    return raw_name(member), raw_descriptor(member)


def parse_javap_methods(text, owner_simple_name):
    """Map physical method identities to the instruction BCIs in their javap Code blocks."""
    lines = text.splitlines()
    starts = []
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped == "static {};":
            starts.append((index, "<clinit>"))
            continue
        if not stripped.endswith(");") or "(" not in stripped:
            continue
        if stripped.startswith(("descriptor:", "Signature:", "flags:", "Code:")):
            continue
        signature = stripped[:-1]
        left = signature.split("(", 1)[0].split()
        if not left:
            continue
        name = left[-1]
        if name == owner_simple_name:
            name = "<init>"
        starts.append((index, name))

    methods = {}
    for ordinal, (start, name) in enumerate(starts):
        end = starts[ordinal + 1][0] if ordinal + 1 < len(starts) else len(lines)
        block = lines[start:end]
        descriptor_match = next((re.match(r"^\s*descriptor:\s*(\S+)\s*$", row)
                                 for row in block if re.match(r"^\s*descriptor:\s*\S+\s*$", row)), None)
        if descriptor_match is None:
            continue
        descriptor = descriptor_match.group(1)
        code_index = next((index for index, row in enumerate(block) if row.strip() == "Code:"), None)
        bcis = set()
        if code_index is not None:
            for row in block[code_index + 1:]:
                match = re.match(r"^\s*(\d+):\s+\S+", row)
                if match:
                    bcis.add(int(match.group(1)))
        key = (name, descriptor)
        assert key not in methods, f"duplicate physical javap method {key}"
        methods[key] = bcis
    return methods


def parse_putstatic(text, expected_owner):
    methods = parse_javap_methods(text, expected_owner)
    # The exact operation details are read from the isolated <clinit> Code block.
    lines = text.splitlines()
    start = next(index for index, line in enumerate(lines) if line.strip() == "static {};")
    end = next((index for index in range(start + 1, len(lines))
                if index != start and lines[index].strip().endswith(");")), len(lines))
    writes = []
    for line in lines[start:end]:
        match = re.match(r"^\s*(\d+):\s+putstatic\b.*//\s*Field\s+([^:]+):(\S+)\s*$", line)
        if match:
            writes.append({"bci": int(match.group(1)), "field": match.group(2),
                           "descriptor": match.group(3)})
    assert ("<clinit>", "()V") in methods
    return methods, writes


def parse_javap_fields(text):
    lines = text.splitlines()
    fields = []
    for index, line in enumerate(lines):
        match = re.match(r"^\s*descriptor:\s*(\S+)\s*$", line)
        if not match or index == 0:
            continue
        header = lines[index - 1].strip()
        if not header.endswith(";") or "(" in header or header == "static {};":
            continue
        words = header[:-1].split()
        if not words or words[0] in {"class", "interface", "enum", "record"}:
            continue
        fields.append((words[-1], match.group(1)))
    return fields


def document_source_map_checks(document, javap_methods, class_bytes,
                               archive_snapshot, expected_entry_name, entry_ordinal,
                               source_map_inventory):
    rows = []
    methods = document.get("methods", [])
    assert len(methods) > 0
    expected_class_metadata = {"digest": b3(class_bytes), "length": len(class_bytes)}
    expected_location = {
        "kind": "archive_entry",
        "entry": {
            "ordinal": entry_ordinal,
            "origin": {"root_container": "root", "snapshot": archive_snapshot, "steps": []},
            "raw_name": list(expected_entry_name.encode("utf-8")),
        },
    }
    document_class = document.get("class", {})
    assert document_class.get("class_bytes") == expected_class_metadata
    assert document_class.get("location") == expected_location
    assert document_class.get("variant") == {"kind": "base"}
    assert len(source_map_inventory) == len(methods)
    for method in methods:
        key = method_key(method)
        identity = method.get("item", {}).get("identity")
        assert identity is not None
        assert identity.get("name") == method.get("item", {}).get("name", {}).get("raw")
        assert identity.get("descriptor") == method.get("item", {}).get("descriptor", {}).get("raw")
        assert identity.get("owner", {}).get("class_bytes") == expected_class_metadata
        assert identity.get("owner", {}).get("location") == expected_location
        assert identity.get("owner", {}).get("variant") == {"kind": "base"}
        assert key in javap_methods, f"JSON method not in javap: {key}"
        outcome = method.get("outcome", {})
        assert outcome.get("kind") == "recovered", f"method was not recovered: {key}"
        report = outcome.get("report", {})
        assert report.get("quality") == "structured", f"method is not structured: {key}"
        assert report.get("representation") == "java", f"method is not Java: {key}"
        assert report.get("content") == "contains_statements", f"method body not represented as statements: {key}"
        assert report.get("fallbacks") == [], f"method contains fallback: {key}"
        source_map = report.get("source_map", {})
        segments = source_map.get("segments", [])
        report_text_bytes = report.get("text", "").encode("utf-8")
        origin_count = 0
        for segment in segments:
            start, end = segment.get("start"), segment.get("end")
            assert isinstance(start, int) and isinstance(end, int)
            assert 0 <= start < end <= len(report_text_bytes), f"source-map span outside UTF-8 report text: {key}"
            origin = segment.get("origin")
            if origin is None:
                continue
            entries = [origin.get("primary"), *origin.get("derived", [])]
            for entry in entries:
                if entry is None:
                    continue
                origin_count += 1
                assert entry.get("method") == identity, f"source-map owner/method mismatch: {key}"
                bci = entry.get("bci")
                assert isinstance(bci, int), f"source-map BCI missing: {key}"
                assert bci in javap_methods[key], f"source-map BCI absent from original javap: {key} @ {bci}"
        rows.append({"method": key, "identity": identity,
                     "source_map_segments": len(segments), "source_map_origins": origin_count})
    expected_inventory = [
        {"method_index": index,
         "name": method_key(method)[0],
         "descriptor": method_key(method)[1],
         "quality": method["outcome"]["report"]["quality"],
         "representation": method["outcome"]["report"]["representation"],
         "segment_count": len(method["outcome"]["report"].get("source_map", {}).get("segments", [])),
         "origin_count": rows[index]["source_map_origins"]}
        for index, method in enumerate(methods)
    ]
    assert source_map_inventory == expected_inventory
    return rows


def main():
    assert not RESULT.exists(), f"refusing to overwrite acceptance report: {RESULT}"
    assert package_version("blake3") == "1.0.11"
    assert OUT.is_dir() and EXECUTION.is_dir()
    assert sha(PREPARE.read_bytes()) == EXPECTED["prepare_sha256"]

    manifest_bytes = (OUT / "manifest.json").read_bytes()
    inventory_bytes = (OUT / "file-inventory.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    inventory = json.loads(inventory_bytes)
    assert manifest["schema"] == "unicode-ci-root-v2"
    assert manifest["status"] == "completed" and manifest["failures"] == []
    inventory_paths = {row["path"] for row in inventory}
    assert len(inventory_paths) == len(inventory)
    actual_paths = {path.relative_to(OUT).as_posix() for path in OUT.rglob("*")
                    if path.is_file() and path.name != "file-inventory.json"}
    assert inventory_paths == actual_paths
    for row in inventory:
        read_record(row)
    assert all(row.get("ok") for row in manifest["preflight"])

    # Independently bind the collector launch and its captured raw streams.
    execution = json.loads((EXECUTION / "execution.json").read_bytes())
    assert execution["exit"] == 0
    assert execution["cwd"] == str(ROOT)
    assert execution["argv"][0] == "/opt/homebrew/opt/python@3.14/bin/python3.14"
    assert Path(execution["argv"][-1]).resolve() == PREPARE.resolve()
    assert len(execution["argv"]) == 3 and execution["argv"][1] == "-B"
    assert sha(PREPARE.read_bytes()) == execution["script_sha256"]
    for stream_name in ("stdout", "stderr"):
        details = execution[stream_name]
        stream = (EXECUTION / details["path"]).read_bytes()
        assert len(stream) == details["bytes"]
        assert sha(stream) == details["sha256"]
    wrapper_stdout = json.loads((EXECUTION / execution["stdout"]["path"]).read_bytes())
    assert wrapper_stdout["status"] == "completed"
    assert wrapper_stdout["case_count"] == 14

    metadata_bytes = CLI_METADATA.read_bytes()
    metadata = json.loads(metadata_bytes)
    assert sha(metadata_bytes) == EXPECTED["cli_metadata_sha256"]
    assert metadata["cli_path"] == str(CLI_PATH)
    assert metadata["cli_sha256"] == EXPECTED["cli_sha256"]
    assert sha(CLI_PATH.read_bytes()) == EXPECTED["cli_sha256"]
    worktree_pin_audit = []
    for key, count in EXPECTED["metadata_counts"].items():
        pins = metadata[key]
        assert len(pins) == count
        for relative, expected_sha in pins.items():
            blob = subprocess.run(
                ["git", "show", "--no-textconv", f"{PINNED_COMMIT}:{relative}"],
                cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
            assert blob.returncode == 0, f"pinned commit blob unavailable: {relative}: {blob.stderr!r}"
            commit_blob_sha = sha(blob.stdout)
            assert commit_blob_sha == expected_sha, relative
            current_path = ROOT / relative
            current_sha = sha(current_path.read_bytes()) if current_path.is_file() else None
            worktree_pin_audit.append({
                "category": key,
                "path": relative,
                "pinned_commit_blob_sha256": commit_blob_sha,
                "current_worktree_sha256": current_sha,
                "current_matches_pin": current_sha == expected_sha,
            })
    assert sha((ROOT / "tests/recover_unicode_identifiers.rs").read_bytes()) == EXPECTED["unicode_test_source_sha256"]
    assert sha((HERE / "unicode-ci-repair-luna-v1.patch").read_bytes()) == EXPECTED["unicode_test_patch_sha256"]
    jdk_manifest_bytes = JDK_MANIFEST.read_bytes()
    assert sha(jdk_manifest_bytes) == EXPECTED["jdk_manifest_sha256"]
    jdk_manifest = json.loads(jdk_manifest_bytes)
    jdk_tools = {}
    for leg in jdk_manifest["legs"]:
        jdk_tools[leg["leg"]] = leg["jdk_tools"]
        for tool, details in leg["jdk_tools"].items():
            tool_path = Path(details["path"])
            assert len(tool_path.read_bytes()) == details["bytes"]
            assert sha(tool_path.read_bytes()) == details["sha256"]
    assert set(jdk_tools) == {"javac8", "javac23"}
    for name, expected_sha in EXPECTED["fixture"].items():
        source_path = ROOT / "tests/fixtures/recover-unicode-identifiers/ut" / name
        assert sha(source_path.read_bytes()) == expected_sha
        copied_path = OUT / ("original-sources/UT.java" if name == "UT.java"
                             else "frozen-inputs/" + name)
        assert copied_path.read_bytes() == source_path.read_bytes()

    commands = manifest["commands"]
    assert len(commands) == 30
    commands_by_label = {row["label"]: row for row in commands}
    assert len(commands_by_label) == len(commands)
    expected_labels = set()
    for leg in ("javac8", "javac23"):
        expected_labels.update({
            f"{leg}-javac-version", f"{leg}-java-version", f"{leg}-javap-version",
            f"{leg}-original-compile", f"{leg}-original-run",
            f"{leg}-original-UT-javap", f"{leg}-original-UT$内部类-javap",
            f"{leg}-UT-default-render", f"{leg}-UT-all-render",
            f"{leg}-UT-inner-内部类-default-render", f"{leg}-UT-inner-内部类-all-render",
            f"{leg}-default-two-class-family-compile", f"{leg}-default-two-class-family-run",
            f"{leg}-all-two-class-family-compile", f"{leg}-all-two-class-family-run",
        })
    assert set(commands_by_label) == expected_labels
    for command in commands:
        assert command["exit"] == 0, command["label"]
        read_record(command["stdout"])
        read_record(command["stderr"])
    rendered_commands = [row for row in commands if row["label"].endswith("-render")]
    assert len(rendered_commands) == 8
    assert len([row for row in commands if row["label"].endswith("-compile")]) == 6
    assert len([row for row in commands if row["label"].endswith("-run")]) == 6
    for row in rendered_commands:
        argv = row["argv"]
        assert argv[0] == str(CLI_PATH)
        assert argv[1:3] == ["class-source", "--input"]
        input_path = Path(argv[3])
        assert input_path.suffix == ".jar"
        assert argv[argv.index("--policy") + 1] == "plain-jar"
        assert argv[argv.index("--release") + 1] == "8"
        assert argv[argv.index("--format") + 1] == "json"
        profile_all = "--evidence" in argv
        if profile_all:
            assert argv[argv.index("--evidence") + 1] == "all"
        else:
            assert "--evidence" not in argv

    cases = manifest["cases"]
    assert len(cases) == 14
    originals = {case["jdk_leg"]: case for case in cases if case["kind"] == "original"}
    recovered = [case for case in cases if case["kind"] == "outer-source-complete-class-family"]
    rendered = [case for case in cases if case["kind"] == "rendered"]
    assert set(originals) == {"javac8", "javac23"}
    assert len(recovered) == 4 and len(rendered) == 8
    for original in originals.values():
        assert original["success"] and original["expected_utf8_stdout"]
        assert original["complete_two_class_output"]
        assert original["outer_physical_clinit_contract"]
        assert original["javap"]["UT"]["physical_clinit"] == {
            "present": True, "descriptor": "()V", "has_code": True}
        assert original["javac8_frozen_class_identity"] == (True if original["jdk_leg"] == "javac8" else None)
        source_row = original["source_files"][0]
        assert Path(source_row["path"]).name == "UT.java"
        source_bytes = read_record(source_row)
        assert source_bytes == (ROOT / "tests/fixtures/recover-unicode-identifiers/ut/UT.java").read_bytes()
        compile_argv = original["compile"]["argv"]
        bind_command_record(original["compile"], commands_by_label)
        bind_command_record(original["runtime"], commands_by_label)
        for option in ("-source", "-target", "-encoding", "-classpath", "-sourcepath", "-d"):
            assert option in compile_argv
        assert compile_argv[compile_argv.index("-source") + 1] == "8"
        assert compile_argv[compile_argv.index("-target") + 1] == "8"
        assert compile_argv[compile_argv.index("-encoding") + 1] == "UTF-8"
        assert len([arg for arg in compile_argv if arg.endswith(".java")]) == 1
        assert Path(next(arg for arg in compile_argv if arg.endswith(".java"))) == (OUT / source_row["path"]).resolve()
        original_classes_dir = (OUT / original["classes"][0]["path"]).resolve().parent
        assert Path(compile_argv[compile_argv.index("-d") + 1]) == original_classes_dir
        assert compile_argv[0] == jdk_tools[original["jdk_leg"]]["javac"]["path"]
        cp = Path(compile_argv[compile_argv.index("-classpath") + 1])
        sp = Path(compile_argv[compile_argv.index("-sourcepath") + 1])
        assert cp == sp and cp.is_dir() and list(cp.iterdir()) == []
        runtime_argv = original["runtime"]["argv"]
        assert "-Xverify:all" in runtime_argv and "-Dfile.encoding=UTF-8" in runtime_argv
        assert runtime_argv[0] == jdk_tools[original["jdk_leg"]]["java"]["path"]
        assert runtime_argv[-1] == "UT"
        original_cp = Path(runtime_argv[runtime_argv.index("-cp") + 1])
        assert original_cp == original_classes_dir
        assert {path.name for path in original_cp.rglob("*.class")} == {"UT.class", "UT$内部类.class"}

    original_raw = {leg: raw_record(case["runtime"]) for leg, case in originals.items()}
    assert original_raw["javac8"] == original_raw["javac23"]
    assert original_raw["javac8"] == (0, EXPECTED["stdout"].encode("utf-8"), b"")

    documents = {}
    javap_by_leg_class = {}
    for leg, original in originals.items():
        for class_name in ("UT", "UT$内部类"):
            command = next(row for row in commands
                           if row["label"] == f"{leg}-original-{class_name}-javap")
            javap_text = read_record(command["stdout"]).decode("utf-8")
            javap_by_leg_class[(leg, class_name)] = parse_javap_methods(
                javap_text, class_name)
            physical_fields = parse_javap_fields(javap_text)
            expected_physical_fields = (
                [("变量", "I"), ("描述", "Ljava/lang/String;")]
                if class_name == "UT" else [("名字", "Ljava/lang/String;")]
            )
            assert physical_fields == expected_physical_fields
            expected_physical_methods = (
                {("<init>", "()V"), ("方法", "(I)I"),
                 ("main", "([Ljava/lang/String;)V"), ("<clinit>", "()V")}
                if class_name == "UT" else {("<init>", "()V")}
            )
            assert set(javap_by_leg_class[(leg, class_name)]) == expected_physical_methods
        class_rows = {Path(row["path"]).name: read_record(row) for row in original["classes"]}
        assert set(class_rows) == {"UT.class", "UT$内部类.class"}
        jar_path = OUT / "input-jars" / leg / "UT.jar"
        with zipfile.ZipFile(jar_path) as archive:
            infos = archive.infolist()
            assert [info.filename for info in infos] == ["UT.class", "UT$内部类.class"]
            assert all(info.compress_type == zipfile.ZIP_STORED for info in infos)
            assert all(info.date_time == (1980, 1, 1, 0, 0, 0) for info in infos)
            assert {info.filename: archive.read(info.filename) for info in infos} == class_rows
        jar_record = original["plain_jar"]["file"]
        jar_bytes = read_record(jar_record)
        assert jar_bytes == jar_path.read_bytes()
        jar_snapshot = b3(jar_bytes)
        if leg == "javac8":
            assert b3(class_rows["UT.class"]) == "1668c25ae79374ff7f09a8a67e9c7efc807be4ec95cb0492399dbd853dbe52fe"
            assert jar_snapshot == "ad31ce3722a82e9495134ca2b868f3cc6ff5ee65eae6d5d75fe5d7f12d40b2d8"
        assert original["plain_jar"]["exact_entry_names"]
        for class_name in ("UT", "UT$内部类"):
            for profile in ("default", "all"):
                case = next(row for row in rendered if row["jdk_leg"] == leg
                            and row["class"] == class_name and row["profile"] == profile)
                assert case["success"]
                render_command = next(row for row in rendered_commands if row["label"] == case["render"]["label"])
                bind_command_record(case["render"], commands_by_label)
                argv = render_command["argv"]
                assert Path(argv[argv.index("--input") + 1]) == jar_path
                assert argv[argv.index("--class") + 1] == class_name
                doc_bytes = read_record(case["document"])
                assert read_record(case["render"]["stdout"]) == doc_bytes
                document = json.loads(doc_bytes)
                assert document["text"].encode("utf-8") == read_record(case["generated_source"])
                assert len(document.get("methods", [])) == case["inspection"]["full_method_count"]
                assert len(document.get("fields", [])) == case["inspection"]["field_projection_count"]
                documents[(leg, class_name, profile)] = document
                class_bytes = class_rows[class_name + ".class"]
                source_map_rows = document_source_map_checks(
                    document, javap_by_leg_class[(leg, class_name)], class_bytes,
                    jar_snapshot, class_name + ".class",
                    0 if class_name == "UT" else 1, case["source_map_inventory"])
                assert all(row["source_map_segments"] > 0 and row["source_map_origins"] > 0
                           for row in source_map_rows)
                assert document.get("execution", {}).get("status") == "complete"

        # Field projections in both default and full-evidence reports must
        # identify the physical members from the same exact archive entries.
        for class_name, field_specs, entry_ordinal in (
            ("UT", [("变量", "I", 8), ("描述", "Ljava/lang/String;", 8)], 0),
            ("UT$内部类", [("名字", "Ljava/lang/String;", None)], 1),
        ):
            physical_bytes = class_rows[class_name + ".class"]
            expected_class_meta = {"digest": b3(physical_bytes), "length": len(physical_bytes)}
            expected_location = {
                "kind": "archive_entry",
                "entry": {"ordinal": entry_ordinal,
                          "origin": {"root_container": "root", "snapshot": jar_snapshot, "steps": []},
                          "raw_name": list((class_name + ".class").encode("utf-8"))},
            }
            for profile in ("default", "all"):
                field_document = documents[(leg, class_name, profile)]
                projected_fields = field_document["fields"]
                assert [(raw_name(field), raw_descriptor(field)) for field in projected_fields] == [
                    (name, descriptor) for name, descriptor, _flags in field_specs]
                for index, (field, (name, descriptor, expected_flags)) in enumerate(
                        zip(projected_fields, field_specs)):
                    item = field["item"]
                    identity = item["identity"]
                    assert item["index"] == index
                    if expected_flags is not None:
                        assert item["access_flags"] == expected_flags
                    assert identity["member"]["kind"] == "field"
                    assert identity["member"]["name"] == item["name"]["raw"]
                    assert identity["member"]["descriptor"] == item["descriptor"]["raw"]
                    assert identity["owner"]["class_bytes"] == expected_class_meta
                    assert identity["owner"]["location"] == expected_location
                    assert identity["owner"]["variant"] == {"kind": "base"}

        outer_docs = [documents[(leg, "UT", profile)] for profile in ("default", "all")]
        assert outer_docs[0]["text"] == outer_docs[1]["text"]
        text = outer_docs[0]["text"]
        exact = [
            "static int 变量 = 1;",
            'static java.lang.String 描述 = new java.lang.StringBuilder().append("变量=").append(UT.变量).toString();',
            "static int 方法(int arg0)",
            "static class 内部类 extends java.lang.Object",
            "java.lang.String 名字;",
            "UT.描述",
            "方法(21)",
            "local1.名字",
        ]
        assert all(value in text for value in exact)
        assert "__" not in text
        assert "is not a Java identifier" not in text
        assert "not recovered: the recovery run" not in text

        # Physical members and initializer proof are tied to actual UTF-8 names,
        # access flags, member identities and original putstatic instruction BCIs.
        fields = outer_docs[0]["fields"]
        assert [raw_name(field) for field in fields] == ["变量", "描述"]
        assert [raw_descriptor(field) for field in fields] == ["I", "Ljava/lang/String;"]
        assert [field["item"]["access_flags"] for field in fields] == [8, 8]
        assert [field["item"]["index"] for field in fields] == [0, 1]
        outer_digest = outer_docs[0]["class"]["class_bytes"]["digest"]
        outer_class_bytes = class_rows["UT.class"]
        outer_location = {
            "kind": "archive_entry",
            "entry": {"ordinal": 0,
                      "origin": {"root_container": "root", "snapshot": jar_snapshot, "steps": []},
                      "raw_name": list(b"UT.class")},
        }
        for field in fields:
            identity = field["item"]["identity"]
            member = identity["member"]
            assert member["kind"] == "field"
            assert member["name"] == field["item"]["name"]["raw"]
            assert member["descriptor"] == field["item"]["descriptor"]["raw"]
            assert identity["owner"]["class_bytes"] == {"digest": b3(outer_class_bytes), "length": len(outer_class_bytes)}
            assert identity["owner"]["location"] == outer_location
            assert identity["owner"]["variant"] == {"kind": "base"}
            assert outer_digest == b3(outer_class_bytes)
        child = documents[(leg, "UT$内部类", "all")]
        assert [raw_name(field) for field in child["fields"]] == ["名字"]
        assert len(child["fields"]) == 1 and len(child["methods"]) == 1
        assert method_key(child["methods"][0]) == ("<init>", "()V")
        child_digest = child["class"]["class_bytes"]["digest"]
        child_class_bytes = class_rows["UT$内部类.class"]
        child_location = {
            "kind": "archive_entry",
            "entry": {"ordinal": 1,
                      "origin": {"root_container": "root", "snapshot": jar_snapshot, "steps": []},
                      "raw_name": list("UT$内部类.class".encode("utf-8"))},
        }
        child_field_identity = child["fields"][0]["item"]["identity"]
        child_field_member = child_field_identity["member"]
        assert child_field_member["kind"] == "field"
        assert child_field_member["name"] == child["fields"][0]["item"]["name"]["raw"]
        assert child_field_member["descriptor"] == child["fields"][0]["item"]["descriptor"]["raw"]
        assert child_field_identity["owner"]["class_bytes"] == {
            "digest": b3(child_class_bytes), "length": len(child_class_bytes)}
        assert child_field_identity["owner"]["location"] == child_location
        assert child_field_identity["owner"]["variant"] == {"kind": "base"}
        assert child_digest == b3(child_class_bytes)
        assert [method_key(method) for method in child["methods"]] == [("<init>", "()V")]
        assert set(javap_by_leg_class[(leg, "UT")]) == {
            ("<init>", "()V"), ("方法", "(I)I"),
            ("main", "([Ljava/lang/String;)V"), ("<clinit>", "()V")}
        assert set(javap_by_leg_class[(leg, "UT$内部类")]) == {("<init>", "()V")}
        assert [method_key(method) for method in outer_docs[0]["methods"]] == [
            ("<init>", "()V"), ("方法", "(I)I"),
            ("main", "([Ljava/lang/String;)V"), ("<clinit>", "()V")]
        proof = outer_docs[0]["initializer_proof"]
        assert outer_docs[1]["initializer_proof"] == proof
        assert proof["kind"] == "proved" and len(proof["fields"]) == 2
        assert [item["field_index"] for item in proof["fields"]] == [0, 1]
        assert [item["write_order"] for item in proof["fields"]] == [0, 1]
        original_clinit_methods, physical_writes = parse_putstatic(
            read_record(next(row for row in commands if row["label"] == f"{leg}-original-UT-javap")["stdout"]).decode("utf-8"), "UT")
        expected_writes = [{"field": "变量", "descriptor": "I"},
                           {"field": "描述", "descriptor": "Ljava/lang/String;"}]
        assert [{"field": row["field"], "descriptor": row["descriptor"]}
                for row in physical_writes] == expected_writes
        assert [item["write_bci"] for item in proof["fields"]] == [item["bci"] for item in physical_writes]
        assert ("<clinit>", "()V") in original_clinit_methods
        clinit = next(method for method in outer_docs[0]["methods"]
                      if method_key(method) == ("<clinit>", "()V"))
        clinit_identity = clinit["item"]["identity"]
        for profile in ("default", "all"):
            report = next(method for method in documents[(leg, "UT", profile)]["methods"]
                          if method_key(method) == ("<clinit>", "()V"))["outcome"]["report"]
            origins = []
            for segment in report.get("source_map", {}).get("segments", []):
                origin = segment.get("origin") or {}
                origins.extend([origin.get("primary"), *origin.get("derived", [])])
            origins = [origin for origin in origins if origin is not None]
            mapped_bcis = {origin["bci"] for origin in origins if origin.get("method") == clinit_identity}
            if origins:
                assert all(origin.get("method") == clinit_identity for origin in origins)
                assert {item["write_bci"] for item in proof["fields"]} <= mapped_bcis

    for case in recovered:
        assert case["success"] and case["generated_compile"]["compile_success"]
        assert case["generated_compile"]["runtime_success"]
        assert case["complete_two_class_output"]
        assert case["generated_source_is_exact_outer_report"]
        assert case["runtime_matches_original_raw"]
        assert case["expected_utf8_stdout"]
        assert raw_record(case["generated_compile"]["runtime"]) == original_raw[case["jdk_leg"]]
        assert Path(case["generated_compile"]["source_files"][0]["path"]).name == "UT.java"
        source_record = case["generated_compile"]["source_files"][0]
        generated_bytes = read_record(source_record)
        rendered_case = next(row for row in rendered
                             if row["jdk_leg"] == case["jdk_leg"]
                             and row["class"] == "UT"
                             and row["profile"] == case["profile"])
        assert generated_bytes == read_record(rendered_case["generated_source"])
        assert generated_bytes == documents[(case["jdk_leg"], "UT", case["profile"])]["text"].encode("utf-8")
        argv = case["generated_compile"]["compile"]["argv"]
        bind_command_record(case["generated_compile"]["compile"], commands_by_label)
        bind_command_record(case["generated_compile"]["runtime"], commands_by_label)
        assert argv[0] == jdk_tools[case["jdk_leg"]]["javac"]["path"]
        assert len([arg for arg in argv if arg.endswith(".java")]) == 1
        assert Path(next(arg for arg in argv if arg.endswith(".java"))) == (OUT / source_record["path"]).resolve()
        for option in ("-source", "-target", "-encoding", "-classpath", "-sourcepath", "-d"):
            assert option in argv
        assert argv[argv.index("-source") + 1] == "8"
        assert argv[argv.index("-target") + 1] == "8"
        assert argv[argv.index("-encoding") + 1] == "UTF-8"
        cp = Path(argv[argv.index("-classpath") + 1])
        sp = Path(argv[argv.index("-sourcepath") + 1])
        assert cp == sp and cp.is_dir() and list(cp.iterdir()) == []
        runtime_argv = case["generated_compile"]["runtime"]["argv"]
        assert runtime_argv[0] == jdk_tools[case["jdk_leg"]]["java"]["path"]
        assert "-Xverify:all" in runtime_argv and "-Dfile.encoding=UTF-8" in runtime_argv
        runtime_cp = Path(runtime_argv[runtime_argv.index("-cp") + 1])
        assert runtime_cp == Path(argv[argv.index("-d") + 1])
        assert {path.name for path in runtime_cp.rglob("*.class")} == {"UT.class", "UT$内部类.class"}

    # Every recorded process must use the pinned executable for its role and
    # JDK leg. This also binds version and javap commands, not just compilation.
    tools_by_leg = jdk_manifest["legs"]
    tools_by_leg = {row["leg"]: row["jdk_tools"] for row in tools_by_leg}
    for command in commands:
        label = command["label"]
        if label.startswith("javac8-"):
            leg = "javac8"
        elif label.startswith("javac23-"):
            leg = "javac23"
        else:
            raise AssertionError(f"unrecognized command label: {label}")
        if label.endswith("-render"):
            expected_executable = str(CLI_PATH)
        elif "-javac-version" in label or label.endswith("-compile"):
            expected_executable = tools_by_leg[leg]["javac"]["path"]
        elif "-java-version" in label or label.endswith("-run"):
            expected_executable = tools_by_leg[leg]["java"]["path"]
        elif "-javap-version" in label or label.endswith("-javap"):
            expected_executable = tools_by_leg[leg]["javap"]["path"]
        else:
            raise AssertionError(f"unrecognized command role: {label}")
        assert command["argv"][0] == expected_executable, label
        assert command["cwd"] == str(ROOT), label

    summary = {
        "schema": "unicode-ci-acceptance-luna-v3",
        "status": "accepted",
        "baseline_commit": PINNED_COMMIT,
        "metadata_pins_verified_from_commit_blobs": len(worktree_pin_audit),
        "worktree_pin_differences_are_informational": True,
        "metadata_pin_audit": worktree_pin_audit,
        "current_worktree_pin_differences": [row for row in worktree_pin_audit
                                             if not row["current_matches_pin"]],
        "manifest_sha256": sha(manifest_bytes),
        "inventory_sha256": sha(inventory_bytes),
        "closed_file_count": len(inventory),
        "command_count": len(commands),
        "render_commands": len(rendered_commands),
        "original_compile_run_legs": len(originals),
        "recovered_compile_run_legs": len(recovered),
        "raw_original_and_recovered_outputs_match": True,
        "plain_jar_exact_two_class_family": True,
        "outer_and_child_full_reports_checked": 8,
        "outer_static_field_proof_write_bcis_verified_against_javap": True,
        "source_map_owner_method_bci_checks": "all mapped origins in 8 full reports; each owner/method identity is exact and each BCI exists in that physical javap method",
        "metadata_sha256": sha(metadata_bytes),
        "claim_boundary": "Independent offline validation of frozen CLI public PlainJar rendering, exact source compilation and raw runtime parity; it does not run Cargo/JDK/CLI or claim CI execution. The Rust test was not locally executed because the root task deferred Cargo below the 20 GiB disk threshold.",
    }
    RESULT.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
