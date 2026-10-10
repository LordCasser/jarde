#!/usr/bin/env python3
"""Verify the frozen integer-array product's exact CI run and build evidence."""

from __future__ import annotations

import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import stat
import subprocess
import sys


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CI_EVIDENCE = HERE / "ci-product-v1"
RESULT = CI_EVIDENCE / "acceptance-int-array-v1.json"

STATIC_HELPER = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/verify-ci-product-root-v12.py"
STATIC_HELPER_SHA256 = "806547956c91e1213b3549ea641497018b97970a5a7796c91c06d345f362d55d"
METADATA_PATH = HERE / "candidate-cli-v1.json"
METADATA_SHA256 = "c97aa02f3af4bab9064e63f9431ecaf61c4c1769e4693529cf5a0d1cdfc72180"
CLI_PATH = Path("/private/tmp/jarde-int-array-names-cli-v1")
CLI_SHA256 = "146c657cdf5baeaa2b9c31e2715547ff9f1a4e129decad025f517a93400af3bf"
BUILD_PATH = HERE / "validation-build-root-v1" / "execution.json"
BUILD_SHA256 = "5106f32591df3c119ca56204c9d9bdd5d84cd61682528e8ac0fb22b05d81d65c"
SOURCE_COMMIT_BASE = "192b0bc13eca631f44ebaf28a998db52d4f14040"

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
    "crates/jarde-java/tests/p3_patterns.rs",
    "tests/class_static_initializer_projection.rs",
    "tests/interface_initializer_proof.rs",
}
CANONICAL_FILES = {
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac23-original/classes/ArrayFieldLiteral.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac23-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac8-original/classes/ArrayFieldLiteral.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac8-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/file-inventory.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/manifest.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/original-sources/ArrayFieldLiteral.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/original-sources/Runner.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac23-original/classes/ArrayFieldInitializers.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac23-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac8-original/classes/ArrayFieldInitializers.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac8-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/file-inventory.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/manifest.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/original-sources/ArrayFieldInitializers.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/original-sources/Runner.java",
}

