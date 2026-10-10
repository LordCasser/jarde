#!/usr/bin/env python3
"""Independently verify the two prepared array-field initializer baselines."""
import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
RESULT = HERE / "array-field-initializers-root-verification-v1.json"
BASELINES = {
    "literal": HERE / "array-field-initializers-literal" / "baseline-root-v1",
    "ordered": HERE / "array-field-initializers" / "baseline-root-v1",
}
EXPECTED = {
    "literal": {
        "class": "ArrayFieldLiteral",
        "oracle": b"static=[10, 20, 30]\nfirst=[40, 50, 60]\nsecond=[40, 50, 60]\nfresh=true\n",
        "methods": {("<init>", "()V", 1), ("<clinit>", "()V", 8)},
        "fields": {("a", "[B", 8), ("b", "[B", 0)},
        "ops": {
            "<init>": {"invokespecial", "newarray", "bastore", "putfield"},
            "<clinit>": {"newarray", "bastore", "putstatic"},
        },
        "writes": {"<init>": {"b"}, "<clinit>": {"a"}},
    },
    "ordered": {
        "class": "ArrayFieldInitializers",
        "oracle": (b"static=12345:1:[2, 3, 4]:5\n"
                   b"first=123456789:[6, 7, 8]\n"
                   b"second=1912282837:[6, 7, 8]\n"),
        "methods": {("<init>", "()V", 1), ("mark", "(I)B", 8), ("<clinit>", "()V", 8)},
        "fields": {("trace", "I", 8), ("before", "B", 8), ("a", "[B", 8),
                   ("after", "B", 8), ("b", "[B", 0)},
        "ops": {
            "<init>": {"invokespecial", "newarray", "invokestatic", "bastore", "putfield"},
            "mark": {"putstatic"},
            "<clinit>": {"invokestatic", "newarray", "bastore", "putstatic"},
        },
        "writes": {"<init>": {"b"}, "mark": {"trace"}, "<clinit>": {"trace", "before", "a", "after"}},
    },
}

checks = 0


def check(condition, label):
    global checks
    checks += 1
    assert condition, label


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def output_path(base, row):
    return base / row["path"]


def verify_row(base, row):
    path = output_path(base, row)
    check(path.is_file() and not path.is_symlink(), "regular file " + row["path"])
    check(path.stat().st_size == row["bytes"], "byte length " + row["path"])
    check(sha(path) == row["sha256"], "sha256 " + row["path"])
    return path


def decode(raw):
    return bytes(raw).decode("utf-8")


def method_key(item):
    return (decode(item["name"]["raw"]), decode(item["descriptor"]["raw"]))


def parse_javap(path, class_name):
    lines = path.read_text().splitlines()
    methods = {}
    code_indices = [index for index, line in enumerate(lines) if line.strip() == "Code:"]
    instruction_re = re.compile(r"^\s*(\d+):\s+([a-z][a-z0-9_]*)\b(.*)$")
    descriptor_re = re.compile(r"^\s*descriptor:\s*(\S+)")
    for ordinal, code_index in enumerate(code_indices):
        previous_code = code_indices[ordinal - 1] if ordinal else -1
        descriptor_index = next((i for i in range(code_index - 1, previous_code, -1)
                                 if descriptor_re.match(lines[i])), None)
        check(descriptor_index is not None, "javap descriptor before Code")
        descriptor = descriptor_re.match(lines[descriptor_index]).group(1)
        header_index = descriptor_index - 1
        while header_index > previous_code and not lines[header_index].strip():
            header_index -= 1
        header = lines[header_index].strip()
        if re.search(r"\bstatic\s*\{\};$", header):
            name = "<clinit>"
        else:
            match = re.search(r"([A-Za-z_$][\w$]*)\([^)]*\);$", header)
            check(match is not None, "javap method header: " + header)
            name = match.group(1)
            if name == class_name:
                name = "<init>"
        next_code = code_indices[ordinal + 1] if ordinal + 1 < len(code_indices) else len(lines)
        instructions = []
        for line in lines[code_index + 1:next_code]:
            match = instruction_re.match(line)
            if match:
                instructions.append((int(match.group(1)), match.group(2), match.group(3).strip()))
        key = (name, descriptor)
        check(key not in methods, "unique javap method " + repr(key))
        methods[key] = instructions
    return methods


