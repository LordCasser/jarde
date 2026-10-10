#!/usr/bin/env python3
"""Prepare a private CF07 full-class replay for the return-arm latch change."""

from __future__ import annotations

import argparse
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
OUT = RESULTS / "cf07-candidate-root-v1"
WRAPPER = RESULTS / "prepare-cf07-candidate-root-v1.py"
FOR_WRAPPER = ROOT / "openspec/changes/preserve-proved-for-latch-origins/results/prepare-cf07-candidate-root-v1.py"
FOR_WRAPPER_SHA256 = "196202bacbcffc09666ff122e1db605d39f7718e10c322f9243d4bcd23311020"
GUARD_SHA256 = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
BASELINE_IF = ROOT / "openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1"
BASELINE_IF_MANIFEST_SHA256 = "d0f6bc9f596105ce7555240c40060146f52ee88298b55a1b5c2008a96e938c7e"
BASELINE_IF_INVENTORY_SHA256 = "79331dc0c3de8fbfb487a17d8df2e64d5269ed3b75ec56528e40191b6798906a"
BASELINE_IF_ACCEPTANCE = ROOT / "openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-acceptance-root-v1.json"
BASELINE_IF_ACCEPTANCE_SHA256 = "229c42b680d51fb8c4add5ea6a29835d35d02de2ffd5a91422ef539535c7545f"


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def load_for_wrapper():
    raw = FOR_WRAPPER.read_bytes()
    if sha(raw) != FOR_WRAPPER_SHA256:
        raise RuntimeError("accepted For CF07 wrapper SHA changed")
    spec = importlib.util.spec_from_file_location("accepted_for_cf07_wrapper_for_return_latch", FOR_WRAPPER)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load accepted For CF07 wrapper")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def frozen_candidate(module, args):
    if not args.metadata_schema.startswith("preserve-proved-return-arm-loop-latch-origins-candidate-cli-v"):
        raise RuntimeError("metadata schema must belong to the return-arm latch change")
    if re.fullmatch(r"preserve-proved-return-arm-loop-latch-origins-validation-build-root-v[1-9][0-9]*",
                    args.build_schema) is None:
        raise RuntimeError("build schema must belong to the return-arm latch change")
    if (not args.product_flag.startswith("uncommitted_") or "return" not in args.product_flag
            or "latch" not in args.product_flag or not args.product_flag.endswith("_product")):
        raise RuntimeError("--product-flag must name this uncommitted return-arm latch product marker")
    cli = Path(args.cli).resolve(strict=True)
    metadata_path = Path(args.metadata).resolve(strict=True)
    build_path = args.build.resolve(strict=True)
    cli_raw, metadata_raw, build_raw = cli.read_bytes(), metadata_path.read_bytes(), build_path.read_bytes()
    if sha(cli_raw) != args.cli_sha256 or sha(metadata_raw) != args.metadata_sha256:
        raise RuntimeError("actual candidate CLI/metadata SHA does not match required arguments")
    if stat.S_IMODE(cli.stat().st_mode) != 0o555:
        raise RuntimeError("new candidate CLI must be frozen mode 0555")
    try:
        metadata_path.relative_to(RESULTS.resolve())
        build_path.relative_to(RESULTS.resolve())
    except ValueError as error:
        raise RuntimeError("--metadata and --build must be within this change's results directory") from error
    match = re.fullmatch(r"validation-build-root-v([1-9][0-9]*)", build_path.parent.name)
    if match is None or build_path.name != "execution.json":
        raise RuntimeError("--build must be validation-build-root-vN/execution.json")
    build_n = match.group(1)
    runner = RESULTS / f"run-validation-build-root-v{build_n}.py"
    if not runner.is_file() or sha(runner.read_bytes()) != args.runner_sha256:
        raise RuntimeError("actual change-specific validation runner SHA does not match required argument")
    metadata = json.loads(metadata_raw)
    execution = json.loads(build_raw)
    if (metadata.get("schema") != args.metadata_schema
            or metadata.get("cli_path") != str(cli)
            or metadata.get("cli_sha256") != args.cli_sha256
            or metadata.get("source_commit_base") != args.source_base
            or metadata.get(args.product_flag) is not True
            or metadata.get("build_result_sha256") != sha(build_raw)):
        raise RuntimeError("metadata does not bind the supplied CLI, source base, build, and product flag")
    if not re.fullmatch(r"[0-9a-f]{40}", args.source_base):
        raise RuntimeError("--source-base must be the actual 40-character source commit")
    if (execution.get("schema") != args.build_schema
            or not execution.get("schema", "").endswith(f"root-v{build_n}")
            or execution.get("status") != "validation-passed-cli-frozen"
            or execution.get("source_commit_base_expected") != args.source_base
            or execution.get(args.product_flag) is not True
            or execution.get("validation_runner") != {"path": str(runner.resolve()), "sha256": args.runner_sha256}
            or execution.get("guarded_runner_template", {}).get("sha256") != GUARD_SHA256
            or sha(Path(execution["guarded_runner_template"]["path"]).read_bytes()) != GUARD_SHA256
            or execution.get("commands") is None
            or len(execution["commands"]) != args.build_command_count
            or execution.get("preflight", {}).get("source_pins_before") != execution.get("preflight", {}).get("source_pins_after")
            or execution.get("freeze", {}).get("cli_path") != str(cli)
            or execution.get("freeze", {}).get("cli_sha256") != args.cli_sha256
            or execution.get("freeze", {}).get("cli_mode") != "0o555"
            or execution.get("freeze", {}).get("metadata_path") != str(metadata_path)
            or execution.get("freeze", {}).get("source_commit_base") != args.source_base
            or execution.get("freeze", {}).get(args.product_flag) is not True
            or metadata.get("cli_mode") != "0o555"
            or metadata.get("validation_runner") != execution.get("validation_runner")
            or metadata.get("guarded_runner_template") != execution.get("guarded_runner_template")):
        raise RuntimeError("validation execution does not match supplied build/schema/runner/CLI/source-base pins")
    if execution.get("guards") != {"minimum_free_bytes": 5 * 1024**3, "maximum_target_bytes": 1024**3}:
        raise RuntimeError("validation build resource guards differ from the approved 5 GiB/1 GiB limits")
    for row in execution["commands"]:
        if (row.get("exit_code") != 0 or row.get("guard_stop") is not None
                or row.get("peak_target_bytes", 1024**3 + 1) > 1024**3
                or row.get("free_bytes_after", 0) < 5 * 1024**3):
            raise RuntimeError(f"validation command failed or exceeded a resource guard: {row.get('index')}")
        for stream in row.get("streams", {}).values():
            raw_path = Path(stream["path"]).resolve(strict=True)
            raw_path.relative_to(build_path.parent.resolve())
            raw = raw_path.read_bytes()
            if len(raw) != stream.get("bytes") or sha(raw) != stream.get("sha256"):
                raise RuntimeError(f"validation command raw changed: {raw_path.name}")
    for group in ("candidate_sources", "test_sources", "canonical_files"):
        pin_map = metadata.get(group)
        if not isinstance(pin_map, dict) or not pin_map:
            raise RuntimeError(f"metadata lacks nonempty {group} pins")
        for relative, expected in pin_map.items():
            path = ROOT / relative
            if not path.is_file() or sha(path.read_bytes()) != expected:
                raise RuntimeError(f"live source pin changed: {group}/{relative}")
    frozen_sets = {group: sorted(metadata[group]) for group in
                   ("candidate_sources", "test_sources", "canonical_files")}
    if execution.get("freeze", {}).get("product_path_sets") != frozen_sets:
        raise RuntimeError("build freeze product path sets differ from metadata")
    return metadata, {"path": str(build_path), "sha256": sha(build_raw),
                      "validation_runner": execution["validation_runner"],
                      "status": execution["status"]}, cli, metadata_path


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
    module = load_for_wrapper()
    metadata, build_record, cli, metadata_path = frozen_candidate(module, args)
    baseline_manifest_raw = (BASELINE_IF / "manifest.json").read_bytes()
    baseline_inventory_raw = (BASELINE_IF / "file-inventory.json").read_bytes()
    baseline_acceptance_raw = BASELINE_IF_ACCEPTANCE.read_bytes()
    if (sha(baseline_manifest_raw) != BASELINE_IF_MANIFEST_SHA256
            or sha(baseline_inventory_raw) != BASELINE_IF_INVENTORY_SHA256
            or sha(baseline_acceptance_raw) != BASELINE_IF_ACCEPTANCE_SHA256):
        raise RuntimeError("accepted If CF07 baseline manifest/inventory/acceptance SHA changed")
    inventory = json.loads(baseline_inventory_raw)
    members = {row["path"]: row for row in inventory}
    actual = {path.relative_to(BASELINE_IF).as_posix(): path for path in BASELINE_IF.rglob("*")
              if path.is_file() and path != BASELINE_IF / "file-inventory.json"}
    if len(members) != 119 or set(members) != set(actual):
        raise RuntimeError("accepted If CF07 baseline inventory is not closed")
    for relative, path in actual.items():
        raw = path.read_bytes()
        if members[relative].get("bytes") != len(raw) or members[relative].get("sha256") != sha(raw):
            raise RuntimeError(f"accepted If CF07 baseline member changed: {relative}")
    acceptance = json.loads(baseline_acceptance_raw)
    if acceptance.get("status") != "accepted_required_scope_with_only_named_last_index_gap":
        raise RuntimeError("accepted If CF07 baseline status changed")
    previous = json.loads(baseline_manifest_raw)
    if previous.get("status") != "candidate-replay-observed" or previous.get("command_count") != 29:
        raise RuntimeError("accepted If CF07 baseline is not the expected complete replay")
    if OUT.exists():
        raise RuntimeError(f"refusing to overwrite candidate output: {OUT}")

    module.RESULTS = RESULTS
    module.OUT = OUT
    module.WRAPPER_PATH = WRAPPER
    module.CLI = cli
    module.METADATA = metadata_path
    module.METADATA_SCHEMA = args.metadata_schema
    module.SOURCE_BASE = args.source_base
    module.TARGETS = {"andWhile(Z)I": 15, "counted(II)I": 30}
    module.OUT_OF_SCOPE = {"counted(II)I": 20, "lastIndexOf([IIII)I": 25}
    module.TARGET_LOOP_TOKENS = {"andWhile(Z)I": "while (", "counted(II)I": "while ("}

    def read_frozen_candidate(cli_digest: str, metadata_digest: str, build_path: Path):
        if cli_digest != args.cli_sha256 or metadata_digest != args.metadata_sha256 or build_path.resolve() != args.build.resolve():
            raise RuntimeError("collector arguments differ from the explicitly frozen candidate inputs")
        return metadata, build_record

    module.read_frozen_candidate = read_frozen_candidate
    # Reuse the reviewed 29-command replay body under its module namespace; its old main is never run.
    saved_argv = sys.argv
    try:
        sys.argv = [str(WRAPPER), "--cli-sha256", args.cli_sha256,
                    "--metadata-sha256", args.metadata_sha256, "--build", str(args.build)]
        result = module.main()
    finally:
        sys.argv = saved_argv
    if result != 0:
        return result

    manifest_path = OUT / "manifest.json"
    manifest = json.loads(manifest_path.read_bytes())
    observation_path = OUT / "candidate-observation.json"
    observation = json.loads(observation_path.read_bytes())
    manifest["schema"] = "preserve-proved-return-arm-loop-latch-origins-cf07-candidate-replay-v1"
    manifest["claim_boundary"] = "Observation only. The independent verifier must confirm all source maps; this draft does not establish product acceptance."
    observation["schema"] = "preserve-proved-return-arm-loop-latch-origins-cf07-candidate-observation-v1"
    observation["candidate_cli"] = {"path": str(cli), "sha256": args.cli_sha256}
    observation["metadata"] = {"path": str(metadata_path), "sha256": args.metadata_sha256}
    observation["build_execution"] = build_record
    observation["source_base"] = args.source_base
    observation["claim_boundary"] = manifest["claim_boundary"]
    wrapper_sha = sha(Path(__file__).read_bytes())
    observation["wrapper"] = {"path": str(WRAPPER), "sha256": wrapper_sha}
    manifest["wrapper_sha256"] = wrapper_sha
    observation_path.write_text(json.dumps(observation, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    collector, _ = module.collector_module()
    collector.OUT = OUT
    helpers = collector.load_physical_helpers()
    (OUT / "file-inventory.json").write_text(json.dumps(helpers["inventory_rows"](), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "candidate-replay-observed", "output": str(OUT),
                      "commands": manifest["command_count"], "cases": manifest["case_counts"]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
