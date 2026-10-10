#!/usr/bin/env python3
"""Independently verify the archived array-literal-boundaries baseline.

This verifier reads recorded evidence and pinned inputs only. It never launches
Javac, Java, JADX, Jarde, Git, Cargo, or another process.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/array-literal-boundaries-next"
BASE = EVIDENCE / "baseline-root-v2"
MANIFEST_PATH = BASE / "manifest.json"
INVENTORY_PATH = BASE / "file-inventory.json"
OUTPUT = EVIDENCE / "results/baseline-verification-luna-v2.json"
METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JADX_LINK = Path("/opt/homebrew/bin/jadx")
UPSTREAM_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays")

CLI_PATH = "/private/tmp/jarde-nonfinal-static-cli-v1"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
PREPARER_SHA256 = "81ab55384513416c311af46ffc56d8587ce8933cb1849824bfcf992f117c72c9"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
SOURCE_SHA256 = {
    "LongArrayLimits": "e42608ddd6fcd766d276e421803960ec95a90cf74023155148ec603e0c84e8f7",
    "ConstantIntArray": "db6c3062d7ddee05df6ea1db091b55b691fb01ac96ee10065a3a0e1b39583f08",
    "DependentArrayStores": "d21029dde313ca1a4b5ce02fc96f620a42cac6363cc0a2076bf5ad958c8f49f6",
    "ArrayLiteralBoundariesRunner": "3690b8528bb7dd0ea785c51973faf72c19aa95f03464949fdb22db9374f7f0ef",
}
UPSTREAM = {
    "LongArrayLimits": ("TestArrayFill4.java", "30dc65aa5749b7287f12d9bd1eceecf9055399386cc82040c583562bc99134a4"),
    "ConstantIntArray": ("TestArrayFillConstReplace.java", "0055730150e31b81ad538497dee3f78725f054ef464f300786d5a23a842fd9e3"),
    "DependentArrayStores": ("TestArrayFillNegative.java", "c1a10209342aba4e3a1fda64e1eee30309b80b613c0e6b3485fa169a595b80a1"),
}
TARGETS = tuple(UPSTREAM)
RUNNER = "ArrayLiteralBoundariesRunner"
ALL_CLASSES = set(TARGETS) | {RUNNER}
EXPECTED_COUNTS = {
    "LongArrayLimits": {"fields": 1, "methods": 2},
    "ConstantIntArray": {"fields": 1, "methods": 2},
    "DependentArrayStores": {"fields": 0, "methods": 2},
}
EXPECTED_MEMBERS = {
    "LongArrayLimits": {
        "fields": {("ARRAY_SIZE", "I", 0x001A)},
        "methods": {("<init>", "()V", 0x0001), ("test", "()[J", 0x0001)},
    },
    "ConstantIntArray": {
        "fields": {("CONST_INT", "I", 0x0019)},
        "methods": {("<init>", "()V", 0x0001), ("test", "()[I", 0x0001)},
    },
    "DependentArrayStores": {
        "fields": set(),
        "methods": {("<init>", "()V", 0x0001), ("test", "()[I", 0x0001)},
    },
}
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
ACCESS_BITS = {
    "ACC_PUBLIC": 0x0001, "ACC_PRIVATE": 0x0002, "ACC_PROTECTED": 0x0004,
    "ACC_STATIC": 0x0008, "ACC_FINAL": 0x0010, "ACC_SYNCHRONIZED": 0x0020,
    "ACC_VOLATILE": 0x0040, "ACC_BRIDGE": 0x0040, "ACC_TRANSIENT": 0x0080,
    "ACC_VARARGS": 0x0080, "ACC_NATIVE": 0x0100, "ACC_ABSTRACT": 0x0400,
    "ACC_STRICT": 0x0800, "ACC_SYNTHETIC": 0x1000,
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def record_bytes(record: dict, base: Path = BASE) -> bytes:
    path = Path(record["path"])
    if not path.is_absolute():
        path = base / path
    require(path.is_file(), f"recorded file missing: {path}")
    data = path.read_bytes()
    require(len(data) == record["bytes"], f"record byte count differs: {path}")
    require(sha(data) == record["sha256"], f"record SHA-256 differs: {path}")
    return data


def verify_inventory(manifest: dict) -> dict:
    inv_ref = manifest["file_inventory"]
    require(inv_ref == {"path": "file-inventory.json", "includes": ["manifest.json"],
                        "excludes": ["file-inventory.json"]}, "inventory policy mismatch")
    require(INVENTORY_PATH.is_file(), "file inventory missing")
    rows = load_json(INVENTORY_PATH)
    require(isinstance(rows, list), "inventory must be a list")
    expected = {}
    for row in rows:
        path = row["path"]
        require(path not in expected, f"duplicate inventory row: {path}")
        expected[path] = (row["bytes"], row["sha256"])
    require("manifest.json" in expected, "inventory does not include manifest")
    actual = {path.relative_to(BASE).as_posix() for path in BASE.rglob("*")
              if path.is_file() and path != INVENTORY_PATH}
    require(actual == set(expected), f"closed inventory mismatch: extra={actual - set(expected)}, missing={set(expected) - actual}")
    for relative, (size, digest) in expected.items():
        data = (BASE / relative).read_bytes()
        require(len(data) == size and sha(data) == digest, f"inventory hash mismatch: {relative}")
    return {"file_count": len(rows), "closed": True,
            "manifest_sha256": sha((BASE / "manifest.json").read_bytes())}


def verify_recorded_streams(commands: list[dict]) -> tuple[dict[str, dict], dict]:
    by_label, stream_paths = {}, set()
    for command in commands:
        label = command["label"]
        require(label not in by_label, f"duplicate command label: {label}")
        by_label[label] = command
        require(isinstance(command["exit"], int), f"missing exit code: {label}")
        for stream_name in ("stdout", "stderr"):
            stream = command[stream_name]
            require(stream["path"] not in stream_paths, f"duplicate raw stream path: {stream['path']}")
            stream_paths.add(stream["path"])
            record_bytes(stream)
    return by_label, {"command_count": len(by_label), "raw_stream_count": len(stream_paths),
                      "unique_labels": True, "unique_stream_paths": True,
                      "all_raw_bytes_match_records": True}


def verify_frozen_inputs(manifest: dict, metadata: dict) -> dict:
    md_path = Path(manifest["cli_metadata"]["path"])
    require(md_path == METADATA, "CLI metadata path mismatch")
    md_bytes = METADATA.read_bytes()
    require(sha(md_bytes) == METADATA_SHA256 == manifest["cli_metadata"]["sha256"], "CLI metadata SHA mismatch")
    md = json.loads(md_bytes)
    frozen = manifest["frozen_cli"]
    require(frozen == {"path": CLI_PATH, "sha256": CLI_SHA256}, "frozen CLI pin mismatch")
    require(md["cli_path"] == CLI_PATH and md["cli_sha256"] == CLI_SHA256, "metadata CLI declaration mismatch")
    require(sha(Path(CLI_PATH).read_bytes()) == CLI_SHA256, "frozen CLI actual bytes mismatch")
    require(manifest["jdk_manifest"] == {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
            "JDK manifest pin mismatch")
    jdk_bytes = JDK_MANIFEST.read_bytes()
    require(sha(jdk_bytes) == JDK_MANIFEST_SHA256, "JDK manifest actual bytes mismatch")
    jdk = json.loads(jdk_bytes)
    jdk_tools = {}
    for leg in jdk["legs"]:
        name = leg["leg"]
        require(name in ("javac8", "javac23"), f"unexpected JDK leg: {name}")
        jdk_tools[name] = {}
        for tool_name in ("java", "javac", "javap"):
            tool = leg["jdk_tools"][tool_name]
            data = Path(tool["path"]).read_bytes()
            require(len(data) == tool["bytes"] and sha(data) == tool["sha256"],
                    f"pinned JDK executable changed: {name}/{tool_name}")
            jdk_tools[name][tool_name] = tool["path"]
    require(set(jdk_tools) == {"javac8", "javac23"}, "required JDK toolchain legs missing")
    require(set(manifest["candidate_sources"]) == set(md["candidate_sources"]), "product pin set differs from CLI metadata")
    require(set(manifest["test_sources"]) == set(md["test_sources"]), "test pin set differs from CLI metadata")
    require(len(manifest["candidate_sources"]) == 10 and len(manifest["test_sources"]) == 4,
            "frozen product/test pin set size changed")
    for category in ("candidate_sources", "test_sources"):
        for relative, digest in manifest[category].items():
            require(digest == md[category][relative], f"{category} pin differs from metadata: {relative}")
            require(sha((ROOT / relative).read_bytes()) == digest, f"current pinned source changed: {relative}")

    jadx = manifest["jadx"]
    require(jadx["path"] == str(JADX_LINK) and jadx["version"] == JADX_VERSION, "JADX executable/version pin mismatch")
    jadx_executable = Path(jadx["executable_path"])
    require(jadx_executable == JADX_LINK.resolve(), "JADX resolved executable path mismatch")
    require(jadx["executable_sha256"] == JADX_SHA256 and sha(jadx_executable.read_bytes()) == JADX_SHA256,
            "JADX executable bytes mismatch")
    require(all(row.get("ok") is True for row in manifest["preflight"]), "manifest records a failed preflight")
    require(manifest["environment_policy"] == {
        "removed_for_each_process": ["JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"],
        "JDK_HOME_and_PATH_set_for_JDK_processes": True,
    }, "recorded process environment policy mismatch")
    return {"metadata_sha256": sha(md_bytes), "cli_sha256": CLI_SHA256,
            "jdk_manifest_sha256": sha(jdk_bytes), "jdk_tools_verified": 6,
            "jadx_executable_sha256": JADX_SHA256,
            "candidate_source_pin_count": len(manifest["candidate_sources"]),
            "test_source_pin_count": len(manifest["test_sources"]), "jdk_tools": jdk_tools}


def expected_command_labels() -> set[str]:
    labels = {"jadx-version", "jadx-default-decompile", "jadx-none-decompile"}
    for leg in ("javac8", "javac23"):
        labels.update({f"{leg}-original-compile", f"{leg}-original-run",
                       f"{leg}-jarde-compile", f"{leg}-jarde-run"})
        for class_name in (*TARGETS, RUNNER):
            labels.add(f"{leg}-original-javap-{class_name}")
        for profile in ("default", "none"):
            labels.update({f"{leg}-jadx-{profile}-compile", f"{leg}-jadx-{profile}-run"})
        for class_name in TARGETS:
            for mode in ("default", "all"):
                labels.add(f"{leg}-jarde-render-{class_name}-{mode}")
    return labels


def verify_command_provenance(commands: dict[str, dict], jdk_tools: dict[str, dict]) -> None:
    require(set(commands) == expected_command_labels(),
            f"unexpected command label set; missing={expected_command_labels() - set(commands)}, extra={set(commands) - expected_command_labels()}")
    for label, command in commands.items():
        require(command["cwd"] == str(ROOT), f"unexpected command cwd: {label}")
        argv = command["argv"]
        require(argv, f"empty argv: {label}")
        leg = "javac8" if label.startswith("javac8-") else "javac23" if label.startswith("javac23-") else None
        if label == "jadx-version":
            require(argv == [str(JADX_LINK), "--version"] and command["java_home"] is None,
                    "JADX version command argv mismatch")
        elif label.startswith("jadx-"):
            profile = "default" if "default" in label else "none"
            expected = [str(JADX_LINK), "--no-res", "--config", "none", "--threads-count", "1"]
            if profile == "none":
                expected += ["--rename-flags", "none"]
            expected += ["-d", str(BASE / "jadx" / profile), str(BASE / "jadx-input/ArrayLiteralBoundaryTargets.jar")]
            require(argv == expected, f"JADX decompile argv mismatch: {label}")
            require(command["java_home"] == str(Path(jdk_tools["javac23"]["java"]).parent.parent),
                    f"JADX profile JDK home mismatch: {label}")
        elif leg is not None:
            executable = None
            if "-javap-" in label:
                executable = jdk_tools[leg]["javap"]
            elif label.endswith("-compile"):
                executable = jdk_tools[leg]["javac"]
            elif label.endswith("-run"):
                executable = jdk_tools[leg]["java"]
            elif "-render-" in label:
                executable = CLI_PATH
            elif "-jadx-" in label:
                executable = jdk_tools[leg]["javac"] if label.endswith("-compile") else jdk_tools[leg]["java"]
            require(argv[0] == executable, f"tool argv mismatch: {label}")
            home = str(Path(executable).parent.parent) if executable != CLI_PATH else str(Path(jdk_tools[leg]["java"]).parent.parent)
            require(command["java_home"] == home, f"tool JAVA_HOME mismatch: {label}")
    version = commands["jadx-version"]
    require(version["exit"] == 0, "JADX version command failed")
    version_stdout, version_stderr = command_output(version)
    require(version_stdout.strip() == JADX_VERSION.encode("ascii") and not version_stderr,
            "actual JADX version output differs from pinned version")


def verify_archived_sources(manifest: dict) -> dict:
    expected_names = {*(name + ".java" for name in TARGETS), RUNNER + ".java",
                      *(filename for filename, _ in UPSTREAM.values())}
    archives = BASE / "original-sources"
    actual_names = {path.name for path in archives.iterdir() if path.is_file()}
    require(actual_names == expected_names, "original source archive set differs")
    rows = {row["class"]: row for row in manifest["source_identities"]}
    require(set(rows) == set(TARGETS), "upstream identity row set mismatch")
    for class_name, (filename, expected) in UPSTREAM.items():
        source_row = rows[class_name]
        snapshot = EVIDENCE / "upstream" / filename
        live = Path(source_row["path"])
        adapter = EVIDENCE / "sources" / f"{class_name}.java"
        require(source_row["snapshot"] == f"upstream/{filename}", f"snapshot path mismatch: {class_name}")
        require(source_row["expected_sha256"] == expected and source_row["ok"] is True,
                f"upstream manifest identity mismatch: {class_name}")
        snapshot_bytes = snapshot.read_bytes()
        live_bytes = live.read_bytes()
        archived = (archives / filename).read_bytes()
        require(sha(snapshot_bytes) == sha(live_bytes) == sha(archived) == expected
                and source_row["snapshot_sha256"] == sha(snapshot_bytes)
                and source_row["live_sha256"] == sha(live_bytes),
                f"upstream source drift: {class_name}")
        adapter_bytes = adapter.read_bytes()
        require(sha(adapter_bytes) == SOURCE_SHA256[class_name], f"adapted source SHA mismatch: {class_name}")
        require((archives / f"{class_name}.java").read_bytes() == adapter_bytes,
                f"adapter archive differs: {class_name}")
    runner_bytes = (EVIDENCE / "sources" / f"{RUNNER}.java").read_bytes()
    require(sha(runner_bytes) == SOURCE_SHA256[RUNNER], "Runner source SHA mismatch")
    require((archives / f"{RUNNER}.java").read_bytes() == runner_bytes, "Runner archive differs")
    for row in manifest["sources"]:
        raw = record_bytes(row, EVIDENCE)
        name = Path(row["path"]).stem
        require(sha(raw) == row["expected_sha256"] == SOURCE_SHA256[name], f"source record mismatch: {name}")
    return {name: SOURCE_SHA256[name] for name in SOURCE_SHA256}


def extract_block(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start:index + 1]
    raise ValueError(f"unclosed method: {signature}")


def normalized(fragment: str) -> str:
    return "\n".join(line.strip() for line in fragment.strip().splitlines())


def verify_selected_bodies() -> dict:
    rows = []
    for class_name, (filename, _) in UPSTREAM.items():
        upstream = (EVIDENCE / "upstream" / filename).read_text(encoding="utf-8")
        adapted = (EVIDENCE / "sources" / f"{class_name}.java").read_text(encoding="utf-8")
        if class_name == "LongArrayLimits":
            expected_field = "private static final int ARRAY_SIZE = 4;"
            old_field = next(line.strip() for line in upstream.splitlines() if "ARRAY_SIZE = 4;" in line)
            new_field = next(line.strip() for line in adapted.splitlines() if "ARRAY_SIZE = 4;" in line)
            require(old_field == new_field == expected_field, "LongArrayLimits field changed")
            signature = "public long[] test() {"
        elif class_name == "ConstantIntArray":
            expected_field = "public static final int CONST_INT = 0xffff;"
            old_field = next(line.strip() for line in upstream.splitlines() if "CONST_INT = 0xffff;" in line)
            new_field = next(line.strip() for line in adapted.splitlines() if "CONST_INT = 0xffff;" in line)
            require(old_field == new_field == expected_field, "ConstantIntArray field changed")
            signature = "public int[] test() {"
        else:
            signature = "public int[] test() {"
        require(normalized(extract_block(upstream, signature)) == normalized(extract_block(adapted, signature)),
                f"selected source body differs: {class_name}")
        rows.append({"class": class_name, "selected_body_matches": True})
    required = {
        ("LongArrayLimits", "field:ARRAY_SIZE"), ("LongArrayLimits", "method:test"),
        ("ConstantIntArray", "field:CONST_INT"), ("ConstantIntArray", "method:test"),
        ("DependentArrayStores", "method:test"),
    }
    reported = {(row["class"], row["member"]) for row in load_json(MANIFEST_PATH)["selected_member_identity"]
                if row["body_matches_after_indent_strip"] is True and row["ok"] is True}
    require(reported == required, "selected-member identity evidence incomplete or false")
    return {"selected_methods_verified": len(rows), "selected_members": len(required)}


def parse_javap(raw: bytes, class_name: str) -> dict:
    lines = raw.decode("utf-8", errors="replace").splitlines()
    members = [index for index, line in enumerate(lines)
               if re.match(r"^\s{2}(?:public|private|protected)\s+.+;\s*$", line)]
    fields, methods, bcis, opcodes = set(), set(), {}, {}
    for pos, start in enumerate(members):
        end = members[pos + 1] if pos + 1 < len(members) else len(lines)
        declaration = lines[start].strip()[:-1]
        section = lines[start + 1:end]
        descriptor_line = next((line.strip() for line in section if line.strip().startswith("descriptor:")), None)
        flags_line = next((line.strip() for line in section if line.strip().startswith("flags:")), None)
        require(descriptor_line is not None and flags_line is not None, f"javap signature metadata missing: {class_name} {declaration}")
        descriptor = descriptor_line.split("descriptor:", 1)[1].strip()
        flags = 0
        for flag in re.findall(r"ACC_[A-Z_]+", flags_line):
            require(flag in ACCESS_BITS, f"unmapped access flag {flag}")
            flags |= ACCESS_BITS[flag]
        if "(" in declaration:
            source_name = declaration.split("(", 1)[0].split()[-1]
            name = "<init>" if source_name == class_name else source_name
            method = (name, descriptor, flags)
            methods.add(method)
            instructions = {}
            for line in section:
                match = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)\s*(.*?)\s*$", line)
                if match:
                    instructions[int(match.group(1))] = (match.group(2), match.group(3))
            require(instructions, f"javap method has no instructions: {class_name} {name}{descriptor}")
            bcis[(name, descriptor)] = set(instructions)
            opcodes[(name, descriptor)] = instructions
        else:
            name = declaration.split()[-1]
            fields.add((name, descriptor, flags))
    require(methods == EXPECTED_MEMBERS[class_name]["methods"], f"physical method inventory differs: {class_name}: {methods}")
    require(fields == EXPECTED_MEMBERS[class_name]["fields"], f"physical field inventory differs: {class_name}: {fields}")
    require(len(bcis) == 2, f"physical method count differs: {class_name}")
    return {"fields": fields, "methods": methods, "bcis": bcis, "opcodes": opcodes}


def verify_class_census(case: dict, package: str | None = None) -> None:
    prefix = package.replace(".", "/") + "/" if package else ""
    expected_paths = {prefix + f"{name}.class" for name in ALL_CLASSES}
    class_rows = case["classes"]
    actual_paths = set()
    for row in class_rows:
        record_bytes(row)
        class_path = Path(row["path"])
        if not class_path.is_absolute():
            class_path = BASE / class_path
        actual_paths.add(class_path.relative_to(Path(case["class_output"])).as_posix())
    require(len(class_rows) == len(expected_paths) and actual_paths == expected_paths,
            f"class output census differs: {case['label']}")
    census = case["class_output_set"]
    require(census["complete"] is True and set(census["expected"]) == expected_paths
            and set(census["actual"]) == expected_paths, f"reported class census differs: {case['label']}")


def verify_compile_runtime(case: dict, leg: str, expected_source_paths: list[str], runner_class: str,
                           tools: dict, original_runtime: dict | None = None,
                           package: str | None = None) -> dict:
    label = case["label"]
    compile_cmd, run_cmd = case["compile"], case["runtime"]
    argv = compile_cmd["argv"]
    require(argv[0] == tools[leg]["javac"], f"{label}: javac executable mismatch")
    require(argv[1:6] == ["-source", "8", "-target", "8", "-g:none"], f"{label}: compile flags mismatch")
    require("-Xlint:-options" in argv, f"{label}: compile lint flag missing")
    cp, sp = argv[argv.index("-classpath") + 1], argv[argv.index("-sourcepath") + 1]
    require(cp == sp == case["empty_classpath_sourcepath"], f"{label}: class/source path mismatch")
    require(Path(cp).is_dir() and not any(Path(cp).iterdir()), f"{label}: class/source path must be empty")
    out_dir = argv[argv.index("-d") + 1]
    require(out_dir == case["class_output"], f"{label}: classes output path mismatch")
    actual_sources = argv[argv.index("-d") + 2:]
    require(actual_sources == expected_source_paths, f"{label}: compile source argv differs from recorded full source set")
    expected_compile_argv = [tools[leg]["javac"], "-source", "8", "-target", "8", "-g:none",
                             "-Xlint:-options", "-classpath", cp, "-sourcepath", sp,
                             "-d", out_dir, *expected_source_paths]
    require(argv == expected_compile_argv, f"{label}: compile argv contains unexpected or reordered arguments")
    require([str(BASE / row["path"]) for row in case["source_files"]] == expected_source_paths,
            f"{label}: source file records differ from argv order")
    for row in case["source_files"]:
        record_bytes(row)
    expected_runtime_argv = [tools[leg]["java"], "-Xverify:all", "-cp", out_dir, runner_class]
    require(run_cmd["argv"] == expected_runtime_argv, f"{label}: verified fresh-classes run argv mismatch")
    compile_out, compile_err = command_output(compile_cmd)
    require(compile_cmd["exit"] == 0 and compile_out == b"" and compile_err == b"", f"{label}: compilation did not complete cleanly")
    run_out, run_err = command_output(run_cmd)
    require(run_cmd["exit"] == 0, f"{label}: runtime failed")
    if original_runtime is not None:
        original_out, original_err = command_output(original_runtime)
        require((run_cmd["exit"], run_out, run_err) == (original_runtime["exit"], original_out, original_err),
                f"{label}: raw runtime triple differs from same-JDK original")
    verify_class_census(case, package)
    return {"label": label, "compile_exit": compile_cmd["exit"], "runtime_exit": run_cmd["exit"],
            "runtime_raw_matches_original": original_runtime is not None}


def command_output(command: dict) -> tuple[bytes, bytes]:
    return record_bytes(command["stdout"]), record_bytes(command["stderr"])


def byte_list(data) -> bytes:
    require(isinstance(data, list) and all(isinstance(x, int) and 0 <= x <= 255 for x in data), "invalid byte identity")
    return bytes(data)


def verify_document_owner(owner: dict, class_name: str, input_bytes: bytes, context: str) -> None:
    class_bytes = owner["class_bytes"]
    location = owner["location"]
    require(class_bytes["length"] == len(input_bytes), f"input byte length mismatch: {context}")
    require(location["kind"] == "standalone_root", f"input location is not standalone: {context}")
    require(location["snapshot"] == class_bytes["digest"], f"snapshot/digest mismatch: {context}")
    require(owner["variant"] == {"kind": "base"}, f"unexpected input variant: {context}")


def verify_jarde_document(render: dict, class_name: str, physical: dict, evidence_mode: str,
                          tools: dict[str, dict]) -> dict:
    profile = next(row for row in render["profiles"] if row["evidence_mode"] == evidence_mode)
    command = profile["command"]
    require(command["exit"] == 0 and profile["success"], f"Jarde render failed: {command['label']}")
    leg = "javac8" if command["label"].startswith("javac8-") else "javac23"
    original_class = render["original_class"]
    input_class = BASE / original_class["path"]
    expected_argv = [CLI_PATH, "class-source", "--input", str(input_class), "--class", class_name,
                     "--policy", "single-class", "--release", "8", "--format", "json"]
    if evidence_mode == "all":
        expected_argv += ["--evidence", "all"]
    require(command["argv"] == expected_argv, f"Jarde render argv mismatch: {command['label']}")
    require(command["java_home"] == str(Path(tools[leg]["java"]).parent.parent),
            f"Jarde render JAVA_HOME mismatch: {command['label']}")
    stdout, stderr = command_output(command)
    require(bool(stdout) and not stderr, f"Jarde render streams invalid: {command['label']}")
    json_bytes = record_bytes(profile["class_source_json"])
    require(json_bytes == stdout, f"Jarde document differs from recorded CLI stdout: {command['label']}")
    source_bytes = record_bytes(profile["generated_source"])
    text_bytes = record_bytes(profile["class_source_text"])
    document = json.loads(json_bytes)
    require(source_bytes == text_bytes == document["text"].encode("utf-8"),
            f"generated source differs from document text: {command['label']}")
    require(document["outcome"] == "performed" and document["declaration"]["name"] == class_name,
            f"class-source envelope mismatch: {command['label']}")
    require(document["coverage"]["artifact_structural"]["state"] == "complete_within_schema",
            f"incomplete class structure: {command['label']}")
    require(len(document["methods"]) == 2 and len(document["fields"]) == EXPECTED_COUNTS[class_name]["fields"],
            f"member count mismatch: {command['label']}")
    owner = document["class"]
    verify_document_owner(owner, class_name, record_bytes(original_class), command["label"])
    require(document["execution"]["status"] == "complete",
            f"class-source execution was not complete: {command['label']}")
    method_ids = set()
    for index, method in enumerate(document["methods"]):
        item = method["item"]
        identity = item["identity"]
        require(item["index"] == index, f"method index is not contiguous: {command['label']}")
        require(identity["owner"] == owner, f"method owner identity is incomplete: {command['label']}")
        name, descriptor = byte_list(identity["name"]).decode(), byte_list(identity["descriptor"]).decode("ascii")
        require(byte_list(item["name"]["raw"]) == byte_list(identity["name"])
                and item["name"]["escaped"] == name
                and byte_list(item["descriptor"]["raw"]) == byte_list(identity["descriptor"])
                and item["descriptor"]["escaped"] == descriptor,
                f"method item name/descriptor differs from its identity: {command['label']}")
        flags = item["access_flags"]
        require((name, descriptor, flags) in physical["methods"], f"method identity/flags differ from javap: {command['label']}")
        require((name, descriptor, flags) not in method_ids,
                f"duplicate method identity: {command['label']} {name}{descriptor}")
        method_ids.add((name, descriptor, flags))
        require(method["outcome"]["kind"] == "recovered" and method["outcome"]["report"]["outcome"] == "produced",
                f"physical method not recovered: {command['label']} {name}{descriptor}")
        analysis = method["outcome"]["analysis"]
        report = method["outcome"]["report"]
        require(analysis["execution"]["status"] == "complete"
                and report["execution"]["status"] == "complete",
                f"method analysis or report execution incomplete: {command['label']} {name}{descriptor}")
        require(report["quality"] == "structured" and report["representation"] == "java"
                and report["content"] == "contains_statements" and report["fallbacks"] == [],
                f"method is not a structured Java statement result: {command['label']} {name}{descriptor}")
        if evidence_mode == "all":
            category = next(row for row in report["evidence"]["categories"] if row["kind"] == "source_map")
            require(category["state"]["state"] == "complete", f"source map incomplete: {command['label']} {name}{descriptor}")
            mapped = set()
            for segment in report["source_map"]["segments"]:
                require(0 <= segment["start"] < segment["end"] <= len(report["text"].encode("utf-8")),
                        f"source-map segment out of bounds: {command['label']}")
                origin = segment["origin"]
                origins = ([origin["primary"]] if origin.get("primary") else []) + origin.get("derived", [])
                require(origins, f"source-map segment has no origin: {command['label']}")
                for source in origins:
                    require(source["method"] == identity, f"source-map full method identity mismatch: {command['label']}")
                    mapped.add(source["bci"])
            require(mapped == physical["bcis"][(name, descriptor)],
                    f"source map fails exact physical BCI coverage: {command['label']} {name}{descriptor}")
    require(method_ids == physical["methods"], f"method inventory incomplete: {command['label']}")
    field_ids = set()
    for index, field in enumerate(document["fields"]):
        item = field["item"]
        identity = item["identity"]
        require(item["index"] == index, f"field index is not contiguous: {command['label']}")
        require(identity["owner"] == owner, f"field owner identity is incomplete: {command['label']}")
        member = identity["member"]
        name, descriptor = byte_list(member["name"]).decode(), byte_list(member["descriptor"]).decode("ascii")
        require(byte_list(item["name"]["raw"]) == byte_list(member["name"])
                and item["name"]["escaped"] == name
                and byte_list(item["descriptor"]["raw"]) == byte_list(member["descriptor"])
                and item["descriptor"]["escaped"] == descriptor,
                f"field item name/descriptor differs from its identity: {command['label']}")
        field_id = (name, descriptor, item["access_flags"])
        require(field_id in physical["fields"], f"field identity/flags differ from javap: {command['label']}")
        require(field_id not in field_ids, f"duplicate field identity: {command['label']} {name}:{descriptor}")
        field_ids.add(field_id)
    require(field_ids == physical["fields"], f"field inventory incomplete: {command['label']}")
    if class_name == "ConstantIntArray":
        require("public static final int CONST_INT = 65535;" in document["text"],
                "constant field must be rendered as literal 65535")
        require("return new int[]{127, 129, 65535};" in document["text"],
                "constant array must preserve the 65535 literal value")
    elif class_name == "LongArrayLimits":
        require("return new long[]{0L, 1L, 9223372036854775807L, -9223372036854775807L};" in document["text"],
                "long array raw 64-bit boundary values differ")
    else:
        expected_lines = (
            "int[] local1 = new int[3];",
            "local1[0] = 1;",
            "local1[1] = local1[0] + 1;",
            "local1[2] = local1[1] + 1;",
            "return local1;",
        )
        require(all(line in document["text"] for line in expected_lines),
                "dependent element stores or temporary array source were not preserved")
    return {"mode": evidence_mode, "field_count": len(field_ids), "method_count": len(method_ids),
            "full_bci_map_verified": evidence_mode == "all", "generated_source_sha256": sha(source_bytes),
            "text": source_bytes.decode("utf-8")}


def verify_originals(cases: dict, tools: dict, commands: dict) -> tuple[dict, dict]:
    originals, physical = {}, {}
    for leg in ("javac8", "javac23"):
        case = cases[f"{leg}-original"]
        require(case["kind"] == "original" and case["jdk_leg"] == leg, f"original case identity mismatch: {leg}")
        source_args = [str(BASE / row["path"]) for row in case["source_files"]]
        require([Path(row["path"]).name for row in case["source_files"]] ==
                ["LongArrayLimits.java", "ConstantIntArray.java", "DependentArrayStores.java", f"{RUNNER}.java"],
                f"original source order/set mismatch: {leg}")
        expected_paths = [str(BASE / "cases" / f"{leg}-original" / "input-sources" / name)
                          for name in ("LongArrayLimits.java", "ConstantIntArray.java", "DependentArrayStores.java", f"{RUNNER}.java")]
        require(source_args == expected_paths, f"original source inputs are not the archived four-class set: {leg}")
        for row, name in zip(case["source_files"], ("LongArrayLimits.java", "ConstantIntArray.java", "DependentArrayStores.java", f"{RUNNER}.java")):
            expected_data = (BASE / "original-sources" / name).read_bytes()
            require(record_bytes(row) == expected_data, f"original compile source differs from archive: {leg}/{name}")
        verify_compile_runtime(case, leg, expected_paths, RUNNER, tools)
        require(case["success"] is True and case["complete_class_set"] is True, f"original case reported failure: {leg}")
        originals[leg] = case
        class_map = {Path(row["path"]).stem: row for row in case["classes"]}
        for class_name in TARGETS:
            rows = [row for row in case["javap"] if row["class"] == class_name]
            require(len(rows) == 1 and rows[0]["success"] and rows[0]["complete_inventory"],
                    f"javap record missing/incomplete: {leg}/{class_name}")
            javap = rows[0]
            require(javap["class_file"]["path"] == class_map[class_name]["path"],
                    f"javap class does not belong to this original compile: {leg}/{class_name}")
            command = javap["command"]
            require(command["label"] == f"{leg}-original-javap-{class_name}", f"javap label mismatch: {leg}/{class_name}")
            require(command["argv"] == [tools[leg]["javap"], "-p", "-c", "-s", "-v",
                                        str(BASE / class_map[class_name]["path"])],
                    f"javap argv mismatch: {leg}/{class_name}")
            stdout, stderr = command_output(command)
            require(stdout and not stderr, f"javap raw streams invalid: {leg}/{class_name}")
            parsed = parse_javap(stdout, class_name)
            require(javap["method_count"] == len(parsed["methods"])
                    and javap["field_count"] == len(parsed["fields"]), f"javap count summary mismatch: {leg}/{class_name}")
            physical[(leg, class_name)] = parsed
        runner_rows = [row for row in case["javap"] if row["class"] == RUNNER]
        require(len(runner_rows) == 1 and runner_rows[0]["success"], f"Runner javap absent: {leg}")
        runner_javap = runner_rows[0]
        runner_command = runner_javap["command"]
        runner_class = next(row for row in case["classes"] if Path(row["path"]).stem == RUNNER)
        require(runner_command["argv"] == [tools[leg]["javap"], "-p", "-c", "-s", "-v",
                                           str(BASE / runner_class["path"])],
                f"Runner javap argv mismatch: {leg}")
        require(runner_javap["field_count"] >= 0 and runner_javap["method_count"] > 0,
                f"Runner javap census missing: {leg}")
    raw = {}
    for leg, case in originals.items():
        stdout, stderr = command_output(case["runtime"])
        raw[leg] = (case["runtime"]["exit"], stdout, stderr)
    require(raw["javac8"] == raw["javac23"], "original full Runner raw triples differ across JDK legs")
    return originals, {"physical": physical, "original_runtime_raw_equal": True,
                       "class_count_per_jdk": 4, "source_count_per_jdk": 4}


def verify_jadx_profiles(manifest: dict, cases: dict, originals: dict, tools: dict) -> dict:
    jadx = manifest["jadx"]
    jar_rec = jadx["jar"]
    jar_path = BASE / jar_rec["path"]
    jar_bytes = record_bytes(jar_rec)
    jar_members = {row["name"]: row for row in jadx["jar_members"]}
    require(set(jar_members) == {f"{name}.class" for name in TARGETS}, "JADX input class member inventory mismatch")
    with zipfile.ZipFile(jar_path) as archive:
        names = [name for name in archive.namelist() if not name.endswith("/")]
        require(names == [f"{name}.class" for name in TARGETS], "JADX input jar entry order/set mismatch")
        for class_name in TARGETS:
            member = archive.read(f"{class_name}.class")
            source_class = next(row for row in cases["javac23-original"]["classes"]
                                if Path(row["path"]).stem == class_name)
            require(member == record_bytes(source_class), f"JADX jar member differs from fresh javac23 class: {class_name}")
            row = jar_members[f"{class_name}.class"]
            require(row["path"] == source_class["path"] and row["bytes"] == len(member)
                    and row["sha256"] == sha(member), f"JADX member record mismatch: {class_name}")

    profiles = {row["profile"]: row for row in jadx["profiles"]}
    require(set(profiles) == {"default", "none"}, "JADX profile set mismatch")
    verified = {}
    for profile in ("default", "none"):
        row = profiles[profile]
        command = row["decompile"]
        require(command["label"] == f"jadx-{profile}-decompile" and command["exit"] == 0
                and row["decompile_success"] is True, f"JADX {profile} extraction failed")
        stdout, stderr = command_output(command)
        require(not stderr, f"JADX {profile} wrote unexpected stderr")
        require(row["input_jar"]["path"] == jar_rec["path"] and record_bytes(row["input_jar"]) == jar_bytes,
                f"JADX {profile} input jar differs")
        source_records = row["generated_sources"]
        require(len(source_records) == 3 and row["generated_source_count"] == 3
                and row["source_name_set_complete"] and row["package_set_consistent"],
                f"JADX {profile} generated-source inventory incomplete")
        sources = {Path(item["path"]).stem: item for item in source_records}
        require(set(sources) == set(TARGETS), f"JADX {profile} generated class set differs")
        package = "defpackage" if profile == "default" else None
        require(row["packages"] == [package], f"JADX {profile} package set mismatch")
        for class_name, record in sources.items():
            source = record_bytes(record)
            text = source.decode("utf-8")
            expected_prefix = "package defpackage;" if package else None
            require((text.startswith(expected_prefix) if expected_prefix else not re.search(r"(?m)^\s*package\s+", text)),
                    f"JADX {profile} generated package mismatch: {class_name}")
        for leg in ("javac8", "javac23"):
            case = cases[f"{leg}-jadx-{profile}"]
            require(case["kind"] == "jadx" and case["profile"] == profile and case["jdk_leg"] == leg,
                    f"JADX case identity mismatch: {case['label']}")
            copied = {Path(item["path"]).stem: item for item in case["generated_sources"]}
            require(set(copied) == set(TARGETS), f"JADX case source set differs: {case['label']}")
            for class_name, source_record in sources.items():
                require(record_bytes(copied[class_name]) == record_bytes(source_record),
                        f"JADX case source differs from extraction: {case['label']}/{class_name}")
            expected_sources = [str(BASE / item["path"]) for item in case["source_files"]]
            require(len(expected_sources) == 4, f"JADX full source set count differs: {case['label']}")
            source_files = {Path(item["path"]).name: item for item in case["source_files"]}
            require(len(source_files) == 4 and
                    set(source_files) == {f"{name}.java" for name in TARGETS} | {f"{RUNNER}.java"},
                    f"JADX compile source file set differs: {case['label']}")
            for class_name, generated_record in sources.items():
                require(record_bytes(source_files[f"{class_name}.java"]) == record_bytes(generated_record),
                        f"JADX actual compile source differs from extracted source: {case['label']}/{class_name}")
            runner_record = next(item for item in case["source_files"] if Path(item["path"]).name == f"{RUNNER}.java")
            runner_original = (BASE / "original-sources" / f"{RUNNER}.java").read_bytes()
            runner_expected = (b"package defpackage;\n\n" + runner_original) if package else runner_original
            require(record_bytes(runner_record) == runner_expected, f"JADX Runner adaptation is not exact: {case['label']}")
            require(record_bytes(case["runner_adaptation"]) == runner_expected, f"JADX Runner adaptation record mismatch: {case['label']}")
            compile_paths = [str(BASE / item["path"]) for item in case["source_files"]]
            require(set(Path(item).name for item in compile_paths) == {f"{name}.java" for name in TARGETS} | {f"{RUNNER}.java"},
                    f"JADX compile source filenames mismatch: {case['label']}")
            expected_runner = (f"{package}." if package else "") + RUNNER
            require(case["runner_class"] == expected_runner,
                    f"JADX runner package/class mismatch: {case['label']}")
            verify_compile_runtime(case, leg, compile_paths, expected_runner, tools,
                                   originals[leg]["runtime"], package)
            require(case["success"] is True and case["runtime_matches_original_raw"] is True,
                    f"JADX public runtime comparison failed: {case['label']}")
            verify_class_census(case, package)
        verified[profile] = {"generated_sources": 3, "rebuilt_jdk_legs": 2,
                             "package": package, "input_jar_bound_to_javac23_original": True}
    return verified


def verify_jarde_cases(cases: dict, originals: dict, physical: dict, tools: dict) -> dict:
    summaries = {}
    for leg in ("javac8", "javac23"):
        case = cases[f"{leg}-jarde"]
        require(case["kind"] == "jarde" and case["jdk_leg"] == leg, f"Jarde case identity mismatch: {leg}")
        require(case["render_success"] is True and case["package_set_consistent"] is True,
                f"Jarde renders are not complete: {leg}")
        rendered_by_name = {row["class"]: row for row in case["rendered_classes"]}
        require(set(rendered_by_name) == set(TARGETS), f"Jarde render class set mismatch: {leg}")
        all_texts, all_sources = {}, {}
        for class_name in TARGETS:
            rendered = rendered_by_name[class_name]
            require(rendered["success"] and rendered["default_all_text_equal"]
                    and rendered["physical_inventory_matches"], f"Jarde class render failed: {leg}/{class_name}")
            original_class = next(row for row in originals[leg]["classes"] if Path(row["path"]).stem == class_name)
            require(record_bytes(rendered["original_class"]) == record_bytes(original_class),
                    f"Jarde render input class differs from same-leg compile: {leg}/{class_name}")
            outputs = {}
            for mode in ("default", "all"):
                output = verify_jarde_document(rendered, class_name, physical[(leg, class_name)], mode, tools)
                outputs[mode] = output
                all_texts[(class_name, mode)] = output["text"]
            require(outputs["default"]["text"] == outputs["all"]["text"],
                    f"Jarde default/all full-class text differs: {leg}/{class_name}")
            all_sources[class_name] = outputs["all"]["text"].encode("utf-8")
        require(case["package_set"] == [None], f"Jarde input classes unexpectedly packaged: {leg}")
        require(len(case["rendered_sources"]) == 3, f"Jarde full source count differs: {leg}")
        compiled_sources = {Path(row["path"]).stem: row for row in case["rendered_sources"]}
        require(set(compiled_sources) == set(TARGETS), f"Jarde compiled render source set differs: {leg}")
        for name, record in compiled_sources.items():
            require(record_bytes(record) == all_sources[name], f"Jarde compile source differs from --evidence all text: {leg}/{name}")
        runner_source = (BASE / "original-sources" / f"{RUNNER}.java").read_bytes()
        require(record_bytes(case["runner_adaptation"]) == runner_source, f"Jarde Runner was changed: {leg}")
        expected_paths = [str(BASE / row["path"]) for row in case["source_files"]]
        require(len(expected_paths) == 4, f"Jarde full source+Runner input count differs: {leg}")
        source_files = {Path(row["path"]).name: row for row in case["source_files"]}
        require(len(source_files) == 4 and
                set(source_files) == {f"{name}.java" for name in TARGETS} | {f"{RUNNER}.java"},
                f"Jarde compile source file set differs: {leg}")
        for name, source_record in compiled_sources.items():
            require(record_bytes(source_files[f"{name}.java"]) == all_sources[name],
                    f"Jarde actual compile source differs from --evidence all text: {leg}/{name}")
        require(record_bytes(source_files[f"{RUNNER}.java"]) == runner_source,
                f"Jarde actual compile Runner differs from original: {leg}")
        verify_compile_runtime(case, leg, expected_paths, RUNNER, tools, originals[leg]["runtime"])
        require(case["success"] is True and case["runtime_matches_original_raw"] is True,
                f"Jarde public runtime comparison failed: {leg}")
        verify_class_census(case)
        summaries[leg] = {"renders": 6, "full_source_files": 4, "methods": 6,
                          "physical_bci_source_maps_complete": True,
                          "raw_runtime_matches_original": True}
    return summaries


def verify_case_accounting(manifest: dict, cases: dict, command_count: int) -> dict:
    expected = {"original": 2, "jadx": 4, "jarde": 2}
    actual = {kind: sum(case["kind"] == kind for case in cases.values()) for kind in expected}
    require(actual == expected == manifest["cases_expected"], f"case-kind count mismatch: {actual}")
    successes = {kind: sum(case["kind"] == kind and case.get("success") is True for case in cases.values())
                 for kind in expected}
    require(successes == manifest["success_counts"], "recorded success counts disagree with case rows")
    failures = manifest["failures"]
    require(manifest["status"] == ("completed" if not failures else "baseline-with-failures"),
            "manifest status does not preserve its failure list")
    require(not failures and manifest["status"] == "completed",
            "the recorded root-v2 baseline is expected to contain eight successful cases")
    require(all(case.get("success") is True for case in cases.values()),
            "a case did not independently verify as successful")
    render_count = sum("class-source" in command["argv"] for command in manifest["commands"])
    require(render_count == manifest["jarde_cli_render_count"] == manifest["jarde_cli_render_count_expected"] == 12,
            "Jarde CLI render command count mismatch")
    require(command_count == len(manifest["commands"]), "command inventory count mismatch")
    return {"case_counts": actual, "success_counts": successes,
            "recorded_failures": failures, "status": manifest["status"],
            "jarde_cli_render_count": render_count,
            "all_current_case_success_flags_independently_verified": True}


def verify_preparation_execution(manifest: dict) -> dict:
    execution_dir = EVIDENCE / "root-execution-v2"
    execution = load_json(execution_dir / "execution.json")
    expected_script = EVIDENCE / "prepare-baseline-luna-v2.py"
    expected_argv = ["python3", "-B", str(expected_script), "--cli", CLI_PATH]
    require(execution["argv"] == expected_argv and execution["cwd"] == str(ROOT),
            "root preparation execution argv/cwd mismatch")
    require(execution["exit"] == 0, "root preparation execution did not exit successfully")
    script_record = manifest["script"]
    preparer_bytes = expected_script.read_bytes()
    require(Path(script_record["path"]) == expected_script
            and script_record["bytes"] == len(preparer_bytes)
            and script_record["sha256"] == PREPARER_SHA256
            and sha(preparer_bytes) == PREPARER_SHA256,
            "prepared-v2 script file_record pin mismatch")
    require(execution["script_sha256"] == PREPARER_SHA256 == script_record["sha256"],
            "root execution script SHA differs from prepared-v2 file_record")
    stream_summary = {}
    for name in ("stdout", "stderr"):
        expected = execution["streams"][name]
        raw = (execution_dir / name).read_bytes()
        require(len(raw) == expected["bytes"] and sha(raw) == expected["sha256"],
                f"root preparation raw {name} differs from execution record")
        stream_summary[name] = {"bytes": len(raw), "sha256": sha(raw)}
    return {"exit": execution["exit"], "argv": expected_argv,
            "script_sha256": PREPARER_SHA256, "raw_streams": stream_summary}


def main() -> int:
    require(not OUTPUT.exists(), f"refusing to overwrite {OUTPUT}")
    manifest_bytes = MANIFEST_PATH.read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("schema") == "array-literal-boundaries-next-baseline-v2", "unexpected baseline schema")
    require(manifest.get("claim_boundary") ==
            "Three focused JADX array-literal fixtures adapted to standalone classes; not a broader suite-completion claim.",
            "claim boundary changed")
    inventory_summary = verify_inventory(manifest)
    md_bytes = METADATA.read_bytes()
    metadata = json.loads(md_bytes)
    frozen_summary = verify_frozen_inputs(manifest, metadata)
    archived_source_summary = verify_archived_sources(manifest)
    selected_body_summary = verify_selected_bodies()
    commands, command_summary = verify_recorded_streams(manifest["commands"])
    verify_command_provenance(commands, frozen_summary["jdk_tools"])
    require(len(commands) == 39, f"actual command row count differs from the 39-label manifest shape: {len(commands)}")

    case_rows = manifest["cases"]
    cases = {case["label"]: case for case in case_rows}
    expected_case_labels = (
        "javac8-original", "javac23-original",
        "javac8-jadx-default", "javac23-jadx-default", "javac8-jadx-none", "javac23-jadx-none",
        "javac8-jarde", "javac23-jarde",
    )
    require(tuple(cases) == expected_case_labels, "case order/set differs from the recorded baseline shape")
    originals, javap_summary = verify_originals(cases, frozen_summary["jdk_tools"], commands)
    verify_command_case_rows(manifest, commands)
    jadx_summary = verify_jadx_profiles(manifest, cases, originals, frozen_summary["jdk_tools"])
    jarde_summary = verify_jarde_cases(cases, originals, javap_summary["physical"], frozen_summary["jdk_tools"])
    accounting = verify_case_accounting(manifest, cases, len(commands))
    require(manifest["original_cross_jdk_raw_equal"] is javap_summary["original_runtime_raw_equal"],
            "original cross-JDK raw equality claim differs")
    preparation = verify_preparation_execution(manifest)

    result = {
        "schema": "array-literal-boundaries-next-baseline-verification-luna-v2",
        "status": "verified-baseline-with-recorded-outcome",
        "manifest_path": str(MANIFEST_PATH.relative_to(ROOT)),
        "manifest_sha256": sha(manifest_bytes),
        "closed_inventory": inventory_summary,
        "frozen_inputs": {key: value for key, value in frozen_summary.items() if key != "jdk_tools"},
        "source_sha256": archived_source_summary,
        "selected_upstream_bodies": selected_body_summary,
        "commands_and_raw_streams": command_summary,
        "preparation_execution": preparation,
        "originals_and_javap": {key: value for key, value in javap_summary.items() if key != "physical"},
        "jadx_rebuilds": jadx_summary,
        "jarde_full_source_rebuilds": jarde_summary,
        "case_accounting": accounting,
        "claim_boundary": manifest["claim_boundary"],
        "instance_initializer_implementation_claim": "not_in_scope_and_not_claimed",
        "semantic_observations": {
            "ConstantIntArray": "the static final CONST_INT field and returned array entries are emitted as the literal value 65535; this confirms the constant value, not preservation of the source symbol reference",
            "LongArrayLimits": "the generated long-array values retain the recorded 64-bit extrema as long literals",
            "DependentArrayStores": "the generated method keeps a new int[3] temporary and the ordered element reads/writes instead of folding them into an array initializer",
        },
    }
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "output": str(OUTPUT.relative_to(ROOT)),
                      "command_count": len(commands), "case_counts": accounting["case_counts"],
                      "success_counts": accounting["success_counts"],
                      "recorded_failures": accounting["recorded_failures"]}, ensure_ascii=False, indent=2))
    return 0


def verify_command_case_rows(manifest: dict, commands: dict[str, dict]) -> None:
    def walk(value):
        if isinstance(value, dict):
            if {"label", "argv", "stdout", "stderr", "exit"}.issubset(value):
                label = value["label"]
                require(label in commands and value == commands[label], f"nested command differs from command table: {label}")
            for item in value.values():
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)
    walk(manifest["jadx"])
    walk(manifest["cases"])


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"verification failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise SystemExit(1)
