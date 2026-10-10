#!/usr/bin/env python3
"""Guarded local validation and CLI freeze for proved discarded-call origins.

This runner is a private preparation draft. It must be reviewed before root execution.
"""
from __future__ import annotations

import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys

ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
TASK = Path("/private/tmp/jarde-discarded-call-validation-luna-v1")
RESULTS = ROOT / "openspec/changes/preserve-proved-discarded-call-origins/results"
OUT = RESULTS / "validation-build-root-v1"
CLI_PATH = Path("/private/tmp/jarde-proved-discarded-call-cli-v1")
METADATA_PATH = RESULTS / "candidate-cli-v1.json"
SOURCE_COMMIT_BASE: str | None = None

TEMPLATE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
TEMPLATE_SHA256 = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
BASELINE_ACCEPTANCE_SHA256 = "b2819eb2632925696a117528634468e73f0834ecef28e0d34c248bc82723e717"
BASELINE_RUNNER_SHA256 = "cdbf43782243a49fd789e2bd6260d569591e8a44f688ccabf58d3d43bebc0955"
BASELINE_EXECUTION_SHA256 = "48a675bf6bf2ffc2c58b8a8ac0e90cad69e5077730a2470ed23f4b52639dbd1e"
BASELINE_INPUT_SHA256S = {
    "DiscardedCallSourceProbe.java": "2f067f8110639ab68467a397c21824eabb8bc434cd1376c8a57369d2ffbdf1ea",
    "DiscardedCallSourceProbeRunner.java": "180f520e94f7250fbbf0a96699376e5878cdc8418e78fa4a90cf738aee5155b5",
    "ProbePop2.java": "914f126077bf34d9551ce9afd9b4f66eb0033f059a9d158da0b3092dcf68d630",
}
BASELINE_CLASS_SHA256S = {
    "javac8": {
        "discardprobe/DiscardedCallSourceProbe.class": "347b1678d4fc3cd6dd168336eccdf354ecc12f61f804c0c36406bbdeee43fa6c",
        "discardprobe/DiscardedCallSourceProbeRunner.class": "b8ec35a7a11d260cfefc7f2695999bdba6017d97c8498cf42c0f7219034b7352",
        "discardprobe/ProbePop2.class": "83901804da90811f6b672df84cb03cbe7dba36e20cc6b67b1d97549921cf5da2",
    },
    "javac23": {
        "discardprobe/DiscardedCallSourceProbe.class": "e913b22b36d114a1eb675459d72e5df8533d5221159388bcba2f5ad4c5504ecb",
        "discardprobe/DiscardedCallSourceProbeRunner.class": "d9fd9492914bade2bf09693773ce702d873896b46984288925618311516c0120",
        "discardprobe/ProbePop2.class": "ea2d95ff242bba7ac66f7693b8a21dcca18e86d976be81728a6304d27e173e85",
    },
}
PRIOR_RETURN_CI = ROOT / "openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/ci-product-v1/acceptance-proved-return-arm-loop-latch-ci-root-v1.json"
PRIOR_RETURN_CI_SHA256 = "d04ac86e2f982783d37382b605a0c34fe1fefcb9a77fc707460e4bb5c3bfea47"
PRIOR_RETURN_CI_EXPECTED = {
    "schema": "preserve-proved-return-arm-loop-latch-origins-ci-product-root-acceptance-v1",
    "status": "accepted",
    "product_commit": "0ae30a7c8d217522f2e5f5b954aa86c9b59356dd",
    "run_id": 38073837512,
    "source_commit_base": "41fe336462448eae3547cafa2a141d52e85a08e0",
}

PRODUCT_PATHS = {
    "Cargo.toml", "Cargo.lock",
    "crates/jarde-jvm/Cargo.toml", "crates/jarde-reader/Cargo.toml",
    "crates/jarde-query/Cargo.toml", "crates/jarde-java/Cargo.toml",
    "crates/jarde-cli/Cargo.toml", "crates/jarde-java/src/region.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/report.rs", "crates/jarde-java/src/lib.rs",
    "src/class_source.rs", "src/facade.rs", "src/lib.rs",
    "crates/jarde-cli/src/main.rs", "crates/jarde-cli/src/task.rs",
}
TEST_PATHS = {
    ".github/workflows/ci.yml",
    "crates/jarde-java/tests/proved_discarded_call_origins.rs",
    "crates/jarde-java/tests/p3_loop_exit_gateways.rs",
    "crates/jarde-java/tests/p3_patterns.rs",
    "tests/p3_twr_discarded_call.rs",
    "tests/p3_popped_static_qualifier.rs",
    "tests/p3_loop_arm_join.rs",
    "tests/p3_loop_boolean_exit.rs",
}
INCLUDE_RE = re.compile(r"(?:include_bytes|include_str)!\s*\(\s*\"([^\"]+)\"\s*,?\s*\)")
SUMMARY_RE = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored")
TEST_NAME_RE = re.compile(r"(?ms)^\s*#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")

