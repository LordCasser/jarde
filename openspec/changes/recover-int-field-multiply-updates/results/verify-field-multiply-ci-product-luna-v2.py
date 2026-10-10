#!/usr/bin/env python3
"""Independently verify the multiply product's frozen build and captured CI evidence."""

from __future__ import annotations

import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import posixpath
import re
import stat
import subprocess
import sys


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CI_EVIDENCE = HERE / "ci-product-v1"
RESULT = CI_EVIDENCE / "acceptance-field-multiply-v2.json"

STATIC_HELPER = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/verify-ci-product-root-v12.py"
STATIC_HELPER_SHA256 = "806547956c91e1213b3549ea641497018b97970a5a7796c91c06d345f362d55d"
INTEGER_VERIFIER = ROOT / "openspec/changes/recover-int-array-constant-names/results/verify-int-array-ci-product-luna-v2.py"
INTEGER_VERIFIER_SHA256 = "e5534e0b2acc6066ba518a1b55f5d0f79b9e5e25a1c4dc5db768040b6d47b425"
METADATA_PATH = HERE / "candidate-cli-v2.json"
CLI_PATH = Path("/private/tmp/jarde-field-multiply-cli-v2")
BUILD_PATH = HERE / "validation-build-root-v2" / "execution.json"
SOURCE_COMMIT_BASE = "977f761d9f68c6cb4de02f42b060de5290a1a947"

PRODUCT_SOURCES = {
    "Cargo.lock",
    "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/build.rs",
    "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/field.rs",
    "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs",
    "src/class_source.rs",
    "src/facade.rs",
}
TEST_SOURCES = {
    ".github/workflows/ci.yml",
    "tests/p3_compound_lvalue_updates.rs",
}
EM23_BASE = "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2"
MULTIPLY_BASE = "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1"
CANONICAL_FIXED = {
    f"{EM23_BASE}/manifest.json",
    f"{EM23_BASE}/file-inventory.json",
    f"{EM23_BASE}/original-sources/InputFieldIncrement2.java",
    f"{EM23_BASE}/original-sources/Runner.java",
    f"{EM23_BASE}/cases/javac23-original/classes/em23/InputFieldIncrement2.class",
    f"{EM23_BASE}/cases/javac23-original/classes/em23/InputFieldIncrement2$A.class",
    f"{MULTIPLY_BASE}/manifest.json",
    f"{MULTIPLY_BASE}/file-inventory.json",
    f"{MULTIPLY_BASE}/original-sources/InputFieldMultiplyControls.java",
    f"{MULTIPLY_BASE}/original-sources/Runner.java",
    f"{MULTIPLY_BASE}/cases/javac23-original/classes/em23/InputFieldMultiplyControls.class",
    f"{MULTIPLY_BASE}/cases/javac23-original/classes/em23/InputFieldMultiplyControls$A.class",
}
INCLUDE_RE = re.compile(r"(?:include_bytes|include_str)!\s*\(\s*\"([^\"]+)\"\s*\)")
WORKSPACE_COMMAND = "cargo test --workspace --all-targets --all-features --locked"
MULTIPLY_TESTS = (
    "nested_int_field_multiply_keeps_every_instruction_and_field_read_presented",
    "multiply_does_not_merge_two_reads_or_cross_field_width_or_consumer_boundaries",
    "multiply_public_recovery_keeps_output_budget_and_cancellation_atomic",
)
ENV_VALUES = {
    "CARGO_BUILD_JOBS": "2",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
}
TARGET_LIMIT = 1024**3
FREE_LIMIT = 20 * 1024**3
SUMMARY_RE = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read(path: Path) -> bytes:
    return path.read_bytes()


def git_blob(product: str, relative: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{product}:{relative}"], cwd=ROOT)


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def load_fixed_helpers():
    helper_bytes = read(STATIC_HELPER)
    assert sha(helper_bytes) == STATIC_HELPER_SHA256, "static CI helper bytes changed"
    helper = load_module("fixed_static_ci_verifier_v12_multiply", STATIC_HELPER)
    assert helper.ROOT == ROOT
    helper.CI_EVIDENCE = CI_EVIDENCE
    helper.METADATA_PATH = METADATA_PATH

    integer_bytes = read(INTEGER_VERIFIER)
    assert sha(integer_bytes) == INTEGER_VERIFIER_SHA256, "accepted integer verifier bytes changed"
    integer = load_module("accepted_integer_ci_verifier_v2_multiply", INTEGER_VERIFIER)
    integer.CI_EVIDENCE = CI_EVIDENCE
    return helper, integer


def expected_canonical_files(product: str) -> set[str]:
    source = git_blob(product, "tests/p3_compound_lvalue_updates.rs").decode("utf-8")
    included = INCLUDE_RE.findall(source)
    assert included, "submitted compound test must retain literal fixture includes"
    paths = set(CANONICAL_FIXED)
    for value in included:
        relative = posixpath.normpath(posixpath.join("tests", value))
        assert relative != ".." and not relative.startswith("../"), value
        paths.add(relative)
    return paths


def expected_clippy_argv(workflow: str) -> list[str]:
    start = workflow.index("      - name: Run Clippy")
    end = workflow.index("      - name: Run workspace tests", start)
    block = workflow[start:end]
    allowed = re.findall(r"(?m)^\s+-A clippy::([a-z0-9_]+)\s*$", block)
    assert len(allowed) == 29 and len(set(allowed)) == 29
    return ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--",
            *sum((["-A", f"clippy::{name}"] for name in allowed), []), "-D", "warnings"]


