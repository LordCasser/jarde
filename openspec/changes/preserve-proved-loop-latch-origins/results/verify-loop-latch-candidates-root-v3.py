#!/usr/bin/env python3
"""Independently accept the frozen Plain and CF07 loop-latch candidate replays.

Read-only: this script never invokes JDK, JADX, Cargo, or the candidate CLI.
It checks the collectors' captured commands, raw files, source maps, and closure.
"""

from __future__ import annotations

from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins"
RESULTS = CHANGE / "results"
PLAIN = RESULTS / "candidate-replay-root-v3"
CF07 = RESULTS / "cf07-candidate-root-v1"
PLAIN_BASELINE_VERIFIER = ROOT / "openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/results/verify-baseline-root-luna-v5.py"
CF07_BASELINE_VERIFIER = ROOT / "openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/verify-baseline-root-v4.py"
CF07_COLLECTOR = CHANGE / "results/prepare-cf07-candidate-root-v9.py"
BUILD_RUNNER = RESULTS / "run-validation-build-root-v8.py"
BUILD_EXECUTION = RESULTS / "validation-build-root-v8/execution.json"
CF07_CLI = Path("/private/tmp/jarde-loop-latch-cli-v1")
CF07_METADATA = RESULTS / "candidate-cli-v1.json"
CF07_TARGETS = {"andWhile(Z)I": 15, "counted(II)I": 30}
CF07_EXPECTED_METHODS = {
    "<init>()V": (1, [0, 1, 4]),
    "andWhile(Z)I": (9, [0, 1, 2, 3, 6, 7, 9, 12, 15, 18, 19]),
    "counted(II)I": (9, [0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 14, 17, 20, 23, 24, 25, 26, 27, 30, 33, 36, 37]),
    "lastIndexOf([IIII)I": (9, [0, 1, 2, 3, 5, 7, 8, 11, 12, 14, 15, 16, 19, 21, 22, 25, 28, 29]),
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def load_module(path: Path, name: str, required: set[str] = frozenset()):
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot load verifier helper: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    missing = required - set(dir(module))
    require(not missing, f"imported helper API is missing {sorted(missing)}: {path}")
    return module


def closed_inventory(bundle: Path) -> tuple[dict, dict[str, dict]]:
    manifest_path, inventory_path = bundle / "manifest.json", bundle / "file-inventory.json"
    manifest = json.loads(manifest_path.read_bytes())
    inventory = json.loads(inventory_path.read_bytes())
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory), f"duplicate inventory path: {bundle}")
    actual = {path.relative_to(bundle).as_posix(): path for path in bundle.rglob("*")
              if path.is_file() and path != inventory_path}
    require(set(rows) == set(actual), f"inventory is not closed: {bundle}")
    for relative, path in actual.items():
        raw = path.read_bytes()
        row = rows[relative]
        require(row["bytes"] == len(raw) and row["sha256"] == sha(raw),
                f"inventory digest mismatch: {bundle}/{relative}")
    return manifest, rows


def ref_bytes(bundle: Path, ref: dict) -> bytes:
    path = bundle / ref["path"]
    raw = path.read_bytes()
    require(ref["bytes"] == len(raw) and ref["sha256"] == sha(raw), f"file record mismatch: {path}")
    return raw


def stream_bytes(bundle: Path, command: dict, key: str) -> bytes:
    return ref_bytes(bundle, command["streams"][key])


def plain_method_key(item: dict) -> str:
    identity = item["identity"]
    name = bytes(identity["name"]).decode("utf-8")
    descriptor = bytes(identity["descriptor"]).decode("ascii")
    require(name == bytes(item["name"]["raw"]).decode("utf-8")
            and descriptor == bytes(item["descriptor"]["raw"]).decode("ascii"),
            "Plain method identity/name/descriptor disagree")
    return name + descriptor


def document_methods(document: dict, key_fn) -> dict[str, dict]:
    result = {}
    for wrapper in document.get("methods", []):
        key = key_fn(wrapper["item"])
        require(key not in result, f"duplicate method in document: {key}")
        result[key] = wrapper
    return result


