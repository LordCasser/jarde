#!/usr/bin/env python3
"""Independently verify the immutable no-clinit/super-argument replay bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import zipfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
OUT = HERE / "baseline-root-v2"
DEFAULT_OUTPUT = HERE / "baseline-root-verification-v1.json"
METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
SOURCE_SHA256 = {
    "ArrayFieldInitBase.java": "16c212caacb1e4fcfb67e78670b4fdd3418cf6e3d5e3977cfd80535403b3d1a9",
    "CommonNoClinitArrayInit.java": "617ff8efc82edec37b50b5d87a3ef43e74fe64374c10caad95ac24211b3521d1",
    "Runner.java": "51336925efdb1ef4abd6270bc63bbb9c752e8a7ea2b98e96b6c2c93bc33fad4d",
}
EXPECTED = {
    "ArrayFieldInitBase": {"fields": {"received:I"}, "methods": {"<init>(I)V"}},
    "CommonNoClinitArrayInit": {
        "fields": {"trace:I", "first:[B", "second:[B"},
        "methods": {"<init>()V", "<init>(I)V", "mark(I)B", "run(I)B"},
    },
}
EXPECTED_OUTPUT = (
    b"noargs-super=7\nnoargs-arrays=[11, 12]/[21]\nnoargs-trace=339975\n"
    b"arg-super=40\narg-arrays=[11, 12]/[21]\narg-trace=339924\nfresh=true\n"
)
PRODUCT_PATHS = {
    "Cargo.lock", "crates/jarde-java/src/asserts.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/field.rs", "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs", "src/class_source.rs", "src/facade.rs",
}
TEST_PATHS = {
    "tests/class_static_initializer_projection.rs", "tests/interface_initializer_proof.rs",
    "crates/jarde-java/tests/p3_patterns.rs", ".github/workflows/ci.yml",
}


class Check:
    def __init__(self) -> None:
        self.rows: list[dict] = []

    def require(self, name: str, ok: bool, detail=None) -> None:
        self.rows.append({"check": name, "ok": bool(ok), "detail": detail})

    def finish(self) -> dict:
        return {"checks": self.rows, "passed": sum(row["ok"] for row in self.rows),
                "failed": sum(not row["ok"] for row in self.rows),
                "success": all(row["ok"] for row in self.rows)}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def safe_file(base: Path, relative: str) -> Path:
    path = base / relative
    if path.is_symlink():
        raise ValueError(f"symlink is not permitted: {relative}")
    resolved = path.resolve(strict=True)
    resolved.relative_to(base.resolve(strict=True))
    if not resolved.is_file():
        raise ValueError(f"not a file: {relative}")
    return resolved


def check_record(check: Check, base: Path, record: dict, label: str) -> bytes | None:
    try:
        path = safe_file(base, record["path"])
        data = path.read_bytes()
        actual = {"bytes": len(data), "sha256": sha(data)}
        expected = {"bytes": record["bytes"], "sha256": record["sha256"]}
        check.require(label, actual == expected, {"path": record["path"], **actual})
        return data
    except (OSError, KeyError, ValueError) as error:
        check.require(label, False, str(error))
        return None


def close_inventory(check: Check, out: Path, manifest: dict) -> None:
    inventory = read_json(out / "file-inventory.json")
    rows = {row["path"]: row for row in inventory}
    actual: dict[str, Path] = {}
    for parent, dirs, files in os.walk(out, followlinks=False):
        parent_path = Path(parent)
        for name in dirs:
            path = parent_path / name
            if path.is_symlink():
                check.require("inventory:no-directory-symlink", False, str(path))
        for name in files:
            path = parent_path / name
            if path.is_symlink():
                check.require("inventory:no-file-symlink", False, str(path))
                continue
            rel = path.relative_to(out).as_posix()
            if rel != "file-inventory.json":
                actual[rel] = path
    check.require("inventory:unique-paths", len(rows) == len(inventory))
    check.require("inventory:closed-file-set", set(actual) == set(rows), {
        "unlisted": sorted(set(actual) - set(rows)), "missing": sorted(set(rows) - set(actual)),
    })
    for rel in sorted(set(actual) & set(rows)):
        data, row = actual[rel].read_bytes(), rows[rel]
        check.require(f"inventory:hash:{rel}", len(data) == row["bytes"] and sha(data) == row["sha256"])
    check.require("inventory:includes-manifest", "manifest.json" in rows)
    check.require("inventory:excludes-itself", "file-inventory.json" not in rows)
    check.require("inventory:manifest-policy",
                  manifest.get("file_inventory", {}).get("includes") == ["manifest.json"]
                  and manifest.get("file_inventory", {}).get("excludes") == ["file-inventory.json"])


def verify_commands(check: Check, manifest: dict) -> dict[str, dict]:
    commands = manifest.get("commands", [])
    by_label = {row.get("label"): row for row in commands}
    expected = {"jadx-version", "jadx-default-decompile", "jadx-none-decompile"}
    for jdk in ("javac8", "javac23"):
        expected.update({f"{jdk}-original-compile", f"{jdk}-original-run"})
        expected.update(f"{jdk}-original-javap-{name}" for name in
                        ("ArrayFieldInitBase", "CommonNoClinitArrayInit", "Runner"))
        for profile in ("default", "none"):
            expected.update({f"{jdk}-jadx-{profile}-compile", f"{jdk}-jadx-{profile}-run"})
        expected.update({f"{jdk}-jarde-compile", f"{jdk}-jarde-run"})
        for name in EXPECTED:
            for profile in ("default", "all"):
                expected.add(f"{jdk}-jarde-render-{name}-{profile}")
    check.require("commands:unique-labels", len(by_label) == len(commands))
    check.require("commands:complete-set", set(by_label) == expected,
                  {"expected": sorted(expected), "actual": sorted(by_label)})
    for label, command in by_label.items():
        check.require(f"command:{label}:exit", command.get("exit") == 0)
        for name in ("stdout", "stderr"):
            check_record(check, OUT, command[name], f"command:{label}:{name}")
    return by_label


def bind_command(check: Check, nested: dict | None, by_label: dict[str, dict], label: str) -> None:
    actual = by_label.get(nested.get("label")) if isinstance(nested, dict) else None
    check.require(f"command-binding:{label}", actual == nested, nested.get("label") if isinstance(nested, dict) else None)


def runtime_triple(check: Check, case: dict) -> tuple[int, bytes, bytes] | None:
    row = case.get("runtime")
    if not row:
        check.require(f"runtime:{case.get('label')}:present", False)
        return None
    stdout = check_record(check, OUT, row["stdout"], f"runtime:{case['label']}:stdout")
    stderr = check_record(check, OUT, row["stderr"], f"runtime:{case['label']}:stderr")
    return (row.get("exit"), stdout, stderr) if stdout is not None and stderr is not None else None


def javap_inventory(text: str, class_name: str) -> tuple[set[str], set[str], bool]:
    fields: set[str] = set()
    methods: set[str] = set()
    lines = text.splitlines()
    for index, line in enumerate(lines[:-1]):
        match = re.fullmatch(r"  ([^ ].*);", line)
        if not match:
            continue
        declaration = match.group(1)
        descriptor = re.match(r"\s*descriptor:\s*(\S+)", lines[index + 1])
        if descriptor is None:
            continue
        desc = descriptor.group(1)
        if "(" in declaration:
            name = declaration.split("(", 1)[0].split()[-1]
            if name == class_name:
                name = "<init>"
            methods.add(name + desc)
        else:
            fields.add(declaration.split()[-1] + ":" + desc)
    has_clinit = bool(re.search(r"(?m)^  (?:static\s*\{\};?|<clinit>\(\);?)\s*$", text))
    return fields, methods, has_clinit


def classfile_inventory(data: bytes) -> dict:
    """Read the class header, constant pool, members and attribute boundaries without a JDK."""
    offset = 0

    def take(size: int) -> bytes:
        nonlocal offset
        if size < 0 or offset + size > len(data):
            raise ValueError("truncated class file")
        chunk = data[offset:offset + size]
        offset += size
        return chunk

    def u1() -> int:
        return take(1)[0]

    def u2() -> int:
        return int.from_bytes(take(2), "big")

    def u4() -> int:
        return int.from_bytes(take(4), "big")

    if take(4) != b"\xca\xfe\xba\xbe":
        raise ValueError("bad classfile magic")
    minor, major = u2(), u2()
    cp_count = u2()
    pool: list[object | None] = [None] * cp_count
    index = 1
    while index < cp_count:
        tag = u1()
        if tag == 1:
            length = u2()
            pool[index] = take(length).decode("utf-8", errors="replace")
        elif tag in (3, 4):
            take(4)
        elif tag in (5, 6):
            take(8)
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pool[index] = u2()
        elif tag in (9, 10, 11, 12, 17, 18):
            take(4)
        elif tag == 15:
            take(3)
        else:
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1

    def utf8(cp_index: int) -> str:
        value = pool[cp_index]
        if isinstance(value, int):
            value = pool[value]
        if not isinstance(value, str):
            raise ValueError(f"constant pool index {cp_index} is not Utf8")
        return value

    access = u2()
    this_class = u2()
    super_class = u2()
    this_name = utf8(int(pool[this_class]))
    interface_count = u2()
    interfaces = [utf8(int(pool[u2()])) for _ in range(interface_count)]

    def members() -> list[dict]:
        result = []
        for _ in range(u2()):
            flags, name_index, descriptor_index = u2(), u2(), u2()
            attrs = []
            for _ in range(u2()):
                attr_name = utf8(u2())
                length = u4()
                take(length)
                attrs.append(attr_name)
            result.append({"name": utf8(name_index), "descriptor": utf8(descriptor_index),
                           "access_flags": flags, "attributes": attrs})
        return result

    fields = members()
    methods = members()
    class_attributes = []
    for _ in range(u2()):
        class_attributes.append(utf8(u2()))
        take(u4())
    if offset != len(data):
        raise ValueError(f"unparsed classfile tail ({len(data) - offset} bytes)")
    return {"minor": minor, "major": major, "access_flags": access, "name": this_name,
            "super_index": super_class, "super_name": utf8(int(pool[super_class])) if super_class else None,
            "interfaces": interfaces, "fields": fields,
            "methods": methods, "class_attributes": class_attributes}


def check_classfile(check: Check, case: dict, record: dict, data: bytes) -> None:
    label = case["label"]
    try:
        parsed = classfile_inventory(data)
    except (ValueError, IndexError) as error:
        check.require(f"{label}:classfile-parse:{record.get('path')}", False, str(error))
        return
    name = parsed["name"].rsplit("/", 1)[-1]
    class_root = Path(case["class_output"]).resolve().relative_to(OUT.resolve()).as_posix()
    expected_internal_name = Path(record["path"]).relative_to(class_root).with_suffix("").as_posix()
    check.require(f"{label}:classfile-internal-name:{name}", parsed["name"] == expected_internal_name,
                  {"expected": expected_internal_name, "actual": parsed["name"]})
    expected_super = "java/lang/Object" if name in ("ArrayFieldInitBase", "Runner") else "ArrayFieldInitBase"
    check.require(f"{label}:classfile-super:{name}", parsed["super_name"] == expected_super, parsed["super_name"])
    check.require(f"{label}:classfile-no-interfaces:{name}", parsed["interfaces"] == [])
    class_methods = {item["name"] + item["descriptor"] for item in parsed["methods"]}
    check.require(f"{label}:classfile-no-clinit:{name}", not any(method.startswith("<clinit>") for method in class_methods))
    if name in EXPECTED:
        class_fields = {item["name"] + ":" + item["descriptor"] for item in parsed["fields"]}
        check.require(f"{label}:classfile-fields:{name}", class_fields == EXPECTED[name]["fields"], sorted(class_fields))
        check.require(f"{label}:classfile-methods:{name}", class_methods == EXPECTED[name]["methods"], sorted(class_methods))
        actual_flags = {item["name"] + ":" + item["descriptor"]: item["access_flags"] for item in parsed["fields"]}
        actual_flags.update({item["name"] + item["descriptor"]: item["access_flags"] for item in parsed["methods"]})
        wanted_flags = ({"received:I": 0x0010, "<init>(I)V": 0}
                        if name == "ArrayFieldInitBase" else
                        {"trace:I": 0x0008, "first:[B": 0x0010, "second:[B": 0x0010,
                         "mark(I)B": 0x0008, "run(I)B": 0x0008,
                         "<init>()V": 0x0001, "<init>(I)V": 0x0001})
        check.require(f"{label}:classfile-access-flags:{name}", actual_flags == wanted_flags, actual_flags)


def instructions(text: str, declaration: str, descriptor: str) -> list[tuple[int, str, str]]:
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if line != "  " + declaration:
            continue
        desc_at = next((n for n in range(index + 1, min(index + 5, len(lines)))
                        if lines[n].strip() == "descriptor: " + descriptor), None)
        if desc_at is None:
            continue
        code_at = next((n for n in range(desc_at + 1, min(desc_at + 8, len(lines)))
                        if lines[n].strip() == "Code:"), None)
        if code_at is None:
            continue
        output = []
        for entry in lines[code_at + 1:]:
            match = re.match(r"\s+\d+:\s+(\S+)(?:\s+(.*?))?\s*$", entry)
            if match:
                output.append((int(match.group(0).split(":", 1)[0].strip()),
                               match.group(1), (match.group(2) or "").strip()))
            elif output and (entry.startswith("  ") or entry == "}"):
                break
        return output
    return []


def instruction_order(check: Check, name: str, code: list[tuple[int, str, str]], wanted: list[tuple[str, str]]) -> None:
    cursor = 0
    for op, operand in wanted:
        match = next((i for i in range(cursor, len(code))
                      if code[i][1] == op and operand in code[i][2]), None)
        if match is None:
            check.require(name, False, {"missing": [op, operand], "instruction_count": len(code)})
            return
        cursor = match + 1
    check.require(name, bool(code))


def instruction_offsets(text: str, declaration: str, descriptor: str) -> set[int] | None:
    code = instructions(text, declaration, descriptor)
    return {offset for offset, _, _ in code} if code else None


def verify_javap(check: Check, case: dict, by_label: dict[str, dict]) -> None:
    label, jdk = case["label"], case["jdk_leg"]
    for class_name in ("ArrayFieldInitBase", "CommonNoClinitArrayInit", "Runner"):
        row = next((item for item in case.get("javap", []) if item.get("class") == class_name), None)
        check.require(f"{label}:javap-row:{class_name}", row is not None)
        if row is None:
            continue
        bind_command(check, row.get("command"), by_label, f"{label}:javap:{class_name}")
        data = check_record(check, OUT, row["raw_javap"], f"{label}:javap-raw:{class_name}")
        check_record(check, OUT, row["class_file"], f"{label}:javap-class:{class_name}")
        if data is None:
            continue
        text = data.decode("utf-8", errors="replace")
        fields, methods, has_clinit = javap_inventory(text, class_name)
        command_argv = row.get("command", {}).get("argv", [])
        class_path = str((OUT / row["class_file"]["path"]).resolve())
        try:
            jdk_doc = read_json(JDK_MANIFEST)
            leg = next(item for item in jdk_doc["legs"] if item["leg"] == jdk)
            expected_javap = leg["jdk_tools"]["javap"]["path"]
        except (OSError, KeyError, StopIteration):
            expected_javap = None
        check.require(f"{label}:javap-exact-command:{class_name}",
                      command_argv == [expected_javap, "-p", "-c", "-s", "-v", class_path])
        command_stdout = check_record(check, OUT, row["command"]["stdout"],
                                      f"{label}:javap-command-stdout:{class_name}")
        check.require(f"{label}:javap-stdout-is-raw:{class_name}", command_stdout == data)
        if class_name in EXPECTED:
            check.require(f"{label}:javap-fields:{class_name}", fields == EXPECTED[class_name]["fields"], sorted(fields))
            check.require(f"{label}:javap-methods:{class_name}", methods == EXPECTED[class_name]["methods"], sorted(methods))
        check.require(f"{label}:javap-no-clinit:{class_name}", not has_clinit)
        allowed_old_jdk_false_flag = jdk == "javac8" and label == "javac8-original" \
            and class_name in EXPECTED and row.get("physical_counts_and_no_clinit") is False
        check.require(f"{label}:javap-recorded-outcome:{class_name}",
                      row.get("success") is True or allowed_old_jdk_false_flag)
        if class_name == "ArrayFieldInitBase":
            code = instructions(text, "ArrayFieldInitBase(int);", "(I)V")
            instruction_order(check, f"{label}:base-constructor", code, [
                ("aload_0", ""), ("invokespecial", 'java/lang/Object."<init>":()V'),
                ("aload_0", ""), ("iload_1", ""), ("putfield", "Field received:I"), ("return", ""),
            ])
        elif class_name == "CommonNoClinitArrayInit":
            zero = instructions(text, "public CommonNoClinitArrayInit();", "()V")
            arg = instructions(text, "public CommonNoClinitArrayInit(int);", "(I)V")
            sequence_zero = [
                ("aload_0", ""), ("bipush", "7"), ("invokespecial", 'ArrayFieldInitBase."<init>":(I)V'),
                ("iconst_2", ""), ("newarray", "byte"), ("bipush", "11"), ("invokestatic", "Method mark:(I)B"),
                ("bastore", ""), ("bipush", "12"), ("invokestatic", "Method mark:(I)B"), ("bastore", ""),
                ("putfield", "Field first:[B"), ("iconst_1", ""), ("newarray", "byte"),
                ("bipush", "21"), ("invokestatic", "Method run:(I)B"), ("bastore", ""),
                ("putfield", "Field second:[B"), ("getstatic", "Field trace:I"), ("bipush", "31"),
                ("imul", ""), ("bipush", "91"), ("iadd", ""), ("putstatic", "Field trace:I"), ("return", ""),
            ]
            sequence_arg = [
                ("aload_0", ""), ("iload_1", ""), ("invokespecial", 'ArrayFieldInitBase."<init>":(I)V'),
                ("iconst_2", ""), ("newarray", "byte"), ("bipush", "11"), ("invokestatic", "Method mark:(I)B"),
                ("bastore", ""), ("bipush", "12"), ("invokestatic", "Method mark:(I)B"), ("bastore", ""),
                ("putfield", "Field first:[B"), ("iconst_1", ""), ("newarray", "byte"),
                ("bipush", "21"), ("invokestatic", "Method run:(I)B"), ("bastore", ""),
                ("putfield", "Field second:[B"), ("getstatic", "Field trace:I"), ("bipush", "31"),
                ("imul", ""), ("iload_1", ""), ("iadd", ""), ("putstatic", "Field trace:I"), ("return", ""),
            ]
            instruction_order(check, f"{label}:noarg-super-array-suffix-order", zero, sequence_zero)
            instruction_order(check, f"{label}:arg-super-array-suffix-order", arg, sequence_arg)


def inspect_cli_doc(check: Check, doc: dict, class_name: str, profile: str, row: dict,
                    original_javap: str | None = None) -> None:
    fields = {m["item"]["name"]["escaped"] + ":" + m["item"]["descriptor"]["escaped"]
              for m in doc.get("fields", [])}
    methods = {m["item"]["name"]["escaped"] + m["item"]["descriptor"]["escaped"]
               for m in doc.get("methods", [])}
    check.require(f"cli:{class_name}:{profile}:fields", fields == EXPECTED[class_name]["fields"], sorted(fields))
    check.require(f"cli:{class_name}:{profile}:methods", methods == EXPECTED[class_name]["methods"], sorted(methods))
    check.require(f"cli:{class_name}:{profile}:no-clinit", not any(m.startswith("<clinit>") for m in methods))
    check.require(f"cli:{class_name}:{profile}:class-execution", doc.get("execution", {}).get("status") == "complete")
    check.require(f"cli:{class_name}:{profile}:class-outcome", doc.get("outcome") == "performed")
    maps = []
    for method in doc.get("methods", []):
        report = method.get("outcome", {}).get("report", {})
        quality = (report.get("quality") == "structured" and report.get("outcome") == "produced"
                   and report.get("execution", {}).get("status") == "complete" and report.get("fallbacks", []) == [])
        check.require(f"cli:{class_name}:{profile}:method-quality:{method['item'].get('index')}", quality)
        source_map = report.get("source_map")
        categories = report.get("evidence", {}).get("categories", [])
        category = next((item for item in categories if item.get("kind") == "source_map"), None)
        complete = (isinstance(source_map, dict) and isinstance(source_map.get("segments"), list)
                    and category is not None and category.get("state", {}).get("state") == "complete")
        if profile == "all":
            check.require(f"cli:{class_name}:all:map:{method['item'].get('index')}", complete)
            if complete:
                identity = method["item"].get("identity")
                expected_origin = {key: identity.get(key) for key in ("name", "descriptor", "owner")}
                bcis = set()
                origins_match = True
                for segment in source_map["segments"]:
                    origin = segment.get("origin", {})
                    for point in ([origin.get("primary")] + origin.get("derived", [])):
                        if point is None:
                            continue
                        bcis.add(point.get("bci"))
                        origins_match &= point.get("method") == expected_origin
                check.require(f"cli:{class_name}:all:map-owner-and-method:{method['item'].get('index')}",
                              origins_match)
                if original_javap is not None:
                    sig = method["item"]["name"]["escaped"] + method["item"]["descriptor"]["escaped"]
                    declaration = {
                        "ArrayFieldInitBase:<init>(I)V": "ArrayFieldInitBase(int);",
                        "CommonNoClinitArrayInit:<init>()V": "public CommonNoClinitArrayInit();",
                        "CommonNoClinitArrayInit:<init>(I)V": "public CommonNoClinitArrayInit(int);",
                        "CommonNoClinitArrayInit:mark(I)B": "static byte mark(int);",
                        "CommonNoClinitArrayInit:run(I)B": "static byte run(int);",
                    }.get(class_name + ":" + sig)
                    raw_offsets = instruction_offsets(original_javap, declaration,
                                                      method["item"]["descriptor"]["escaped"]) if declaration else set()
                    check.require(f"cli:{class_name}:all:map-bci-set:{method['item'].get('index')}",
                                  raw_offsets is not None and bcis == raw_offsets,
                                  {"mapped": sorted(value for value in bcis if isinstance(value, int)),
                                   "bytecode": sorted(raw_offsets or set())})
            maps.append({
                "index": method["item"].get("index"),
                "signature": method["item"]["name"]["escaped"] + method["item"]["descriptor"]["escaped"],
                "identity": method["item"].get("identity"),
                "access_flags": method["item"].get("access_flags"),
                "quality_and_execution_complete": quality,
                "source_map_complete": complete,
                "source_map": source_map,
            })
    if profile == "all":
        check.require(f"cli:{class_name}:all:manifest-map-equality", maps == row.get("method_source_maps", []))


def verify_metadata(check: Check, manifest: dict, by_label: dict[str, dict]) -> dict[str, dict]:
    data = METADATA.read_bytes()
    metadata = json.loads(data)
    check.require("metadata:frozen-sha", sha(data) == METADATA_SHA256)
    check.require("metadata:cli-pin", metadata.get("cli_path") == "/private/tmp/jarde-nonfinal-static-cli-v1"
                  and metadata.get("cli_sha256") == CLI_SHA256)
    check.require("metadata:source-pin-sets", set(metadata.get("candidate_sources", {})) == PRODUCT_PATHS
                  and set(metadata.get("test_sources", {})) == TEST_PATHS
                  and len(metadata.get("canonical_files", {})) == 16)
    for group in ("candidate_sources", "test_sources", "canonical_files"):
        for relative, expected in metadata[group].items():
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() and not path.is_symlink() else None
            check.require(f"metadata:{group}:{relative}", actual == expected)
    check.require("manifest:prepared-source-identity", {
        row.get("path"): row.get("sha256") for row in manifest.get("source_files", [])
    } == SOURCE_SHA256)
    cli = Path(metadata["cli_path"])
    actual_cli = sha(cli.read_bytes()) if cli.is_file() else None
    check.require("cli:binary-sha", actual_cli == CLI_SHA256)
    check.require("manifest:cli-pins", manifest.get("frozen_cli") == {"path": metadata["cli_path"], "sha256": CLI_SHA256}
                  and manifest.get("cli_metadata", {}).get("path") == str(METADATA.resolve())
                  and manifest.get("cli_metadata", {}).get("sha256") == METADATA_SHA256)
    jadx = Path(manifest["jadx"]["path"])
    actual_jadx = sha(jadx.read_bytes()) if jadx.is_file() else None
    check.require("jadx:binary-sha", actual_jadx == JADX_SHA256
                  and str(jadx) == "/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx"
                  and manifest["jadx"].get("version") == "1.5.6")
    jdk_data = JDK_MANIFEST.read_bytes()
    jdk = json.loads(jdk_data)
    check.require("jdk:manifest-sha", sha(jdk_data) == JDK_MANIFEST_SHA256 and jdk.get("status") == "complete")
    legs = {row.get("leg"): row for row in jdk.get("legs", [])}
    check.require("jdk:legs", set(legs) == {"javac8", "javac23"})
    for name, leg in legs.items():
        tools = leg.get("jdk_tools", {})
        check.require(f"jdk:{name}:tools", set(tools) == {"java", "javac", "javap"})
        for tool, row in tools.items():
            path = Path(row["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            check.require(f"jdk:{name}:{tool}:sha", actual == row.get("sha256"))
    check.require("jadx:version-command", by_label.get("jadx-version", {}).get("exit") == 0)
    version_row = manifest.get("jadx", {}).get("version_command", {})
    bind_command(check, version_row, by_label, "jadx-version")
    version_out = check_record(check, OUT, by_label["jadx-version"]["stdout"], "jadx:version-stdout")
    check.require("jadx:version-output", version_out == b"1.5.6\n")
    jar_record = manifest.get("jadx", {}).get("jar", {})
    jar_data = check_record(check, OUT, jar_record, "jadx:input-jar")
    expected_members = {row["name"]: row for row in manifest.get("jadx", {}).get("jar_members", [])}
    if jar_data is not None:
        try:
            with zipfile.ZipFile(OUT / jar_record["path"]) as archive:
                names = set(archive.namelist())
                check.require("jadx:jar-member-set", names == set(expected_members), sorted(names))
                for name, record in expected_members.items():
                    member = archive.read(name)
                    check.require(f"jadx:jar-member:{name}", len(member) == record["bytes"]
                                  and sha(member) == record["sha256"])
        except (OSError, KeyError, zipfile.BadZipFile) as error:
            check.require("jadx:jar-readable", False, str(error))
    return legs


def verify_cases(check: Check, manifest: dict, by_label: dict[str, dict], jdk_legs: dict[str, dict]) -> None:
    cases = manifest.get("cases", [])
    expected = {"javac8-original", "javac23-original"}
    expected.update(f"javac{jdk}-jadx-{profile}" for jdk in (8, 23) for profile in ("default", "none"))
    expected.update({"javac8-jarde", "javac23-jarde"})
    cases_by_label = {case.get("label"): case for case in cases}
    check.require("cases:eight-complete", len(cases) == 8 and set(cases_by_label) == expected)
    check.require("cases:counts", manifest.get("case_counts") == {"original": 2, "jadx": 4, "jarde": 2})
    check.require("cases:only-recorded-preparation-flag-failure", manifest.get("failures") == ["javac8-original"]
                  and manifest.get("success_counts") == {"original": 1, "jadx": 4, "jarde": 2})
    check.require("jarde:eight-complete-class-renders", manifest.get("jarde_cli_render_count") == 8
                  and manifest.get("jarde_cli_render_count_expected") == 8)
    original_javap: dict[tuple[str, str], str] = {}
    for jdk in ("javac8", "javac23"):
        original_case = cases_by_label.get(f"{jdk}-original", {})
        for class_name in EXPECTED:
            javap_row = next((row for row in original_case.get("javap", [])
                              if row.get("class") == class_name), None)
            if javap_row:
                raw_path = OUT / javap_row["raw_javap"]["path"]
                if raw_path.is_file() and not raw_path.is_symlink():
                    original_javap[(jdk, class_name)] = raw_path.read_text(encoding="utf-8", errors="replace")
    originals = {}
    for label, case in cases_by_label.items():
        check.require(f"case:{label}:compile-run", case.get("compile_success") is True and case.get("runtime_success") is True)
        check.require(f"case:{label}:success-boundary",
                      case.get("success") is (label != "javac8-original"))
        bind_command(check, case.get("compile"), by_label, f"{label}:compile")
        bind_command(check, case.get("runtime"), by_label, f"{label}:run")
        triple = runtime_triple(check, case)
        if case.get("kind") == "original":
            originals[case["jdk_leg"]] = triple
        elif triple is not None:
            check.require(f"case:{label}:same-jdk-raw", triple == originals.get(case["jdk_leg"]))
        check.require(f"case:{label}:complete-source-class-lists",
                      len(case.get("source_files", [])) == 3 and len(case.get("classes", [])) == 3)
        for kind in ("source_files", "classes"):
            for row in case.get(kind, []):
                data = check_record(check, OUT, row, f"case:{label}:{kind}:{row.get('path')}")
                if kind == "classes" and data is not None:
                    check_classfile(check, case, row, data)
        class_root = Path(case.get("class_output", ""))
        recorded_classes = {row["path"] for row in case.get("classes", [])}
        actual_classes = {path.relative_to(OUT).as_posix() for path in class_root.rglob("*.class")
                          if path.is_file() and not path.is_symlink()}
        check.require(f"case:{label}:disk-class-census", actual_classes == recorded_classes,
                      {"actual": sorted(actual_classes), "recorded": sorted(recorded_classes)})
        check.require(f"case:{label}:class-output-census", case.get("class_output_set", {}).get("complete") is True
                      and case.get("produced_class_names", ["ArrayFieldInitBase", "CommonNoClinitArrayInit", "Runner"])
                      == ["ArrayFieldInitBase", "CommonNoClinitArrayInit", "Runner"])
        empty = Path(case.get("empty_classpath_sourcepath", ""))
        check.require(f"case:{label}:empty-cp-sp", empty.is_dir() and not empty.is_symlink() and not any(empty.iterdir()))
        argv = case.get("compile", {}).get("argv", [])
        empty_path = str(empty)
        class_path = case.get("class_output")
        source_paths = [str((OUT / row["path"]).resolve()) for row in case.get("source_files", [])]
        check.require(f"case:{label}:javac-exact-mode", len(argv) >= 14
                      and argv[0] == jdk_legs[case["jdk_leg"]]["jdk_tools"]["javac"]["path"]
                      and argv[1:8] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options", "-classpath"]
                      and argv[8] == empty_path and argv[9:11] == ["-sourcepath", empty_path]
                      and argv[11:13] == ["-d", class_path] and argv[13:] == source_paths)
        run_argv = case.get("runtime", {}).get("argv", [])
        expected_runner = case.get("runner_class", "Runner")
        check.require(f"case:{label}:jvm-exact-mode", run_argv == [
            jdk_legs[case["jdk_leg"]]["jdk_tools"]["java"]["path"], "-Xverify:all", "-cp", class_path, expected_runner])
        check.require(f"case:{label}:java-home-binding",
                      case.get("compile", {}).get("java_home") == jdk_legs[case["jdk_leg"]]["jdk_tools"]["java"]["path"].rsplit("/bin/", 1)[0]
                      and case.get("runtime", {}).get("java_home") == case.get("compile", {}).get("java_home"))
        if case.get("kind") == "original":
            for name, expected_hash in SOURCE_SHA256.items():
                row = next((item for item in case["source_files"] if Path(item["path"]).name == name), None)
                check.require(f"case:{label}:frozen-source:{name}", row is not None and row.get("sha256") == expected_hash)
            verify_javap(check, case, by_label)
        elif case.get("kind") == "jadx":
            dec = case.get("decompilation", {})
            bind_command(check, dec.get("decompile"), by_label, f"{label}:jadx")
            check.require(f"case:{label}:jadx-all-sources", dec.get("profile") == case.get("profile")
                          and dec.get("decompile_success") is True
                          and dec.get("source_name_set_complete") is True and len(dec.get("generated_sources", [])) == 2
                          and {Path(row["path"]).stem for row in dec.get("generated_sources", [])} == set(EXPECTED))
            decompile_argv = dec.get("decompile", {}).get("argv", [])
            expected_jadx_argv = [manifest["jadx"]["version_command"]["argv"][0], "--no-res", "--config",
                                  "none", "--threads-count", "1"]
            if case["profile"] == "none":
                expected_jadx_argv.extend(["--rename-flags", "none"])
            expected_jadx_argv.extend(["-d", str((OUT / f"jadx/{case['profile']}").resolve()),
                                       str((OUT / manifest["jadx"]["jar"]["path"]).resolve())])
            check.require(f"case:{label}:jadx-exact-command", decompile_argv == expected_jadx_argv)
            check.require(f"case:{label}:jadx-same-frozen-jar",
                          dec.get("input_jar") == manifest["jadx"].get("jar")
                          and dec.get("jar_members") == manifest["jadx"].get("jar_members"))
            for row in dec.get("generated_sources", []):
                check_record(check, OUT, row, f"case:{label}:jadx-source:{row['path']}")
            copied = {Path(row["path"]).stem: row for row in case.get("source_files", [])}
            generated = {Path(row["path"]).stem: row for row in dec.get("generated_sources", [])}
            check.require(f"case:{label}:jadx-source-copy-identity", all(
                copied.get(name, {}).get("bytes") == generated[name].get("bytes")
                and copied.get(name, {}).get("sha256") == generated[name].get("sha256")
                for name in EXPECTED))
            runner_copy = case.get("runner_adaptation", {})
            check_record(check, OUT, runner_copy, f"case:{label}:runner-adaptation")
            check.require(f"case:{label}:runner-source", copied.get("Runner", {}).get("sha256") == runner_copy.get("sha256"))
            expected_package = "defpackage" if case.get("profile") == "default" else None
            check.require(f"case:{label}:runner-package", case.get("runner_class") == ((expected_package + ".") if expected_package else "") + "Runner")
        elif case.get("kind") == "jarde":
            check.require(f"case:{label}:render-count", case.get("cli_render_count") == 4)
            rendered = {row.get("class"): row for row in case.get("rendered_classes", [])}
            check.require(f"case:{label}:render-targets", set(rendered) == set(EXPECTED))
            for class_name, item in rendered.items():
                original_class = item.get("original_class", {})
                check_record(check, OUT, original_class, f"case:{label}:original-class:{class_name}")
                profiles = {row.get("profile"): row for row in item.get("profiles", [])}
                check.require(f"case:{label}:{class_name}:profiles", set(profiles) == {"default", "all"})
                profile_sources = {}
                for profile, row in profiles.items():
                    bind_command(check, row.get("command"), by_label, f"{label}:{class_name}:{profile}")
                    command_argv = row.get("command", {}).get("argv", [])
                    flag_values = {}
                    for flag in ("--input", "--class", "--policy", "--release", "--format", "--evidence"):
                        if flag in command_argv and command_argv.index(flag) + 1 < len(command_argv):
                            flag_values[flag] = command_argv[command_argv.index(flag) + 1]
                    expected_argv = [manifest["frozen_cli"]["path"], "class-source", "--input",
                                     str((OUT / original_class["path"]).resolve()), "--class", class_name,
                                     "--policy", "single-class", "--release", "8", "--format", "json"]
                    if profile == "all":
                        expected_argv.extend(["--evidence", "all"])
                    check.require(f"case:{label}:{class_name}:{profile}:cli-mode",
                                  command_argv == expected_argv
                                  and flag_values.get("--input") == str((OUT / original_class["path"]).resolve())
                                  and flag_values.get("--class") == class_name
                                  and flag_values.get("--policy") == "single-class"
                                  and flag_values.get("--release") == "8"
                                  and flag_values.get("--format") == "json"
                                  and ((flag_values.get("--evidence") == "all") if profile == "all"
                                       else "--evidence" not in command_argv))
                    raw_json = check_record(check, OUT, row["class_source_json"], f"case:{label}:{class_name}:{profile}:json")
                    source = check_record(check, OUT, row["generated_source"], f"case:{label}:{class_name}:{profile}:source")
                    if raw_json is None or source is None:
                        continue
                    command_label = row["command"]["label"]
                    command_stdout = check_record(check, OUT, by_label[command_label]["stdout"],
                                                  f"case:{label}:{class_name}:{profile}:cli-stdout")
                    check.require(f"case:{label}:{class_name}:{profile}:raw-json-is-command-output",
                                  command_stdout == raw_json)
                    doc = json.loads(raw_json)
                    check.require(f"case:{label}:{class_name}:{profile}:json-source-match",
                                  doc.get("text", "").encode("utf-8") == source
                                  and sha(source) == row.get("source_text_sha256"))
                    profile_sources[profile] = source
                    inspect_cli_doc(check, doc, class_name, profile, row,
                                    original_javap.get((case["jdk_leg"], class_name)))
                check.require(f"case:{label}:{class_name}:default-all-equal",
                              profile_sources.get("default") == profile_sources.get("all"))
                check.require(f"case:{label}:{class_name}:physical-inventory", item.get("physical_inventory_matches") is True)
                all_row = profiles.get("all", {}).get("generated_source")
                if all_row:
                    text = (OUT / all_row["path"]).read_text(encoding="utf-8")
                    if class_name == "ArrayFieldInitBase":
                        fragments = ["super();", "this.received = arg1;"]
                        expected_fragment_counts = [1, 1]
                    else:
                        fragments = ["super(7);", "this.first = new byte[]{mark(11), mark(12)};",
                                     "this.second = new byte[]{run(21)};",
                                     "CommonNoClinitArrayInit.trace = CommonNoClinitArrayInit.trace * 31 + 91;",
                                     "super(arg1);",
                                     "CommonNoClinitArrayInit.trace = CommonNoClinitArrayInit.trace * 31 + arg1;"]
                        expected_fragment_counts = [1, 2, 2, 1, 1, 1]
                    positions = [text.find(fragment) for fragment in fragments]
                    counts = [text.count(fragment) for fragment in fragments]
                    check.require(f"case:{label}:{class_name}:source-write-order",
                                  all(position >= 0 for position in positions) and positions == sorted(positions)
                                  and counts == expected_fragment_counts)
                    check.require(f"case:{label}:{class_name}:source-has-no-static-block",
                                  not re.search(r"(?m)^\s*static\s*\{", text))
            rendered_source_rows = {Path(row["path"]).name: row for row in case.get("rendered_sources", [])}
            compile_source_rows = {Path(row["path"]).name: row for row in case.get("source_files", [])}
            check.require(f"case:{label}:all-rendered-sources-compiled", all(
                name in compile_source_rows
                and compile_source_rows[name].get("sha256") == row.get("sha256")
                and compile_source_rows[name].get("bytes") == row.get("bytes")
                for name, row in rendered_source_rows.items())
                and set(rendered_source_rows) == {"ArrayFieldInitBase.java", "CommonNoClinitArrayInit.java"})
            runner_copy = case.get("runner_adaptation", {})
            check_record(check, OUT, runner_copy, f"case:{label}:runner-adaptation")
            source_runner = next((row for row in case.get("source_files", [])
                                  if Path(row["path"]).name == "Runner.java"), {})
            check.require(f"case:{label}:runner-source", source_runner.get("sha256") == runner_copy.get("sha256")
                          and runner_copy.get("sha256") == SOURCE_SHA256["Runner.java"]
                          and case.get("runner_class") == "Runner")
    check.require("oracle:original-runner-output", all(value == (0, EXPECTED_OUTPUT, b"") for value in originals.values()))
    check.require("oracle:two-jdk-raw-equality", originals.get("javac8") == originals.get("javac23"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=OUT / "manifest.json")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit(f"refusing to overwrite {args.output}")
    manifest_path = args.manifest.resolve(strict=True)
    manifest = read_json(manifest_path)
    check = Check()
    check.require("manifest-schema", manifest.get("schema") == "no-clinit-super-args-baseline-v1")
    check.require("manifest-status-records-jdk8-preparation-check-failure",
                  manifest.get("status") == "baseline-with-failures")
    check.require("manifest-path", manifest_path.parent == OUT.resolve())
    close_inventory(check, OUT, manifest)
    commands = verify_commands(check, manifest)
    jdk_legs = verify_metadata(check, manifest, commands)
    verify_cases(check, manifest, commands, jdk_legs)
    result = check.finish()
    result.update({
        "schema": "no-clinit-super-args-independent-verification-v1",
        "manifest": str(manifest_path),
        "diagnosis": "javac8-original was marked unsuccessful because Corretto/JDK 8 javap omits the `interfaces: N, fields: N, methods: N` class-header line required by the preparation script. Its javap processes exited 0. Parsing each declaration with its following descriptor independently recovers the expected physical fields and methods; the raw code and runtime legs are preserved for verification.",
        "claim_boundary": "Verifier prepared only; this agent did not execute it.",
    })
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: result[key] for key in ("success", "passed", "failed", "manifest", "diagnosis")},
                     ensure_ascii=False))
    return 0 if result["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