def source_origins(report, key):
    origins = set()
    for segment in report["source_map"]["segments"]:
        origin = segment["origin"]
        for item in [origin["primary"], *origin.get("derived", [])]:
            method = item["method"]
            origin_key = (decode(method["name"]), decode(method["descriptor"]))
            check(origin_key == key, "source origin belongs to its method " + repr(key))
            origins.add(item["bci"])
    return origins


def verify_tool_pins(base, manifest, commands, cli_meta_path, jdk_manifest_path):
    check(manifest["jdk_manifest"]["path"] == str(jdk_manifest_path), "JDK manifest path")
    check(sha(jdk_manifest_path) == manifest["jdk_manifest"]["sha256"], "JDK manifest pin")
    check(manifest["cli_metadata"]["path"] == str(cli_meta_path), "CLI metadata path")
    check(sha(cli_meta_path) == manifest["cli_metadata"]["sha256"], "CLI metadata pin")
    cli_meta = json.loads(cli_meta_path.read_bytes())
    check(manifest["frozen_cli"]["path"] == cli_meta["cli_path"], "frozen CLI path")
    check(manifest["frozen_cli"]["sha256"] == cli_meta["cli_sha256"], "frozen CLI recorded pin")
    cli_path = Path(cli_meta["cli_path"])
    check(cli_path.is_file() and sha(cli_path) == cli_meta["cli_sha256"], "frozen CLI current pin")
    for row in manifest["preflight"]:
        check(row["ok"], "recorded preflight " + row["label"])
        if "expected_sha256" in row:
            pinned_path = Path(row["path"]) if row.get("path") else ROOT / row["label"].split(":", 1)[1]
            check(pinned_path.is_file() and sha(pinned_path) == row["expected_sha256"],
                  "current tool/source pin " + row["label"])
    jdk_meta = json.loads(jdk_manifest_path.read_bytes())
    frozen_jdks = {leg["leg"]: {tool: fact["path"] for tool, fact in leg["jdk_tools"].items()}
                   for leg in jdk_meta["legs"]}
    check(set(frozen_jdks) == {"javac8", "javac23"}, "both frozen JDK tool legs")
    for leg, tools in frozen_jdks.items():
        for tool in ("java", "javac", "javap"):
            command = next(item for item in commands if item["label"] == leg + "-" + tool + "-version")
            check(command["argv"] == [tools[tool], "-version"], "frozen JDK version argv " + leg + ":" + tool)
    version = next(command for command in commands if command["label"] == "jadx-version")
    check(version["argv"] == [manifest["jadx"], "--version"], "JADX version argv")
    check(version["exit"] == 0, "JADX version exit")
    check(output_path(base, version["stdout"]).read_bytes() == b"1.5.6\n", "JADX 1.5.6 raw version")
    check(manifest["jadx"] == "/opt/homebrew/bin/jadx", "pinned JADX path")
    return frozen_jdks