def verify_plain_document(document: dict, baseline_document: dict, class_bytes: bytes,
                          physical: dict, label: str, old: object) -> None:
    require(document.get("text") == baseline_document.get("text")
            and document.get("fields") == [] and document.get("execution", {}).get("status") == "complete",
            f"Plain complete class text/shape changed: {label}")
    methods = document_methods(document, plain_method_key)
    baseline_methods = document_methods(baseline_document, plain_method_key)
    expected_keys = set(old.METHOD_ORDER)
    require(set(methods) == expected_keys == set(baseline_methods), f"Plain physical method set differs: {label}")
    owner = document["class"]
    class_digest = old.blake3.blake3(class_bytes).hexdigest()
    require(owner.get("class_bytes") == {"digest": class_digest, "length": len(class_bytes)}
            and owner.get("location") == {"kind": "standalone_root", "snapshot": class_digest}
            and owner.get("variant") == {"kind": "base"}, f"Plain candidate owner mismatch: {label}")
    for key in old.METHOD_ORDER:
        wrapper, base = methods[key], baseline_methods[key]
        item, base_item = wrapper["item"], base["item"]
        report, base_report = wrapper.get("outcome", {}).get("report"), base.get("outcome", {}).get("report")
        require(isinstance(report, dict) and isinstance(base_report, dict), f"Plain method report absent: {label}/{key}")
        require(item.get("identity", {}).get("owner") == owner
                and item.get("access_flags") == physical[key]["access_flags"]
                and item.get("index") == base_item.get("index")
                and item.get("identity") == base_item.get("identity"),
                f"Plain method physical identity/flags/index mismatch: {label}/{key}")
        require(report.get("method") == key and report.get("text") == base_report.get("text")
                and (report.get("quality"), report.get("representation"), report.get("content"), report.get("fallbacks"))
                == (base_report.get("quality"), base_report.get("representation"), base_report.get("content"), base_report.get("fallbacks")),
                f"Plain method text/presentation differs: {label}/{key}")
        physical_bcis = set(physical[key]["instructions"])
        mapped, text_bytes = set(), report["text"].encode("utf-8")
        for segment in report.get("source_map", {}).get("segments", []):
            start, end = segment.get("start"), segment.get("end")
            require(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(text_bytes),
                    f"Plain source span invalid: {label}/{key}")
            origin = segment.get("origin", {})
            points = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
            for point in points:
                method = point.get("method", {})
                require(method.get("owner") == owner
                        and bytes(method.get("name", [])).decode("utf-8")
                        + bytes(method.get("descriptor", [])).decode("ascii") == key
                        and point.get("bci") in physical_bcis,
                        f"Plain origin does not bind to physical owner/method/BCI: {label}/{key}")
                mapped.add(point["bci"])
        if key != "noPrefix(ZI)I":
            require(report.get("source_map") == base_report.get("source_map"),
                    f"Plain non-target source map changed: {label}/{key}")
            continue
        before, after = cf07_point_facts(base_report), cf07_point_facts(report)
        require(Counter((s.get("start"), s.get("end")) for s in base_report["source_map"]["segments"])
                == Counter((s.get("start"), s.get("end")) for s in report["source_map"]["segments"]),
                f"Plain noPrefix source spans changed: {label}")
        require(not (before - after), f"Plain noPrefix removed or changed an existing origin: {label}")
        added = after - before
        require(len(list(added.elements())) == 1, f"Plain noPrefix delta is not one origin: {label}")
        fact = next(iter(added.elements()))
        span = text_bytes[fact[0]:fact[1]].decode("utf-8")
        require(fact[2] == "derived" and fact[3] == tuple(b"noPrefix")
                and fact[4] == tuple(b"(ZI)I") and fact[5] == class_digest
                and fact[6] == len(class_bytes) and fact[7] == "standalone_root"
                and fact[8] == class_digest and json.loads(fact[9]) == {"kind": "base"}
                and fact[10] == 14 and span.strip().startswith("while (") and span.rstrip().endswith("}"),
                f"Plain sole added origin is not derived noPrefix@14 on its while statement: {label}")
        require(mapped == physical_bcis, f"Plain noPrefix physical BCI coverage is incomplete: {label}")