# Frozen from the accepted 38073837512 return-arm CI record, not inferred from the new patch.
BASELINE_GATEWAY_TESTS = {
    "cf07_if_join_origin_rejects_nonjoin_and_nontransfer_terminals",
    "cf07_if_join_proof_budget_and_cancellation_publish_no_partial_source",
    "cf07_nonempty_if_join_goto_is_derived_from_complete_if",
    "cf07_return_arm_latch_budget_and_cancellation_publish_no_partial_source",
    "cf07_return_arm_latch_origin_rejects_wrong_target_nonterminal_and_nonreturn",
    "cf07_return_arm_loop_latch_is_derived_from_complete_while",
    "cf08_gateway_budget_and_cancellation_publish_no_partial_source",
    "cf08_two_gateways_have_one_loop_owner_and_complete_sources",
    "cf08_unproved_gateways_keep_physical_quotes",
    "no_prefix_latch_budget_and_cancellation_publish_no_partial_source",
    "no_prefix_while_latch_keeps_physical_source_and_one_owner",
    "non_backedge_transfer_does_not_gain_a_loop_latch_origin",
    "one_arm_straight_prefix_loop_and_optional_tail_are_closed",
    "prefixed_one_arm_loop_budget_and_cancellation_publish_no_partial_source",
    "rejected_for_update_slot_keeps_the_independent_while_latch",
}
REQUIRED_ORIGIN_TESTS: tuple[str, ...] = ()
EXPECTED_LIB_COUNT = 0
EXPECTED_ORIGIN_TEST_COUNT = 0

# Extra canonical pins lock the raw source inputs and accepted observation used by this run.
BASE = "openspec/changes/preserve-proved-discarded-call-origins/results/"
CANONICAL_EXTRA = {
    BASE + "original-inputs-root-v1/DiscardedCallSourceProbe.java",
    BASE + "original-inputs-root-v1/DiscardedCallSourceProbeRunner.java",
    BASE + "original-inputs-root-v1/ProbePop2.java",
    BASE + "baseline-root-v1/acceptance-root-v1.json",
    BASE + "baseline-root-v1/execution.json",
    BASE + "run-baseline-root-v1.py",
    "openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/ci-product-v1/acceptance-proved-return-arm-loop-latch-ci-root-v1.json",
}
for jdk in ("javac8", "javac23"):
    for binary in ("DiscardedCallSourceProbe.class", "DiscardedCallSourceProbeRunner.class", "ProbePop2.class"):
        CANONICAL_EXTRA.add(BASE + f"baseline-root-v1/{jdk}/original/discardprobe/{binary}")

# Expected legacy binary summaries are pinned from current source / prior accepted runner records.
# The library and new-origin counts are caller inputs, because these tests are changing in this patch.
FIXED_EXPECTED = {
    4: [(8, 0, 0)],       # tests/p3_twr_discarded_call.rs: eight existing #[test] cases
    5: [(4, 0, 0)],       # tests/p3_popped_static_qualifier.rs: four existing #[test] cases
    6: [(15, 0, 0)],      # exact accepted gateway-name set above
    7: [(4, 0, 0)],       # tests/p3_loop_arm_join.rs
    8: [(3, 0, 0)],       # tests/p3_loop_boolean_exit.rs
    9: [(85, 0, 0)],      # accepted source's p3_patterns target
    10: [(178, 0, 0)],    # accepted return-arm validation run, command 8
}
TEST_COMMANDS = set(range(2, 11))