def verify_build_and_candidate(product: str, metadata_sha256: str, cli_sha256: str,
                               build_sha256: str) -> dict:
    for label, value in (("metadata", metadata_sha256), ("CLI", cli_sha256), ("build", build_sha256)):
        assert re.fullmatch(r"[0-9a-f]{64}", value), f"{label} SHA-256 must be 64 lowercase hex characters"

    metadata_bytes = read(METADATA_PATH)
    assert sha(metadata_bytes) == metadata_sha256
    metadata = json.loads(metadata_bytes)
    assert metadata["cli_path"] == str(CLI_PATH)
    assert metadata["cli_sha256"] == cli_sha256
    assert metadata["build_result_sha256"] == build_sha256
    assert metadata["source_commit_base"] == SOURCE_COMMIT_BASE
    assert metadata["uncommitted_field_multiply_product"] is True

    expected_canonical = expected_canonical_files(product)
    assert set(metadata["candidate_sources"]) == PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == TEST_SOURCES
    assert set(metadata["canonical_files"]) == expected_canonical
    assert len(PRODUCT_SOURCES) == 10 and len(TEST_SOURCES) == 2

    git_pins = {}
    for category, expected_paths in (("candidate_sources", PRODUCT_SOURCES),
                                     ("test_sources", TEST_SOURCES),
                                     ("canonical_files", expected_canonical)):
        pins = metadata[category]
        assert set(pins) == expected_paths, category
        for relative, expected_sha in pins.items():
            data = git_blob(product, relative)
            assert sha(data) == expected_sha, (category, relative)
            git_pins[relative] = {"category": category, "bytes": len(data), "sha256": expected_sha}
    assert len(git_pins) == len(PRODUCT_SOURCES | TEST_SOURCES | expected_canonical)

    cli_bytes = read(CLI_PATH)
    cli_mode = stat.S_IMODE(CLI_PATH.stat().st_mode)
    assert sha(cli_bytes) == cli_sha256 and cli_mode == 0o555

    build_bytes = read(BUILD_PATH)
    assert sha(build_bytes) == build_sha256
    build = json.loads(build_bytes)
    assert build["schema"] == "int-field-multiply-validation-build-root-v2"
    assert build["status"] == "validation-passed-cli-frozen"
    assert build["source_commit_base_expected"] == SOURCE_COMMIT_BASE
    assert build["uncommitted_field_multiply_product"] is True
    assert build["environment_overrides"] == ENV_VALUES
    assert build["guards"] == {"minimum_free_bytes": FREE_LIMIT, "maximum_target_bytes": TARGET_LIMIT}
    assert build["preflight"]["git_head"]["matches_expected"] is True
    assert build["preflight"]["git_head"]["value"] == SOURCE_COMMIT_BASE
    head_record = build["preflight"]["git_head"]
    assert head_record["argv"] == ["git", "rev-parse", "HEAD"] and head_record["exit_code"] == 0
    head_stdout = head_record["stdout"]
    head_stderr = head_record["stderr"]
    for stream_name, stream in (("git-head stdout", head_stdout), ("git-head stderr", head_stderr)):
        raw_path = Path(stream["path"])
        if not raw_path.is_absolute():
            raw_path = ROOT / raw_path
        try:
            raw_path.resolve().relative_to(ROOT.resolve())
        except ValueError as error:
            raise AssertionError(f"{stream_name} path escapes the project root") from error
        raw = read(raw_path)
        assert len(raw) == stream["bytes"] and sha(raw) == stream["sha256"], stream_name
        if stream_name == "git-head stdout":
            assert raw.decode("ascii").strip() == SOURCE_COMMIT_BASE
    before = build["preflight"]["source_pins_before"]
    after = build["preflight"]["source_pins_after"]
    metadata_pins = {"candidate_sources": metadata["candidate_sources"],
                     "test_sources": metadata["test_sources"],
                     "canonical_files": metadata["canonical_files"]}
    assert before == after == metadata_pins

    expected_commands = [
        ["cargo", "fmt", "--all", "--", "--check"],
        expected_clippy_argv(git_blob(product, ".github/workflows/ci.yml").decode("utf-8")),
        ["cargo", "test", "-p", "jarde-java", "--lib", "--locked"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_compound_lvalue_updates", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde-java", "--lib", "integer_array_name_projection_tests", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde", "--lib", "integer_constant_name_tests", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde-reader", "--lib", "--locked"],
        ["cargo", "test", "-p", "jarde", "--test", "p5_corpus_fingerprint", "--locked"],
        ["cargo", "build", "-p", "jarde-cli", "--locked"],
    ]
    commands = build["commands"]
    assert len(commands) == len(expected_commands) == 9
    expected_summary = {
        2: None, 3: [(7, 0, 0)], 4: [(2, 0, 0)], 5: [(8, 0, 0)],
        6: [(178, 0, 0)], 7: [(5, 0, 1)],
    }
    recomputed_summaries = {}
    for index, (row, argv) in enumerate(zip(commands, expected_commands)):
        assert row["index"] == index and row["argv"] == argv, index
        assert row["cwd"] == str(ROOT), index
        assert row["exit_code"] == 0 and row["guard_stop"] is None, index
        assert row["env_overrides"] == ENV_VALUES, index
        assert row["peak_target_bytes"] <= TARGET_LIMIT
        assert row["free_bytes_after"] >= FREE_LIMIT
        captured_streams = {}
        for stream_name, stream in row["streams"].items():
            raw_path = Path(stream["path"])
            if not raw_path.is_absolute():
                raw_path = ROOT / raw_path
            try:
                raw_path.resolve().relative_to(ROOT.resolve())
            except ValueError as error:
                raise AssertionError(f"command {index} {stream_name} path escapes the project root") from error
            raw = read(raw_path)
            assert len(raw) == stream["bytes"] and sha(raw) == stream["sha256"], (index, stream_name)
            captured_streams[stream_name] = raw

        if index in expected_summary:
            check = row["test_summary_check"]
            assert check is not None and check["ok"] is True, index
            assert check["required"] is True and check["expected"] == expected_summary[index], index
            assert "stdout" in captured_streams, index
            stdout_text = captured_streams["stdout"].decode("utf-8", errors="strict")
            actual = [tuple(map(int, match)) for match in SUMMARY_RE.findall(stdout_text)]
            recorded = [tuple(record) for record in check["actual"]]
            assert actual, f"command {index} stdout has no test result records"
            assert all(failed == 0 for _, failed, _ in actual), (index, actual)
            assert actual == recorded, (index, actual, recorded)
            if expected_summary[index] is not None:
                assert actual == expected_summary[index], (index, actual, expected_summary[index])
            recomputed_summaries[str(index)] = [list(record) for record in actual]
        else:
            assert row["test_summary_check"] is None
    assert sum(len(row["streams"]) for row in commands) == 18

    freeze = build["freeze"]
    assert freeze["built_binary"] == str(ROOT / "target/debug/jarde-cli")
    assert freeze["cli_path"] == str(CLI_PATH) and freeze["cli_sha256"] == cli_sha256
    assert freeze["cli_mode"] == "0o555" and freeze["metadata_path"] == str(METADATA_PATH)
    assert freeze["source_commit_base"] == SOURCE_COMMIT_BASE
    assert freeze["uncommitted_field_multiply_product"] is True
    assert freeze["product_path_sets"] == {
        "candidate_sources": sorted(PRODUCT_SOURCES),
        "test_sources": sorted(TEST_SOURCES),
        "canonical_files": sorted(expected_canonical),
    }
    return {
        "candidate_metadata_sha256": metadata_sha256,
        "candidate_cli": {"path": str(CLI_PATH), "sha256": cli_sha256, "mode": oct(cli_mode)},
        "candidate_build_sha256": build_sha256,
        "product_commit_git_blob_pins_verified": len(git_pins),
        "pin_counts": {"candidate_sources": len(PRODUCT_SOURCES),
                        "test_sources": len(TEST_SOURCES), "canonical_files": len(expected_canonical)},
        "source_commit_base": SOURCE_COMMIT_BASE,
        "uncommitted_field_multiply_product": True,
        "build_commands": len(commands), "build_streams_verified": 18,
        "build_test_result_records_recomputed_from_stdout": recomputed_summaries,
        "all_build_commands_exit_zero_no_guard_stop": True,
        "workspace_source_bytes_used_as_acceptance_gate": False,
    }


