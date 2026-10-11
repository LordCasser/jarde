#!/usr/bin/env python3
"""Read-only verifier for the completed computed-initializer call-order control."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import struct
import sys

EXPECTED_CLI_SHA256 = "8e70234797af43e114e94cd08ba0a8f120d59d0e9c638a20a8cf64e9876fd64b"
EXPECTED_CLI_PATH = "/private/tmp/jarde-cf12-constant-candidate-cli-root-v2"
EXPECTED_CONTROL_DIR = "/private/tmp/jarde-computed-for-call-order-control-luna-v2"
METHOD_NAME = "find"
METHOD_DESC = "([IIIIII)I"
PRODUCT_PINS = {
    "crates/jarde-jvm/src/canonical.rs": "79d7725b697433ae9f00360818cf5c3deece227c4d15101412a730fd8bed9a86",
    "crates/jarde-jvm/src/ssa.rs": "03e01dcda42e6022f217b4238184fc1b5d45a63b05811f7692b39d7fefa2c90a",
    "crates/jarde-java/src/build.rs": "f4660755cf2ebc699dccf8ee7f7c984a94f6695a22a62cf649d4b728c76a1ade",
    "crates/jarde-java/src/region.rs": "acdb5d22195ce0b5d283fe1b7a3003f35830c85811b18e2e713b7f4728e93168",
    "crates/jarde-java/tests/cf12_proved_local_source_types.rs": "1eb2c9e039a6aa2dcb706f29969e7006f4ec6029340763b1d883accfb58e98db",
}
EXPECTED_ROWS = [
    "normal-hit outcome=normal result=1 f=1 g=1 order=12 end=0",
    "normal-miss outcome=normal result=-1 f=1 g=1 order=12 end=0",
    "normal-empty-range outcome=normal result=-1 f=1 g=1 order=12 end=0",
    "fail-f outcome=fail-f result=777 f=1 g=0 order=1 end=3",
    "fail-g outcome=fail-g result=777 f=1 g=1 order=12 end=0",
]


def need(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def pin_file(path, expected, label):
    path = Path(path)
    need(path.is_file(), f"{label}: missing file {path}")
    data = path.read_bytes()
    need(len(data) == expected.get("bytes", len(data)), f"{label}: byte length differs for {path}")
    need(sha256(data) == expected["sha256"], f"{label}: SHA-256 differs for {path}")
    return {"path": str(path), "bytes": len(data), "sha256": sha256(data)}


def record_pin(record, label):
    return pin_file(record["path"], record, label)


def verify_streams(commands, label):
    pins = []
    for command in commands:
        for stream_name in ("stdout", "stderr"):
            stream = command["streams"][stream_name]
            pins.append(record_pin(stream, f"{label} command {command.get('index')} {stream_name}"))
    return pins


def u1(data, at):
    return data[at]


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def parse_class_method(data, want_name, want_descriptor):
    need(data[:4] == b"\xca\xfe\xba\xbe", "class input has invalid magic")
    cp_count = u2(data, 8)
    cp = [None] * cp_count
    at = 10
    index = 1
    while index < cp_count:
        tag = u1(data, at)
        at += 1
        if tag == 1:
            length = u2(data, at)
            at += 2
            cp[index] = (tag, data[at:at + length].decode("utf-8"))
            at += length
        elif tag in (3, 4):
            cp[index] = (tag, data[at:at + 4])
            at += 4
        elif tag in (5, 6):
            cp[index] = (tag, data[at:at + 8])
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, at))
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[index] = (tag, u2(data, at), u2(data, at + 2))
            at += 4
        elif tag == 15:
            cp[index] = (tag, u1(data, at), u2(data, at + 1))
            at += 3
        else:
            raise ValueError(f"unsupported class constant-pool tag {tag}")
        index += 1

    def utf8(cp_index):
        entry = cp[cp_index]
        need(entry is not None and entry[0] == 1, f"constant-pool {cp_index} is not UTF8")
        return entry[1]

    def class_name(cp_index):
        entry = cp[cp_index]
        need(entry is not None and entry[0] == 7, f"constant-pool {cp_index} is not Class")
        return utf8(entry[1])

    def method_ref(cp_index):
        entry = cp[cp_index]
        if entry is None or entry[0] not in (10, 11):
            return None
        nat = cp[entry[2]]
        need(nat[0] == 12, "method reference has invalid NameAndType")
        return (class_name(entry[1]), utf8(nat[1]), utf8(nat[2]))

    this_index = u2(data, at + 2)
    internal_name = class_name(this_index)
    at += 6
    interface_count = u2(data, at)
    at += 2 + 2 * interface_count

    def skip_members(pos, count):
        for _ in range(count):
            attr_count = u2(data, pos + 6)
            pos += 8
            for _ in range(attr_count):
                attr_len = u4(data, pos + 2)
                pos += 6 + attr_len
        return pos

    field_count = u2(data, at)
    at = skip_members(at + 2, field_count)
    method_count = u2(data, at)
    at += 2
    result = None
    for _ in range(method_count):
        name = utf8(u2(data, at + 2))
        descriptor = utf8(u2(data, at + 4))
        attr_count = u2(data, at + 6)
        attr_at = at + 8
        for _ in range(attr_count):
            attr_name = utf8(u2(data, attr_at))
            attr_len = u4(data, attr_at + 2)
            payload = attr_at + 6
            if name == want_name and descriptor == want_descriptor and attr_name == "Code":
                code_len = u4(data, payload + 4)
                code_start = payload + 8
                exc_count = u2(data, code_start + code_len)
                result = {
                    "owner": internal_name,
                    "code": data[code_start:code_start + code_len],
                    "exception_table_length": exc_count,
                    "method_ref": method_ref,
                }
            attr_at += 6 + attr_len
        at = attr_at
    need(result is not None, f"class lacks {want_name}{want_descriptor} Code")
    return result


def instruction_length(code, pc):
    op = code[pc]
    if op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
              0x36, 0x37, 0x38, 0x39, 0x3A, 0xA9, 0xBC):
        return 2
    if op in (0x11, 0x13, 0x14, 0x84,
              0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E,
              0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6,
              0xA7, 0xA8, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7,
              0xB8, 0xBB, 0xBD, 0xC0, 0xC1, 0xC6, 0xC7):
        return 3
    if op in (0xB9, 0xBA):
        return 5
    if op == 0xC5:
        return 4
    if op in (0xC8, 0xC9):
        return 5
    if op == 0xAA:
        base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3)
        low, high = struct.unpack_from(">ii", code, base + 4)
        return base - pc + 12 + 4 * (high - low + 1)
    if op == 0xAB:
        base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3)
        pairs = struct.unpack_from(">i", code, base + 4)[0]
        return base - pc + 8 + 8 * pairs
    if op == 0xC4:
        return 6 if code[pc + 1] == 0x84 else 4
    return 1


def decode(code):
    rows = []
    pc = 0
    while pc < len(code):
        size = instruction_length(code, pc)
        need(size > 0 and pc + size <= len(code), f"malformed bytecode instruction at BCI {pc}")
        rows.append((pc, code[pc], bytes(code[pc:pc + size])))
        pc += size
    need(pc == len(code), "bytecode decoder did not reach Code end")
    return rows


def verify_fixture_order(path, variant):
    data = Path(path).read_bytes()
    info = parse_class_method(data, METHOD_NAME, METHOD_DESC)
    rows = decode(info["code"])
    ref = info["method_ref"]

    def call(row, name, desc):
        return row[1] == 0xB8 and ref(u2(row[2], 1)) == ("ComputedForEffectControl", name, desc)

    sequences = []
    for i in range(len(rows) - 4):
        a, b, c, d, e = rows[i:i + 5]
        common = call(a, "f", "()I") and b[1] == 0x04 and c[1] == 0x64
        original = common and d[1] == 0x36 and d[2] == b"\x36\x06" and call(e, "g", "()V")
        pending = common and call(d, "g", "()V") and e[1] == 0x36 and e[2] == b"\x36\x06"
        if original:
            sequences.append("original")
        if pending:
            sequences.append("pending")
    need(sequences == [variant], f"{path}: expected exactly {variant} init/store/g ordering, found {sequences}")
    need(info["exception_table_length"] == 0, f"{path}: find has a handler")
    return {"path": str(path), "class_sha256": sha256(data), "class_bytes": len(data),
            "owner_internal_name": info["owner"], "method_descriptor": METHOD_DESC,
            "code_bytes": len(info["code"]), "instruction_bcis": [pc for pc, _, _ in rows],
            "init_call_order": variant, "find_exception_table_length": 0}


def method_report(doc):
    rows = [m["outcome"]["report"] for m in doc["methods"]
            if isinstance(m.get("outcome"), dict)
            and isinstance(m["outcome"].get("report"), dict)
            and m["outcome"]["report"].get("method") == METHOD_NAME + METHOD_DESC]
    need(len(rows) == 1, "CLI report must contain exactly one find method report")
    return rows[0]


def origin_rows(report):
    out = []
    for segment in report["source_map"]["segments"]:
        origin = segment["origin"]
        primary = origin.get("primary")
        if primary is not None:
            out.append((segment, primary))
        for derived in origin.get("derived", []):
            out.append((segment, derived))
    return out


def verify_source_map(doc, report, class_path, record_class_sha, record_class_len):
    class_data = Path(class_path).read_bytes()
    need(sha256(class_data) == record_class_sha and len(class_data) == record_class_len,
         f"source-map input differs from execution class pin: {class_path}")
    physical = parse_class_method(class_data, METHOD_NAME, METHOD_DESC)
    instruction_bcis = {pc for pc, _, _ in decode(physical["code"])}
    class_identity = doc["class"]["class_bytes"]
    class_digest = class_identity["digest"]
    need(re.fullmatch(r"[0-9a-f]{64}", class_digest) is not None,
         "class_bytes.digest must be a 32-byte hex identity (BLAKE3 field, not SHA-256)")
    need(class_identity["length"] == len(class_data), "report class byte length differs from input class")
    text_bytes = report["text"].encode("utf-8")
    seen = set()
    span_count = 0
    for segment, fact in origin_rows(report):
        start, end = segment["start"], segment["end"]
        need(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(text_bytes),
             "source map span lies outside method text")
        # Check both endpoints and the span itself are valid UTF-8 boundaries.
        text_bytes[:start].decode("utf-8")
        text_bytes[:end].decode("utf-8")
        text_bytes[start:end].decode("utf-8")
        method = fact["method"]
        need(bytes(method["name"]).decode("ascii") == METHOD_NAME, "source-map method owner name differs")
        need(bytes(method["descriptor"]).decode("ascii") == METHOD_DESC, "source-map method descriptor differs")
        owner = method["owner"]["class_bytes"]
        need(owner["digest"] == class_digest, "source-map owner BLAKE3 identity differs from report class identity")
        need(owner["length"] == len(class_data), "source-map owner length differs from input class length")
        bci = fact["bci"]
        need(bci in instruction_bcis, f"source map refers to non-instruction physical BCI {bci}")
        seen.add(bci)
        span_count += 1
    need(seen == instruction_bcis,
         f"source map physical BCI coverage differs: missing={sorted(instruction_bcis-seen)}, extra={sorted(seen-instruction_bcis)}")
    return {"input_class_sha256": record_class_sha, "input_class_bytes": record_class_len,
            "class_identity_field": "BLAKE3 (compared as reported; no SHA-256 relabeling)",
            "class_identity_digest": class_digest, "method": METHOD_NAME + METHOD_DESC,
            "physical_instruction_bcis": sorted(instruction_bcis), "source_map_bci_coverage": sorted(seen),
            "source_map_spans": span_count, "utf8_spans_valid": True}


def verify_live_product_sources(workspace, build):
    pins_outer = build["source_pins_before"]
    need(set(pins_outer) == {"full_product_source_and_include_closure"}, "unexpected build source-pin groups")
    before = pins_outer["full_product_source_and_include_closure"]
    after = build["source_pins_after"]["full_product_source_and_include_closure"]
    product = build["freeze"]["product_source_pins"]
    need(before == after == product, "build source pins differ before/after/freeze")
    paths_before = build["source_paths_before"]
    paths_after = build["source_paths_after"]
    need(paths_before == paths_after, "source path closure changed during CLI build/clean")
    need(set(paths_before) == set(before), "source path list and pin map differ")
    need(len(before) == build["product_source_pin_count"], "dynamic product pin count differs")
    named = {}
    for rel, expected in PRODUCT_PINS.items():
        need(before.get(rel) == expected, f"74b product source pin differs for {rel}")
        data = (workspace / rel).read_bytes()
        need(sha256(data) == expected, f"live workspace source differs from frozen 74b pin: {rel}")
        named[rel] = {"sha256": expected, "bytes": len(data)}
    checked = 0
    for rel, expected in before.items():
        data = (workspace / rel).read_bytes()
        need(sha256(data) == expected, f"live product source differs from frozen pin: {rel}")
        checked += 1
    return {"dynamic_product_source_pin_count": len(before), "live_source_files_checked": checked,
            "source_paths_before_after_equal": True, "product_source_pins_before_after_freeze_equal": True,
            "named_74b_product_source_pins": named}


def verify_build(build_path, workspace):
    build_bytes = build_path.read_bytes()
    build = json.loads(build_bytes)
    need(build["schema"] == "jarde-final-cli-build-clean-root-v1", "wrong final CLI build schema")
    need(build["status"] == "validation-passed-cli-frozen-target-cleaned", "final CLI build not accepted/clean")
    need(build["failure"] is None and build["pre_build_target_absent"] is True
         and build["target_absent_after_clean"] is True, "final CLI build/clean guard incomplete")
    need(build["guard_limits"] == {"free_bytes": 5368709120, "target_bytes": 1073741824},
         "final CLI build used unexpected resource guard limits")
    need(len(build["commands"]) == 2 and all(c["exit_code"] == 0 and c["guard_stop"] is None for c in build["commands"]),
         "final CLI build/clean commands did not both pass")
    build_stream_pins = verify_streams(build["commands"], "build")
    frozen = build["freeze"]["frozen_cli"]
    built = build["freeze"]["built_binary"]
    need(frozen["path"] == EXPECTED_CLI_PATH and frozen["sha256"] == EXPECTED_CLI_SHA256,
         "final CLI freeze does not match expected accepted CLI identity")
    need(built["sha256"] == frozen["sha256"] and built["size_bytes"] == frozen["size_bytes"],
         "built and frozen CLI identities differ")
    cli_path = Path(frozen["path"])
    cli_bytes = cli_path.read_bytes()
    need(len(cli_bytes) == frozen["size_bytes"] and sha256(cli_bytes) == frozen["sha256"],
         "frozen CLI file bytes differ from final build receipt")
    need(stat.S_IMODE(cli_path.stat().st_mode) == 0o555, "frozen CLI mode differs from accepted executable mode")
    sources = verify_live_product_sources(workspace, build)
    runner_pins = []
    for key in ("runner", "guard_template", "target_size_scan_adapter"):
        pin = build[key]
        runner_pins.append(pin_file(pin["path"], {"bytes": Path(pin["path"]).stat().st_size,
                                                "sha256": pin["sha256"]}, "build " + key))
    return build, {"execution_json": {"path": str(build_path), "bytes": len(build_bytes), "sha256": sha256(build_bytes)},
                   "schema": build["schema"], "status": build["status"], "source_base": build["source_base"],
                   "frozen_cli": {"path": frozen["path"], "bytes": frozen["size_bytes"],
                                  "sha256": frozen["sha256"], "mode": oct(stat.S_IMODE(cli_path.stat().st_mode))},
                   "build_runner_pins": runner_pins, "build_raw_stream_pins": build_stream_pins,
                   "product_source_verification": sources, "build_clean_passed": True}


def verify_prior_failure(path):
    data = path.read_bytes()
    rec = json.loads(data)
    need(rec.get("status") == "failed" and len(rec.get("commands", [])) == 5,
         "v1 failed single-class-policy control record missing or changed")
    last = rec["commands"][-1]
    need(last.get("index") == 104 and last.get("exit_code") == 2,
         "v1 expected fifth-command failure does not match")
    argv = last["argv"]
    need("--policy" not in argv, "v1 fifth command unexpectedly contains single-class policy")
    need(argv[0] == EXPECTED_CLI_PATH, "v1 failure came from a different CLI")
    failure_stream_pins = verify_streams(rec["commands"], "v1-preserved-failure")
    outer_streams = []
    for suffix in ("stdout.raw", "stderr.raw"):
        outer = path.parent.parent / (path.parent.name + "." + suffix)
        if outer.is_file():
            raw = outer.read_bytes()
            outer_streams.append({"path": str(outer), "bytes": len(raw), "sha256": sha256(raw),
                                 "post_run_observed": True})
    return {"execution_json": {"path": str(path), "bytes": len(data), "sha256": sha256(data)},
            "status": rec["status"], "command_count": len(rec["commands"]),
            "failure_command_index": last["index"], "failure_exit_code": last["exit_code"],
            "missing_policy_flag": True, "preserved_unchanged": True,
            "raw_stream_pins": failure_stream_pins, "outer_failure_capture_pins": outer_streams}


def output_rows(stream_record):
    content = Path(stream_record["path"]).read_bytes()
    lines = content.decode("utf-8").splitlines()
    return lines


def verify_runtime(run, workspace, root_dir, render=None, expected_variant=None):
    need(run["exit_code"] == 0 and run["guard_stop"] is None, "runtime command failed or guard stopped")
    argv = run["argv"]
    need(Path(argv[0]).name == "java" and "-Xverify:all" in argv, "runtime omitted -Xverify:all")
    cp = argv[argv.index("-cp") + 1].split(os.pathsep)
    need(argv[-1] == "Runner", "runtime main class is not Runner")
    if render is None:
        need(len(cp) == 1 and expected_variant in ("original", "pending"),
             "raw-class runtime has unexpected classpath/variant")
        raw_dir = Path(cp[0])
        need(raw_dir.name == expected_variant and raw_dir.parent.name in ("jdk8", "jdk23"),
             "raw-class runtime classpath is unexpected")
        need(raw_dir.is_dir(), "raw-class runtime directory is missing")
        need((raw_dir / "ComputedForEffectControl.class").is_file(), "raw control class missing from runtime classpath")
        need((Path(cp[0]) / "Runner.class").is_file(), "Runner.class missing from raw runtime classpath")
        priority = {"mode": "raw control runtime", "first_classpath": cp[0],
                    "control_exists_first": True, "runner_exists_first": True}
    else:
        source_dir = Path(render["source"]["path"]).parent
        jdk_name = source_dir.parent.name
        variant = render["variant"]
        original_dir = root_dir / jdk_name / "original"
        need(len(cp) == 2 and Path(cp[0]) == source_dir and Path(cp[1]) == original_dir,
             "compiled Jarde source is not first on the runtime classpath")
        generated_control = source_dir / "ComputedForEffectControl.class"
        need(generated_control.is_file(), "Jarde-compiled class missing from first classpath entry")
        need(not (source_dir / "Runner.class").exists(), "compiled output unexpectedly shadows Runner")
        need((original_dir / "Runner.class").is_file(), "original Runner.class was not available as runtime harness")
        base_input = root_dir / jdk_name / variant / "ComputedForEffectControl.class"
        need(generated_control.read_bytes() != base_input.read_bytes(),
             "Jarde runtime could fall through to original control class; generated class is absent/identical")
        priority = {"mode": "Jarde source runtime", "first_classpath": cp[0], "second_classpath": cp[1],
                    "generated_control_exists_first": True, "runner_borrowed_from_original_harness": True,
                    "generated_control_differs_from_input": True}
    rows = output_rows(run["streams"]["stdout"])
    need(rows == EXPECTED_ROWS, "runtime rows differ from five expected inputs/results/call orders")
    return {"index": run["index"], "argv": argv, "classpath_proof": priority,
            "stdout": record_pin(run["streams"]["stdout"], "runtime stdout"),
            "stderr": record_pin(run["streams"]["stderr"], "runtime stderr"), "rows": rows}


def verify_class_inventory(execution, root_dir):
    expected_dirs = set()
    compile_dirs = set()
    patched_dirs = set()
    for command in execution["commands"]:
        argv = command["argv"]
        if Path(argv[0]).name == "javac":
            out = Path(argv[argv.index("-d") + 1])
            expected_dirs.add(out)
            compile_dirs.add(out)
        elif Path(argv[0]).name.startswith("python") and "swap_init_store_with_g.py" in argv[1]:
            out = Path(argv[argv.index("--output") + 1]).parent
            expected_dirs.add(out)
            patched_dirs.add(out)
    observed = []
    for jdk_dir in (root_dir / "jdk8", root_dir / "jdk23"):
        need(jdk_dir.is_dir(), f"missing output tree {jdk_dir}")
        found = sorted(jdk_dir.rglob("*.class"))
        for path in found:
            need(any(path.is_relative_to(directory) for directory in expected_dirs),
                 f"unaccounted generated class path: {path}")
            data = path.read_bytes()
            in_compile = any(path.is_relative_to(directory) for directory in compile_dirs)
            in_patch = any(path.is_relative_to(directory) for directory in patched_dirs)
            if path.name == "Runner.class":
                provenance = "post-run-observed-runner-class (pending copy is not separately command-pinned)"
            elif in_patch:
                provenance = "post-run-observed-patched-class"
            elif in_compile:
                provenance = "post-run-observed-javac-output-class"
            else:
                provenance = "post-run-observed-unclassified"
            observed.append({"path": str(path), "bytes": len(data), "sha256": sha256(data),
                             "pin_timing": "observed after completed run; not recorded in execution.json",
                             "provenance": provenance})
    expected = set()
    for directory in expected_dirs:
        need(directory.is_dir(), f"recorded class output directory missing: {directory}")
        expected.update(directory.rglob("*.class"))
    found_all = set()
    for jdk_dir in (root_dir / "jdk8", root_dir / "jdk23"):
        found_all.update(jdk_dir.rglob("*.class"))
    need(found_all == expected, "complete generated-class inventory differs from recorded output directories")
    need(len(observed) == len(found_all), "generated class manifest omitted a class")
    runner_copies = {}
    for jdk in ("jdk8", "jdk23"):
        original_runner = root_dir / jdk / "original" / "Runner.class"
        pending_runner = root_dir / jdk / "pending" / "Runner.class"
        need(original_runner.is_file() and pending_runner.is_file(), f"{jdk} direct runtime Runner class missing")
        original_bytes, pending_bytes = original_runner.read_bytes(), pending_runner.read_bytes()
        need(original_bytes == pending_bytes, f"{jdk} pending harness Runner differs from original compiled Runner")
        runner_copies[jdk] = {"sha256": sha256(original_bytes), "bytes": len(original_bytes),
                              "pending_copy_equals_original": True,
                              "pending_copy_timing": "post-run observed; runner script source states shutil.copy2"}
    return {"pin_timing": "post-run observed; root execution record did not contain full generated-class hash inventory",
            "class_count": len(observed), "classes": observed, "runner_harness_copies": runner_copies}


def verify_execution(execution_path, build, prior_failure_path, workspace):
    raw = execution_path.read_bytes()
    execution = json.loads(raw)
    need(execution["status"] == "observed_complete_pending_root_review", "control execution is not complete")
    need(len(execution["commands"]) == 36, "control replay command count differs")
    need([c["index"] for c in execution["commands"]] == list(range(100, 136)), "control command indices are not exact")
    need(all(c["exit_code"] == 0 and c["guard_stop"] is None for c in execution["commands"]),
         "one or more control commands failed or stopped")
    need(len(execution["profiles"]) == 2 and {p["jdk"] for p in execution["profiles"]} == {"jdk8", "jdk23"},
         "control execution lacks both JDK legs")
    need(execution["cli"]["path"] == EXPECTED_CLI_PATH
         and execution["cli"]["sha256"] == build["freeze"]["frozen_cli"]["sha256"],
         "control CLI differs from the final frozen build CLI")
    pin_file(execution["cli"]["path"], execution["cli"], "control CLI")
    for source in execution["source_pins"]:
        record_pin(source, "control source input")
    for key in ("runner", "guard", "scanner"):
        record_pin(execution[key], "control " + key)
    control_stream_pins = verify_streams(execution["commands"], "control")

    command_by_index = {c["index"]: c for c in execution["commands"]}
    all_cli_args = [c["argv"] for c in execution["commands"] if c["argv"] and c["argv"][0] == EXPECTED_CLI_PATH]
    need(len(all_cli_args) == 8, "expected four class-source CLI renders per JDK")
    for argv in all_cli_args:
        need("--policy" in argv and argv[argv.index("--policy") + 1] == "single-class",
             "accepted control CLI invocation does not use single-class policy")

    inventory = verify_class_inventory(execution, execution_path.parent)
    profiles_report = []
    all_runtime_rows = []
    for profile in execution["profiles"]:
        jdk = profile["jdk"]
        for tool_name, tool_pin in profile["tools"].items():
            record_pin(tool_pin, f"{jdk} {tool_name} tool")
        classes = {"original": profile["original_class"], "pending": profile["pending_class"]}
        class_order = {}
        for variant, pin in classes.items():
            record_pin(pin, f"{jdk} {variant} input class")
            class_order[variant] = verify_fixture_order(pin["path"], variant)
        need(class_order["original"]["code_bytes"] == class_order["pending"]["code_bytes"],
             f"{jdk} original/pending Code lengths changed")

        runtime_summaries = []
        for variant, run in profile["runtimes"].items():
            summary = verify_runtime(run, workspace, execution_path.parent, expected_variant=variant)
            runtime_summaries.append(summary)
            all_runtime_rows.append({"jdk": jdk, "variant": variant, "kind": "input-class", "rows": summary["rows"]})

        renders_by_variant = {variant: {} for variant in ("original", "pending")}
        for render in profile["renders"]:
            variant, mode = render["variant"], render["mode"]
            need(mode in ("default", "all") and mode not in renders_by_variant[variant],
                 f"duplicate or unknown render profile {jdk}/{variant}/{mode}")
            report_pin = render["report"]
            raw_doc_bytes = Path(report_pin["path"]).read_bytes()
            need(len(raw_doc_bytes) == report_pin["bytes"] and sha256(raw_doc_bytes) == report_pin["sha256"],
                 f"{jdk}/{variant}/{mode} CLI report raw differs from execution pin")
            doc = json.loads(raw_doc_bytes)
            report = method_report(doc)
            source = render["source"]
            source_bytes = Path(source["path"]).read_bytes()
            need(len(source_bytes) == source["bytes"] and sha256(source_bytes) == source["sha256"],
                 f"{jdk}/{variant}/{mode} generated Java source differs from record pin")
            need(source_bytes == doc["text"].encode("utf-8"),
                 f"{jdk}/{variant}/{mode} generated full-class source differs from report text")
            need(render["matches_original_variant"] is True, "recorded render runtime comparison failed")
            source_map_proof = verify_source_map(doc, report, classes[variant]["path"],
                                                classes[variant]["sha256"], classes[variant]["bytes"])
            need(report["outcome"] == "produced" and report["quality"] == "structured"
                 and report["execution"]["status"] == "complete", "find report not structured/complete")
            text = report["text"]
            if variant == "original":
                need("local6 = f() - 1;" in text and "g();" in text and "saved0" not in text,
                     "original render no longer exhibits expected while baseline text")
                need(text.index("local6 = f() - 1;") < text.index("g();") < text.index("while ("),
                     "original render initializer/call/loop order differs")
            else:
                need("saved0 = f();" in text and "g();" in text and "local6 = saved0 - 1;" in text,
                     "pending render no longer exhibits expected saved producer ordering")
                need(text.index("saved0 = f();") < text.index("g();")
                     < text.index("local6 = saved0 - 1;") < text.index("while ("),
                     "pending render does not place saved producer, g, initializer, loop in exact order")
            input_arg = render["report"]["path"]
            cli_command = next((c for c in execution["commands"]
                                if c["streams"]["stdout"]["path"] == input_arg), None)
            need(cli_command is not None and cli_command["argv"][0] == EXPECTED_CLI_PATH,
                 "CLI report is not bound to the frozen final CLI argv")
            argv = cli_command["argv"]
            need(argv[argv.index("--input") + 1] == classes[variant]["path"],
                 "CLI argv input class differs from source-map input class")
            need(("--evidence" in argv) == (mode == "all"), "CLI evidence mode differs from report profile")
            run_summary = verify_runtime(render["runtime"], workspace, execution_path.parent, render)
            runtime_summaries.append(run_summary)
            all_runtime_rows.append({"jdk": jdk, "variant": variant, "kind": "generated-source-" + mode,
                                     "rows": run_summary["rows"]})
            renders_by_variant[variant][mode] = {"doc": doc, "report": report,
                                                 "source_sha256": source["sha256"],
                                                 "source_map_proof": source_map_proof}
        need(all(set(rows) == {"default", "all"} for rows in renders_by_variant.values()),
             f"{jdk} lacks complete default/all render matrix")
        text_map_summary = {}
        for variant, modes in renders_by_variant.items():
            default_doc, all_doc = modes["default"]["doc"], modes["all"]["doc"]
            need(default_doc["text"] == all_doc["text"], f"{jdk}/{variant} whole-class default/all text differs")
            dmethods, amethods = default_doc["methods"], all_doc["methods"]
            need(len(dmethods) == len(amethods), f"{jdk}/{variant} default/all method count differs")
            for dm, am in zip(dmethods, amethods):
                need(dm["text"] == am["text"], f"{jdk}/{variant} method source differs default/all")
                dr = dm.get("outcome", {}).get("report")
                ar = am.get("outcome", {}).get("report")
                if dr is not None or ar is not None:
                    need(dr is not None and ar is not None and dr.get("text") == ar.get("text")
                         and dr.get("source_map") == ar.get("source_map"),
                         f"{jdk}/{variant} method text/source map differs default/all")
            need(modes["default"]["report"]["text"] == modes["all"]["report"]["text"]
                 and modes["default"]["report"]["source_map"] == modes["all"]["report"]["source_map"],
                 f"{jdk}/{variant} find text/source map differs default/all")
            text_map_summary[variant] = {"whole_class_text_equal": True, "all_method_texts_equal": True,
                                         "all_method_source_maps_equal": True,
                                         "find_text_bytes": len(modes["default"]["report"]["text"].encode("utf-8")),
                                         "find_source_map_segments": len(modes["default"]["report"]["source_map"]["segments"]),
                                         "find_source_map_physical_identity": {
                                             "default": modes["default"]["source_map_proof"],
                                             "all": modes["all"]["source_map_proof"]}}
        profiles_report.append({"jdk": jdk, "tools": profile["tools"], "class_input_order": class_order,
                                "default_all_text_map_equal": text_map_summary,
                                "runtime_sides": runtime_summaries})

    need(len(all_runtime_rows) == 12, "expected exactly twelve runtime sides")
    need(all(item["rows"] == EXPECTED_ROWS for item in all_runtime_rows), "runtime result rows differ across sides")
    return execution, {"execution_json": {"path": str(execution_path), "bytes": len(raw), "sha256": sha256(raw)},
                       "runner": record_pin(execution["runner"], "root control runner"),
                       "guard": record_pin(execution["guard"], "guard helper"),
                       "scanner": record_pin(execution["scanner"], "target-size scanner"),
                       "control_source_pins": [record_pin(p, "control source") for p in execution["source_pins"]],
                       "command_count": len(execution["commands"]), "all_commands_passed": True,
                       "recorded_command_stream_count": 2 * len(execution["commands"]),
                       "raw_stream_pins": control_stream_pins,
                       "profiles": profiles_report, "generated_classes": inventory,
                       "runtime_side_count": len(all_runtime_rows), "runtime_rows": EXPECTED_ROWS,
                       "all_runtime_sides_match": True}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--execution", required=True, type=Path)
    parser.add_argument("--build", required=True, type=Path)
    parser.add_argument("--prior-failure", required=True, type=Path)
    parser.add_argument("--workspace", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    args.workspace = args.workspace.resolve()
    need(args.workspace.is_dir(), "workspace directory does not exist")
    result = {"schema": "computed-for-call-order-independent-verifier-v1",
              "scope": "74b product / current CLI while baseline; not a computed-for candidate acceptance",
              "inputs": {"execution": str(args.execution), "build": str(args.build),
                         "prior_v1_failure": str(args.prior_failure), "workspace": str(args.workspace)}}
    try:
        build, build_summary = verify_build(args.build, args.workspace)
        prior = verify_prior_failure(args.prior_failure)
        execution, execution_summary = verify_execution(args.execution, build, args.prior_failure, args.workspace)
        result.update({"status": "accepted", "build": build_summary,
                       "preserved_v1_failure": prior, "control_execution": execution_summary})
    except Exception as error:
        result.update({"status": "failed", "error": type(error).__name__ + ": " + str(error)})
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        print(result["error"], file=sys.stderr)
        return 1
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print("status=accepted")
    print("scope=" + result["scope"])
    print("commands=" + str(execution_summary["command_count"]) + " all_passed=true")
    print("runtime_sides=" + str(execution_summary["runtime_side_count"]))
    print("generated_classes=" + str(execution_summary["generated_classes"]["class_count"])
          + " (post-run observed pins)")
    for row in EXPECTED_ROWS:
        print(row)
    print("report=" + str(args.out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