def sha_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def load_runner():
    if sha_file(TEMPLATE) != TEMPLATE_SHA256:
        raise RuntimeError(f"guarded runner template SHA-256 mismatch: {TEMPLATE}")
    spec = importlib.util.spec_from_file_location("proved_loop_latch_guard_v9", TEMPLATE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load guarded runner template: {TEMPLATE}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def discovered_literal_paths() -> set[str]:
    result: set[str] = set()
    for relative in sorted(TEST_PATHS):
        if not relative.endswith(".rs"):
            continue
        source = (ROOT / relative).read_text(encoding="utf-8")
        for included in INCLUDE_RE.findall(source):
            resolved = (ROOT / relative).parent.joinpath(included).resolve()
            result.add(resolved.relative_to(ROOT).as_posix())
    return result


def test_names(path: Path) -> set[str]:
    return set(TEST_NAME_RE.findall(path.read_text(encoding="utf-8")))


def summary_check(index: int, stdout: bytes) -> dict:
    text = stdout.decode("utf-8", errors="replace")
    actual = [tuple(map(int, found)) for found in SUMMARY_RE.findall(text)]
    expected = FIXED_EXPECTED.get(index)
    if index == 2:
        expected = [(EXPECTED_LIB_COUNT, 0, 0)]
    elif index == 3:
        expected = [(EXPECTED_ORIGIN_TEST_COUNT, 0, 0)]
    required = index in TEST_COMMANDS
    nonempty_success = bool(actual) and all(failed == 0 for _, failed, _ in actual)
    has_tests = any(passed + failed + ignored > 0 for passed, failed, ignored in actual)
    names: tuple[str, ...] = ()
    names_ok = True
    if index == 3:
        names = REQUIRED_ORIGIN_TESTS
    elif index == 6:
        names = tuple(sorted(BASELINE_GATEWAY_TESTS))
    if names:
        names_ok = all(re.search(r"(?m)^test " + re.escape(name) + r" \.\.\. ok$", text) for name in names)
    ok = (expected is None or actual == expected) and (not required or (nonempty_success and has_tests)) and names_ok
    reason = None
    if expected is not None and actual != expected:
        reason = f"expected summaries {expected}, got {actual}"
    elif required and (not nonempty_success or not has_tests):
        reason = "required successful non-empty test summary was absent or failed"
    elif not names_ok:
        reason = "one or more exact required test names were absent or did not pass"
    return {
        "expected": expected, "actual": actual, "required": required,
        "required_test_names": list(names), "required_test_names_present": names_ok,
        "ok": ok, "failure_reason": reason,
    }


def validate_baseline_observation() -> dict:
    acceptance = RESULTS / "baseline-root-v1/acceptance-root-v1.json"
    runner_path = RESULTS / "run-baseline-root-v1.py"
    if sha_file(acceptance) != BASELINE_ACCEPTANCE_SHA256:
        raise RuntimeError("frozen discarded-call baseline acceptance SHA-256 mismatch")
    if sha_file(runner_path) != BASELINE_RUNNER_SHA256:
        raise RuntimeError("frozen discarded-call baseline runner SHA-256 mismatch")
    record = json.loads(acceptance.read_bytes())
    execution_path = RESULTS / "baseline-root-v1/execution.json"
    if sha_file(execution_path) != BASELINE_EXECUTION_SHA256 or record.get("manifest_sha256") != BASELINE_EXECUTION_SHA256:
        raise RuntimeError("discarded-call baseline execution manifest SHA mismatch")
    execution = json.loads(execution_path.read_bytes())
    if (execution.get("schema") != "discarded-call-source-baseline-root-v1"
            or execution.get("status") != "observed-baseline-not-accepted"
            or execution.get("runner_sha256") != BASELINE_RUNNER_SHA256
            or execution.get("inputs") != BASELINE_INPUT_SHA256S):
        raise RuntimeError("discarded-call baseline runner/input identity changed")
    for name, expected_sha in BASELINE_INPUT_SHA256S.items():
        source_path = RESULTS / "original-inputs-root-v1" / name
        if sha_file(source_path) != expected_sha:
            raise RuntimeError(f"frozen source input hash changed: {source_path}")
    for leg, expected_classes in BASELINE_CLASS_SHA256S.items():
        if execution.get("legs", {}).get(leg, {}).get("original_classes") != expected_classes:
            raise RuntimeError(f"baseline {leg} original class hashes changed in execution record")
        for relative, expected_sha in expected_classes.items():
            class_path = RESULTS / "baseline-root-v1" / leg / "original" / relative
            if sha_file(class_path) != expected_sha:
                raise RuntimeError(f"frozen baseline class hash changed: {class_path}")
    if (record.get("schema") != "discarded-call-source-baseline-root-v1"
            or record.get("status") != "observed-baseline-not-accepted"
            or record.get("commands") != 26 or record.get("complete_runtime_legs") != 8
            or record.get("method_profiles") != 28
            or record.get("expected_missing_pop_bcis") != {"discardStatic": [4], "discardAppend": [13], "discardListAdd": [15]}):
        raise RuntimeError("frozen discarded-call source baseline facts changed")
    return {"acceptance_path": str(acceptance), "acceptance_sha256": BASELINE_ACCEPTANCE_SHA256,
            "runner_path": str(runner_path), "runner_sha256": BASELINE_RUNNER_SHA256,
            "execution_path": str(execution_path), "execution_sha256": BASELINE_EXECUTION_SHA256,
            "input_source_sha256s": BASELINE_INPUT_SHA256S,
            "original_class_sha256s": BASELINE_CLASS_SHA256S,
            "status": record["status"], "commands": record["commands"],
            "complete_runtime_legs": record["complete_runtime_legs"],
            "method_profiles": record["method_profiles"],
            "expected_missing_pop_bcis": record["expected_missing_pop_bcis"]}

def validate_old_acceptance() -> dict:
    accepted_path = ROOT / "openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/ci-product-v1/acceptance-proved-return-arm-loop-latch-ci-root-v1.json"
    raw = accepted_path.read_bytes()
    if sha_file(accepted_path) != PRIOR_RETURN_CI_SHA256:
        raise RuntimeError("accepted return-arm CI artifact SHA-256 mismatch")
    record = json.loads(raw)
    if any(record.get(key) != expected for key, expected in PRIOR_RETURN_CI_EXPECTED.items()):
        raise RuntimeError("accepted return-arm CI record identity mismatch")
    ci = record.get("ci", {})
    if ci.get("jobs") != 4 or ci.get("step_count") != 52 or not ci.get("all_jobs_and_steps_success"):
        raise RuntimeError("accepted return-arm CI record is not four successful jobs / 52 steps")
    ci_seeds = ci.get("workspace_seed_runs", [])
    if len(ci_seeds) != 2 or any(
            row.get("binaries", {}).get("tests/p3_patterns.rs") != {"passed": 85, "failed": 0, "ignored": 0}
            for row in ci_seeds):
        raise RuntimeError("accepted return-arm CI p3_patterns binary count changed")
    accepted_summaries = record.get("frozen_cli_and_build", {}).get("build_test_result_records_recomputed_from_stdout", {})
    for command_index, expected in {"4": [[4, 0, 0]], "8": [[178, 0, 0]], "10": [[3, 0, 0]]}.items():
        if accepted_summaries.get(command_index) != expected:
            raise RuntimeError(f"accepted return-arm local validation summary {command_index} changed")
    seeds = record.get("workspace_regressions", {}).get("workspace_seed_runs", [])
    if len(seeds) != 2:
        raise RuntimeError("accepted return-arm CI record must carry both workspace seeds")
    for row in seeds:
        if (row.get("library_test_count") != 337
                or row.get("workspace_expected_totals") != [3393, 0, 97]
                or row.get("workspace_result_records") != 354
                or set(row.get("gateway_test_names", [])) != BASELINE_GATEWAY_TESTS):
            raise RuntimeError("accepted return-arm CI gateway/library baseline changed")
    return {
        "path": str(accepted_path), "sha256": PRIOR_RETURN_CI_SHA256,
        "product_commit": record["product_commit"], "run_id": record["run_id"],
        "jobs": ci["jobs"], "step_count": ci["step_count"],
        "gateway_test_names": sorted(BASELINE_GATEWAY_TESTS),
        "baseline_library_test_count": 337,
    }


def write_execution(runner, rows: list[dict], preflight: dict, status: str, freeze: dict | None = None) -> None:
    runner.write_json(OUT / "execution.json", {
        "schema": "preserve-proved-discarded-call-origins-validation-build-root-v1",
        "validation_runner": {"path": str(Path(__file__).resolve()), "sha256": sha_file(Path(__file__).resolve())},
        "guarded_runner_template": {"path": str(TEMPLATE), "sha256": TEMPLATE_SHA256},
        "status": status,
        "source_commit_base_expected": SOURCE_COMMIT_BASE,
        "uncommitted_discarded_call_product": True,
        "environment_overrides": runner.ENV_VALUES,
        "required_origin_tests": list(REQUIRED_ORIGIN_TESTS),
        "expected_origin_test_count": EXPECTED_ORIGIN_TEST_COUNT,
        "expected_library_test_count": EXPECTED_LIB_COUNT,
        "guards": {"minimum_free_bytes": runner.FREE_LIMIT, "maximum_target_bytes": runner.TARGET_LIMIT},
        "preflight": preflight, "commands": rows, "freeze": freeze,
    })


def main() -> int:
    global SOURCE_COMMIT_BASE, EXPECTED_LIB_COUNT, EXPECTED_ORIGIN_TEST_COUNT, REQUIRED_ORIGIN_TESTS
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-base", required=True, help="exact expected HEAD after the product edit")
    parser.add_argument("--expected-lib-count", required=True, type=int)
    parser.add_argument("--expected-origin-test-count", required=True, type=int)
    parser.add_argument("--new-test-name", action="append", required=True,
                        help="exact newly added test name in proved_discarded_call_origins")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.source_base):
        raise SystemExit("--source-base must be a lowercase 40-hex commit")
    if args.expected_lib_count < 337:
        raise SystemExit("--expected-lib-count cannot be below the accepted return-arm baseline 337")
    if args.expected_origin_test_count <= 0 or args.expected_origin_test_count != len(args.new_test_name):
        raise SystemExit("origin count must be positive and equal the number of exact --new-test-name arguments")
    if len(set(args.new_test_name)) != len(args.new_test_name):
        raise SystemExit("--new-test-name values must be distinct")
    if any(not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name) for name in args.new_test_name):
        raise SystemExit("origin test names must be Rust identifiers")

    SOURCE_COMMIT_BASE = args.source_base
    EXPECTED_LIB_COUNT = args.expected_lib_count
    EXPECTED_ORIGIN_TEST_COUNT = args.expected_origin_test_count
    REQUIRED_ORIGIN_TESTS = tuple(sorted(args.new_test_name))
    runner = load_runner()
    runner.ROOT = ROOT
    runner.HERE = TASK
    runner.RESULTS = RESULTS
    runner.OUT = OUT
    runner.CLI_PATH = CLI_PATH
    runner.METADATA_PATH = METADATA_PATH
    runner.SOURCE_COMMIT_BASE = SOURCE_COMMIT_BASE
    runner.TEST_COMMANDS = TEST_COMMANDS
    runner.EXPECTED_SUMMARIES = FIXED_EXPECTED
    runner.expected_test_summaries = summary_check
    runner.command_stream = lambda path: {
        "path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": sha_file(path)
    }
    runner.PRODUCT_PATHS = set(PRODUCT_PATHS)
    runner.TEST_PATHS = set(TEST_PATHS)
    literal_paths = discovered_literal_paths()
    runner.CANONICAL_PATHS = literal_paths | CANONICAL_EXTRA
    runner.COMMANDS = [
        ["cargo", "fmt", "--all", "--", "--check"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--",
         *sum((["-A", f"clippy::{name}"] for name in runner.CI_CLIPPY_ALLOW_LINTS), []), "-D", "warnings"],
        ["cargo", "test", "-p", "jarde-java", "--lib", "--locked"],
        ["cargo", "test", "-p", "jarde-java", "--test", "proved_discarded_call_origins", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_twr_discarded_call", "--locked"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_popped_static_qualifier", "--locked"],
        ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_exit_gateways", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_loop_arm_join", "--locked"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_loop_boolean_exit", "--locked"],
        ["cargo", "test", "-p", "jarde-java", "--test", "p3_patterns", "--locked"],
        ["cargo", "test", "-p", "jarde-reader", "--lib", "--locked"],
        ["cargo", "build", "-p", "jarde-cli", "--locked"],
    ]
    if len(runner.COMMANDS) != 12:
        raise RuntimeError(f"expected exactly 12 validation commands, got {len(runner.COMMANDS)}")
    if (len(runner.PRODUCT_PATHS), len(runner.TEST_PATHS)) != (17, 8):
        raise RuntimeError("the reviewed source-pin closure must contain 17 product and 8 test paths")
    missing_extra = CANONICAL_EXTRA - runner.CANONICAL_PATHS
    if missing_extra:
        raise RuntimeError(f"required frozen probe/acceptance inputs are absent from canonical pins: {sorted(missing_extra)}")

    source_test_file = ROOT / "crates/jarde-java/tests/proved_discarded_call_origins.rs"
    if test_names(source_test_file) != set(REQUIRED_ORIGIN_TESTS):
        raise RuntimeError("caller-supplied origin names must exactly match the newly pinned integration test source")
    if test_names(ROOT / "crates/jarde-java/tests/p3_loop_exit_gateways.rs") != BASELINE_GATEWAY_TESTS:
        raise RuntimeError("gateway source must exactly preserve all 15 names from accepted return-arm CI")
    observed_legacy = {
        4: len(test_names(ROOT / "tests/p3_twr_discarded_call.rs")),
        5: len(test_names(ROOT / "tests/p3_popped_static_qualifier.rs")),
        7: len(test_names(ROOT / "tests/p3_loop_arm_join.rs")),
        8: len(test_names(ROOT / "tests/p3_loop_boolean_exit.rs")),
        9: len(test_names(ROOT / "crates/jarde-java/tests/p3_patterns.rs")),
    }
    source_expected = {4: 8, 5: 4, 7: 4, 8: 3, 9: 85}
    if observed_legacy != source_expected:
        raise RuntimeError(f"legacy source test counts changed: {observed_legacy}")

    if OUT.exists() or CLI_PATH.exists() or METADATA_PATH.exists():
        raise FileExistsError("refusing to overwrite validation output, candidate CLI, or candidate metadata")
    if shutil.disk_usage(ROOT).free < runner.FREE_LIMIT:
        raise RuntimeError("5 GiB free-space guard failed before output creation")
    if runner.target_bytes() > runner.TARGET_LIMIT:
        raise RuntimeError("1 GiB target-size guard failed before output creation")
    baseline_record = validate_baseline_observation()
    accepted_record = validate_old_acceptance()

    OUT.mkdir(parents=True)
    rows: list[dict] = []
    pins_before = {
        "candidate_sources": runner.source_pins(runner.PRODUCT_PATHS),
        "test_sources": runner.source_pins(runner.TEST_PATHS),
        "canonical_files": runner.source_pins(runner.CANONICAL_PATHS),
    }
    preflight = {
        "git_head": None, "source_pins_before": pins_before, "source_pins_after": None,
        "discarded_call_source_baseline": baseline_record,
        "accepted_return_arm_ci": accepted_record,
        "source_test_counts": {"twr_discarded_call": 8, "popped_static_qualifier": 4,
                                "loop_arm_join": 4, "loop_boolean_exit": 3, "p3_patterns": 85,
                                "gateway": 15, "new_origin": EXPECTED_ORIGIN_TEST_COUNT,
                                "java_lib": EXPECTED_LIB_COUNT},
        "canonical_closure": {"test_paths": len(runner.TEST_PATHS),
                               "discovered_literal_includes": len(literal_paths),
                               "additional_frozen_inputs": len(CANONICAL_EXTRA),
                               "unique_canonical_files": len(runner.CANONICAL_PATHS)},
        "uncommitted_discarded_call_product": True,
    }
    env = os.environ.copy()
    stripped = []
    for key in runner.STRIPPED_ENV_KEYS:
        if key in env:
            env.pop(key)
            stripped.append(key)
    env.update(runner.ENV_VALUES)
    try:
        preflight["stripped_environment_keys"] = stripped
        preflight["jdk23"] = runner.configure_jdk23(env)
        head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, check=False)
        head_out, head_err = OUT / "git-head.stdout.raw", OUT / "git-head.stderr.raw"
        head_out.write_bytes(head.stdout)
        head_err.write_bytes(head.stderr)
        head_value = head.stdout.decode("ascii", errors="replace").strip()
        preflight["git_head"] = {
            "argv": ["git", "rev-parse", "HEAD"], "exit_code": head.returncode, "value": head_value,
            "matches_expected": head.returncode == 0 and head_value == SOURCE_COMMIT_BASE,
            "stdout": runner.command_stream(head_out), "stderr": runner.command_stream(head_err),
        }
        if not preflight["git_head"]["matches_expected"]:
            raise RuntimeError("HEAD does not exactly match caller-supplied --source-base")

        cleanup: list[dict] = []
        runner.clean_check_only_metadata(cleanup)
        preflight["check_only_metadata_cleanup"] = cleanup
        write_execution(runner, rows, preflight, "running")
        if shutil.disk_usage(ROOT).free < runner.FREE_LIMIT or runner.target_bytes() > runner.TARGET_LIMIT:
            raise RuntimeError("disk guard failed after check-only metadata cleanup")

        for index, argv in enumerate(runner.COMMANDS):
            row = runner.run_command(index, argv, env)
            rows.append(row)
            write_execution(runner, rows, preflight, "running")
            print(json.dumps(row, ensure_ascii=False), flush=True)
            is_test = argv[:2] == ["cargo", "test"]
            if (row["exit_code"] != 0 or row["guard_stop"] is not None
                    or (is_test and (row["test_summary_check"] is None or not row["test_summary_check"]["ok"]))
                    or (row["test_summary_check"] is not None and not row["test_summary_check"]["ok"])):
                raise RuntimeError(f"validation command {index} failed exit, summary, or disk guard")
            row["cleanup"] = {"test_binaries": [], "check_only_metadata": []}
            if index == 1:
                runner.clean_check_only_metadata(row["cleanup"]["check_only_metadata"])
            if is_test:
                runner.clean_successful_test_binary(index, row["cleanup"]["test_binaries"])
            write_execution(runner, rows, preflight, "running")

        pins_after = {
            "candidate_sources": runner.source_pins(runner.PRODUCT_PATHS),
            "test_sources": runner.source_pins(runner.TEST_PATHS),
            "canonical_files": runner.source_pins(runner.CANONICAL_PATHS),
        }
        preflight["source_pins_after"] = pins_after
        if pins_after != pins_before:
            raise RuntimeError("source pins changed during validation")
        binary = ROOT / "target/debug/jarde-cli"
        if not binary.is_file():
            raise FileNotFoundError(f"successful CLI build did not produce {binary}")
        CLI_PATH.parent.mkdir(parents=True, exist_ok=True)
        temporary = CLI_PATH.with_name(f".{CLI_PATH.name}.tmp-{os.getpid()}")
        if temporary.exists() or CLI_PATH.exists():
            raise FileExistsError("refusing to overwrite frozen CLI or temporary destination")
        shutil.copyfile(binary, temporary)
        os.chmod(temporary, 0o555)
        os.replace(temporary, CLI_PATH)
        freeze = {
            "cli_path": str(CLI_PATH), "cli_sha256": sha_file(CLI_PATH),
            "cli_mode": oct(stat.S_IMODE(CLI_PATH.stat().st_mode)),
            "metadata_path": str(METADATA_PATH), "source_commit_base": SOURCE_COMMIT_BASE,
            "uncommitted_discarded_call_product": True,
            "validation_runner": {"path": str(Path(__file__).resolve()), "sha256": sha_file(Path(__file__).resolve())},
            "guarded_runner_template": {"path": str(TEMPLATE), "sha256": TEMPLATE_SHA256},
            "product_path_sets": {"candidate_sources": sorted(runner.PRODUCT_PATHS),
                                  "test_sources": sorted(runner.TEST_PATHS),
                                  "canonical_files": sorted(runner.CANONICAL_PATHS)},
            "required_origin_tests": list(REQUIRED_ORIGIN_TESTS),
            "expected_origin_test_count": EXPECTED_ORIGIN_TEST_COUNT,
            "expected_library_test_count": EXPECTED_LIB_COUNT,
        }
        write_execution(runner, rows, preflight, "validation-passed-cli-frozen", freeze)
        metadata = {
            "schema": "preserve-proved-discarded-call-origins-candidate-cli-v1",
            **freeze,
            "candidate_sources": pins_after["candidate_sources"],
            "test_sources": pins_after["test_sources"],
            "canonical_files": pins_after["canonical_files"],
            "build_result_sha256": sha_file(OUT / "execution.json"),
        }
        with METADATA_PATH.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(metadata, ensure_ascii=False, indent=2) + "\n")
        return 0
    except Exception as error:
        write_execution(runner, rows, {**preflight, "failure": f"{type(error).__name__}: {error}"}, "failed")
        print(f"validation stopped: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
