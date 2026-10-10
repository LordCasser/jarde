#!/usr/bin/env python3
"""Independently check the frozen CF07 loop-latch evidence bundle (read-only)."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

try:
    import blake3
except ImportError as exc:  # no dependency installation is attempted
    raise SystemExit("Python blake3 module is required to verify class-byte digests") from exc


ROOT = Path(__file__).resolve().parent
BUNDLE = ROOT / "baseline-root-v2"
OUTPUT = ROOT / "independent-verification-root-v3.json"
COLLECTOR_SHA256 = "87c693a53f31898374d5c963c6fbe14b550db20439e0409486b9d191a00cec8f"
PREPARED_SOURCE_SHA256 = "3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38"
EXPECTED_METHODS = {
    "<init>()V": (1, [0, 1, 4]),
    "andWhile(Z)I": (9, [0, 1, 2, 3, 6, 7, 9, 12, 15, 18, 19]),
    "counted(II)I": (9, [0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 14, 17, 20, 23, 24, 25, 26, 27, 30, 33, 36, 37]),
    "lastIndexOf([IIII)I": (9, [0, 1, 2, 3, 5, 7, 8, 11, 12, 14, 15, 16, 19, 21, 22, 25, 28, 29]),
}
EXPECTED_GAPS = {
    "andWhile(Z)I": [15],
    "counted(II)I": [20, 30],
    "lastIndexOf([IIII)I": [25],
}


def check(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def tracked_file(path: str) -> bytes:
    data = (BUNDLE / path).read_bytes()
    return data


def check_ref(ref: dict, *, compare_sha=True) -> bytes:
    data = tracked_file(ref["path"])
    check(len(data) == ref["bytes"], f"byte length mismatch: {ref['path']}")
    if compare_sha:
        check(sha256(data) == ref["sha256"], f"SHA-256 mismatch: {ref['path']}")
    return data


def method_key(item: dict) -> str:
    # The identity arrays are the document's canonical method key.
    identity = item["identity"]
    name = bytes(identity["name"]).decode("utf-8")
    desc = bytes(identity["descriptor"]).decode("utf-8")
    check(name == bytes(item["name"]["raw"]).decode("utf-8"), "method name identity mismatch")
    check(desc == bytes(item["descriptor"]["raw"]).decode("utf-8"), "method descriptor identity mismatch")
    return name + desc


def parse_javap(path: Path) -> dict:
    """Read member facts and bytecode instructions from the captured javap text."""
    text = path.read_text(encoding="utf-8", errors="strict")
    # Start parsing only inside the class body; constant-pool entries are outside it.
    body_start = re.search(r"(?m)^\s*\{\s*$", text)
    check(body_start is not None, f"javap class body missing: {path}")
    body = text[body_start.end():]
    methods = {}
    physical_field_count = 0
    seen_method = False
    current = None
    flags_for = {"ACC_PUBLIC": 1, "ACC_PRIVATE": 2, "ACC_PROTECTED": 4, "ACC_STATIC": 8,
                 "ACC_FINAL": 16, "ACC_SYNCHRONIZED": 32, "ACC_BRIDGE": 64, "ACC_VARARGS": 128,
                 "ACC_NATIVE": 256, "ACC_ABSTRACT": 1024, "ACC_STRICT": 2048, "ACC_SYNTHETIC": 4096}
    for line in body.splitlines():
        declaration = re.match(r"^  (.+\([^;]*\));\s*$", line)
        if declaration:
            seen_method = True
            current = {"declaration": declaration.group(1), "flags": None, "instructions": []}
            continue
        if current is None:
            if line.strip() == "}":
                break
            if not seen_method and re.match(r"^  [^/].+;\s*$", line):
                physical_field_count += 1
            continue
        desc_line = re.match(r"^\s+descriptor:\s+(\S+)\s*$", line)
        if desc_line:
            decl = current["declaration"]
            if decl.startswith("public cf07.LoopCases("):
                name = "<init>"
            else:
                name_match = re.search(r"([A-Za-z_$][\w$]*)\([^()]*\)$", decl)
                check(name_match is not None, f"cannot parse javap method declaration: {decl}")
                name = name_match.group(1)
            current["key"] = name + desc_line.group(1)
            methods[current["key"]] = current
        if line.strip().startswith("flags:"):
            flag_line = line.split(":", 1)[1]
            hex_flag = re.search(r"0x([0-9a-fA-F]+)", flag_line)
            if hex_flag:
                current["flags"] = int(hex_flag.group(1), 16)
            else:
                current["flags"] = sum(bit for flag, bit in flags_for.items() if flag in flag_line)
        inst = re.match(r"^\s+(\d+):\s+(.+)$", line)
        if inst:
            current["instructions"].append({"bci": int(inst.group(1)), "instruction": inst.group(2).strip()})
    return {"methods": methods, "physical_field_count": physical_field_count}


def command_stream(command: dict, stream_name: str) -> bytes:
    stream = command["streams"][stream_name]
    raw = (BUNDLE / stream["path"]).read_bytes()
    check(len(raw) == stream["bytes"], f"stream byte length mismatch: {stream['path']}")
    check(sha256(raw) == stream["sha256"], f"stream SHA-256 mismatch: {stream['path']}")
    return raw


def main() -> None:
    check(not OUTPUT.exists(), f"refusing to overwrite {OUTPUT}")
    manifest_path = BUNDLE / "manifest.json"
    manifest_bytes = manifest_path.read_bytes()
    check(sha256(manifest_bytes) == "1a374647a971b416635bc114b9c197d05c855c51eeabd274f3ee571e868a64ee", "fixed manifest SHA mismatch")
    manifest = json.loads(manifest_bytes)
    check(manifest["frozen_jarde_cli"]["sha256"] == "b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59", "expected old CLI identity mismatch")
    check(manifest["frozen_jarde_cli"]["metadata_sha256"] == "f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8", "expected old CLI metadata mismatch")
    jdk_manifest = Path(manifest["jdk_manifest"]["path"]).read_bytes()
    check(sha256(jdk_manifest) == "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec", "JDK manifest mismatch")
    inventory = read_json(BUNDLE / manifest["file_inventory"]["path"])
    check(len(inventory) == 118, f"expected 118 inventory entries, got {len(inventory)}")
    expected_paths = {entry["path"] for entry in inventory}
    check("manifest.json" in expected_paths, "manifest missing from inventory")
    check(manifest["file_inventory"]["path"] not in expected_paths, "inventory must exclude itself")
    actual_paths = {str(p.relative_to(BUNDLE)) for p in BUNDLE.rglob("*") if p.is_file() and p.name != "file-inventory.json"}
    check(actual_paths == expected_paths, "inventory does not close over bundle files")
    for entry in inventory:
        data = tracked_file(entry["path"])
        check(len(data) == entry["bytes"] and sha256(data) == entry["sha256"], f"inventory mismatch: {entry['path']}")

    check(manifest["prepared_input_sha256"]["source"] == PREPARED_SOURCE_SHA256, "prepared source identity mismatch")
    original_source = check_ref(manifest["source"])
    original_runner_source = check_ref(manifest["runner"])
    check(sha256(original_source) == PREPARED_SOURCE_SHA256, "original source bytes mismatch")
    check(sha256(original_runner_source) == "28b0a08ceb89b6ff31d12056831a2d7cc62bfda0a61ebc39835d2f62bc89bdf5", "original Runner bytes mismatch")
    collector = manifest["prepared_script"]
    collector_bytes = (ROOT / Path(collector["path"]).name).read_bytes()
    check(len(collector_bytes) == collector["bytes"] and sha256(collector_bytes) == COLLECTOR_SHA256 == collector["sha256"], "collector identity mismatch")
    cli = manifest["frozen_jarde_cli"]
    check(sha256(Path(cli["path"]).read_bytes()) == cli["sha256"], "frozen Jarde CLI digest mismatch")
    metadata = Path(cli["metadata_path"]).read_bytes()
    check(sha256(metadata) == cli["metadata_sha256"], "frozen Jarde CLI metadata digest mismatch")
    jadx = manifest["jadx"]
    check(sha256(Path(jadx["resolved_launcher"]).read_bytes()) == jadx["sha256"], "pinned JADX launcher digest mismatch")
    check(jadx["expected_version"] == "1.5.6", "pinned JADX version mismatch")
    jar = check_ref(jadx["input_jar"])
    check(len(jar) == jadx["input_jar"]["bytes"], "JADX input jar length mismatch")
    check(manifest["command_count"] == len(manifest["commands"]) == 29, "command inventory mismatch")
    commands = {c["label"]: c for c in manifest["commands"]}
    check(len(commands) == 29, "command labels are not unique")
    command_facts = []
    for label, command in commands.items():
        check(command["exit"] == 0, f"nonzero captured exit: {label}")
        out = command_stream(command, "stdout")
        err = command_stream(command, "stderr")
        command_facts.append({"label": label, "exit": command["exit"], "stdout_sha256": sha256(out), "stderr_sha256": sha256(err)})
    jadx_version = commands["jadx-version"]
    check(command_stream(jadx_version, "stdout").decode("utf-8").strip() == jadx["expected_version"], "captured JADX version does not match pin")

    # Original class bytes are the input oracle for every rendered document.
    originals = {}
    javap = {}
    jdk_pin_facts = {}
    for leg in ("javac8", "javac23"):
        original = next(c for c in manifest["cases"] if c["label"] == f"{leg}-original")
        cls = original["actual_class"]
        raw = tracked_file(cls["path"])
        check(len(raw) == cls["bytes"] and sha256(raw) == cls["sha256"], f"original class mismatch {leg}")
        check(blake3.blake3(raw).hexdigest() == cls["blake3"], f"original class BLAKE3 mismatch {leg}")
        originals[leg] = {"path": cls["path"], "bytes": len(raw), "sha256": sha256(raw), "blake3": blake3.blake3(raw).hexdigest()}
        for tool_name, tool in manifest["jdk_legs"][leg]["tools"].items():
            tool_bytes = Path(tool["path"]).read_bytes()
            check(sha256(tool_bytes) == tool["sha256"], f"pinned {leg} {tool_name} binary mismatch")
            jdk_pin_facts[f"{leg}.{tool_name}"] = {"path": tool["path"], "sha256": sha256(tool_bytes)}
        fact = next(x for x in manifest["original_physical_facts"] if x["command"]["label"] == f"{leg}-original-javap")
        text_bytes = check_ref(fact["text"])
        parsed = parse_javap(BUNDLE / fact["text"]["path"])
        physical = parsed["methods"]
        check(parsed["physical_field_count"] == fact["physical_field_count"] == 0, f"physical field count mismatch {leg}")
        check(set(physical) == set(EXPECTED_METHODS), f"physical method set mismatch {leg}")
        for key, (flags, bcis) in EXPECTED_METHODS.items():
            check(key in physical, f"javap method missing {leg} {key}")
            actual = physical[key]
            check(actual["flags"] == flags, f"javap flags mismatch {leg} {key}: {actual['flags']}")
            actual_bcis = [x["bci"] for x in actual["instructions"]]
            check(actual_bcis == bcis, f"javap BCI mismatch {leg} {key}")
        javap[leg] = {"text_path": fact["text"]["path"], "text_sha256": sha256(text_bytes), "physical_field_count": parsed["physical_field_count"], "methods": physical}

    cases = manifest["cases"]
    check(len(cases) == 10, f"expected ten cases, got {len(cases)}")
    expected_case_labels = {f"{leg}-original" for leg in ("javac8", "javac23")}
    expected_case_labels |= {f"{leg}-jadx-{profile}" for leg in ("javac8", "javac23") for profile in ("default", "none")}
    expected_case_labels |= {f"{leg}-jarde-{profile}" for leg in ("javac8", "javac23") for profile in ("default", "all")}
    check({case["label"] for case in cases} == expected_case_labels, "case labels are not the expected ten cases")
    raw_oracle = {}
    for leg in ("javac8", "javac23"):
        original = next(c for c in cases if c["label"] == f"{leg}-original")
        cmd = original["runtime"]
        raw_oracle[leg] = {"stdout": command_stream(cmd, "stdout"), "stderr": command_stream(cmd, "stderr")}
    check(raw_oracle["javac8"] == raw_oracle["javac23"], "original raw triples differ across JDKs")
    with zipfile.ZipFile(BUNDLE / jadx["input_jar"]["path"]) as jar_file:
        check(jar_file.namelist() == ["cf07/LoopCases.class"], "JADX input jar contains another member")
        check(jar_file.read("cf07/LoopCases.class") == tracked_file(originals["javac23"]["path"]), "JADX input is not javac23 original target")
    case_results = []
    render_results = []
    render_docs = {}
    for case in cases:
        wrapped = case.get("compiled_case", case)
        label = case["label"]
        leg = case["jdk_leg"]
        compile_cmd = wrapped["compile"]
        runtime_cmd = wrapped["runtime"]
        check(compile_cmd["exit"] == runtime_cmd["exit"] == 0, f"compile/runtime exit for {label}")
        check(commands[compile_cmd["label"]] == compile_cmd and commands[runtime_cmd["label"]] == runtime_cmd, f"case command differs from command log {label}")
        argv = compile_cmd["argv"]
        tool = manifest["jdk_legs"][leg]["tools"]["javac"]["path"]
        check(argv[0] == tool and argv[1:7] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"], f"compile tool/options mismatch {label}")
        check(argv[7] == "-classpath" and argv[9] == "-sourcepath" and argv[8] == argv[10], f"empty classpath/sourcepath mismatch {label}")
        cp_dir = Path(argv[8])
        check(cp_dir == (BUNDLE / "cases" / label / "empty-classpath-sourcepath").resolve() and cp_dir.is_dir() and not list(cp_dir.iterdir()), f"empty CP/SP path mismatch {label}")
        d_index = argv.index("-d")
        out_dir = argv[d_index + 1]
        check(out_dir == wrapped["class_output"], f"compile output dir mismatch {label}")
        check(out_dir == str((BUNDLE / "cases" / label / "classes").resolve()), f"compile output is not this case's fresh directory {label}")
        source_refs = wrapped["source_files"]
        target_source = (BUNDLE / "cases" / label / "LoopCases.java").read_bytes()
        check((BUNDLE / "cases" / label / "Runner.java").read_bytes() == original_runner_source, f"Runner source modified {label}")
        if case["kind"] == "original":
            check(target_source == original_source, f"original source modified {label}")
        elif case["kind"] == "jadx":
            check(target_source == check_ref(case["decompilation"]["generated_sources"][0]), f"JADX generated source modified {label}")
        check(len(argv[d_index + 2:]) == len(source_refs) == 2, f"compile source count mismatch {label}")
        expected_sources = [str((BUNDLE / ref["path"]).resolve()) for ref in source_refs]
        check(argv == [tool, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options", "-classpath", str(cp_dir), "-sourcepath", str(cp_dir), "-d", out_dir, *expected_sources], f"compile argv not exact frozen invocation {label}")
        check([Path(p).read_bytes() for p in argv[d_index + 2:]] == [check_ref(ref) for ref in source_refs], f"compile sources do not match frozen inputs {label}")
        check(runtime_cmd["argv"] == [manifest["jdk_legs"][leg]["tools"]["java"]["path"], "-Xverify:all", "-cp", out_dir, "cf07.Runner"], f"runtime argv mismatch {label}")
        check(len(wrapped["classes"]) == 2 and {Path(x["path"]).name for x in wrapped["classes"]} == {"LoopCases.class", "Runner.class"}, f"fresh class set mismatch {label}")
        expected_classes = set(wrapped["expected_class_paths"])
        actual_classes = {str((BUNDLE / x["path"]).relative_to(Path(out_dir))) for x in wrapped["classes"]}
        check(actual_classes == {"cf07/LoopCases.class", "cf07/Runner.class"}, f"fresh class set mismatch {label}")
        check({str(p.relative_to(Path(out_dir))) for p in Path(out_dir).rglob("*.class")} == actual_classes, f"actual fresh directory class set mismatch {label}")
        check(expected_classes == actual_classes == set(wrapped["actual_class_paths"]), f"class path set mismatch {label}")
        for class_ref in wrapped["classes"]:
            raw_class = tracked_file(class_ref["path"])
            check(len(raw_class) == class_ref["bytes"] and sha256(raw_class) == class_ref["sha256"], f"class output digest mismatch {label}")
            if Path(class_ref["path"]).name == "Runner.class":
                original_runner = next(x for x in next(c for c in cases if c["label"] == f"{leg}-original")["classes"] if Path(x["path"]).name == "Runner.class")
                check(raw_class == tracked_file(original_runner["path"]), f"runner class differs from original {label}")
        out = command_stream(runtime_cmd, "stdout")
        err = command_stream(runtime_cmd, "stderr")
        check(out == raw_oracle[leg]["stdout"] and err == raw_oracle[leg]["stderr"], f"runtime raw differs from same-JDK original: {label}")
        case_results.append({"label": label, "jdk_leg": leg, "compile_exit": compile_cmd["exit"], "runtime_exit": runtime_cmd["exit"], "runtime_stdout_sha256": sha256(out), "runtime_stderr_sha256": sha256(err), "class_paths": [x["path"] for x in wrapped["classes"]]})
        if case["kind"] == "jarde":
            mode = case["evidence_mode"]
            profile = case["rendered_profile"]
            generated = check_ref(case["generated_source"])
            rendered_source = check_ref(profile["generated_text"])
            check(generated == rendered_source, f"generated text differs from rendered profile {label}")
            doc_bytes = check_ref(profile["document"])
            render_command = profile["command"]
            original_path = str((BUNDLE / originals[leg]["path"]).resolve())
            expected_render = [cli["path"], "class-source", "--input", original_path, "--class", "cf07.LoopCases", "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                expected_render += ["--evidence", "all"]
            check(render_command["argv"] == expected_render, f"render command pin/arguments mismatch {label}")
            check(render_command["exit"] == 0 and command_stream(render_command, "stdout") == doc_bytes, f"render raw/document mismatch {label}")
            doc = json.loads(doc_bytes)
            check(doc["text"].encode("utf-8") == generated == target_source, f"JSON full text differs from compiled source {label}")
            check(doc["fields"] == [] and len(doc["methods"]) == 4 and doc["execution"]["status"] == "complete", f"class report member/execution mismatch {label}")
            render_docs[(leg, mode)] = (doc, generated.decode("utf-8"))
            owner = doc["class"]
            raw_original = tracked_file(originals[leg]["path"])
            digest = blake3.blake3(raw_original).hexdigest()
            owner_facts = owner["class_bytes"]
            check(owner_facts == {"digest": digest, "length": len(raw_original)}, f"document class byte identity mismatch {label}")
            check(owner["location"] == {"kind": "standalone_root", "snapshot": digest}, f"document owner location mismatch {label}")
            check(owner["variant"] == {"kind": "base"}, f"document owner variant mismatch {label}")
            method_facts = []
            for index, method in enumerate(doc["methods"]):
                item = method["item"]
                key = method_key(item)
                check(key in EXPECTED_METHODS, f"unexpected method in document {label}: {key}")
                check(item["access_flags"] == EXPECTED_METHODS[key][0] and item["index"] == index, f"reported flags/index mismatch {label} {key}")
                check(method["outcome"]["kind"] == "recovered", f"method outcome mismatch {label} {key}")
                report = method["outcome"]["report"]
                check(report["outcome"] == "produced" and report["execution"]["status"] == "complete", f"method execution mismatch {label} {key}")
                check(report["method"] == key, f"report method identity mismatch {label} {key}")
                check(item["identity"]["owner"] == owner, f"method physical owner mismatch {label} {key}")
                physical = javap[leg]["methods"][key]
                mapped, origin_facts = set(), []
                text_len = len(report["text"].encode("utf-8"))
                for segment in report["source_map"]["segments"]:
                    start, end = segment["start"], segment["end"]
                    check(0 <= start < end <= text_len, f"empty/out-of-bounds source span {label} {key}")
                    for role in ("primary",):
                        origin = segment["origin"].get(role)
                        if origin:
                            check(origin["method"] == item["identity"], f"origin method/owner mismatch {label} {key}")
                            mapped.add(origin["bci"])
                            origin_facts.append({"role": role, "bci": origin["bci"], "provenance": origin["provenance"], "span": [start, end]})
                    for origin in segment["origin"].get("derived", []):
                        check(origin["method"] == item["identity"], f"derived origin method/owner mismatch {label} {key}")
                        check(origin["bci"] in EXPECTED_METHODS[key][1], f"nonphysical derived BCI {label} {key}")
                        mapped.add(origin["bci"])
                        origin_facts.append({"role": "derived", "bci": origin["bci"], "provenance": origin["provenance"], "span": [start, end]})
                missing = sorted(set(EXPECTED_METHODS[key][1]) - mapped)
                check(mapped <= set(EXPECTED_METHODS[key][1]), f"source map has nonphysical BCI {label} {key}")
                check(missing == EXPECTED_GAPS.get(key, []), f"source-map gap changed {label} {key}: {missing}")
                method_facts.append({"method": key, "physical_flags": physical["flags"], "physical_bcis": [x["bci"] for x in physical["instructions"]], "source_map_all_origin_bcis": sorted(mapped), "missing_origin_bcis": missing, "origin_spans": origin_facts})
            check({m["method"] for m in method_facts} == set(EXPECTED_METHODS), f"document method set mismatch {label}")
            render_results.append({"case": label, "profile": mode, "document_path": profile["document"]["path"], "document_sha256": sha256(doc_bytes), "owner": owner, "physical_field_count": javap[leg]["physical_field_count"], "methods": method_facts})

    equality = {}
    for leg in ("javac8", "javac23"):
        default, default_text = render_docs[(leg, "default")]
        all_doc, all_text = render_docs[(leg, "all")]
        check(default_text == all_text, f"default/all generated source differs {leg}")
        equality[leg] = {"generated_text_equal": True, "method_text_equal": {}, "source_map_equal": {}}
        for key in EXPECTED_METHODS:
            dm = next(m for m in default["methods"] if method_key(m["item"]) == key)
            am = next(m for m in all_doc["methods"] if method_key(m["item"]) == key)
            dr, ar = dm["outcome"]["report"], am["outcome"]["report"]
            check(dr["text"] == ar["text"], f"default/all method text differs {leg} {key}")
            check(dr["source_map"] == ar["source_map"], f"default/all source map differs {leg} {key}")
            equality[leg]["method_text_equal"][key] = True
            equality[leg]["source_map_equal"][key] = True

    transfer_observations = {}
    for leg in ("javac8", "javac23"):
        facts = {}
        for key, (bci, target) in [("andWhile(Z)I", (15, 2)), ("counted(II)I", (20, 27)), ("counted(II)I", (30, 6)), ("lastIndexOf([IIII)I", (25, 5))]:
            inst = next(i["instruction"] for i in javap[leg]["methods"][key]["instructions"] if i["bci"] == bci)
            check(re.fullmatch(rf"goto\s+{target}", inst) is not None, f"physical goto mismatch {leg} {key}@{bci}: {inst}")
            facts[f"{key}@{bci}"] = {"instruction": inst, "target": target, "source_origin_missing": bci in EXPECTED_GAPS.get(key, [])}
        transfer_observations[leg] = facts

    result = {
        "schema": "cf07-independent-verification-root-v3",
        "verifier_sha256": sha256(Path(__file__).read_bytes()),
        "status": "verified_from_frozen_raw_evidence",
        "scope": {"bundle": str(BUNDLE), "collector_sha256": COLLECTOR_SHA256, "manifest_sha256": sha256(manifest_bytes), "inventory_entries": len(inventory), "commands": 29, "cases": 10},
        "tool_pins": {"jarde_cli": {"path": cli["path"], "sha256": cli["sha256"]}, "jarde_cli_metadata": {"path": cli["metadata_path"], "sha256": cli["metadata_sha256"]}, "jdk": jdk_pin_facts, "jadx": {"launcher": jadx["resolved_launcher"], "sha256": jadx["sha256"], "version": jadx["expected_version"], "input_jar_sha256": sha256(jar)}},
        "oracle": {"original_class_bytes": originals, "physical_javap": javap},
        "commands": command_facts,
        "cases": case_results,
        "jarde_render_profiles": render_results,
        "render_equality": equality,
        "jarde_profile_gaps": EXPECTED_GAPS,
        "physical_goto_observations": transfer_observations,
        "claim_boundary": "These facts verify the frozen original/JADX/Jarde evidence and source maps. Missing origins at andWhile@15, counted@20/@30, and lastIndexOf@25 are recorded as source-map gaps; no candidate acceptance or complete BCI coverage is claimed.",
    }
    OUTPUT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUTPUT}")


if __name__ == "__main__":
    try:
        main()
    except (KeyError, ValueError, OSError, json.JSONDecodeError) as exc:
        print(f"verification failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