WORKSPACE_TESTS = (
    "facade::integer_constant_name_tests::incomplete_field_table_never_proves_an_integer_name",
    "facade::integer_constant_name_tests::fixed_switch_names_keep_physical_keys_returns_and_origins",
    "facade::integer_constant_name_tests::output_limit_or_cancellation_never_publishes_partial_name_projection",
    "facade::integer_constant_name_tests::direct_int_array_names_have_exact_leaf_spans_and_keep_switch_projection",
    "facade::integer_constant_name_tests::staged_same_method_text_is_not_overwritten_after_a_valid_replay",
    "facade::integer_constant_name_tests::direct_integer_return_names_independently_of_a_numeric_case_key",
    "facade::integer_constant_name_tests::direct_array_names_reject_unsupported_shapes_shadowing_duplicates_and_accept_assert_body",
    "facade::integer_constant_name_tests::case_and_return_candidates_are_independent_with_narrow_refusals",
    "report::integer_array_name_projection_tests::array_leaf_ranges_are_exact_and_same_bci_ambiguity_refuses",
    "emit::tests::class_source_body_replay_propagates_an_evidence_budget_stop",
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read(path: Path) -> bytes:
    return path.read_bytes()


def git_blob(product: str, relative: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{product}:{relative}"], cwd=ROOT)


def load_fixed_helper():
    helper_bytes = read(STATIC_HELPER)
    assert sha(helper_bytes) == STATIC_HELPER_SHA256
    spec = importlib.util.spec_from_file_location("fixed_static_ci_verifier_v12", STATIC_HELPER)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    assert module.ROOT == ROOT
    module.CI_EVIDENCE = CI_EVIDENCE
    module.METADATA_PATH = METADATA_PATH
    return module


def verify_metadata_and_build(product: str) -> dict:
    metadata_bytes = read(METADATA_PATH)
    assert sha(metadata_bytes) == METADATA_SHA256
    metadata = json.loads(metadata_bytes)
    assert metadata["cli_path"] == str(CLI_PATH)
    assert metadata["cli_sha256"] == CLI_SHA256
    assert metadata["build_result_sha256"] == BUILD_SHA256
    assert metadata["source_commit_base"] == SOURCE_COMMIT_BASE
    assert metadata["uncommitted_int_product"] is True
    assert set(metadata["candidate_sources"]) == PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == TEST_SOURCES
    assert set(metadata["canonical_files"]) == CANONICAL_FILES
    assert len(PRODUCT_SOURCES) == 10 and len(TEST_SOURCES) == 4 and len(CANONICAL_FILES) == 16

    cli_bytes = read(CLI_PATH)
    assert sha(cli_bytes) == CLI_SHA256
    cli_mode = stat.S_IMODE(CLI_PATH.stat().st_mode)
    assert cli_mode == 0o555

    verified_blobs = {}
    for category in ("candidate_sources", "test_sources", "canonical_files"):
        for relative, expected in metadata[category].items():
            blob = git_blob(product, relative)
            assert sha(blob) == expected, (category, relative)
            verified_blobs[relative] = {"category": category, "sha256": expected}
    assert len(verified_blobs) == 30

    build_bytes = read(BUILD_PATH)
    assert sha(build_bytes) == BUILD_SHA256
    build = json.loads(build_bytes)
    assert build["schema"] == "int-array-constant-names-validation-build-root-v1"
    assert build["status"] == "validation-passed-cli-frozen"
    assert build["source_commit_base_expected"] == SOURCE_COMMIT_BASE
    assert len(build["commands"]) == 7
    assert build["preflight"]["git_head"]["matches_expected"] is True
    pins_before = build["preflight"]["source_pins_before"]
    pins_after = build["preflight"]["source_pins_after"]
    assert pins_before == pins_after
    assert pins_after == {
        "candidate_sources": metadata["candidate_sources"],
        "test_sources": metadata["test_sources"],
        "canonical_files": metadata["canonical_files"],
    }

    commands = build["commands"]
    expected_exact = {
        1: ["cargo", "test", "-p", "jarde-java", "--lib", "instance_array", "--locked", "--", "--nocapture"],
        2: ["cargo", "test", "-p", "jarde", "--lib", "common_instance_array_initializer_tests", "--locked", "--", "--nocapture"],
        3: ["cargo", "test", "-p", "jarde", "--test", "class_static_initializer_projection", "--test", "interface_initializer_proof", "--locked"],
        4: ["cargo", "test", "-p", "jarde-reader", "--lib", "--locked"],
        5: ["cargo", "test", "-p", "jarde", "--test", "p5_corpus_fingerprint", "--locked"],
        6: ["cargo", "build", "-p", "jarde-cli", "--locked"],
    }
    for index, argv in expected_exact.items():
        assert commands[index]["argv"] == argv, (index, commands[index]["argv"])
    clippy = commands[0]["argv"]
    assert clippy[:10] == ["cargo", "clippy", "-p", "jarde", "-p", "jarde-java",
                           "--lib", "--tests", "--all-features", "--locked"]
    assert clippy[-2:] == ["-D", "warnings"]
    for command in commands:
        assert command["exit_code"] == 0 and command["guard_stop"] is None
        for stream in command["streams"].values():
            path = Path(stream["path"])
            if not path.is_absolute():
                path = ROOT / path
            data = read(path)
            assert len(data) == stream["bytes"] and sha(data) == stream["sha256"]

    freeze = build["freeze"]
    assert freeze["built_binary"] == str(ROOT / "target/debug/jarde-cli")
    assert freeze["cli_path"] == str(CLI_PATH) and freeze["cli_sha256"] == CLI_SHA256
    assert freeze["cli_mode"] == "0o555"
    assert freeze["metadata_path"] == str(METADATA_PATH)
    assert freeze["source_commit_base"] == SOURCE_COMMIT_BASE
    assert freeze["uncommitted_int_product"] is True
    assert freeze["product_path_sets"] == {
        "candidate_sources": sorted(PRODUCT_SOURCES),
        "test_sources": sorted(TEST_SOURCES),
        "canonical_files": sorted(CANONICAL_FILES),
    }
    return {
        "metadata_sha256": METADATA_SHA256,
        "cli_path": str(CLI_PATH),
        "cli_sha256": CLI_SHA256,
        "cli_mode": oct(cli_mode),
        "source_commit_base": SOURCE_COMMIT_BASE,
        "uncommitted_int_product": True,
        "pin_counts": {"candidate_sources": 10, "test_sources": 4, "canonical_files": 16},
        "product_git_blobs_verified": len(verified_blobs),
        "worktree_source_bytes_used_as_gate": False,
        "build_execution_sha256": BUILD_SHA256,
        "build_commands": len(commands),
        "all_build_commands_exit_zero_no_guard_stop": True,
        "build_streams_verified": sum(len(row["streams"]) for row in commands),
    }


def verify_integer_tests_in_both_seeds(helper, product: str) -> list[dict]:
    workflow = git_blob(product, ".github/workflows/ci.yml").decode("utf-8")
    seeds = re.findall(r'PROPTEST_RNG_SEED:\s*"(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    stable_gzip = read(CI_EVIDENCE / "ci-stable-job-v1.log.gz")
    stable_raw = gzip.decompress(stable_gzip)
    stable_text = helper.strip_terminal(stable_raw)
    groups = helper.command_groups(stable_text.splitlines(), helper.WORKSPACE_COMMAND)
    assert len(groups) == 2
    results = []
    step_names = (
        "Run workspace tests with two fixed independent seeds (pass 1)",
        "Run workspace tests with two fixed independent seeds (pass 2)",
    )
    for (index, text), step_name, seed in zip(groups, step_names, seeds):
        assert seed in text and all(other not in text for other in seeds if other != seed)
        for test_name in WORKSPACE_TESTS:
            assert re.search(r"test " + re.escape(test_name) + r" \.\.\. ok", text), test_name
        counts = helper.test_counts(text)
        assert counts and all(failed == 0 for _, failed, _ in counts)
        assert not re.search(r"test result: FAILED|error: test failed", text)
        results.append({
            "step": step_name,
            "command_group_index": index,
            "seed": seed,
            "required_tests": list(WORKSPACE_TESTS),
            "required_tests_ok": len(WORKSPACE_TESTS),
            "actual_test_result_records": len(counts),
            "actual_passed": sum(passed for passed, _, _ in counts),
            "actual_failed": sum(failed for _, failed, _ in counts),
            "actual_ignored": sum(ignored for _, _, ignored in counts),
        })
    return results


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: verify-int-array-ci-product-luna-v1.py PRODUCT_SHA RUN_ID")
    product, run_id_text = sys.argv[1:]
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", run_id_text):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(run_id_text)
    if RESULT.exists():
        raise SystemExit(f"refusing to overwrite {RESULT}")
    if not CI_EVIDENCE.is_dir():
        raise SystemExit(f"missing captured CI evidence directory: {CI_EVIDENCE}")

    helper = load_fixed_helper()
    ci = helper.verify_ci(product, run_id)
    assert ci["jobs"] == 4 and ci["step_count"] == 52
    assert ci["all_jobs_and_steps_success"] is True
    build = verify_metadata_and_build(product)
    int_tests = verify_integer_tests_in_both_seeds(helper, product)
    assert len(int_tests) == 2

    result = {
        "schema": "int-array-ci-product-luna-acceptance-v1",
        "status": "accepted",
        "product_commit": product,
        "run_id": run_id,
        "shared_static_ci_helper": {
            "path": str(STATIC_HELPER.relative_to(ROOT)),
            "sha256": STATIC_HELPER_SHA256,
            "verify_ci_called": True,
            "main_called": False,
        },
        "ci": ci,
        "integer_array_workspace_tests": int_tests,
        "frozen_cli_and_build": build,
        "scope": "CI and source identity are checked against the submitted commit's Git blobs and its pinned CLI/build evidence; current worktree source bytes do not gate acceptance.",
    }
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "product_commit": product,
                      "run_id": run_id, "jobs": ci["jobs"], "steps": ci["step_count"],
                      "required_seed_runs": len(int_tests)}))


if __name__ == "__main__":
    main()
