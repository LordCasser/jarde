#!/usr/bin/env python3
"""Independent verifier for the focused TestArrays2 primitive-array fixture.

This verifies the archived baseline evidence. It does not run javac, Java,
JADX, or Jarde, and does not claim completion of the wider EM18 audit.
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path
from textwrap import dedent


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/primitive-array-branches"
BASE = EVIDENCE / "baseline-root-v1"
MANIFEST_PATH = BASE / "manifest.json"
INVENTORY_PATH = BASE / "file-inventory.json"
OUTPUT = EVIDENCE / "results/baseline-verification-root-v3.json"
METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
UPSTREAM_CURRENT = Path("/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrays2.java")

EXPECTED = {
    "cli": ("/private/tmp/jarde-nonfinal-static-cli-v1", "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"),
    "metadata": "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a",
    "jdk_manifest": "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec",
    "upstream": "8f3f006efc74144fedb359b3830a78c131d4d8ac0a9ef095c4d05fe6b302664d",
    "adapter": "504986e4a62260aa3b9fec5721e6d29631bde2de8b260af340999ab1b37ccbb5",
    "runner": "fbacb4d827fe517da19c5fcb8b1c6972b7e8cc9ba29aa419a4c2e50e044ab8a4",
    "jadx": "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7",
    "jar": "648b903e386993d2ea4bb8173ca517addee464d907b440eb5a37a02ac7fee6d8",
}
EXPECTED_CASES = (
    "javac8-original", "javac23-original",
    "javac8-jadx-default", "javac23-jadx-default",
    "javac8-jadx-none", "javac23-jadx-none",
    "javac8-jarde", "javac23-jarde",
)
EMPTY_SHA = hashlib.sha256(b"").hexdigest()
INSTRUCTIONS = {
    6: "newarray int", 11: "iastore", 15: "iastore", 16: "areturn",
    23: "newarray float", 28: "fastore", 32: "fastore", 33: "areturn",
    40: "newarray short", 45: "sastore", 49: "sastore", 50: "areturn",
    57: "newarray byte", 62: "bastore", 66: "bastore", 67: "areturn",
    69: "areturn",
}


def fail(message: str) -> None:
    raise ValueError(message)


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def bytes_at(rel: str) -> bytes:
    path = BASE / rel
    require(path.is_file(), f"missing evidence file: {rel}")
    return path.read_bytes()


def check_record(record: dict, *, root: Path = BASE) -> bytes:
    path = root / record["path"]
    require(path.is_file(), f"missing recorded file: {path}")
    data = path.read_bytes()
    require(len(data) == record["bytes"], f"byte length mismatch: {path}")
    require(sha(data) == record["sha256"], f"SHA-256 mismatch: {path}")
    return data


def verify_inventory(manifest: dict) -> None:
    inv_ref = manifest["file_inventory"]
    inv_path = BASE / inv_ref["path"]
    inv = load_json(inv_path)
    files = inv
    require(isinstance(files, list), "inventory must be an array")
    expected = {row["path"]: (row["bytes"], row["sha256"]) for row in files}
    require(len(expected) == len(files), "duplicate inventory path")
    require("manifest.json" in expected, "inventory omits manifest")
    actual_paths = set()
    for p in BASE.rglob("*"):
        if p.is_file() and p != inv_path:
            actual_paths.add(p.relative_to(BASE).as_posix())
    require(actual_paths == set(expected), "baseline file set differs from closed inventory")
    for rel, (size, digest) in expected.items():
        data = bytes_at(rel)
        require(len(data) == size and sha(data) == digest, f"inventory hash mismatch: {rel}")
    require(inv_ref.get("path") == "file-inventory.json", "unexpected inventory path")
    require("file-inventory.json" in inv_ref.get("excludes", []), "inventory exclusion not explicit")
    require("manifest.json" in inv_ref.get("includes", []), "manifest inclusion not explicit")


def verify_raw(command: dict) -> tuple[bytes, bytes]:
    require(command.get("exit") == 0, f"nonzero command: {command.get('label')}")
    outputs = []
    for stream in ("stdout", "stderr"):
        rec = command[stream]
        data = bytes_at(rec["path"])
        require(len(data) == rec["bytes"], f"{command['label']} {stream} length mismatch")
        require(sha(data) == rec["sha256"], f"{command['label']} {stream} hash mismatch")
        outputs.append(data)
    return outputs[0], outputs[1]


def byte_string(values: list[int]) -> str:
    return bytes(values).decode("utf-8")


def parse_javap(text: str) -> dict:
    """Read physical method signatures, flags, and instruction BCI/opcodes."""
    lines = text.splitlines()
    methods = {}
    current = None
    for i, line in enumerate(lines):
        m = re.match(r"\s{2}(public|private|protected)\s+(.+);\s*$", line)
        if not m:
            continue
        declaration = m.group(2)
        source_name = declaration.split("(", 1)[0].split()[-1]
        name = "<init>" if source_name == "PrimitiveArrayBranches" else source_name
        desc = next((x.split("descriptor:", 1)[1].strip() for x in lines[i + 1:i + 8] if "descriptor:" in x), None)
        flags_line = next((x for x in lines[i + 1:i + 8] if "flags:" in x), "")
        require(desc is not None, f"javap descriptor absent for {name}")
        methods[(name, desc)] = {"flags": flags_line, "instructions": {}}
        current = (name, desc)
        # Method Code starts after the descriptor/flags; instruction rows carry explicit BCI.
        for row in lines[i + 1:]:
            if re.match(r"\s{2}(public|private|protected)\s+.+;\s*$", row):
                break
            ins = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)\s*(.*?)\s*$", row)
            if ins:
                methods[current]["instructions"][int(ins.group(1))] = (ins.group(2), ins.group(3))
    return methods


def normalized_method(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    # Normalize each line because the extracted source sits at different
    # nesting depths and the sliced first line has already lost its indent.
    return "\n".join(line.strip() for line in source[start:end].strip().splitlines())


def verify_source_identity(manifest: dict) -> None:
    upstream_snap = bytes_at(manifest["upstream"]["snapshot"]["path"])
    adapter = bytes_at(manifest["source"]["path"])
    runner = bytes_at(manifest["runner"]["path"])
    require(sha(upstream_snap) == EXPECTED["upstream"], "upstream snapshot hash mismatch")
    require(sha(adapter) == EXPECTED["adapter"], "adapter hash mismatch")
    require(sha(runner) == EXPECTED["runner"], "runner hash mismatch")
    require(sha(UPSTREAM_CURRENT.read_bytes()) == EXPECTED["upstream"], "current upstream source identity changed")
    original_method = normalized_method(
        upstream_snap.decode(), "private static Object test4(int type) {", "public void check() {"
    )
    adapter_method = normalized_method(
        adapter.decode(), "private static Object test4(int type) {", "public static Object choose(int type) {"
    )
    require(original_method == adapter_method, "adapted test4 body differs beyond indentation/context")
    wrapper = normalized_method(adapter.decode(), "public static Object choose(int type) {", "\n}")
    require("return test4(type);" in wrapper, "adapter wrapper mismatch")
    for value in ("int[]", "float[]", "short[]", "byte[]", "return null"):
        require(value in adapter.decode(), f"adapter is missing expected source branch: {value}")


def verify_metadata(manifest: dict) -> dict:
    require(Path(manifest["cli_metadata"]["path"]) == METADATA, "metadata path mismatch")
    md_bytes = METADATA.read_bytes()
    require(sha(md_bytes) == EXPECTED["metadata"], "metadata SHA mismatch")
    md = json.loads(md_bytes)
    cli_path, cli_hash = EXPECTED["cli"]
    require(md["cli_path"] == cli_path and md["cli_sha256"] == cli_hash, "frozen CLI metadata mismatch")
    require(manifest["frozen_cli"] == {"path": cli_path, "sha256": cli_hash}, "manifest CLI pin mismatch")
    require(sha(Path(cli_path).read_bytes()) == cli_hash, "frozen CLI binary hash mismatch")
    require(manifest["jdk_manifest"]["path"] == str(JDK_MANIFEST), "JDK manifest path mismatch")
    require(sha(JDK_MANIFEST.read_bytes()) == EXPECTED["jdk_manifest"], "JDK manifest hash mismatch")
    jdk = load_json(JDK_MANIFEST)
    require(jdk.get("status") == "complete", "pinned JDK baseline is incomplete")
    tool_manifest_ref = jdk["toolchain_manifest"]
    tool_manifest_path = Path(tool_manifest_ref["path"])
    require(sha(tool_manifest_path.read_bytes()) == tool_manifest_ref["sha256"], "JDK executable manifest hash mismatch")
    tool_manifest = load_json(tool_manifest_path)
    pinned_tools = {}
    for leg in tool_manifest["legs"]:
        for tool_name, tool in leg["tools"].items():
            pinned_tools[tool["path"]] = (tool["bytes"], tool["sha256"])
    for path, (size, digest) in pinned_tools.items():
        data = Path(path).read_bytes()
        require(len(data) == size and sha(data) == digest, f"actual JDK tool binary mismatch: {path}")
    return md, pinned_tools


def verify_commands(manifest: dict) -> dict[str, dict]:
    commands = manifest["commands"]
    by_label = {c["label"]: c for c in commands}
    require(len(by_label) == len(commands) == 25, "command set count/uniqueness mismatch")
    for c in commands:
        verify_raw(c)
    require(by_label["jadx-version"]["argv"] == ["/opt/homebrew/bin/jadx", "--version"], "JADX version argv mismatch")
    out, _ = verify_raw(by_label["jadx-version"])
    require(out.strip() == b"1.5.6", "JADX version output mismatch")
    return by_label


def verify_compile_and_runtime(case: dict, original: dict, package: str | None = None) -> None:
    label = case["label"]
    compile_cmd = case["compile"]
    run_cmd = case["runtime"]
    argv = compile_cmd["argv"]
    require("-source" in argv and argv[argv.index("-source") + 1] == "8", f"{label}: source level")
    require("-target" in argv and argv[argv.index("-target") + 1] == "8", f"{label}: target level")
    require("-g:none" in argv, f"{label}: debug info not disabled")
    require("-classpath" in argv and "-sourcepath" in argv, f"{label}: explicit empty paths absent")
    cp = argv[argv.index("-classpath") + 1]
    sp = argv[argv.index("-sourcepath") + 1]
    require(cp == sp == case["empty_classpath_sourcepath"], f"{label}: class/source path mismatch")
    require(Path(cp).is_dir() and not any(Path(cp).iterdir()), f"{label}: class/source path not empty")
    require("-d" in argv and argv[argv.index("-d") + 1] == case["class_output"], f"{label}: class output mismatch")
    require(run_cmd["argv"][1:3] == ["-Xverify:all", "-cp"], f"{label}: runtime verifier/classpath flags")
    require(run_cmd["argv"][3] == case["class_output"], f"{label}: runtime uses unexpected classpath")
    require(run_cmd["argv"][-1] == (package + "." if package else "") + "PrimitiveArrayBranchesRunner", f"{label}: runner main class")
    require(case["success"] and case["compile_success"] and case["runtime_success"], f"{label}: success flags false")
    out, err = verify_raw(run_cmd)
    orig_out, orig_err = verify_raw(original["runtime"])
    require((run_cmd["exit"], out, err) == (original["runtime"]["exit"], orig_out, orig_err), f"{label}: raw runtime differs from original")
    srcs = case["source_files"]
    require(len(srcs) == 2, f"{label}: must compile complete target and Runner source set")
    source_args = [x for x in argv if x.endswith(".java")]
    require(set(source_args) == {str(BASE / row["path"]) for row in srcs}, f"{label}: compile argv omits/changes source files")
    for row in srcs:
        check_record(row)
    for row in case["classes"]:
        check_record(row)
    require(case["complete_class_set"], f"{label}: incomplete class set")


def verify_javap(original_cases: list[dict], commands: dict[str, dict]) -> dict[str, dict]:
    expected_sigs = {("<init>", "()V"), ("test4", "(I)Ljava/lang/Object;"), ("choose", "(I)Ljava/lang/Object;")}
    parsed_by_jdk = {}
    for case in original_cases:
        jdk = case["jdk_leg"]
        target_rows = [x for x in case["javap"] if x["class"] == "PrimitiveArrayBranches"]
        require(len(target_rows) == 1, f"{case['label']}: missing target javap")
        row = target_rows[0]
        raw, _ = verify_raw(row["command"])
        parsed = parse_javap(raw.decode())
        require(set(parsed) == expected_sigs, f"{case['label']}: unexpected physical method signature set {set(parsed)}")
        # The class member area before the first method declaration must contain no fields.
        text = raw.decode()
        class_open = re.search(r"(?m)^.*\bclass PrimitiveArrayBranches.*\{\s*$", text)
        first_method = re.search(r"(?m)^\s{2}(?:public|private|protected)\s+.+;\s*$", text)
        require(class_open is not None and first_method is not None, f"{case['label']}: javap class/member area missing")
        member_area = text[class_open.end():first_method.start()]
        require(not re.search(r"(?m)^\s{2}.+;\s*$", member_area), f"{case['label']}: unexpected field")
        flags = {sig: item["flags"] for sig, item in parsed.items()}
        require("ACC_PUBLIC" in flags[("<init>", "()V")], f"{case['label']}: ctor not public")
        require("ACC_PRIVATE" in flags[("test4", "(I)Ljava/lang/Object;")] and "ACC_STATIC" in flags[("test4", "(I)Ljava/lang/Object;")], f"{case['label']}: test4 flags")
        require("ACC_PUBLIC" in flags[("choose", "(I)Ljava/lang/Object;")] and "ACC_STATIC" in flags[("choose", "(I)Ljava/lang/Object;")], f"{case['label']}: choose flags")
        test4 = parsed[("test4", "(I)Ljava/lang/Object;")]["instructions"]
        for bci, expected in INSTRUCTIONS.items():
            require(bci in test4, f"{case['label']}: missing test4 BCI {bci}")
            opcode, operand = test4[bci]
            if expected.startswith("newarray"):
                require(opcode == "newarray" and expected.split()[1] in operand, f"{case['label']}: BCI {bci} expected {expected}, got {opcode} {operand}")
            else:
                require(opcode == expected, f"{case['label']}: BCI {bci} expected {expected}, got {opcode}")
        parsed_by_jdk[jdk] = parsed
    return parsed_by_jdk


def verify_jarde(case: dict, original: dict, physical: dict) -> None:
    label = case["label"]
    doc_bytes = check_record(case["class_source_json"])
    doc = json.loads(doc_bytes)
    text = check_record(case["class_source_text"])
    source = check_record(case["generated_source"])
    require(text == source == doc["text"].encode(), f"{label}: emitted source is not exact JSON text")
    require(doc["fields"] == [] and case["field_count"] == 0, f"{label}: unexpected fields")
    methods = doc["methods"]
    require(len(methods) == case["member_count"] == 3, f"{label}: method/member count")
    identity_map = {}
    owner = doc["class"]
    for method in methods:
        item = method["item"]
        ident = item["identity"]
        require(ident["owner"] == owner, f"{label}: physical method owner identity mismatch")
        name = byte_string(ident["name"])
        desc = byte_string(ident["descriptor"])
        require(name == item["name"]["escaped"] and desc == item["descriptor"]["escaped"], f"{label}: raw method identity inconsistent")
        expected_flags = {("<init>", "()V"): 0x0001, ("test4", "(I)Ljava/lang/Object;"): 0x000A, ("choose", "(I)Ljava/lang/Object;"): 0x0009}
        require(item["access_flags"] == expected_flags.get((name, desc)), f"{label}: CLI method flags differ from expected physical flags")
        identity_map[(name, desc)] = method
    require(set(identity_map) == set(physical), f"{label}: CLI method identities differ from javap")
    # BLAKE3 class digest is intentionally not compared with a SHA-256 value.
    target_class = BASE / case["original_class"]["path"]
    target_data = target_class.read_bytes()
    require(len(target_data) == case["original_class"]["bytes"] and sha(target_data) == case["original_class"]["sha256"], f"{label}: original input class hash/length mismatch")
    require(doc["class"]["class_bytes"]["length"] == target_class.stat().st_size, f"{label}: class byte length not bound to input")
    require(doc["class"]["location"]["snapshot"] == doc["class"]["class_bytes"]["digest"], f"{label}: class snapshot identity inconsistent")
    for sig, parsed_method in physical.items():
        method = identity_map[sig]
        segs = method["outcome"]["report"]["source_map"]["segments"]
        anchors = set()
        for seg in segs:
            for key in ("primary", "derived"):
                origin = seg.get("origin", {}).get(key)
                if origin and origin.get("method") == method["item"]["identity"] and isinstance(origin.get("bci"), int):
                    anchors.add(origin["bci"])
        expected_bcis = set(parsed_method["instructions"])
        require(expected_bcis <= anchors, f"{label}: source-map misses physical {sig} BCIs {sorted(expected_bcis - anchors)}")
    require(case["render"]["argv"][0] == EXPECTED["cli"][0], f"{label}: render CLI path")
    argv = case["render"]["argv"]
    require(argv[1:3] == ["class-source", "--input"], f"{label}: CLI subcommand/input ordering")
    require(argv[3] == str(target_class), f"{label}: CLI input not original class")
    require(argv[4:] == ["--class", "PrimitiveArrayBranches", "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all"], f"{label}: class-source options")
    verify_compile_and_runtime(case, original)
    runner = check_record(case["runner_adaptation"])
    original_runner = bytes_at("original-sources/PrimitiveArrayBranchesRunner.java")
    require(runner == original_runner, f"{label}: Runner adaptation changed")


def main() -> int:
    require(not OUTPUT.exists(), f"refusing to overwrite {OUTPUT}")
    manifest = load_json(MANIFEST_PATH)
    require(manifest.get("schema") == "primitive-array-branches-baseline-root-v1", "unexpected baseline schema")
    require(manifest.get("status") == "completed", "baseline is not complete")
    require(manifest.get("claim_boundary") == "Focused adaptation of TestArrays2.test4(int) with JUnit removed and a public choose(int) wrapper added. This does not represent the full TestArrays2 test class or the full EM18 suite.", "claim boundary must explicitly exclude whole EM18")
    verify_inventory(manifest)
    verify_source_identity(manifest)
    metadata, pinned_tools = verify_metadata(manifest)
    commands = verify_commands(manifest)
    for command in manifest["commands"]:
        exe = command["argv"][0]
        if Path(exe).name in {"java", "javac", "javap"}:
            require(exe in pinned_tools, f"command uses an unpinned JDK executable: {exe}")
    cases = {c["label"]: c for c in manifest["cases"]}
    require(tuple(cases) == EXPECTED_CASES, "case order/set mismatch")
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 2}, "case counts mismatch")
    require(manifest["success_counts"] == manifest["case_counts"], "not all cases passed")
    original_cases = [cases["javac8-original"], cases["javac23-original"]]
    for original in original_cases:
        verify_compile_and_runtime(original, original)
        target_source = next(row for row in original["source_files"] if row["path"].endswith("PrimitiveArrayBranches.java"))
        runner_source = next(row for row in original["source_files"] if row["path"].endswith("PrimitiveArrayBranchesRunner.java"))
        require(check_record(target_source) == bytes_at(manifest["source"]["path"]), f"{original['label']}: source differs from pinned adapter")
        require(check_record(runner_source) == bytes_at(manifest["runner"]["path"]), f"{original['label']}: Runner differs from pinned harness")
    j8_out, j8_err = verify_raw(cases["javac8-original"]["runtime"])
    j23_out, j23_err = verify_raw(cases["javac23-original"]["runtime"])
    require((j8_out, j8_err) == (j23_out, j23_err), "original cross-JDK raw streams differ")
    physical = verify_javap(original_cases, commands)
    # Both JDK original bytecode layouts must retain the same verified branch instructions.
    require(set(physical["javac8"]) == set(physical["javac23"]), "JDK physical method identities differ")
    for sig in physical["javac8"]:
        require(physical["javac8"][sig]["instructions"] == physical["javac23"][sig]["instructions"], f"JDK physical instruction layout differs for {sig}")

    for label in EXPECTED_CASES[2:6]:
        case = cases[label]
        original = cases["javac8-original" if case["jdk_leg"] == "javac8" else "javac23-original"]
        profile = "default" if case["label"].endswith("default") else "none"
        pkg = "defpackage" if profile == "default" else None
        verify_compile_and_runtime(case, original, pkg)
        profile_record = next(p for p in manifest["jadx"]["profiles"] if p["profile"] == profile)
        require(profile_record["generated_source_count"] == 1 and profile_record["decompile_success"], f"JADX {profile} extraction incomplete")
        check_record(profile_record["input_jar"])
        require(len(profile_record["jar_members"]) == 1 and profile_record["jar_members"][0]["name"] == "PrimitiveArrayBranches.class", "JADX input jar membership")
        jar_bytes = check_record(profile_record["input_jar"])
        with zipfile.ZipFile(BASE / profile_record["input_jar"]["path"]) as jar:
            names = [n for n in jar.namelist() if not n.endswith("/")]
            require(names == ["PrimitiveArrayBranches.class"], "JADX jar must contain exactly one class")
            member = jar.read(names[0])
        orig_target = BASE / cases["javac23-original"]["classes"][0]["path"]
        require(member == orig_target.read_bytes(), "JADX jar class differs from fresh javac23 original")
        generated = profile_record["generated_sources"][0]
        check_record(generated)
        generated_path = BASE / generated["path"]
        compiled_target = [r for r in case["classes"] if r["path"].endswith("PrimitiveArrayBranches.class")][0]
        require((BASE / compiled_target["path"]).is_file(), f"{label}: target class missing")
        expected_source = next(r for r in case["source_files"] if r["path"].endswith("PrimitiveArrayBranches.java"))
        require((BASE / expected_source["path"]).read_bytes() == generated_path.read_bytes(), f"{label}: compiled source differs from archived extraction")
        runner_source = next(r for r in case["source_files"] if r["path"].endswith("PrimitiveArrayBranchesRunner.java"))
        runner_bytes = check_record(runner_source)
        original_runner = bytes_at("original-sources/PrimitiveArrayBranchesRunner.java")
        expected_runner = (b"package defpackage;\n\n" + original_runner) if profile == "default" else original_runner
        require(runner_bytes == expected_runner, f"{label}: Runner package adaptation is not exact")

    jarde_cases = [cases["javac8-jarde"], cases["javac23-jarde"]]
    for case in jarde_cases:
        original = cases["javac8-original" if case["jdk_leg"] == "javac8" else "javac23-original"]
        verify_jarde(case, original, physical[case["jdk_leg"]])

    # Pin executable and archive provenance from the manifest, including the actual target bytes.
    jadx = manifest["jadx"]
    require(jadx["version"] == "1.5.6" and jadx["executable_sha256"] == EXPECTED["jadx"], "JADX executable/version pin")
    require(sha(Path(jadx["executable_path"]).read_bytes()) == EXPECTED["jadx"], "actual JADX executable hash mismatch")
    require(jadx["input_jar"]["sha256"] == EXPECTED["jar"], "JADX jar hash pin")
    require(len(metadata["candidate_sources"]) == 10 and len(metadata["test_sources"]) == 4, "frozen product/test source set changed")
    for category in ("candidate_sources", "test_sources"):
        for path, digest in metadata[category].items():
            require(sha((ROOT / path).read_bytes()) == digest, f"current {category} identity drift: {path}")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    result = {
        "schema": "primitive-array-branches-baseline-verification-root-v3",
        "status": "verified",
        "scope": "focused TestArrays2.test4 primitive-array branch adaptation only",
        "whole_em18_complete": False,
        "manifest_sha256": sha(MANIFEST_PATH.read_bytes()),
        "inventory_files": len(load_json(INVENTORY_PATH)),
        "verified_cases": list(EXPECTED_CASES),
        "verified_commands": len(commands),
        "verified_jdk_legs": ["javac8", "javac23"],
        "physical_methods_per_class": 3,
        "physical_fields": 0,
        "source_map_bci_coverage": "all physical method BCIs for both Jarde legs",
        "raw_runtime_equal": True,
        "cli_sha256": EXPECTED["cli"][1],
        "metadata_sha256": EXPECTED["metadata"],
        "jadx_version": "1.5.6",
    }
    OUTPUT.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"verification failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