def verify_multiply_workspace_tests(helper, product: str, integer_verifier) -> dict:
    workflow = git_blob(product, ".github/workflows/ci.yml").decode("utf-8")
    seeds = re.findall(r'PROPTEST_RNG_SEED:\s*"(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    assert workflow.count(f"run: {WORKSPACE_COMMAND}") == 2
    stable_gzip = read(CI_EVIDENCE / "ci-stable-job-v1.log.gz")
    stable_text = helper.strip_terminal(gzip.decompress(stable_gzip))
    groups = helper.command_groups(stable_text.splitlines(), WORKSPACE_COMMAND)
    assert len(groups) == 2
    seed_runs = []
    steps = ("Run workspace tests with two fixed independent seeds (pass 1)",
             "Run workspace tests with two fixed independent seeds (pass 2)")
    for (group_index, text), step, seed in zip(groups, steps, seeds):
        assert seed in text and all(other not in text for other in seeds if other != seed)
        lines = text.splitlines()
        block, counts = helper.binary_group(lines, "tests/p3_compound_lvalue_updates.rs", 7)
        for test_name in MULTIPLY_TESTS:
            assert re.search(r"test " + re.escape(test_name) + r" \.\.\. ok", block), (step, test_name)
        assert all(failed == 0 for _, failed, _ in counts)
        totals = helper.test_counts(text)
        assert totals and all(failed == 0 for _, failed, _ in totals)
        seed_runs.append({
            "step": step, "seed": seed, "command_group_index": group_index,
            "focused_binary": "tests/p3_compound_lvalue_updates.rs",
            "focused_test_counts": [list(row) for row in counts],
            "multiply_tests": list(MULTIPLY_TESTS),
            "multiply_test_results": {name: "ok" for name in MULTIPLY_TESTS},
            "workspace_result_records": len(totals),
            "workspace_passed": sum(row[0] for row in totals),
            "workspace_failed": sum(row[1] for row in totals),
            "workspace_ignored": sum(row[2] for row in totals),
        })
    assert len(seed_runs) == 2

    integer_verifier.CI_EVIDENCE = CI_EVIDENCE
    integer_runs = integer_verifier.verify_integer_tests_in_both_seeds(helper, product)
    assert len(integer_verifier.WORKSPACE_TESTS) == 11 and len(integer_runs) == 2
    return {"multiply_workspace_tests": seed_runs,
            "prior_integer_array_tests": integer_runs,
            "prior_integer_array_test_count": len(integer_verifier.WORKSPACE_TESTS)}


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("product_sha", help="submitted product commit SHA")
    parser.add_argument("run_id", help="captured GitHub Actions run ID")
    parser.add_argument("--metadata-sha256", required=True)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--build-sha256", required=True)
    args = parser.parse_args()
    product = args.product_sha
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", args.run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(args.run_id)
    if RESULT.exists():
        raise SystemExit(f"refusing to overwrite {RESULT}")
    if not CI_EVIDENCE.is_dir():
        raise SystemExit(f"missing captured CI evidence directory: {CI_EVIDENCE}")

    helper, integer_verifier = load_fixed_helpers()
    build = verify_build_and_candidate(product, args.metadata_sha256.lower(),
                                       args.cli_sha256.lower(), args.build_sha256.lower())
    ci = helper.verify_ci(product, run_id)
    assert ci["jobs"] == 4 and ci["step_count"] == 52
    assert ci["all_jobs_and_steps_success"] is True
    workspace = verify_multiply_workspace_tests(helper, product, integer_verifier)

    result = {
        "schema": "int-field-multiply-ci-product-luna-acceptance-v2",
        "status": "accepted",
        "product_commit": product,
        "run_id": run_id,
        "source_commit_base": SOURCE_COMMIT_BASE,
        "shared_ci_verification": {
            "static_helper": {"path": str(STATIC_HELPER.relative_to(ROOT)),
                              "sha256": STATIC_HELPER_SHA256,
                              "verify_ci_called": True, "main_called": False},
            "accepted_integer_verifier": {"path": str(INTEGER_VERIFIER.relative_to(ROOT)),
                                          "sha256": INTEGER_VERIFIER_SHA256,
                                          "integer_test_helper_called": True},
        },
        "ci": ci,
        "workspace_regressions": workspace,
        "frozen_cli_and_build": build,
        "acceptance_scope": "submitted product Git blobs, frozen candidate CLI/build, four successful CI jobs and all recorded steps, and test-result counts independently parsed from frozen stdout, plus both fixed-seed multiply and prior integer regression tests",
    }
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "product_commit": product,
                      "run_id": run_id, "jobs": ci["jobs"], "steps": ci["step_count"],
                      "multiply_tests_per_seed": len(MULTIPLY_TESTS),
                      "prior_integer_tests_per_seed": workspace["prior_integer_array_test_count"]}))


if __name__ == "__main__":
    main()
