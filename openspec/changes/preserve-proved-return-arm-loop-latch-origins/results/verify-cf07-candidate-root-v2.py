#!/usr/bin/env python3
"""Independently verify the return-arm latch CF07 replay without running toolchains."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import stat
import sys


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
CHANGE = ROOT / "openspec/changes/preserve-proved-return-arm-loop-latch-origins"
RESULTS = CHANGE / "results"
BUNDLE = RESULTS / "cf07-candidate-root-v1"
ACCEPTANCE = RESULTS / "cf07-candidate-acceptance-root-v1.json"
WRAPPER = RESULTS / "prepare-cf07-candidate-root-v1.py"
FOR_VERIFIER = ROOT / "openspec/changes/preserve-proved-for-latch-origins/results/verify-cf07-candidate-root-v2.py"
FOR_VERIFIER_SHA256 = "bd9690976089431103802b3b96de9c168419b59c6cd51e6b8eb74ed9133759ee"
IF_BUNDLE = ROOT / "openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1"
IF_ACCEPTANCE = ROOT / "openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-acceptance-root-v1.json"
IF_MANIFEST_SHA256 = "d0f6bc9f596105ce7555240c40060146f52ee88298b55a1b5c2008a96e938c7e"
IF_INVENTORY_SHA256 = "79331dc0c3de8fbfb487a17d8df2e64d5269ed3b75ec56528e40191b6798906a"
IF_ACCEPTANCE_SHA256 = "229c42b680d51fb8c4add5ea6a29835d35d02de2ffd5a91422ef539535c7545f"
GUARD_SHA256 = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def load_for_verifier():
    raw = FOR_VERIFIER.read_bytes()
    if sha(raw) != FOR_VERIFIER_SHA256:
        raise RuntimeError("accepted For CF07 verifier source SHA changed")
    spec = importlib.util.spec_from_file_location("accepted_for_cf07_verifier_for_return_latch", FOR_VERIFIER)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load accepted For CF07 verifier")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_build(module, args) -> tuple[dict, dict, dict]:
    require = module.require
    require(args.metadata_schema.startswith("preserve-proved-return-arm-loop-latch-origins-candidate-cli-v"),
            "metadata schema must belong to the return-arm latch change")
    require(re.fullmatch(r"preserve-proved-return-arm-loop-latch-origins-validation-build-root-v[1-9][0-9]*",
                         args.build_schema) is not None,
            "build schema must belong to the return-arm latch change")
    require(args.product_flag.startswith("uncommitted_") and "return" in args.product_flag
            and "latch" in args.product_flag and args.product_flag.endswith("_product"),
            "product marker key must name this uncommitted return-arm latch product")
    cli = Path(args.cli).resolve(strict=True)
    metadata_path = Path(args.metadata).resolve(strict=True)
    build_path = args.build.resolve(strict=True)
    cli_raw, meta_raw, build_raw = cli.read_bytes(), metadata_path.read_bytes(), build_path.read_bytes()
    require(sha(cli_raw) == args.cli_sha256 and stat.S_IMODE(cli.stat().st_mode) == 0o555,
            "actual new CLI SHA/mode differs from required inputs")
    require(sha(meta_raw) == args.metadata_sha256, "actual new metadata SHA differs from required input")
    require(build_path.name == "execution.json" and build_path.parent.name.startswith("validation-build-root-v"),
            "--build must name validation-build-root-vN/execution.json")
    try:
        metadata_path.relative_to(RESULTS.resolve())
        build_path.relative_to(RESULTS.resolve())
    except ValueError as error:
        raise RuntimeError("--metadata and --build must be inside the new change results directory") from error
    build_n = build_path.parent.name.removeprefix("validation-build-root-v")
    require(build_n.isdecimal() and int(build_n) > 0, "invalid validation-build version")
    runner = RESULTS / f"run-validation-build-root-v{build_n}.py"
    require(runner.is_file() and sha(runner.read_bytes()) == args.runner_sha256,
            "actual change runner path/SHA differs from required input")
    metadata, execution = json.loads(meta_raw), json.loads(build_raw)
    require(metadata.get("schema") == args.metadata_schema
            and metadata.get("cli_path") == str(cli)
            and metadata.get("cli_sha256") == args.cli_sha256
            and metadata.get("source_commit_base") == args.source_base
            and metadata.get(args.product_flag) is True
            and metadata.get("build_result_sha256") == sha(build_raw),
            "metadata does not bind the supplied CLI/source base/build/product flag")
    require(re.fullmatch(r"[0-9a-f]{40}", args.source_base) is not None,
            "source base must be an actual 40-character commit")
    require(execution.get("schema") == args.build_schema
            and execution.get("schema", "").endswith(f"root-v{build_n}")
            and execution.get("status") == "validation-passed-cli-frozen"
            and execution.get("source_commit_base_expected") == args.source_base
            and execution.get(args.product_flag) is True
            and execution.get("validation_runner") == {"path": str(runner.resolve()), "sha256": args.runner_sha256}
            and metadata.get("validation_runner") == execution.get("validation_runner")
            and metadata.get("guarded_runner_template") == execution.get("guarded_runner_template")
            and execution.get("guarded_runner_template", {}).get("sha256") == GUARD_SHA256
            and sha(Path(execution["guarded_runner_template"]["path"]).read_bytes()) == GUARD_SHA256,
            "build schema/status/runner/base does not match explicit frozen inputs")
    require(execution.get("guards") == {"minimum_free_bytes": 5 * 1024**3,
                                         "maximum_target_bytes": 1024**3},
            "validation resource guards differ from 5 GiB free / 1 GiB target limits")
    require(len(execution.get("commands", [])) == args.build_command_count,
            "build command count differs from explicit frozen count")
    pins = {}
    for group in ("candidate_sources", "test_sources", "canonical_files"):
        pin_map = metadata.get(group)
        require(isinstance(pin_map, dict) and bool(pin_map), f"metadata lacks {group} pins")
        for relative, digest in pin_map.items():
            path = (ROOT / relative).resolve(strict=True)
            path.relative_to(ROOT.resolve())
            require(sha(path.read_bytes()) == digest, f"live pin differs: {group}/{relative}")
        pins[group] = pin_map
    expected_sets = {name: sorted(paths) for name, paths in pins.items()}
    require(execution.get("preflight", {}).get("source_pins_before") == pins
            and execution.get("preflight", {}).get("source_pins_after") == pins
            and execution.get("freeze", {}).get("product_path_sets") == expected_sets
            and execution.get("freeze", {}).get("cli_path") == str(cli)
            and execution.get("freeze", {}).get("cli_sha256") == args.cli_sha256
            and execution.get("freeze", {}).get("cli_mode") == "0o555"
            and execution.get("freeze", {}).get("metadata_path") == str(metadata_path)
            and execution.get("freeze", {}).get("source_commit_base") == args.source_base
            and execution.get("freeze", {}).get(args.product_flag) is True
            and metadata.get("cli_mode") == "0o555"
            and metadata.get("metadata_path") == str(metadata_path),
            "build freeze/source pin record differs from actual metadata")
    for row in execution["commands"]:
        require(row.get("exit_code") == 0 and row.get("guard_stop") is None
                and row.get("peak_target_bytes", 1024**3 + 1) <= 1024**3
                and row.get("free_bytes_after", 0) >= 5 * 1024**3,
                f"validation command failed or exceeded resource guard: {row.get('index')}")
        for stream in row.get("streams", {}).values():
            path = Path(stream["path"]).resolve(strict=True)
            path.relative_to(build_path.parent.resolve())
            raw = path.read_bytes()
            require(len(raw) == stream.get("bytes") and sha(raw) == stream.get("sha256"),
                    f"validation raw stream changed: {path.name}")
    template = execution["guarded_runner_template"]
    require(template.get("path") == str(ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"),
            "guarded runner template path is not the pinned v9 template")
    return metadata, execution, {"path": str(build_path), "sha256": sha(build_raw),
                                  "runner": execution["validation_runner"]}


def load_if_baseline(module):
    require = module.require
    acceptance_raw = IF_ACCEPTANCE.read_bytes()
    manifest_raw = (IF_BUNDLE / "manifest.json").read_bytes()
    inventory_raw = (IF_BUNDLE / "file-inventory.json").read_bytes()
    require(sha(acceptance_raw) == IF_ACCEPTANCE_SHA256
            and sha(manifest_raw) == IF_MANIFEST_SHA256
            and sha(inventory_raw) == IF_INVENTORY_SHA256,
            "previously accepted If CF07 files changed")
    acceptance, manifest, inventory = json.loads(acceptance_raw), json.loads(manifest_raw), json.loads(inventory_raw)
    require(acceptance.get("status") == "accepted_required_scope_with_only_named_last_index_gap"
            and acceptance.get("full_physical_bci_coverage") is False
            and acceptance.get("candidate_bundle", {}).get("manifest_sha256") == IF_MANIFEST_SHA256,
            "previous If CF07 acceptance record differs")
    rows = {row["path"]: row for row in inventory}
    actual = {path.relative_to(IF_BUNDLE).as_posix(): path for path in IF_BUNDLE.rglob("*")
              if path.is_file() and path != IF_BUNDLE / "file-inventory.json"}
    require(len(rows) == len(inventory) == 119 and set(rows) == set(actual),
            "previous If CF07 bundle inventory is not closed")
    for name, path in actual.items():
        raw = path.read_bytes()
        require(rows[name].get("bytes") == len(raw) and rows[name].get("sha256") == sha(raw),
                f"previous If CF07 member changed: {name}")
    require(manifest.get("command_count") == 29 and len(manifest.get("cases", [])) == 10,
            "previous If CF07 29-command/10-case replay differs")
    return manifest


def while_span(text: str) -> str:
    matches = list(re.finditer(r"(?m)^(?P<indent>[ \t]*)while\s*\([^\n]*\)\s*\{\n", text))
    if len(matches) != 1:
        raise ValueError(f"expected exactly one while statement in lastIndexOf report, got {len(matches)}")
    start = matches[0].start()
    depth = 0
    opened = False
    index = matches[0].end() - 2
    while index < len(text):
        char = text[index]
        if char == "{":
            opened = True
            depth += 1
        elif char == "}":
            depth -= 1
            if opened and depth == 0:
                end = index + 1
                if end < len(text) and text[end] == "\n":
                    end += 1
                return text[start:end]
        index += 1
    raise ValueError("unterminated while statement in lastIndexOf report")


def verify_maps(module, manifest, cases, old_if_manifest, old):
    require = module.require
    old_cases = {row["label"]: row for row in old_if_manifest["cases"]}
    physical_rows = {row["command"]["label"]: row for row in manifest["original_physical_facts"]}
    results, gaps = {}, {}
    expected_methods = set(module.EXPECTED_METHODS)
    for leg in ("javac8", "javac23"):
        original = module.unwrap(cases[f"{leg}-original"])
        javap = physical_rows[f"{leg}-original-javap"]
        command = javap["command"]
        javap_raw = module.read_ref(BUNDLE, javap["text"])
        require(module.command_stream(BUNDLE, command, "stdout") == javap_raw
                and command["argv"] == [module._JDK_TOOLS[leg]["javap"], "-p", "-c", "-s", "-v",
                                        str((BUNDLE / original["actual_class"]["path"]).resolve())],
                f"candidate javap raw/argv mismatch: {leg}")
        parsed = old.parse_javap(module.safe_path(BUNDLE, javap["text"]["path"]))
        recomputed = {key: {"flags": val["flags"], "bcis": [ins["bci"] for ins in val["instructions"]],
                            "instructions": val["instructions"], "declaration": val["declaration"]}
                      for key, val in parsed["methods"].items()}
        require(parsed.get("physical_field_count") == 0
                and set(recomputed) == expected_methods
                and recomputed == javap["physical_methods"],
                f"candidate physical methods were not reproduced from raw javap: {leg}")
        for method_id, (flags, bcis) in module.EXPECTED_METHODS.items():
            require(recomputed[method_id]["flags"] == flags and recomputed[method_id]["bcis"] == bcis,
                    f"physical method/BCI census differs from the frozen CF07 class: {leg}/{method_id}")
        for method_id, bci, target in (("andWhile(Z)I", 15, 2), ("counted(II)I", 30, 6),
                                       ("counted(II)I", 20, 27), ("lastIndexOf([IIII)I", 25, 5)):
            instruction = next(row["instruction"] for row in recomputed[method_id]["instructions"]
                               if row["bci"] == bci)
            require(" ".join(instruction.split()) == f"goto {target}",
                    f"physical transfer target changed: {leg}/{method_id}@{bci}")
        old_original = module.unwrap(old_cases[f"{leg}-original"])
        require(module.read_ref(BUNDLE, original["actual_class"]) == old.check_ref(old_original["actual_class"]),
                f"fresh input class differs from accepted If oracle: {leg}")
        old_original_sources = [old.check_ref(ref) for ref in old_original["source_files"]]
        require([module.read_ref(BUNDLE, ref) for ref in original["source_files"]] == old_original_sources,
                f"fresh original input/Runner differs from accepted If oracle: {leg}")

        default_all = {}
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = cases[label]
            rendered = case["rendered_profile"]
            document_raw = module.read_ref(BUNDLE, rendered["document"])
            document = json.loads(document_raw)
            generated = module.read_ref(BUNDLE, rendered["generated_text"])
            require(generated == document.get("text", "").encode("utf-8"), f"generated text differs from document: {label}")
            argv = [str(module.CLI), "class-source", "--input",
                    str((BUNDLE / original["actual_class"]["path"]).resolve()),
                    "--class", "cf07.LoopCases", "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                argv += ["--evidence", "all"]
            require(rendered["command"]["argv"] == argv
                    and module.command_stream(BUNDLE, rendered["command"], "stdout") == document_raw,
                    f"raw Jarde render command/document mismatch: {label}")
            owner = document["class"]
            raw_class = module.read_ref(BUNDLE, original["actual_class"])
            require(owner.get("class_bytes") == {"digest": module.blake3(raw_class).hexdigest(),
                    "length": len(raw_class)}
                    and owner.get("location") == {"kind": "standalone_root",
                    "snapshot": module.blake3(raw_class).hexdigest()}
                    and owner.get("variant") == {"kind": "base"},
                    f"owner does not bind exact original class bytes: {label}")
            methods = module.document_methods(document, old)
            require(set(methods) == expected_methods, f"physical method set mismatch: {label}")
            default_all[mode] = document
            old_doc_ref = module.unwrap(old_cases[label])["rendered_profile"]["document"]
            old_doc = json.loads(old.check_ref(old_doc_ref))
            old_methods = module.document_methods(old_doc, old)
            require(document.get("text") == old_doc.get("text"), f"method body/class text changed from accepted If: {label}")
            for method_id in sorted(expected_methods):
                item = methods[method_id]["item"]
                identity = item.get("identity", {})
                if "member" in identity:
                    identity = identity["member"]
                require(old.method_key(item) == method_id
                        and item.get("access_flags") == recomputed[method_id]["flags"]
                        and identity.get("owner") == owner,
                        f"reported physical owner/flags differ from javap: {label}/{method_id}")
                report = methods[method_id]["outcome"]["report"]
                old_report = old_methods[method_id]["outcome"]["report"]
                require((report.get("text"), report.get("quality"), report.get("representation"),
                         report.get("content"), report.get("fallbacks"))
                        == (old_report.get("text"), old_report.get("quality"), old_report.get("representation"),
                            old_report.get("content"), old_report.get("fallbacks")),
                        f"existing method text/presentation changed: {label}/{method_id}")
                if method_id == "lastIndexOf([IIII)I":
                    require("while (" in report["text"] and "for (" not in report["text"],
                            "lastIndexOf spelling changed from the accepted while body")
                physical = recomputed[method_id]
                physical_bcis = set(physical["bcis"])
                text_bytes = report["text"].encode("utf-8")
                mapped = set()
                for segment in report.get("source_map", {}).get("segments", []):
                    start, end = segment.get("start"), segment.get("end")
                    require(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(text_bytes),
                            f"invalid UTF-8 source span: {label}/{method_id}")
                    origin = segment.get("origin", {})
                    points = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
                    for point in points:
                        method = point.get("method", {})
                        require(method.get("owner") == owner
                                and bytes(method.get("name", [])).decode("utf-8")
                                + bytes(method.get("descriptor", [])).decode("ascii") == method_id
                                and point.get("bci") in physical_bcis,
                                f"origin is not bound to physical owner/method/BCI: {label}/{method_id}")
                        mapped.add(point["bci"])
                missing = sorted(physical_bcis - mapped)
                require(not missing, f"physical BCI coverage incomplete: {label}/{method_id}: {missing}")
                before = module.point_facts(old_report)
                after = module.point_facts(report)
                require(not (before - after), f"candidate removed/changed accepted If origins: {label}/{method_id}")
                additions = list((after - before).elements())
                if method_id != "lastIndexOf([IIII)I":
                    require(not additions, f"unexpected new source origin outside lastIndexOf: {label}/{method_id}")
                else:
                    require(len(additions) == 1, f"lastIndexOf must add exactly one origin, got {len(additions)}")
                    fact = additions[0]
                    expected_span = while_span(old_report["text"])
                    actual_span = text_bytes[fact[0]:fact[1]].decode("utf-8")
                    require(fact[2] == "derived" and fact[10] == 25 and actual_span == expected_span,
                            "lastIndexOf@25 must be one derived origin on the exact complete while span")
                    instruction = next(row["instruction"] for row in physical["instructions"] if row["bci"] == 25)
                    require(" ".join(instruction.split()) == "goto 5", "lastIndexOf@25 is not the frozen goto 5")
                results[f"{leg}/{mode}/{method_id}"] = {
                    "physical_bcis": sorted(physical_bcis), "mapped_bcis": sorted(mapped),
                    "missing_bcis": [], "added_origins": [list(fact) for fact in additions]}
        require(default_all["default"]["text"] == default_all["all"]["text"],
                f"default/all whole-class text differs: {leg}")
        for method_id in expected_methods:
            left = module.document_methods(default_all["default"], old)[method_id]["outcome"]["report"]
            right = module.document_methods(default_all["all"], old)[method_id]["outcome"]["report"]
            require(left.get("text") == right.get("text") and left.get("source_map") == right.get("source_map"),
                    f"default/all source map differs: {leg}/{method_id}")
    return {"physical_methods_per_profile": len(expected_methods), "method_profile_rows": results,
            "all_physical_bcis_covered": True, "default_all_text_and_maps_equal": True,
            "accepted_if_bodies_and_origins_preserved": True,
            "only_new_origin": {"method": "lastIndexOf([IIII)I", "bci": 25, "role": "derived",
                                "span": "complete while statement including indentation and newline"}}


def verify(args) -> dict:
    module = load_for_verifier()
    module.BUNDLE = BUNDLE
    module.ACCEPTANCE = ACCEPTANCE
    module.WRAPPER = WRAPPER
    module.CLI = Path(args.cli).resolve(strict=True)
    module.CLI_SHA256 = args.cli_sha256
    module.METADATA_PATH = Path(args.metadata).resolve(strict=True)
    module.METADATA_SHA256 = args.metadata_sha256
    module.SOURCE_BASE = args.source_base
    module.BUILD_RESULTS = RESULTS / args.build.parent.name
    module.BUILD_RUNNER = RESULTS / f"run-validation-build-root-v{args.build.parent.name.removeprefix('validation-build-root-v')}.py"
    module.BUILD_RUNNER_SHA256 = args.runner_sha256
    module.GUARD_SOURCE = Path("/Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py")

    # Validate the historical closed controls and the exact, already accepted If map baseline.
    old = module.load_baseline_verifier()
    baseline_controls = module.verify_baseline(old)
    old_if_manifest = load_if_baseline(module)
    metadata, execution, build = verify_build(module, args)
    manifest, inventory = module.closed_bundle()
    require = module.require
    require(manifest.get("schema") == "preserve-proved-return-arm-loop-latch-origins-cf07-candidate-replay-v1"
            and manifest.get("status") == "candidate-replay-observed"
            and manifest.get("failures") == []
            and manifest.get("command_count") == 29 and len(manifest.get("cases", [])) == 10
            and manifest.get("case_counts") == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest.get("expected_case_counts") == {"original": 2, "jadx": 4, "jarde": 4},
            "return-arm CF07 candidate manifest schema/matrix differs")
    observation = json.loads((BUNDLE / "candidate-observation.json").read_bytes())
    require(observation.get("schema") == "preserve-proved-return-arm-loop-latch-origins-cf07-candidate-observation-v1"
            and observation.get("status") == "candidate_replay_observed"
            and observation.get("candidate_cli") == {"path": str(module.CLI), "sha256": args.cli_sha256}
            and observation.get("metadata") == {"path": str(module.METADATA_PATH), "sha256": args.metadata_sha256}
            and observation.get("source_base") == args.source_base
            and observation.get("build_execution", {}).get("path") == str(args.build.resolve())
            and observation.get("build_execution", {}).get("sha256") == build["sha256"],
            "collector observation is not bound to the explicit candidate/build inputs")
    require(manifest.get("frozen_jarde_cli", {}).get("path") == str(module.CLI)
            and manifest.get("frozen_jarde_cli", {}).get("sha256") == args.cli_sha256
            and manifest.get("frozen_jarde_cli", {}).get("metadata_path") == str(module.METADATA_PATH)
            and manifest.get("frozen_jarde_cli", {}).get("metadata_sha256") == args.metadata_sha256,
            "candidate manifest frozen CLI/metadata identity differs")
    wrapper_sha = sha(WRAPPER.read_bytes())
    require(manifest.get("wrapper_sha256") == wrapper_sha
            and observation.get("wrapper") == {"path": str(WRAPPER), "sha256": wrapper_sha},
            "candidate wrapper path/SHA provenance differs")
    commands = module.verify_commands(manifest)
    cases = {row["label"]: row for row in manifest["cases"]}
    expected_labels = {f"{leg}-original" for leg in ("javac8", "javac23")}
    expected_labels |= {f"{leg}-jadx-{profile}" for leg in ("javac8", "javac23") for profile in ("default", "none")}
    expected_labels |= {f"{leg}-jarde-{mode}" for leg in ("javac8", "javac23") for mode in ("default", "all")}
    require(set(cases) == expected_labels, "candidate whole-class case labels differ")
    module.verify_jdk_and_jadx(manifest, commands)

    historical_manifest = json.loads((module.BASELINE / "manifest.json").read_bytes())
    historical_cases = {row["label"]: row for row in historical_manifest["cases"]}
    runtime = {}
    original_runtime = {}
    for leg in ("javac8", "javac23"):
        original = cases[f"{leg}-original"]
        fresh = module.unwrap(original)
        base_original = module.unwrap(historical_cases[f"{leg}-original"])
        fresh_raw = tuple(module.command_stream(BUNDLE, fresh["runtime"], stream) for stream in ("stdout", "stderr"))
        require(fresh_raw == tuple(old.command_stream(base_original["runtime"], stream) for stream in ("stdout", "stderr")),
                f"fresh original runtime raw differs from same-JDK oracle: {leg}")
        original_runtime[leg] = fresh_raw
        for kind, profiles in (("original", (None,)), ("jadx", ("default", "none")), ("jarde", ("default", "all"))):
            for profile in profiles:
                label = f"{leg}-original" if profile is None else f"{leg}-{kind}-{profile}"
                case = cases[label]
                baseline_case = historical_cases[label]
                source_bytes = [old.check_ref(ref) for ref in module.unwrap(baseline_case)["source_files"]]
                require(len(source_bytes) == 2, f"historical control source pair differs: {label}")
                if kind == "original":
                    require([module.read_ref(BUNDLE, ref) for ref in module.unwrap(case)["source_files"]] == source_bytes,
                            f"fresh original Java/Runner differs from frozen control: {label}")
                runtime[label] = module.verify_case(case, commands, module.unwrap(original),
                    source_bytes[0], source_bytes[1], old, baseline_case, leg)
    require(original_runtime["javac8"] == original_runtime["javac23"]
            and manifest.get("original_cross_jdk_raw_equal") is True,
            "fresh original runtime raw differs across the two JDK oracle legs")

    for leg in ("javac8", "javac23"):
        for profile in ("default", "none"):
            label = f"{leg}-jadx-{profile}"
            case, decomp = cases[label], cases[label]["decompilation"]
            command = decomp["command"]
            expected_argv = [str(module.JADX), "--no-res", "--config", "none", "--threads-count", "1"]
            if profile == "none":
                expected_argv += ["--rename-flags", "none"]
            expected_argv += ["-d", str((BUNDLE / "jadx-output" / profile).resolve()),
                              str((BUNDLE / "jadx-input/cf07/LoopCases.class.jar").resolve())]
            require(commands.get(command["label"]) == command and command.get("exit") == 0
                    and command.get("argv") == expected_argv
                    and decomp.get("decompile_success") is True
                    and decomp.get("source_name_set_exact") is True
                    and decomp.get("packages") == ["cf07"]
                    and decomp.get("package_set_single") is True
                    and decomp.get("input_jar_exact") is True,
                    f"JADX command/source/package/input jar differs: {label}")
            generated_refs = decomp.get("generated_sources", [])
            require(len(generated_refs) == 1 and generated_refs[0]["path"].endswith("/cf07/LoopCases.java"),
                    f"JADX source member set differs: {label}")
            generated = module.read_ref(BUNDLE, generated_refs[0])
            runner = module.read_ref(BUNDLE, case["runner_adaptation"])
            old_runner = old.check_ref(next(ref for ref in module.unwrap(historical_cases[f"{leg}-original"])["source_files"]
                                            if ref["path"].endswith("Runner.java")))
            package = re.search(rb"(?m)^\s*package\s+([\w.]+)\s*;", generated)
            adapted = module.adapt_runner_bytes(old_runner, package.group(1).decode("ascii") if package else None)
            require(runner == adapted
                    and [module.read_ref(BUNDLE, ref) for ref in module.unwrap(case)["source_files"]] == [generated, runner],
                    f"JADX compile sources contain changes beyond package-only Runner adaptation: {label}")
    for leg in ("javac8", "javac23"):
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = cases[label]
            document = json.loads(module.read_ref(BUNDLE, case["rendered_profile"]["document"]))
            generated = module.read_ref(BUNDLE, case["rendered_profile"]["generated_text"])
            runner = module.read_ref(BUNDLE, case["runner_adaptation"])
            old_runner = old.check_ref(next(ref for ref in module.unwrap(historical_cases[f"{leg}-original"])["source_files"]
                                            if ref["path"].endswith("Runner.java")))
            package = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", document["text"])
            adapted = module.adapt_runner_bytes(old_runner, package.group(1) if package else None)
            require(generated == document["text"].encode("utf-8") and runner == adapted
                    and [module.read_ref(BUNDLE, ref) for ref in module.unwrap(case)["source_files"]] == [generated, runner],
                    f"Jarde compile input differs from raw document or package-only Runner: {label}")

    maps = verify_maps(module, manifest, cases, old_if_manifest, old)
    return {
        "schema": "preserve-proved-return-arm-loop-latch-origins-cf07-candidate-acceptance-root-v1",
        "status": "accepted_full_physical_bci_coverage",
        "verified_observations": True,
        "full_physical_bci_coverage": True,
        "uncovered_physical_bcis": {},
        "candidate_bundle": {"path": str(BUNDLE), "manifest_sha256": sha((BUNDLE / "manifest.json").read_bytes()),
                             "inventory_members": len(inventory), "commands": len(commands), "cases": len(cases)},
        "candidate_cli": {"path": str(module.CLI), "sha256": args.cli_sha256},
        "metadata": {"path": str(module.METADATA_PATH), "sha256": args.metadata_sha256,
                     "build_result_path": build["path"], "build_result_sha256": build["sha256"]},
        "source_base": args.source_base,
        "baseline_if": {"path": str(IF_BUNDLE), "manifest_sha256": IF_MANIFEST_SHA256,
                        "acceptance_sha256": IF_ACCEPTANCE_SHA256},
        "historical_control": baseline_controls,
        "runtime_cases": runtime,
        "source_map_checks": maps,
        "claim_boundary": "Full CF07 physical BCI coverage is accepted for this candidate; existing accepted If/loop origins and bodies are unchanged, with only the exact derived lastIndexOf@25 while-span origin added. This does not claim for syntax recovery.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--metadata", required=True)
    parser.add_argument("--metadata-sha256", required=True)
    parser.add_argument("--metadata-schema", required=True)
    parser.add_argument("--source-base", required=True)
    parser.add_argument("--product-flag", required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--build-schema", required=True)
    parser.add_argument("--build-command-count", type=int, required=True)
    parser.add_argument("--runner-sha256", required=True)
    args = parser.parse_args()
    if args.build_command_count < 1:
        raise SystemExit("--build-command-count must be positive")
    if ACCEPTANCE.exists():
        raise SystemExit(f"refusing to overwrite acceptance result: {ACCEPTANCE}")
    try:
        result = verify(args)
        ACCEPTANCE.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    except Exception as error:
        print(f"return-arm CF07 independent verification rejected: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