def verify_baseline(name, base, expected, jdk_manifest_path, cli_meta_path):
    manifest_path = base / "manifest.json"
    inventory_path = base / "file-inventory.json"
    manifest = json.loads(manifest_path.read_bytes())
    inventory = json.loads(inventory_path.read_bytes())
    entries = {row["path"] for row in inventory}
    check(len(entries) == len(inventory), name + " unique inventory")
    actual_entries = {path.relative_to(base).as_posix() for path in base.rglob("*")
                      if path.is_file() and path != inventory_path}
    check(entries == actual_entries, name + " closed inventory")
    for row in inventory:
        verify_row(base, row)
    check(manifest["status"] == "completed", name + " completed")
    check(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 2}, name + " eight legs")
    check(manifest["success_counts"] == manifest["case_counts"], name + " all legs successful")
    actual_counts = {kind: sum(case.get("kind") == kind for case in manifest["cases"])
                     for kind in ("original", "jadx", "jarde")}
    check(len(manifest["cases"]) == 8 and actual_counts == manifest["case_counts"], name + " actual case inventory")
    check(manifest["original_cross_jdk_equal"], name + " original cross-JDK raw equal")
    check(len(manifest["commands"]) == 31, name + " 31 recorded commands")
    command_by_label = {command["label"]: command for command in manifest["commands"]}
    check(len(command_by_label) == len(manifest["commands"]), name + " unique command labels")
    for command in manifest["commands"]:
        check(command["exit"] == 0, name + " command exit " + command["label"])
        verify_row(base, command["stdout"])
        verify_row(base, command["stderr"])
    frozen_jdks = verify_tool_pins(base, manifest, manifest["commands"], cli_meta_path, jdk_manifest_path)

    fixture_dir = base.parent
    source_top = fixture_dir / (expected["class"] + ".java")
    runner_top = fixture_dir / "Runner.java"
    copied_source = verify_row(base, manifest["source"])
    copied_runner = verify_row(base, manifest["runner"])
    check(copied_source.read_bytes() == source_top.read_bytes(), name + " frozen source copy")
    check(copied_runner.read_bytes() == runner_top.read_bytes(), name + " frozen Runner copy")
    script_path = Path(manifest["script"]["path"])
    check(script_path.is_file() and sha(script_path) == manifest["script"]["sha256"], name + " prepared script hash")

    original_by_jdk = {case["jdk_leg"]: case for case in manifest["cases"] if case["kind"] == "original"}
    check(set(original_by_jdk) == {"javac8", "javac23"}, name + " both original JDK legs")
    input_rows = {row["leg"]: row for row in manifest["inputs"]}
    check(set(input_rows) == {"javac8", "javac23"}, name + " two fresh original class inputs")
    for jdk, original in original_by_jdk.items():
        check(original["runtime"]["exit"] == 0, name + " original oracle exit " + jdk)
        check(output_path(base, original["runtime"]["stdout"]).read_bytes() == expected["oracle"],
              name + " observed original stdout oracle " + jdk)
        check(output_path(base, original["runtime"]["stderr"]).read_bytes() == b"",
              name + " observed original stderr oracle " + jdk)
        original_class = expected_class_path(base, jdk, expected["class"])
        input_path = verify_row(base, input_rows[jdk])
        check(input_path == original_class and input_path.is_file(), name + " exact fresh original class input " + jdk)
        check(original["javap"]["argv"] == [original["javap"]["argv"][0], "-p", "-c", "-s", "-v",
                                               str(original_class)], name + " javap complete physical input " + jdk)
        check(original["javap"]["argv"][0] == frozen_jdks[jdk]["javap"], name + " javap pinned JDK tool " + jdk)

    for case in manifest["cases"]:
        label = case["label"]
        check(case["success"] and case["compile_success"] and case["runtime_success"], name + " successful leg " + label)
        compile = case["compile"]
        runtime = case["runtime"]
        compiler_argv = compile["argv"]
        runtime_argv = runtime["argv"]
        expected_class_dir = base / "cases" / label / "classes"
        empty_cp = Path(compiler_argv[compiler_argv.index("-classpath") + 1])
        empty_sp = Path(compiler_argv[compiler_argv.index("-sourcepath") + 1])
        check(empty_cp == empty_sp and empty_cp.is_dir() and not any(empty_cp.iterdir()),
              name + " empty classpath/sourcepath " + label)
        check(compiler_argv[1:compiler_argv.index("-classpath")] ==
              ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"],
              name + " Java 8 full-source flags " + label)
        check(Path(compiler_argv[compiler_argv.index("-d") + 1]) == expected_class_dir,
              name + " fresh classes target " + label)
        sources = [verify_row(base, row) for row in case["source_files"]]
        argv_sources = [Path(path) for path in compiler_argv[compiler_argv.index("-d") + 2:]]
        check(argv_sources == sources and len(sources) == len(set(sources)),
              name + " compile argv includes every exact source " + label)
        check(len(case["classes"]) == len(list(expected_class_dir.rglob("*.class"))),
              name + " class inventory count " + label)
        check({verify_row(base, row).resolve() for row in case["classes"]} ==
              {path.resolve() for path in expected_class_dir.rglob("*.class")},
              name + " closed generated classes " + label)
        check("-Xverify:all" in runtime_argv and Path(runtime_argv[runtime_argv.index("-cp") + 1]) == expected_class_dir,
              name + " verified run uses only fresh classes " + label)
        check(Path(runtime_argv[0]).parent == Path(compiler_argv[0]).parent,
              name + " matching compiler/runtime JDK " + label)
        check(compiler_argv[0] == frozen_jdks[case["jdk_leg"]]["javac"]
              and runtime_argv[0] == frozen_jdks[case["jdk_leg"]]["java"],
              name + " frozen compiler/runtime binaries " + label)
        check(runtime_argv == [runtime_argv[0], "-Xverify:all", "-cp", str(expected_class_dir),
                               (case.get("runner_package") + "." if case.get("runner_package") else "") + "Runner"],
              name + " fixed Runner entry " + label)
        check(command_by_label[compile["label"]] == compile and command_by_label[runtime["label"]] == runtime,
              name + " compile/run command records " + label)

        if case["kind"] == "original":
            check([row["path"] for row in case["source_files"]] ==
                  [manifest["source"]["path"], manifest["runner"]["path"]], name + " original source argv " + label)
        elif case["kind"] == "jadx":
            original_class = expected_class_path(base, case["jdk_leg"], expected["class"])
            decompile_argv = case["decompile"]["argv"]
            check(decompile_argv[0] == manifest["jadx"] and decompile_argv[-1] == str(original_class),
                  name + " JADX consumes fresh original class " + label)
            check("--no-res" in decompile_argv and "--config" in decompile_argv
                  and decompile_argv[decompile_argv.index("--config") + 1] == "none"
                  and "--threads-count" in decompile_argv
                  and decompile_argv[decompile_argv.index("--threads-count") + 1] == "1",
                  name + " deterministic JADX profile " + label)
            if case["profile"] == "none":
                check(decompile_argv[decompile_argv.index("--rename-flags") + 1] == "none",
                      name + " JADX rename-none profile " + label)
            else:
                check("--rename-flags" not in decompile_argv, name + " JADX default profile " + label)
            check(Path(decompile_argv[decompile_argv.index("-d") + 1]) ==
                  base / "cases" / label / "jadx", name + " fresh JADX output directory " + label)
            check(command_by_label[case["decompile"]["label"]] == case["decompile"],
                  name + " JADX command record " + label)
            generated = case["generated_sources"]
            generated_paths = [verify_row(base, row) for row in generated]
            check(generated_paths and all(path.suffix == ".java" for path in generated_paths),
                  name + " all fresh JADX Java sources " + label)
            package_names = set()
            for path in generated_paths:
                match = re.search(r"^package ([\w.]+);", path.read_text(), re.M)
                package_names.add(match.group(1) if match else None)
            check(len(package_names) == 1, name + " one generated package " + label)
            package = next(iter(package_names))
            check(case.get("runner_package") == package, name + " recorded runner package " + label)
            runner = next(path for path in sources if path.name == "Runner.java")
            expected_runner = (("package " + package + ";\n\n") if package else "").encode() + runner_top.read_bytes()
            check(runner.read_bytes() == expected_runner, name + " Runner only package adaptation " + label)
            check(set(sources) == set(generated_paths + [runner]), name + " no omitted/extra JADX source " + label)
        elif case["kind"] == "jarde":
            original_class = expected_class_path(base, case["jdk_leg"], expected["class"])
            render_argv = case["render"]["argv"]
            check(render_argv[0] == manifest["frozen_cli"]["path"]
                  and render_argv[1:] == ["class-source", "--input", str(original_class), "--class", expected["class"],
                                          "--policy", "single-class", "--release", "8", "--format", "json",
                                          "--evidence", "all"], name + " Jarde full evidence argv " + label)
            check(command_by_label[case["render"]["label"]] == case["render"],
                  name + " Jarde render command record " + label)
            document_path = verify_row(base, case["document"])
            document = json.loads(document_path.read_bytes())
            generated = next(path for path in sources if path.name == expected["class"] + ".java")
            check(generated.read_bytes() == document["text"].encode(), name + " exact full CLI JSON source " + label)
            runner = next(path for path in sources if path.name == "Runner.java")
            check(runner.read_bytes() == runner_top.read_bytes(), name + " unchanged Jarde Runner " + label)
            check(set(sources) == {generated, runner}, name + " full Jarde class plus Runner " + label)
            verify_jarde_document(name, expected, case, document, base)

        oracle = original_by_jdk[case["jdk_leg"]]["runtime"]
        check(runtime["exit"] == oracle["exit"], name + " raw runtime exit oracle " + label)
        for stream in ("stdout", "stderr"):
            check(output_path(base, runtime[stream]).read_bytes() == output_path(base, oracle[stream]).read_bytes(),
                  name + " raw runtime " + stream + " oracle " + label)

    # The fixed values below are copies of the two fresh original javac8/javac23 raw observations above,
    # recorded after execution. They are not test-author predictions or candidate expectations.
    check(manifest["review_only_expected_values"] == [], name + " no pre-execution expected output")
    return manifest