def verify_plain(old: object) -> dict:
    baseline = old.verify_candidate_baseline()
    require(baseline.get("verified") is True, "accepted one-arm baseline verifier did not accept")
    baseline_manifest = old.read_json(old.BASE / "manifest.json")
    manifest, rows = closed_inventory(PLAIN)
    require(manifest.get("schema") == "preserve-proved-loop-latch-origins-candidate-replay-root-v3"
            and manifest.get("status") == "completed" and manifest.get("failures") == [],
            "Plain candidate manifest schema/status/failures differ")
    summary = json.loads((PLAIN / "summary.json").read_bytes())
    require(summary.get("schema") == "preserve-proved-loop-latch-origins-candidate-summary-root-v3"
            and summary.get("status") == "completed" and summary.get("failures") == []
            and summary.get("candidate_case_counts") == {"original": 2, "jadx": 4, "jarde": 4},
            "Plain candidate summary differs")
    require(manifest.get("candidate_case_counts") == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest.get("captured_command_count") == 31,
            "Plain candidate matrix/command count differs")
    cli = manifest["candidate_cli"]
    cli_path, metadata_path = Path(cli["path"]), Path(cli["metadata_path"])
    cli_sha, metadata_sha = sha(cli_path.read_bytes()), sha(metadata_path.read_bytes())
    require(cli_sha == cli["sha256"] and metadata_sha == cli["metadata_sha256"],
            "Plain frozen candidate CLI or metadata pin differs")
    metadata = json.loads(metadata_path.read_bytes())
    require(metadata.get("cli_path") == str(cli_path) and metadata.get("cli_sha256") == cli_sha,
            "Plain candidate metadata does not bind the CLI")
    original_source, original_runner = old.verify_source_inputs(baseline_manifest)
    require((PLAIN / "original-sources/PlainOneArmLoops.java").read_bytes() == original_source
            and (PLAIN / "original-sources/Runner.java").read_bytes() == original_runner,
            "Plain frozen original source/Runner differs from accepted input")

    commands = {row["label"]: row for row in manifest["commands"]}
    require(len(commands) == 31, "Plain command labels are not unique")
    # Fresh original and JADX legs must preserve the accepted compile/runtime oracle.
    for case in manifest["cases"]:
        label, kind, leg = case["label"], case["kind"], case["jdk_leg"]
        compile_cmd = case["compile"]
        require(commands[compile_cmd["label"]] == compile_cmd, f"Plain compile command detached: {label}")
        argv = compile_cmd["argv"]
        tool = baseline_manifest["jdk_legs"][leg]["tools"]["javac"]["path"]
        require(argv[0] == tool and argv[1:6] == ["-source", "8", "-target", "8", "-g:none"],
                f"Plain compile tool/options mismatch: {label}")
        require(argv[6] == "-Xlint:-options" and argv[7] == "-classpath"
                and argv[9] == "-sourcepath" and argv[8] == argv[10],
                f"Plain compile isolation options differ: {label}")
        cp = Path(argv[argv.index("-classpath") + 1])
        require(cp == Path(case["empty_classpath_sourcepath"])
                == (PLAIN / "cases" / label / "empty-classpath-sourcepath").resolve()
                and cp.is_dir() and not any(cp.iterdir()),
                f"Plain classpath/sourcepath not fresh and empty: {label}")
        output = Path(argv[argv.index("-d") + 1])
        require(output == Path(case["class_output"]) == (PLAIN / "cases" / label / "classes").resolve()
                and output.is_dir(), f"Plain fresh output absent: {label}")
        d_index = argv.index("-d")
        expected_sources = [str((PLAIN / ref["path"]).resolve()) for ref in case["source_files"]]
        require(argv == [tool, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                         "-classpath", str(cp), "-sourcepath", str(cp), "-d", str(output), *expected_sources],
                f"Plain compile argv/source set is not exact: {label}")
        require(len(argv[d_index + 2:]) == len(case["source_files"]),
                f"Plain compile source count differs: {label}")
        if kind in ("original", "jadx"):
            require(compile_cmd["exit"] == 0 and case.get("compile_success") is True,
                    f"Plain oracle compile failed: {label}")
            runtime = case["runtime"]
            require(runtime is not None and commands[runtime["label"]] == runtime and runtime["exit"] == 0,
                    f"Plain oracle runtime absent/failed: {label}")
            require(runtime["argv"][0] == baseline_manifest["jdk_legs"][leg]["tools"]["java"]["path"]
                    and runtime["argv"][1:4] == ["-Xverify:all", "-cp", str(output)]
                    and runtime["argv"][-1], f"Plain runtime verifier/classpath missing: {label}")
            expected_names = {"PlainOneArmLoops.class", "Runner.class"}
            require(case.get("complete_class_set") is True and case.get("classes")
                    and {Path(row["path"]).name for row in case["classes"]} == expected_names,
                    f"Plain fresh class set differs: {label}")
            for class_ref in case["classes"]:
                ref_bytes(PLAIN, class_ref)
            if kind == "original":
                frozen = next(row for row in baseline_manifest["cases"] if row["label"] == label)
                require(ref_bytes(PLAIN, case["source_files"][0]) == original_source
                        and ref_bytes(PLAIN, case["source_files"][1]) == original_runner
                        and ref_bytes(PLAIN, case["actual_class"]) == old.recorded_bytes(old.BASE, frozen["actual_class"]),
                        f"Plain original fresh source/class differs from accepted oracle: {label}")
                javap_row = case["javap"]
                javap_cmd = javap_row["command"]
                expected_javap = [baseline_manifest["jdk_legs"][leg]["tools"]["javap"]["path"],
                                  "-p", "-c", "-s", "-v", str((PLAIN / case["actual_class"]["path"]).resolve())]
                require(javap_cmd["argv"] == expected_javap and javap_cmd["exit"] == 0
                        and ref_bytes(PLAIN, javap_row["text"]) == stream_bytes(PLAIN, javap_cmd, "stdout"),
                        f"Plain fresh javap command/output differs: {label}")
                javap_text = stream_bytes(PLAIN, javap_cmd, "stdout").decode("utf-8", errors="strict")
                require(re.search(r"(?m)^\s*14:\s+goto\s+6\s*$", javap_text) is not None,
                        f"Plain physical noPrefix latch is not goto 6 at BCI14: {label}")
            ref_baseline = next(item for item in old.read_json(old.BASE / "manifest.json")["cases"]
                                if item["label"] == label)
            require(stream_bytes(PLAIN, runtime, "stdout") == old.recorded_bytes(old.BASE, ref_baseline["runtime"]["streams"]["stdout"])
                    and stream_bytes(PLAIN, runtime, "stderr") == old.recorded_bytes(old.BASE, ref_baseline["runtime"]["streams"]["stderr"]),
                    f"Plain oracle runtime raw differs from accepted baseline: {label}")
        else:
            mode = case["evidence_mode"]
            observation = case.get("whole_class_observation", {})
            require(compile_cmd["exit"] != 0 and case.get("compile_success") is False
                    and case.get("runtime") is None and observation.get("runtime_attempted") is False,
                    f"Plain whole-class failure/runtime boundary differs: {label}")
            require(not list(output.rglob("*.class")), f"Plain failed compile left class files: {label}")
            before = next(item for item in old.read_json(old.BASE / "manifest.json")["cases"]
                          if item["label"] == label)
            require(compile_cmd["exit"] == before["compile"]["exit"]
                    and stream_bytes(PLAIN, compile_cmd, "stdout")
                    == old.recorded_bytes(old.BASE, before["compile"]["streams"]["stdout"]),
                    f"Plain whole-class compile exit/stdout differs from baseline: {label}")
            before_err = old.recorded_bytes(old.BASE, before["compile"]["streams"]["stderr"])
            got_err = stream_bytes(PLAIN, compile_cmd, "stderr")
            normalize_path = lambda value: re.sub(rb"(?m)^.*PlainOneArmLoops\.java:", b"<PlainOneArmLoops.java>:", value)
            require(normalize_path(got_err) == normalize_path(before_err),
                    f"Plain expected compile diagnostic changed: {label}/{mode}")
    # Independently recompute each rendered document's physical origin delta.
    base_manifest = old.read_json(old.BASE / "manifest.json")
    for leg in ("javac8", "javac23"):
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = next(row for row in manifest["cases"] if row["label"] == label)
            render_row = next(row for row in manifest["render_profiles"]
                              if row["label"] == f"{leg}-jarde-render")
            profile = render_row["profiles"][mode]
            raw = ref_bytes(PLAIN, profile["document"])
            doc = json.loads(raw)
            baseline_case = next(row for row in old.read_json(old.BASE / "manifest.json")["cases"]
                                 if row["label"] == label)
            baseline_doc = json.loads(old.recorded_bytes(old.BASE, baseline_case["rendered_profile"]["document"]))
            javap_row = next(row for row in manifest["cases"] if row["label"] == f"{leg}-original")
            javap = old.parse_javap((PLAIN / javap_row["javap"]["text"]["path"]).read_text())
            input_bytes = ref_bytes(PLAIN, javap_row["actual_class"])
            profile_command = profile["command"]
            expected_render = [str(cli_path), "class-source", "--input",
                               str((PLAIN / javap_row["actual_class"]["path"]).resolve()), "--class",
                               "PlainOneArmLoops", "--policy", "single-class", "--release", "8",
                               "--format", "json"]
            if mode == "all":
                expected_render += ["--evidence", "all"]
            require(profile_command["argv"] == expected_render and profile_command["exit"] == 0
                    and commands[profile_command["label"]] == profile_command
                    and stream_bytes(PLAIN, profile_command, "stdout") == raw,
                    f"Plain render CLI/argv/raw report binding differs: {label}")
            verify_plain_document(doc, baseline_doc, input_bytes, javap, label, old)
    return {"closed_files": len(rows), "captured_commands": len(commands),
            "candidate_cli_sha256": cli_sha, "candidate_metadata_sha256": metadata_sha,
            "full_class_compile": "four failures, zero runtime", "new_origin": "noPrefix@14 derived while",
            "all_existing_method_text_and_other_maps_equal": True}


