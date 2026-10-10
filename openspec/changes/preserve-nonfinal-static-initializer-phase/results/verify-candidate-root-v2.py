#!/usr/bin/env python3
"""Independently verify the four frozen nonfinal static-initializer candidate legs."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
OUT = HERE / "candidate-luna-v1"
RESULT = HERE / "candidate-root-verification-v1.json"
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
CLI_METADATA = HERE / "candidate-cli-v1.json"
CLI_PATH = Path("/private/tmp/jarde-nonfinal-static-cli-v1")
PRODUCT_PINS = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
EXPECTED = {
    "literal": {
        "directory": "array-field-initializers-literal", "class": "ArrayFieldLiteral",
        "static_fields": ["a"],
        "required_source": ["static byte[] a = new byte[]{10, 20, 30};"],
        "source_sha256": "184d4d068f785c75d5db8a30579284ed42817eaf91b012e55681581ab145a67b",
        "runner_sha256": "fd69838bf2a372d1bceb27b83fffc3b24b23a2ba1967bb51038e4473b1d4e163",
        "manifest_sha256": "a021eedcde975dcb7bf7b7821d5097141b93292181015a45415752b21ddbafe1",
        "class_sha256": {"javac8": "6ead8bb0f3c466ba4dc10958a4cb349699f587bc9d5115f0656d0e2df60aa891",
                         "javac23": "dc388f1ed9c0a44c9b87f9d61057b7a3f6c09fa1387b50d8f992a049e8544827"},
        "fields": {("a", "[B", 8), ("b", "[B", 0)},
        "methods": {("<init>", "()V", 1), ("<clinit>", "()V", 8)},
        "opcodes": {"<init>": {"invokespecial", "newarray", "bastore", "putfield"},
                    "<clinit>": {"newarray", "bastore", "putstatic"}},
        "writes": {"<init>": {"b"}, "<clinit>": {"a"}},
        "mark_calls": {"<init>": 0, "<clinit>": 0},
    },
    "ordered": {
        "directory": "array-field-initializers", "class": "ArrayFieldInitializers",
        "static_fields": ["trace", "before", "a", "after"],
        "required_source": [
            "static int trace = 0;", "static byte before = mark(1);",
            "static byte[] a = new byte[]{mark(2), mark(3), mark(4)};",
            "static byte after = mark(5);",
        ],
        "source_sha256": "8ed268bad1dff3025b1d02e3c95a26b06889c02a1c14f52cc953f8e944e61650",
        "runner_sha256": "b6541a755e1e9b6c2c4b7d601b14fcc2a27f820650ec968fe0f2d150f79a28cd",
        "manifest_sha256": "93f14c0b125a738378c18a7a1a1f63dc6fb8c86c8c349c879b8e0e567c7c89d5",
        "class_sha256": {"javac8": "5bd4b86f40a268640dd391df54a050f44056918928ae240232ebed6671fa24ee",
                         "javac23": "3660a28f19cb7304ddc9c952a89f1b43378834faf0251f0910770ba9f89a6789"},
        "fields": {("trace", "I", 8), ("before", "B", 8), ("a", "[B", 8),
                   ("after", "B", 8), ("b", "[B", 0)},
        "methods": {("<init>", "()V", 1), ("mark", "(I)B", 8), ("<clinit>", "()V", 8)},
        "opcodes": {"<init>": {"invokespecial", "newarray", "invokestatic", "bastore", "putfield"},
                    "mark": {"getstatic", "putstatic", "imul", "iadd", "i2b", "ireturn"},
                    "<clinit>": {"invokestatic", "newarray", "bastore", "putstatic"}},
        "writes": {"<init>": {"b"}, "mark": {"trace"},
                   "<clinit>": {"trace", "before", "a", "after"}},
        "mark_calls": {"<init>": 4, "<clinit>": 5},
    },
}
ACCESS_BITS = {
    "ACC_PUBLIC": 0x0001, "ACC_PRIVATE": 0x0002, "ACC_PROTECTED": 0x0004,
    "ACC_STATIC": 0x0008, "ACC_FINAL": 0x0010, "ACC_SYNCHRONIZED": 0x0020,
    "ACC_VOLATILE": 0x0040, "ACC_BRIDGE": 0x0040, "ACC_TRANSIENT": 0x0080,
    "ACC_VARARGS": 0x0080, "ACC_NATIVE": 0x0100, "ACC_INTERFACE": 0x0200,
    "ACC_ABSTRACT": 0x0400, "ACC_STRICT": 0x0800, "ACC_SYNTHETIC": 0x1000,
    "ACC_ANNOTATION": 0x2000, "ACC_ENUM": 0x4000, "ACC_MODULE": 0x8000,
}
CHECKS = 0


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check(condition, message):
    global CHECKS
    CHECKS += 1
    if not condition:
        raise AssertionError(message)


def read_json(path):
    return json.loads(path.read_bytes())


def decode(raw):
    return bytes(raw).decode("utf-8")


def output_file(base, record):
    path = base / record["path"]
    check(path.is_file() and not path.is_symlink(), "regular file " + record["path"])
    data = path.read_bytes()
    check(len(data) == record["bytes"], "byte length " + record["path"])
    check(sha(data) == record["sha256"], "sha256 " + record["path"])
    return path


def verify_closed_inventory(base):
    inventory_path = base / "file-inventory.json"
    rows = read_json(inventory_path)
    listed = {row["path"] for row in rows}
    check(len(listed) == len(rows), "inventory paths unique")
    actual = {path.relative_to(base).as_posix() for path in base.rglob("*")
              if path.is_file() and path != inventory_path}
    check(listed == actual, "closed evidence inventory")
    for row in rows:
        output_file(base, row)
    check("manifest.json" in listed, "manifest is included in inventory")
    return rows


def command_stream(base, command, name):
    record = command[name]
    return output_file(base, record).read_bytes()


def method_key(item):
    return decode(item["name"]["raw"]), decode(item["descriptor"]["raw"])


def parse_javap(text, class_name):
    lines = text.splitlines()
    methods = {}
    code_indices = [i for i, line in enumerate(lines) if line.strip() == "Code:"]
    insn_re = re.compile(r"^\s*(\d+):\s+([a-z][a-z0-9_]*)\b(.*)$")
    descriptor_re = re.compile(r"^\s*descriptor:\s*(\S+)")
    for ordinal, code_index in enumerate(code_indices):
        previous = code_indices[ordinal - 1] if ordinal else -1
        desc_index = next((i for i in range(code_index - 1, previous, -1)
                           if descriptor_re.match(lines[i])), None)
        check(desc_index is not None, "javap Code has descriptor")
        descriptor = descriptor_re.match(lines[desc_index]).group(1)
        header_index = desc_index - 1
        while header_index > previous and not lines[header_index].strip():
            header_index -= 1
        header = lines[header_index].strip()
        if re.search(r"\bstatic\s*\{\};$", header):
            name = "<clinit>"
        else:
            match = re.search(r"([A-Za-z_$][\w$]*)\([^)]*\);$", header)
            check(match is not None, "javap method header " + header)
            name = match.group(1)
            if name == class_name:
                name = "<init>"
        end = code_indices[ordinal + 1] if ordinal + 1 < len(code_indices) else len(lines)
        insns = []
        for line in lines[code_index + 1:end]:
            match = insn_re.match(line)
            if match:
                insns.append((int(match.group(1)), match.group(2), match.group(3).strip()))
        key = (name, descriptor)
        check(key not in methods, "unique javap Code member " + repr(key))
        methods[key] = insns
    return methods


def parse_physical_members(text, class_name):
    lines = text.splitlines()
    headers = []
    for index, line in enumerate(lines):
        stripped = line.strip()
        if (not line.startswith("  ") or line.startswith("   ") or not stripped
                or stripped.startswith("#") or not stripped.endswith(";")):
            continue
        if stripped == "static {};":
            headers.append((index, "method", "<clinit>"))
        elif re.search(r"\([^)]*\)(?:\s+throws\s+[^;]+)?;$", stripped):
            prefix = stripped[:stripped.index("(")].split()
            if prefix:
                name = prefix[-1]
                headers.append((index, "method", "<init>" if name == class_name else name))
        elif "(" not in stripped:
            names = stripped[:-1].split()
            if len(names) >= 2:
                headers.append((index, "field", names[-1]))
    result = {"fields": [], "methods": []}
    for slot, (start, kind, name) in enumerate(headers):
        end = headers[slot + 1][0] if slot + 1 < len(headers) else len(lines)
        block = lines[start:end]
        desc_line = next((line for line in block if re.match(r"^\s*descriptor:\s*\S+\s*$", line)), None)
        flags_line = next((line for line in block if re.match(r"^\s*flags:\s*", line)), None)
        check(desc_line is not None and flags_line is not None, "javap member descriptor/flags " + name)
        descriptor = re.match(r"^\s*descriptor:\s*(\S+)", desc_line).group(1)
        flags = 0
        for flag in re.findall(r"ACC_[A-Z_]+", flags_line):
            check(flag in ACCESS_BITS, "known javap access flag " + flag)
            flags |= ACCESS_BITS[flag]
        result[kind + "s"].append({
            "name": name, "descriptor": descriptor, "flags": flags,
            "has_code": any(line.strip() == "Code:" for line in block),
            "has_constant_value": any("ConstantValue:" in line for line in block),
            "block": block,
        })
    return result


def source_origins(report, key, identity):
    origins = set()
    for segment in report["source_map"]["segments"]:
        origin = segment["origin"]
        for item in [origin["primary"], *origin.get("derived", [])]:
            method = item["method"]
            origin_key = (decode(method["name"]), decode(method["descriptor"]))
            check(origin_key == key and method == identity, "source-map full method identity " + repr(key))
            origins.add(item["bci"])
    return origins


def baseline_manifest(family):
    spec = EXPECTED[family]
    base = EVIDENCE / spec["directory"] / "baseline-root-v1"
    manifest_path = base / "manifest.json"
    inventory_path = base / "file-inventory.json"
    manifest = read_json(manifest_path)
    rows = read_json(inventory_path)
    listed = {row["path"] for row in rows}
    check(len(listed) == len(rows), family + " baseline inventory uniqueness")
    actual = {path.relative_to(base).as_posix() for path in base.rglob("*")
              if path.is_file() and path != inventory_path}
    check(actual == listed, family + " closed baseline inventory")
    for row in rows:
        output_file(base, row)
    check(sha(manifest_path.read_bytes()) == spec["manifest_sha256"], family + " baseline manifest pin")
    check(manifest["status"] == "completed", family + " completed frozen baseline")
    check(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 2}, family + " baseline case counts")
    check(manifest["success_counts"] == manifest["case_counts"], family + " baseline success counts")
    check(len(manifest["cases"]) == 8 and len(manifest["commands"]) == 31,
          family + " baseline actual cases and commands")
    check(manifest["review_only_expected_values"] == [], family + " baseline oracle is observed")
    for command in manifest["commands"]:
        check(command["exit"] == 0, family + " baseline command exit " + command["label"])
        command_stream(base, command, "stdout")
        command_stream(base, command, "stderr")
    source = base / manifest["source"]["path"]
    runner = base / manifest["runner"]["path"]
    check(sha(source.read_bytes()) == spec["source_sha256"], family + " baseline original source pin")
    check(sha(runner.read_bytes()) == spec["runner_sha256"], family + " baseline original Runner pin")
    cases = {}
    for leg in ("javac8", "javac23"):
        for kind in ("original", "jarde"):
            match = [case for case in manifest["cases"]
                     if case.get("kind") == kind and case.get("jdk_leg") == leg]
            check(len(match) == 1 and match[0].get("success"), family + " baseline " + kind + " " + leg)
            cases[(kind, leg)] = match[0]
        inputs = [row for row in manifest["inputs"] if row["leg"] == leg]
        check(len(inputs) == 1, family + " one original class input " + leg)
        class_path = output_file(base, inputs[0])
        check(sha(class_path.read_bytes()) == spec["class_sha256"][leg], family + " original class pin " + leg)
        cases[("class", leg)] = class_path
        original_runtime = cases[("original", leg)]["runtime"]
        jarde_runtime = cases[("jarde", leg)]["runtime"]
        for runtime in (original_runtime, jarde_runtime):
            check(runtime["exit"] == 0, family + " baseline runtime success " + leg)
        original_triple = (original_runtime["exit"], command_stream(base, original_runtime, "stdout"),
                           command_stream(base, original_runtime, "stderr"))
        jarde_triple = (jarde_runtime["exit"], command_stream(base, jarde_runtime, "stdout"),
                        command_stream(base, jarde_runtime, "stderr"))
        check(original_triple == jarde_triple, family + " frozen Jarde equals original raw triple " + leg)
    originals = [cases[("original", leg)]["runtime"] for leg in ("javac8", "javac23")]
    check((originals[0]["exit"], command_stream(base, originals[0], "stdout"), command_stream(base, originals[0], "stderr"))
          == (originals[1]["exit"], command_stream(base, originals[1], "stdout"), command_stream(base, originals[1], "stderr")),
          family + " original raw triples equal across JDKs")
    return base, manifest, cases


def check_document(family, case, document, javap_text, bytecode):
    spec = EXPECTED[family]
    label = case["label"]
    fields = document["fields"]
    methods = document["methods"]
    actual_fields = {(method_key(item["item"])[0], method_key(item["item"])[1], item["item"]["access_flags"])
                     for item in fields}
    actual_methods = {(method_key(item["item"])[0], method_key(item["item"])[1], item["item"]["access_flags"])
                      for item in methods}
    check(actual_fields == spec["fields"] and len(fields) == len(spec["fields"]), label + " exact JSON physical fields")
    check(actual_methods == spec["methods"] and len(methods) == len(spec["methods"]), label + " exact JSON physical methods")
    proof = document["initializer_proof"]
    check(proof["kind"] == "proved", label + " complete static field group proved")
    proof_fields = proof["fields"]
    field_names = [decode(item["item"]["name"]["raw"]) for item in fields]
    proof_names = [field_names[item["field_index"]] for item in proof_fields]
    check(proof_names == spec["static_fields"], label + " JSON proof group field order")
    check(len(proof_fields) == (4 if family == "ordered" else 1), label + " expected JSON group cardinality")

    text = document["text"]
    positions = [text.find(part) for part in spec["required_source"]]
    check(all(position >= 0 for position in positions) and positions == sorted(positions),
          label + " exact static declaration order")
    check(re.search(r"^\s*static\s*\{", text, re.M) is None, label + " no duplicate root static block")
    check(re.search(r"(?m)^\s*byte\[\]\s+b\s*;", text) is not None,
          label + " instance b remains an uninitialized declaration")
    constructor = next((method for method in methods if method_key(method["item"]) == ("<init>", "()V")), None)
    constructor_text = constructor["outcome"]["report"]["text"] if constructor else ""
    check("this.b = new byte[]" in constructor_text, label + " instance b remains assigned in constructor")

    clinit = next((method for method in methods if method_key(method["item"]) == ("<clinit>", "()V")), None)
    check(clinit is not None, label + " physical <clinit> JSON member retained")
    clinit_report = clinit["outcome"]["report"]
    check(clinit["outcome"]["kind"] == "recovered" and clinit_report["quality"] == "structured"
          and clinit_report["representation"] == "java", label + " physical <clinit> recovered structurally")
    check("static {" in clinit["text"] and clinit_report["source_map"]["segments"],
          label + " physical <clinit> and source map retained")
    for method in methods:
        method_key_value = method_key(method["item"])
        report = method["outcome"]["report"]
        check(method["outcome"]["kind"] == "recovered" and report["quality"] == "structured"
              and report["representation"] == "java", label + " structured method " + repr(method_key_value))
        origins = source_origins(report, method_key_value, method["item"]["identity"])
        opcodes = spec["opcodes"].get(method_key_value[0], set())
        # Parse all original physical methods once; assert each relevant instruction BCI maps back
        # to the same physical member identity in the JSON source map.
        physical_code = bytecode.get(method_key_value)
        check(physical_code is not None, label + " javap physical method " + repr(method_key_value))
        present_opcodes = {opcode for _, opcode, _ in physical_code}
        check(opcodes <= present_opcodes, label + " required JVM operations " + repr(method_key_value))
        relevant = [(bci, opcode, operand) for bci, opcode, operand in physical_code if opcode in opcodes]
        check(bool(relevant) and all(bci in origins for bci, _, _ in relevant),
              label + " field/array/call BCIs have same-method source origins " + repr(method_key_value))
        writes = set()
        for _, opcode, operand in relevant:
            if opcode in ("putfield", "putstatic"):
                match = re.search(r"Field\s+(?:[^\s]+\.)?([^:\s]+):", operand)
                check(match is not None, label + " field write operand " + operand)
                writes.add(match.group(1))
        if method_key_value[0] in spec["writes"]:
            check(writes == spec["writes"][method_key_value[0]],
                  label + " exact physical write targets " + repr(method_key_value))
        expected_calls = spec["mark_calls"].get(method_key_value[0], 0)
        actual_calls = sum(opcode == "invokestatic" and bool(re.search(r"(?:^|[ .])mark:\(I\)B", operand))
                           for _, opcode, operand in physical_code)
        check(actual_calls == expected_calls, label + " physical marker call count " + repr(method_key_value))
        if method_key_value[0] == "<init>":
            check(any(opcode == "invokespecial" and "java/lang/Object.\"<init>\"" in operand
                      for _, opcode, operand in relevant), label + " constructor super call has origin")

    physical = parse_physical_members(javap_text, spec["class"])
    for category, wanted in (("fields", spec["fields"]), ("methods", spec["methods"])):
        rows = physical[category]
        actual = {(row["name"], row["descriptor"], row["flags"]) for row in rows}
        check(actual == wanted and len(rows) == len(wanted), label + " javap -p exact physical " + category)
    check(all(not row["has_constant_value"] for row in physical["fields"]),
          label + " no physical ConstantValue attributes")
    check(all(row["has_code"] for row in physical["methods"]), label + " all physical methods retain Code")
    if family == "ordered":
        trace = next(field for field in fields if decode(field["item"]["name"]["raw"]) == "trace")
        check(trace["item"]["access_flags"] & 0x0010 == 0, label + " trace remains nonfinal")


def verify_candidate():
    check(not RESULT.exists(), "refuse to overwrite verification result")
    manifest_path = OUT / "manifest.json"
    inventory_path = OUT / "file-inventory.json"
    manifest = read_json(manifest_path)
    inventory = verify_closed_inventory(OUT)
    check(manifest["schema"] == "preserve-nonfinal-static-initializer-candidate-luna-v1", "candidate schema")
    check(manifest["status"] == "completed" and not manifest["failures"], "candidate run completed without failures")
    check(manifest["file_inventory"] == {"path": "file-inventory.json", "includes": ["manifest.json"],
                                          "excludes": ["file-inventory.json"]}, "inventory contract")

    metadata_record = manifest["metadata"]
    metadata_path = Path(metadata_record["path"])
    check(metadata_path.resolve() == CLI_METADATA.resolve(), "expected frozen metadata path")
    metadata_bytes = metadata_path.read_bytes()
    check(len(metadata_bytes) == metadata_record["bytes"] and sha(metadata_bytes) == metadata_record["sha256"],
          "metadata raw identity from candidate record")
    check(sha(metadata_bytes) == "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a", "root fixed metadata SHA")
    metadata = json.loads(metadata_bytes)
    check(metadata["cli_path"] == str(CLI_PATH), "metadata pins dedicated nonfinal CLI path")
    check(manifest["arguments"]["metadata"] == str(CLI_METADATA), "candidate invoked with pinned metadata")
    check(Path(manifest["arguments"]["cli"]).resolve() == CLI_PATH.resolve(), "candidate invoked pinned CLI")
    cli_hash = sha(CLI_PATH.read_bytes())
    check(cli_hash == "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e", "root fixed CLI SHA")
    check(metadata["cli_sha256"] == cli_hash == manifest["candidate_cli"]["sha256"], "frozen CLI current SHA")
    check(manifest["candidate_cli"]["path"] == str(CLI_PATH), "candidate CLI path record")
    check(manifest["candidate_sources"] == metadata["candidate_sources"], "candidate manifest source pins equal metadata")
    candidate_sources = metadata["candidate_sources"]
    check(set(candidate_sources) == PRODUCT_PINS, "exact ten frozen product source pins")
    for relative, expected_hash in candidate_sources.items():
        path = ROOT / relative
        check(path.is_file() and sha(path.read_bytes()) == expected_hash, "current product source pin " + relative)
    test_sources = metadata["test_sources"]
    check("tests/class_static_initializer_projection.rs" in test_sources,
          "test metadata pins the static projection test")
    for relative, expected_hash in test_sources.items():
        path = ROOT / relative
        check(path.is_file() and sha(path.read_bytes()) == expected_hash, "current test source pin " + relative)
    canonical = metadata["canonical_files"]
    check(bool(canonical), "metadata has canonical fixture pins")
    for relative, expected_hash in canonical.items():
        path = ROOT / relative
        check(path.is_file() and sha(path.read_bytes()) == expected_hash, "current canonical fixture pin " + relative)

    jdk_record = manifest["jdk_manifest"]
    check(Path(jdk_record["path"]).resolve() == JDK_MANIFEST.resolve(), "JDK controls manifest path")
    jdk_bytes = JDK_MANIFEST.read_bytes()
    check(sha(jdk_bytes) == jdk_record["sha256"], "JDK controls manifest raw pin")
    check(sha(jdk_bytes) == "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec", "root fixed JDK manifest SHA")
    jdk = json.loads(jdk_bytes)
    jdk_legs = {row["leg"]: row for row in jdk["legs"]}
    check(set(jdk_legs) == {"javac8", "javac23"}, "two frozen JDK legs")
    jdk_tools = {}
    for leg, row in jdk_legs.items():
        jdk_tools[leg] = {}
        for tool, record in row["jdk_tools"].items():
            path = Path(record["path"])
            check(path.is_file() and sha(path.read_bytes()) == record["sha256"],
                  "current frozen JDK tool " + leg + ":" + tool)
            jdk_tools[leg][tool] = path
        check(set(jdk_tools[leg]) >= {"java", "javac", "javap"}, "complete toolchain " + leg)

    baseline_data = {family: baseline_manifest(family) for family in EXPECTED}
    commands = manifest["commands"]
    by_label = {command["label"]: command for command in commands}
    check(len(by_label) == len(commands), "unique actual command labels")
    expected_labels = {f"{family}-javac{jdk}-candidate-{kind}"
                       for family in EXPECTED for jdk in (8, 23) for kind in ("render", "compile", "run", "javap")}
    check(len(commands) == 16 and set(by_label) == expected_labels, "four candidate legs and sixteen real commands")
    for command in commands:
        check(command["exit"] == 0, "candidate command exit " + command["label"])
        command_stream(OUT, command, "stdout")
        command_stream(OUT, command, "stderr")

    cases = manifest["cases"]
    expected_case_labels = {f"{family}-javac{jdk}-candidate" for family in EXPECTED for jdk in (8, 23)}
    check(len(cases) == 4 and {case["label"] for case in cases} == expected_case_labels,
          "four candidate cases exactly")
    case_results = []
    baseline_summaries = {}
    for family, spec in EXPECTED.items():
        baseline_root, baseline, reference_cases = baseline_data[family]
        baseline_summaries[family] = {"manifest_sha256": sha((baseline_root / "manifest.json").read_bytes()),
                                      "inventory_sha256": sha((baseline_root / "file-inventory.json").read_bytes())}
        baseline_manifest_record = manifest["baseline_manifests"][family]
        baseline_path = baseline_root / "manifest.json"
        check(Path(baseline_manifest_record["path"]).resolve() == baseline_path.resolve(),
              family + " baseline manifest path")
        check(baseline_manifest_record["sha256"] == sha(baseline_path.read_bytes()), family + " baseline manifest pin")
        for jdk_leg in ("javac8", "javac23"):
            label = f"{family}-{jdk_leg}-candidate"
            case = next(item for item in cases if item["label"] == label)
            check(case["family"] == family and case["jdk_leg"] == jdk_leg, label + " family/leg")
            check(case.get("success") and case.get("render_success") and case.get("compile_success")
                  and case.get("runtime_success"), label + " recorded summary agrees")
            source_class = reference_cases[("class", jdk_leg)]
            original_case = reference_cases[("original", jdk_leg)]
            jarde_case = reference_cases[("jarde", jdk_leg)]
            original_runtime = original_case["runtime"]
            jarde_runtime = jarde_case["runtime"]
            recorded_original = case["original_class"]
            check(Path(recorded_original["path"]).resolve() == source_class.resolve()
                  and sha(source_class.read_bytes()) == recorded_original["sha256"], label + " exact baseline class input")
            check(recorded_original["sha256"] == spec["class_sha256"][jdk_leg], label + " baseline original class pin")

            render = case["render"]
            check(render == by_label[label + "-render"], label + " render command record")
            render_argv = render["argv"]
            check(render_argv == [str(CLI_PATH), "class-source", "--input", str(source_class), "--class", spec["class"],
                                  "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all"],
                  label + " exact class-source evidence-all CLI argv")
            document_record = case["document"]
            document_path = output_file(OUT, document_record)
            raw_render = command_stream(OUT, render, "stdout")
            check(document_path.read_bytes() == raw_render, label + " full raw CLI JSON preserved")
            document = json.loads(raw_render)
            check(case["field_count"] == len(document["fields"])
                  and case["member_count"] == len(document["methods"]), label + " inventory counts from JSON")

            source_files = [output_file(OUT, row) for row in case["source_files"]]
            generated = next((path for path in source_files if path.name == spec["class"] + ".java"), None)
            runner = next((path for path in source_files if path.name == "Runner.java"), None)
            check(generated is not None and runner is not None and len(source_files) == 2,
                  label + " exactly full class source plus Runner")
            check(generated.read_bytes() == document["text"].encode(), label + " generated class source unchanged")
            original_runner = baseline_root / "original-sources/Runner.java"
            runner_text = original_runner.read_text()
            package = re.search(r"^\s*package\s+([\w.]+)\s*;", generated.read_text(), re.M)
            generated_package = package.group(1) if package else None
            adaptation = (("package " + generated_package + ";\n\n") if generated_package else "").encode()
            check(runner.read_bytes() == adaptation + original_runner.read_bytes(), label + " Runner only package adaptation")
            check(case["runner_package_adaptation"] == generated_package, label + " recorded Runner package")

            compile_record = case["compile"]
            check(compile_record == by_label[label + "-compile"], label + " compile record")
            argv = compile_record["argv"]
            check(argv[0] == str(jdk_tools[jdk_leg]["javac"]), label + " frozen javac binary")
            empty_cp = Path(argv[argv.index("-classpath") + 1])
            empty_sp = Path(argv[argv.index("-sourcepath") + 1])
            classes = Path(argv[argv.index("-d") + 1])
            check(empty_cp == empty_sp and empty_cp.is_dir() and not any(empty_cp.iterdir()),
                  label + " empty classpath/sourcepath")
            check(argv[1:argv.index("-classpath")] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"],
                  label + " exact Java 8 compiler flags")
            check(classes == OUT / "cases" / label / "classes", label + " fresh classes directory")
            expected_sources = [str(path) for path in source_files]
            check(argv[argv.index("-d") + 2:] == expected_sources, label + " javac includes exactly all full sources")
            class_rows = case["classes"]
            class_paths = [output_file(OUT, row) for row in class_rows]
            actual_classes = sorted(path for path in classes.rglob("*.class") if path.is_file())
            check({path.resolve() for path in class_paths} == {path.resolve() for path in actual_classes},
                  label + " closed generated class inventory")

            runtime = case["runtime"]
            check(runtime == by_label[label + "-run"], label + " runtime record")
            runner_name = (generated_package + "." if generated_package else "") + "Runner"
            check(runtime["argv"] == [str(jdk_tools[jdk_leg]["java"]), "-Xverify:all", "-cp", str(classes), runner_name],
                  label + " fresh classes only -Xverify:all runtime")
            check(Path(runtime["argv"][0]).parent == Path(argv[0]).parent, label + " compiler/runtime same JDK")
            candidate_triple = (runtime["exit"], command_stream(OUT, runtime, "stdout"),
                                command_stream(OUT, runtime, "stderr"))
            for baseline_kind, expected_case in (("original", original_case), ("jarde", jarde_case)):
                expected_runtime = expected_case["runtime"]
                expected_triple = (expected_runtime["exit"], command_stream(baseline_root, expected_runtime, "stdout"),
                                   command_stream(baseline_root, expected_runtime, "stderr"))
                check(candidate_triple == expected_triple,
                      label + " raw exit/stdout/stderr equals " + baseline_kind + " oracle")

            javap_record = case["javap"]
            check(javap_record == by_label[label + "-javap"], label + " javap record")
            target_class = next((path for path in class_paths if path.name == spec["class"] + ".class"), None)
            check(target_class is not None, label + " compiled target class exists")
            check(case["javap_class"]["sha256"] == sha(target_class.read_bytes()), label + " javap class identity")
            check(javap_record["argv"] == [str(jdk_tools[jdk_leg]["javap"]), "-p", "-c", "-s", "-v", str(target_class)],
                  label + " physical javap command")
            javap_text = command_stream(OUT, javap_record, "stdout").decode(errors="strict")
            # JSON origins describe the original input, never the recompiled candidate.
            original_javap = next(command for command in baseline["commands"]
                                  if command["label"] == jdk_leg + "-javap")
            check(original_javap["argv"] == [str(jdk_tools[jdk_leg]["javap"]), "-p", "-c", "-s", "-v", str(source_class)],
                  label + " original physical javap argv")
            original_javap_text = command_stream(baseline_root, original_javap, "stdout").decode()
            bytecode = parse_javap(original_javap_text, spec["class"])
            original_members = parse_physical_members(original_javap_text, spec["class"])
            for category in ("fields", "methods"):
                check({(x["name"], x["descriptor"], x["flags"]) for x in original_members[category]} == spec[category],
                      label + " original physical member flags " + category)
            physical_writes = [(bci, re.search(r"Field\s+(?:[^\s]+\.)?([^:\s]+):", operand).group(1))
                               for bci, op, operand in bytecode[("<clinit>", "()V")]
                               if op == "putstatic"]
            proof = document["initializer_proof"]["fields"]
            names = [decode(field["item"]["name"]["raw"]) for field in document["fields"]]
            check([(item["write_bci"], names[item["field_index"]]) for item in proof] == physical_writes,
                  label + " proof BCIs bind exact original field writes")
            check([item["write_order"] for item in proof] == list(range(len(proof))), label + " exact proof write orders")
            check_document(family, case, document, javap_text, bytecode)
            case_results.append({"label": label, "family": family, "jdk_leg": jdk_leg,
                                 "member_count": len(document["methods"]),
                                 "static_group_fields": len(document["initializer_proof"]["fields"]),
                                 "runtime_sha256": {"stdout": sha(candidate_triple[1]), "stderr": sha(candidate_triple[2])},
                                 "raw_matches_original_and_baseline_jarde": True})

    check(manifest["case_counts"] == {"candidate": 4} and manifest["success_counts"] == {"candidate": 4},
          "candidate four-leg summary counts")
    result = {
        "schema": "preserve-nonfinal-static-initializer-candidate-root-verification-v1",
        "status": "accepted", "checks": CHECKS,
        "candidate_manifest_sha256": sha(manifest_path.read_bytes()),
        "candidate_inventory_sha256": sha(inventory_path.read_bytes()),
        "candidate_cli": {"path": str(CLI_PATH), "sha256": cli_hash,
                          "metadata_path": str(CLI_METADATA), "metadata_sha256": sha(metadata_bytes)},
        "candidate_sources": {relative: sha((ROOT / relative).read_bytes()) for relative in sorted(candidate_sources)},
        "baseline_manifests": baseline_summaries,
        "command_count": len(commands), "candidate_legs": case_results,
        "static_group_sizes": {family: len(EXPECTED[family]["static_fields"]) for family in EXPECTED},
        "claim_boundary": manifest["claim_boundary"],
        "oracle": "Each fresh candidate runtime exit/stdout/stderr is byte-equal to both the fresh original and already accepted baseline Jarde raw triple for that same class and JDK leg.",
        "verifier": "verify-candidate-root-v2.py", "errors": [],
        "verification_scope": "Original-input BCIs and full owner identities; closed inventories and raw command streams; frozen CLI/product metadata and JDK tool pins; complete-source empty-CP/SP compile and fresh -Xverify runtime; physical javap members/flags/ConstantValue; ordered JSON field groups; method-identity-bound array, field, and call BCIs in source maps.",
    }
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "checks": CHECKS,
                      "candidate_legs": case_results, "result": str(RESULT)}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    verify_candidate()