def verify_jarde_document(name, expected, case, document, base):
    methods = document["methods"]
    fields = document["fields"]
    actual_methods = {(method_key(item["item"])[0], method_key(item["item"])[1], item["item"]["access_flags"])
                      for item in methods}
    actual_fields = {(method_key(item["item"])[0], method_key(item["item"])[1], item["item"]["access_flags"])
                     for item in fields}
    check(actual_methods == expected["methods"], name + " all physical methods/descriptor/flags " + case["label"])
    check(actual_fields == expected["fields"], name + " all physical fields/descriptor/flags " + case["label"])
    check(len(methods) == len(expected["methods"]) and len(fields) == len(expected["fields"]),
          name + " physical member cardinality " + case["label"])
    baseline_manifest = json.loads((base / "manifest.json").read_bytes())
    original_case = next(c for c in baseline_manifest["cases"]
                         if c["kind"] == "original" and c["jdk_leg"] == case["jdk_leg"])
    javap_path = output_path(base, original_case["javap"]["stdout"])
    bytecode = parse_javap(javap_path, expected["class"])
    for item in methods:
        report = item["outcome"]["report"]
        key = method_key(item["item"])
        check(item["outcome"]["kind"] == "recovered" and report["quality"] == "structured"
              and report["representation"] == "java", name + " structured member " + repr(key))
        check(report["source_map"]["segments"], name + " source map present " + repr(key))
        origins = source_origins(report, key)
        required_opcodes = expected["ops"].get(key[0], set())
        if required_opcodes:
            instructions = bytecode.get(key)
            check(instructions is not None, name + " javap method exists " + repr(key))
            targeted = [(bci, opcode, operand) for bci, opcode, operand in instructions
                        if opcode in required_opcodes]
            check(targeted and all(bci in origins for bci, _, _ in targeted),
                  name + " physical opcode BCIs mapped to source " + repr(key))
            write_targets = {match.group(1) for _, opcode, operand in targeted
                             if opcode in ("putfield", "putstatic")
                             for match in [re.search(r"Field ([^:]+):", operand)] if match}
            if key[0] in expected["writes"]:
                check(write_targets == expected["writes"][key[0]],
                      name + " exact physical field writes and source origins " + repr(key))
            if key[0] == "<init>":
                check(any(opcode == "invokespecial" and "java/lang/Object.\"<init>\"" in operand
                          for _, opcode, operand in targeted), name + " constructor invocation origin")
    text = document["text"]
    check("@bytecode" not in text and "jarde_refused_body" not in text,
          name + " no refusal marker in class source " + case["label"])
    if name == "literal":
        check(document["initializer_proof"]["kind"] == "proved", "literal static initializer group proved")
        check("static byte[] a = new byte[]{10, 20, 30};" in text, "literal a initializer projected")
        check("byte[] b;" in text and "this.b = new byte[]{40, 50, 60};" in text,
              "literal instance b stays at constructor seam")
    else:
        proof = document["initializer_proof"]
        check(proof["kind"] == "refused", "ordered scalar/array static group refused")
        check(proof["reason"] == "RHS of field `trace` is a Java constant expression and would change initialization phase",
              "ordered refusal reason is the explicit trace initialization phase")
        check("static {" in text and "ArrayFieldInitializers.trace = 0;" in text
              and "ArrayFieldInitializers.a = new byte[]{mark(2), mark(3), mark(4)};" in text,
              "ordered full static block retained")