def cf07_point_facts(report: dict) -> Counter:
    result = []
    for segment in report.get("source_map", {}).get("segments", []):
        origin = segment.get("origin", {})
        points = ([('primary', origin["primary"])] if origin.get("primary") is not None else [])
        points.extend(("derived", point) for point in origin.get("derived", []))
        for role, point in points:
            method, owner = point["method"], point["method"]["owner"]
            result.append((segment["start"], segment["end"], role, tuple(method["name"]),
                           tuple(method["descriptor"]), owner["class_bytes"]["digest"],
                           owner["class_bytes"]["length"], owner["location"]["kind"],
                           owner["location"]["snapshot"], json.dumps(owner["variant"], sort_keys=True,
                           separators=(",", ":")), point["bci"], point["provenance"],
                           json.dumps(point.get("cp"), sort_keys=True, separators=(",", ":"))))
    return Counter(result)


def verify_cf07() -> dict:
    old = load_module(CF07_BASELINE_VERIFIER, "cf07_accepted_baseline_verifier",
                      {"EXPECTED_METHODS", "check_ref", "command_stream", "method_key",
                       "parse_javap", "read_json", "BUNDLE", "blake3"})
    collector = load_module(CF07_COLLECTOR, "cf07_candidate_collector_helpers",
                            {"close_baseline", "BASELINE", "read_frozen_candidate"})
    baseline = collector.close_baseline()
    manifest, inventory = closed_inventory(CF07)
    require(manifest.get("schema") == "cf07-loop-latch-candidate-replay-root-v9"
            and manifest.get("status") == "candidate-replay-observed"
            and manifest.get("failures") == [], "CF07 candidate manifest schema/status/failures differ")
    observation = json.loads((CF07 / "candidate-observation.json").read_bytes())
    require(observation.get("schema") == "cf07-loop-latch-candidate-observation-root-v1"
            and observation.get("status") == "candidate_replay_observed",
            "CF07 observation schema/status differs")
    metadata_raw = CF07_METADATA.read_bytes()
    metadata = json.loads(metadata_raw)
    cli_raw = CF07_CLI.read_bytes()
    cli_sha, metadata_sha = sha(cli_raw), sha(metadata_raw)
    execution_raw = BUILD_EXECUTION.read_bytes()
    execution = json.loads(execution_raw)
    require(metadata.get("cli_path") == str(CF07_CLI) and metadata.get("cli_sha256") == cli_sha
            and metadata.get("build_result_sha256") == sha(execution_raw),
            "CF07 CLI/metadata/build result binding differs")
    pins = {key: metadata[key] for key in ("candidate_sources", "test_sources", "canonical_files")}
    for group, paths in pins.items():
        require(isinstance(paths, dict) and paths, f"CF07 metadata missing {group}")
        for relative, digest in paths.items():
            require(sha((ROOT / relative).read_bytes()) == digest, f"CF07 source pin differs: {group}/{relative}")
    freeze = execution.get("freeze", {})
    require(execution.get("schema") == "preserve-proved-loop-latch-validation-build-root-v8"
            and execution.get("status") == "validation-passed-cli-frozen"
            and execution.get("guards") == {"minimum_free_bytes": 5 * 1024**3,
                                             "maximum_target_bytes": 1024**3}
            and execution.get("validation_runner") == {"path": str(BUILD_RUNNER.resolve()),
                                                          "sha256": sha(BUILD_RUNNER.read_bytes())}
            and execution.get("preflight", {}).get("source_pins_before") == pins
            and execution.get("preflight", {}).get("source_pins_after") == pins
            and freeze.get("cli_path") == str(CF07_CLI) and freeze.get("cli_sha256") == cli_sha
            and freeze.get("metadata_path") == str(CF07_METADATA)
            and metadata.get("uncommitted_loop_latch_product") is True,
            "CF07 build v8 freeze/execution proof does not bind candidate")
    build_commands = execution.get("commands", [])
    require(len(build_commands) == 12 and all(row.get("exit_code") == 0
            and row.get("guard_stop") is None
            and row.get("peak_target_bytes", 1024**3 + 1) <= 1024**3
            and row.get("free_bytes_after", 0) >= 5 * 1024**3
            for row in build_commands),
            "CF07 guarded build execution is incomplete or has a failed validation command")
    require(observation.get("candidate_cli_sha256") == cli_sha
            and observation.get("metadata_sha256") == metadata_sha
            and observation.get("build_execution", {}).get("sha256") == sha(execution_raw),
            "CF07 candidate observation CLI/metadata/build pins differ")
    require(manifest.get("command_count") == 29 and len(manifest.get("cases", [])) == 10
            and manifest.get("case_counts") == {"original": 2, "jadx": 4, "jarde": 4},
            "CF07 raw matrix counts differ")

    # Reuse the accepted parser and compare fresh raw Jarde executions to exact same-JDK originals.
    old_methods = old.EXPECTED_METHODS
    commands = {row["label"]: row for row in manifest["commands"]}
    require(len(commands) == 29, "CF07 command labels are not unique")
    cases = {row["label"]: row for row in manifest["cases"]}
    baseline_cases = {row["label"]: row for row in json.loads((collector.BASELINE / "manifest.json").read_bytes())["cases"]}
    for leg in ("javac8", "javac23"):
        original = cases[f"{leg}-original"]
        base_original = baseline_cases[f"{leg}-original"]
        wrapped_original = original.get("compiled_case", original)
        base_wrapped_original = base_original.get("compiled_case", base_original)
        require([ref_bytes(CF07, row) for row in wrapped_original["source_files"]]
                == [old.check_ref(row) for row in base_wrapped_original["source_files"]]
                and ref_bytes(CF07, original["actual_class"]) == old.check_ref(base_original["actual_class"]),
                f"CF07 fresh original source/class differs from accepted oracle: {leg}")
        oracle = {name: stream_bytes(CF07, original["runtime"], name) for name in ("stdout", "stderr")}
        require(all(oracle[name] == old.command_stream(base_original["runtime"], name)
                    for name in ("stdout", "stderr")),
                f"CF07 fresh original raw differs from accepted same-JDK oracle: {leg}")
        javap_fact = next(row for row in manifest["original_physical_facts"]
                          if row["command"]["label"] == f"{leg}-original-javap")
        javap_ref = javap_fact["text"]
        parsed = old.parse_javap(CF07 / javap_ref["path"])
        require(parsed["physical_field_count"] == 0 and set(parsed["methods"]) == set(old_methods),
                f"CF07 fresh physical member set differs: {leg}")
        for key, (flags, bcis) in old_methods.items():
            physical = parsed["methods"][key]
            require(physical["flags"] == flags and [i["bci"] for i in physical["instructions"]] == bcis,
                    f"CF07 fresh physical javap facts differ: {leg}/{key}")
        javap_command = javap_fact["command"]
        require(javap_command["argv"] == [manifest["jdk_legs"][leg]["tools"]["javap"]["path"],
                "-p", "-c", "-s", "-v", str((CF07 / original["actual_class"]["path"]).resolve())]
                and javap_command["exit"] == 0
                and stream_bytes(CF07, javap_command, "stdout") == ref_bytes(CF07, javap_ref),
                f"CF07 physical javap argv/raw differs: {leg}")
        for kind, profile in (("original", None),
                              *(('jadx', profile) for profile in ("default", "none")),
                              *(('jarde', profile) for profile in ("default", "all"))):
            label = f"{leg}-original" if kind == "original" else f"{leg}-{kind}-{profile}"
            case = cases[label]
            wrapped = case.get("compiled_case", case)
            base_case = baseline_cases[label]
            base_wrapped = base_case.get("compiled_case", base_case)
            require([ref_bytes(CF07, row) for row in wrapped["source_files"]]
                    == [old.check_ref(row) for row in base_wrapped["source_files"]],
                    f"CF07 fresh source inputs differ from accepted baseline: {label}")
            comp, runtime = wrapped["compile"], wrapped["runtime"]
            require(comp["exit"] == runtime["exit"] == 0 and commands[comp["label"]] == comp
                    and commands[runtime["label"]] == runtime, f"CF07 fresh compile/runtime failed: {label}")
            argv = comp["argv"]
            require(argv[0] == manifest["jdk_legs"][leg]["tools"]["javac"]["path"]
                    and argv[1:7] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"],
                    f"CF07 compile argv tool/options differ: {label}")
            cp = Path(argv[8])
            require(argv[7] == "-classpath" and argv[9] == "-sourcepath" and argv[8] == argv[10]
                    and cp == (CF07 / "cases" / label / "empty-classpath-sourcepath").resolve()
                    and cp.is_dir() and not any(cp.iterdir()), f"CF07 empty compile isolation differs: {label}")
            output = Path(argv[argv.index("-d") + 1])
            require(output == Path(wrapped["class_output"]) == (CF07 / "cases" / label / "classes").resolve()
                    and output.is_dir(), f"CF07 fresh class output absent: {label}")
            expected_sources = [str((CF07 / ref["path"]).resolve()) for ref in wrapped["source_files"]]
            tool = manifest["jdk_legs"][leg]["tools"]["javac"]["path"]
            require(argv == [tool, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                             "-classpath", str(cp), "-sourcepath", str(cp), "-d", str(output), *expected_sources],
                    f"CF07 compile argv/source set is not exact: {label}")
            actual_classes = {path.relative_to(output).as_posix() for path in output.rglob("*.class")}
            recorded_classes = {str((CF07 / ref["path"]).relative_to(output)) for ref in wrapped["classes"]}
            require(actual_classes == recorded_classes == set(wrapped["actual_class_paths"])
                    == set(wrapped["expected_class_paths"]), f"CF07 fresh class set differs: {label}")
            for class_ref in wrapped["classes"]:
                ref_bytes(CF07, class_ref)
            require(runtime["argv"] == [manifest["jdk_legs"][leg]["tools"]["java"]["path"],
                    "-Xverify:all", "-cp", str(output), "cf07.Runner"], f"CF07 runtime argv differs: {label}")
            require(stream_bytes(CF07, runtime, "stdout") == oracle["stdout"]
                    and stream_bytes(CF07, runtime, "stderr") == oracle["stderr"],
                    f"CF07 raw runtime differs from same-JDK original: {label}")
    # The candidate delta is recomputed from the document, never accepted from its summary.
    for leg in ("javac8", "javac23"):
        physical_fact = next(row for row in manifest["original_physical_facts"]
                             if row["command"]["label"] == f"{leg}-original-javap")
        leg_physical = old.parse_javap(CF07 / physical_fact["text"]["path"])["methods"]
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = cases[label]
            doc_ref = case["rendered_profile"]["document"]
            doc = json.loads(ref_bytes(CF07, doc_ref))
            render = case["rendered_profile"]["command"]
            original_path = str((CF07 / cases[f"{leg}-original"]["actual_class"]["path"]).resolve())
            expected_render_argv = [str(CF07_CLI), "class-source", "--input", original_path,
                                    "--class", "cf07.LoopCases", "--policy", "single-class",
                                    "--release", "8", "--format", "json"]
            if mode == "all":
                expected_render_argv += ["--evidence", "all"]
            require(render["argv"] == expected_render_argv and render["exit"] == 0
                    and commands[render["label"]] == render
                    and stream_bytes(CF07, render, "stdout") == ref_bytes(CF07, doc_ref),
                    f"CF07 render CLI/argv/raw document binding differs: {label}")
            base_case = next(row for row in json.loads((collector.BASELINE / "manifest.json").read_bytes())["cases"]
                             if row["label"] == label)
            base_doc = json.loads(old.check_ref(base_case["rendered_profile"]["document"]))
            fresh_methods = document_methods(doc, old.method_key)
            base_methods = document_methods(base_doc, old.method_key)
            require(set(fresh_methods) == set(CF07_EXPECTED_METHODS) == set(base_methods),
                    f"CF07 candidate method identities differ: {label}")
            original = cases[f"{leg}-original"]
            original_raw = ref_bytes(CF07, original["actual_class"])
            digest = old.blake3.blake3(original_raw).hexdigest()
            owner = doc["class"]
            require(owner.get("class_bytes") == {"digest": digest, "length": len(original_raw)}
                    and owner.get("location") == {"kind": "standalone_root", "snapshot": digest}
                    and owner.get("variant") == {"kind": "base"},
                    f"CF07 candidate class owner does not bind fresh physical class: {label}")
            for method_id, wrapper in fresh_methods.items():
                report = wrapper["outcome"]["report"]
                baseline_report = base_methods[method_id]["outcome"]["report"]
                require(report["text"] == baseline_report["text"]
                        and (report.get("quality"), report.get("representation"), report.get("content"), report.get("fallbacks"))
                        == (baseline_report.get("quality"), baseline_report.get("representation"), baseline_report.get("content"), baseline_report.get("fallbacks")),
                        f"CF07 method text/presentation changed: {label}/{method_id}")
                fresh_origins, baseline_origins = cf07_point_facts(report), cf07_point_facts(baseline_report)
                require(not (baseline_origins - fresh_origins), f"CF07 existing physical origin removed: {label}/{method_id}")
                physical_bcis = {instruction["bci"] for instruction in leg_physical[method_id]["instructions"]}
                item = wrapper["item"]
                identity = item.get("identity", {})
                if "member" in identity:
                    identity = identity["member"]
                require(old.method_key(wrapper["item"]) == method_id
                        and item.get("access_flags") == leg_physical[method_id]["flags"]
                        and identity.get("owner") == owner,
                        f"CF07 method name/descriptor/flags/owner mismatch: {label}/{method_id}")
                text_bytes = report["text"].encode("utf-8")
                for segment in report.get("source_map", {}).get("segments", []):
                    start, end = segment.get("start"), segment.get("end")
                    require(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(text_bytes),
                            f"CF07 source span is invalid: {label}/{method_id}")
                    origin = segment.get("origin", {})
                    points = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
                    for point in points:
                        method_fact = point.get("method", {})
                        require(method_fact.get("owner") == owner
                                and bytes(method_fact.get("name", [])).decode("utf-8")
                                + bytes(method_fact.get("descriptor", [])).decode("ascii") == method_id
                                and point.get("bci") in physical_bcis,
                                f"CF07 origin does not bind to physical owner/method/BCI: {label}/{method_id}")
                delta = fresh_origins - baseline_origins
                target_bci = CF07_TARGETS.get(method_id)
                if target_bci is None:
                    require(not delta and report["source_map"] == baseline_report["source_map"],
                            f"CF07 out-of-scope map changed: {label}/{method_id}")
                else:
                    require(len(list(delta.elements())) == 1, f"CF07 expected one added loop origin: {label}/{method_id}")
                    fact = next(iter(delta.elements()))
                    target_span = report["text"].encode()[fact[0]:fact[1]].decode()
                    require(fact[2] == "derived" and fact[10] == target_bci
                            and target_span.strip().startswith("while (") and target_span.rstrip().endswith("}")
                            and any(p.get("bci") == target_bci for seg in report["source_map"]["segments"]
                                    for p in seg.get("origin", {}).get("derived", [])),
                            f"CF07 new origin is not derived target BCI on while span: {label}/{method_id}")
    for leg in ("javac8", "javac23"):
        docs = [json.loads(ref_bytes(CF07, cases[f"{leg}-jarde-{mode}"]["rendered_profile"]["document"]))
                for mode in ("default", "all")]
        require(docs[0]["text"] == docs[1]["text"], f"CF07 default/all full class text differs: {leg}")
        for mode_doc in docs:
            require(mode_doc["text"] == base_doc_text(collector, old, leg), f"CF07 generated full class changed: {leg}")
        dm, am = document_methods(docs[0], old.method_key), document_methods(docs[1], old.method_key)
        for key in CF07_EXPECTED_METHODS:
            require(dm[key]["outcome"]["report"]["text"] == am[key]["outcome"]["report"]["text"]
                    and dm[key]["outcome"]["report"]["source_map"] == am[key]["outcome"]["report"]["source_map"],
                    f"CF07 default/all method report differs: {leg}/{key}")
    return {"closed_files": len(inventory), "commands": len(commands), "cases": 10,
            "candidate_cli_sha256": cli_sha, "metadata_sha256": metadata_sha,
            "build_execution_sha256": sha(execution_raw), "whole_class": "2 original + 4 JADX + 4 Jarde all successful",
            "derived_loop_origins": {"javac8": sorted(CF07_TARGETS.items()), "javac23": sorted(CF07_TARGETS.items())},
            "default_all_equal": True, "existing_origins_preserved": True}


def base_doc_text(collector: object, old: object, leg: str) -> str:
    manifest = json.loads((collector.BASELINE / "manifest.json").read_bytes())
    case = next(row for row in manifest["cases"] if row["label"] == f"{leg}-jarde-default")
    return json.loads(old.check_ref(case["rendered_profile"]["document"]))["text"]


def main() -> int:
    try:
        plain_old = load_module(PLAIN_BASELINE_VERIFIER, "plain_accepted_baseline_verifier",
                                {"METHOD_ORDER", "EXPECTED_BCIS", "parse_javap", "read_json",
                                 "recorded_bytes", "verify_candidate_baseline", "verify_source_inputs",
                                 "BASE", "blake3"})
        result = {"schema": "preserve-proved-loop-latch-independent-acceptance-root-v3",
                  "status": "accepted", "verified": True,
                  "plain": verify_plain(plain_old), "cf07": verify_cf07(),
                  "evidence_scope": "closed candidate inventories; frozen CLI/metadata/build-v8 pins; replay argv and raw compile/runtime streams; exact method text and physical origin deltas"}
    except Exception as error:
        result = {"schema": "preserve-proved-loop-latch-independent-acceptance-root-v3",
                  "status": "rejected", "verified": False,
                  "failure": f"{type(error).__name__}: {error}"}
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["verified"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