def expected_class_path(base, jdk, class_name):
    return base / "cases" / (jdk + "-original") / "classes" / (class_name + ".class")


def observed_oracle(base):
    manifest = json.loads((base / "manifest.json").read_bytes())
    original = next(case for case in manifest["cases"]
                    if case["kind"] == "original" and case["jdk_leg"] == "javac8")
    stdout = output_path(base, original["runtime"]["stdout"])
    stderr = output_path(base, original["runtime"]["stderr"])
    return {
        "source": original["runtime"]["stdout"]["path"],
        "stdout": stdout.read_text(),
        "stdout_sha256": sha(stdout),
        "stderr_sha256": sha(stderr),
        "exit": original["runtime"]["exit"],
        "provenance": "raw stdout/stderr/exit from fresh original javac8 run; javac23 checked byte-equal; recorded after execution",
    }


def main():
    check(not RESULT.exists(), "verification result is not overwritten")
    jdk_manifest_path = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
    cli_meta_path = ROOT / "openspec/changes/recover-returned-int-array-compound-updates/results/candidate-cli-v2.json"
    verified = {name: verify_baseline(name, base, EXPECTED[name], jdk_manifest_path, cli_meta_path)
                for name, base in BASELINES.items()}
    total_commands = sum(len(manifest["commands"]) for manifest in verified.values())
    total_cases = sum(sum(manifest["case_counts"].values()) for manifest in verified.values())
    check(total_commands == 62, "62 actual commands across both baselines")
    check(total_cases == 16, "16 actual class/JADX/Jarde legs across both baselines")
    result = {
        "status": "accepted",
        "checks": checks,
        "errors": [],
        "baselines": {
            name: {
                "manifest_sha256": sha(base / "manifest.json"),
                "inventory_sha256": sha(base / "file-inventory.json"),
                "commands": len(verified[name]["commands"]),
                "case_counts": verified[name]["case_counts"],
                "success_counts": verified[name]["success_counts"],
            }
            for name, base in BASELINES.items()
        },
        "total_commands": total_commands,
        "total_class_jadx_jarde_legs": total_cases,
        "observed_original_raw_oracles": {
            name: observed_oracle(BASELINES[name]) for name in BASELINES
        },
        "source_map_scope": "per-method allocation, byte-array element stores, putfield/putstatic, helper calls, and constructor invokespecial BCIs are sourced",
        "interpretation": {
            "literal": "static array initializer group proved and projected; instance array remains at constructor; this is a presentation boundary, not an EM-18 gap",
            "ordered": "static group refusal is caused by the explicit trace=0 initialization-phase constraint; all members remain structured and full-source raw execution matches; this is not an array semantic gap",
            "em18": "partial; these two initializer-position slices do not complete EM-18",
            "oracle": "fixed stdout byte strings were read from fresh original runs after execution and are not prior expected values",
        },
    }
    RESULT.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
