#!/usr/bin/env python3
"""Verify the committed instance-array product CI and fixed local evidence."""

from __future__ import annotations

import gzip
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CI_EVIDENCE = HERE / "ci-product-v1"
RESULT = CI_EVIDENCE / "instance-ci-acceptance-root-v3.json"
PRODUCT = "192b0bc13eca631f44ebaf28a998db52d4f14040"
RUN_ID = 38022628853

STATIC_HELPER = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/verify-ci-product-root-v12.py"
STATIC_HELPER_SHA256 = "806547956c91e1213b3549ea641497018b97970a5a7796c91c06d345f362d55d"
METADATA = HERE / "candidate-cli-v1.json"
METADATA_SHA256 = "b86961182a352dfa663215c47bdd3e64cf7f8a67c5e36951f2c98d020f044cfa"
CLI = Path("/private/tmp/jarde-instance-array-cli-v1")
CLI_SHA256 = "5abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663"
BUILD = HERE / "validation-build-root-v3/execution.json"
BUILD_SHA256 = "838a69752acb2e23afb22024b6fe706d0fce14801b80fcbcc7b0a8a76de85705"

CANDIDATE_ROOT = HERE / "instance-candidate-replay-root-v1"
CANDIDATE_MANIFEST = CANDIDATE_ROOT / "manifest.json"
CANDIDATE_INVENTORY = CANDIDATE_ROOT / "file-inventory.json"
CANDIDATE_MANIFEST_SHA256 = "01ac7281244e5d95d92b0218098d2b2e0637fab9b3b27bd2db7210c85b991c50"
CANDIDATE_INVENTORY_SHA256 = "295647aa0748c6db0bc09a6d6c960db98d567dafe8c26f5e1d49066862e0f684"
CANDIDATE_ACCEPTANCE = HERE / "instance-candidate-replay-independent-v6.json"
CANDIDATE_ACCEPTANCE_SHA256 = "910df190ceaddb2b57fda15e7b1da6922cf82a359cdee57be91bdbcc405546c6"

STATIC_ROOT = HERE / "static-array-regression-root-v3"
STATIC_MANIFEST = STATIC_ROOT / "manifest.json"
STATIC_INVENTORY = STATIC_ROOT / "file-inventory.json"
STATIC_MANIFEST_SHA256 = "b3080025560171adc84f8dd755240170c1abc218468ef2cd8868df9070a639be"
STATIC_INVENTORY_SHA256 = "46aa8f23f1ade97d0129ba8b7febaf2e6ef5d41bda2e04329b7b1ab1ae38cc95"
STATIC_ACCEPTANCE = HERE / "static-array-regression-root-acceptance-v1.json"
STATIC_ACCEPTANCE_SHA256 = "3661f2b685d4f1f8b52fe882d7e4b59d99793024eae6ef2a94fcffbc1ee50c94"

SCOPED_RUST = HERE / "scoped-rust-root-v7/execution.json"
SCOPED_RUST_SHA256 = "ac756708c9bc0ac659ff26e6fc5d25b97e8926a1b5b2274ad3bad747d812b70c"
READER_FINGERPRINT = HERE / "reader-fingerprint-root-v1/execution.json"
READER_FINGERPRINT_SHA256 = "6380d3e80499e0cc17ff4c6ecc09c501e0e712cd7fe774f8a18e7f9e62c65980"
CHECKPOINT = HERE / "root-checkpoint-validation-v1/execution.json"
CHECKPOINT_SHA256 = "ffb8879251b5722c50f3d703123b450656a95df8a74594b2f4be94611695ab70"

INSTANCE_UNIT_TESTS = (
    "report::instance_array_prefix_tests::budget_stop_returns_no_candidate",
    "report::instance_array_prefix_tests::rhs_comparison_preserves_long_and_float_double_raw_bits",
    "report::instance_array_prefix_tests::static_rhs_call_arguments_and_overload_descriptor_are_exact",
    "report::instance_array_prefix_tests::non_super_handlers_unclaimed_duplicate_and_rhs_mismatch_refuse",
    "report::instance_array_prefix_tests::prefix_emission_keeps_super_arguments_and_final_return",
)
FACADE_TESTS = (
    "facade::common_instance_array_initializer_tests::public_class_source_honors_pre_cancelled_request",
    "facade::common_instance_array_initializer_tests::public_class_source_budget_stop_does_not_publish_a_partial_field_group",
    "facade::common_instance_array_initializer_tests::no_clinit_two_constructor_group_preserves_super_calls_suffixes_and_physical_maps",
    "facade::common_instance_array_initializer_tests::constant_value_on_any_nonstatic_field_refuses_the_whole_group",
    "facade::common_instance_array_initializer_tests::direct_constructor_group_and_single_constructor_literal_are_projected",
    "facade::common_instance_array_initializer_tests::unequal_this_delegating_and_reverse_order_groups_keep_constructor_writes",
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read(path: Path) -> bytes:
    return path.read_bytes()


def read_json(path: Path) -> dict:
    return json.loads(read(path))


def record_data(base: Path, record: dict) -> bytes:
    path = Path(record["path"])
    if not path.is_absolute():
        path = base / path
    data = read(path)
    assert len(data) == record["bytes"] and sha(data) == record["sha256"], str(path)
    return data


def close_inventory(root: Path, manifest_sha: str, inventory_sha: str) -> tuple[dict, list]:
    manifest_bytes, inventory_bytes = read(root / "manifest.json"), read(root / "file-inventory.json")
    assert sha(manifest_bytes) == manifest_sha
    assert sha(inventory_bytes) == inventory_sha
    manifest, inventory = json.loads(manifest_bytes), json.loads(inventory_bytes)
    rows = {row["path"]: row for row in inventory}
    assert len(rows) == len(inventory)
    actual = {}
    for parent, dirs, files in os.walk(root, followlinks=False):
        parent = Path(parent)
        assert all(not (parent / name).is_symlink() for name in dirs)
        for name in files:
            path = parent / name
            assert not path.is_symlink()
            relative = path.relative_to(root).as_posix()
            if relative != "file-inventory.json":
                actual[relative] = path
    assert set(actual) == set(rows), {"unlisted": sorted(set(actual)-set(rows)),
                                      "missing": sorted(set(rows)-set(actual))}
    for relative, path in actual.items():
        data = read(path)
        assert len(data) == rows[relative]["bytes"] and sha(data) == rows[relative]["sha256"], relative
    assert len(inventory) == len(actual)
    policy = manifest["file_inventory"]
    if root == CANDIDATE_ROOT:
        assert policy == {"excludes": ["file-inventory.json"],
                          "includes": ["manifest.json"]}
    elif root == STATIC_ROOT:
        assert policy == {"path": "file-inventory.json", "includes": ["manifest.json"],
                          "excludes": ["file-inventory.json"]}
    else:
        raise AssertionError(f"unexpected inventory root: {root}")
    return manifest, inventory


def raw_triplet(base: Path, row: dict) -> tuple[int, bytes, bytes]:
    return row["exit"], record_data(base, row["stdout"]), record_data(base, row["stderr"])


def load_static_helper():
    helper_bytes = read(STATIC_HELPER)
    assert sha(helper_bytes) == STATIC_HELPER_SHA256
    spec = importlib.util.spec_from_file_location("static_ci_verifier_v12", STATIC_HELPER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    assert module.ROOT == ROOT
    module.CI_EVIDENCE = CI_EVIDENCE
    return module, sha(helper_bytes)


def verify_commit_metadata(helper) -> dict:
    metadata_bytes = read(METADATA)
    assert sha(metadata_bytes) == METADATA_SHA256
    metadata = json.loads(metadata_bytes)
    assert metadata["cli_path"] == str(CLI) and metadata["cli_sha256"] == CLI_SHA256
    assert len(metadata["candidate_sources"]) == 10
    assert len(metadata["test_sources"]) == 4
    assert len(metadata["canonical_files"]) == 16
    assert set(metadata["candidate_sources"]) == helper.PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == helper.TEST_SOURCES
    assert sha(read(CLI)) == CLI_SHA256
    # Only immutable blobs at the submitted product commit gate these pins; current worktree bytes are not read.
    verified = {}
    for category in ("candidate_sources", "test_sources", "canonical_files"):
        for path, expected in metadata[category].items():
            blob = helper.git_blob(PRODUCT, path)
            assert sha(blob) == expected, (category, path)
            verified[path] = expected
    assert len(verified) == 30
    build_bytes = read(BUILD)
    assert sha(build_bytes) == BUILD_SHA256 == metadata["build_result_sha256"]
    build = json.loads(build_bytes)
    assert build["schema"] == "instance-array-validation-build-root-v3"
    assert len(build["commands"]) == 3
    assert [row["argv"][1:3] for row in build["commands"]] == [
        ["clippy", "-p"], ["test", "-p"], ["build", "-p"]]
    assert build["commands"][-1]["argv"] == ["cargo", "build", "-p", "jarde-cli"]
    for row in build["commands"]:
        assert row["exit_code"] == 0 and row["guard_stop"] is None
    return {"metadata_sha256": METADATA_SHA256, "cli_sha256": CLI_SHA256,
            "pin_counts": {"candidate_sources": 10, "test_sources": 4, "canonical_files": 16},
            "commit_blob_pins_verified": len(verified), "worktree_hashes_used_as_gate": False,
            "build_execution_sha256": BUILD_SHA256}


def verify_candidate_replay() -> dict:
    manifest, inventory = close_inventory(CANDIDATE_ROOT, CANDIDATE_MANIFEST_SHA256,
                                          CANDIDATE_INVENTORY_SHA256)
    assert manifest["schema"] == "instance-array-candidate-complete-class-replay-v1"
    assert manifest["status"] == "completed" and manifest["failures"] == []
    assert len(manifest["commands"]) == 58 and len(manifest["cases"]) == 52
    assert manifest["success_count"] == manifest["case_count"] == 52
    assert manifest["inputs"]["cli"]["path"] == str(CLI)
    assert manifest["inputs"]["cli"]["actual_sha256"] == CLI_SHA256
    assert manifest["inputs"]["cli"]["metadata_path"] == str(METADATA)
    assert manifest["inputs"]["candidate_metadata_copy"]["sha256"] == METADATA_SHA256
    assert read(CANDIDATE_ROOT / "inputs/candidate-cli-metadata.json") == read(METADATA)
    accepted_bytes = read(CANDIDATE_ACCEPTANCE)
    assert sha(accepted_bytes) == CANDIDATE_ACCEPTANCE_SHA256
    accepted = json.loads(accepted_bytes)
    assert accepted["schema"] == "instance-array-candidate-independent-verification-v6"
    assert accepted["status"] == "accepted"
    assert accepted["candidate_manifest_sha256"] == CANDIDATE_MANIFEST_SHA256
    assert accepted["candidate_inventory_sha256"] == CANDIDATE_INVENTORY_SHA256
    assert accepted["closed_inventory_file_count"] == 361
    assert accepted["commands"] == 58 and accepted["cases"] == 52
    assert accepted["candidate_cli_sha256"] == CLI_SHA256
    assert accepted["candidate_cli_metadata_sha256"] == METADATA_SHA256
    assert accepted["build_execution_sha256"] == BUILD_SHA256
    assert accepted["complete_source_legs"] == 4
    assert accepted["runtime_legs_compared_to_original_raw"] == 6
    assert accepted["negative_control_texts_equal_accepted_baseline"] == [
        "ThisDelegatingByteArray", "DifferentRhsByteArray", "MissingWriteByteArray",
        "DuplicateWriteByteArray", "InterveningEffectByteArray", "ParameterRhsByteArray",
        "ReverseFieldOrderByteArray", "HandlerArrayByteArray"]
    assert accepted["class_outputs_exact_no_helpers"] is True
    return {"manifest_sha256": CANDIDATE_MANIFEST_SHA256,
            "inventory_sha256": CANDIDATE_INVENTORY_SHA256,
            "closed_files": len(inventory), "commands": 58, "cases": 52,
            "independent_acceptance_sha256": CANDIDATE_ACCEPTANCE_SHA256,
            "candidate_cli_sha256": CLI_SHA256, "metadata_sha256": METADATA_SHA256}


def verify_static_eight_legs() -> dict:
    manifest, inventory = close_inventory(STATIC_ROOT, STATIC_MANIFEST_SHA256, STATIC_INVENTORY_SHA256)
    assert manifest["schema"] == "common-instance-array-static-regression-root-v3"
    assert manifest["status"] == "completed" and manifest["failures"] == []
    assert manifest["case_counts"] == {"candidate_legs": 8, "expected_candidate_legs": 8}
    assert manifest["success_counts"] == {"candidate_legs": 8}
    assert manifest["candidate_cli"]["sha256"] == CLI_SHA256
    assert manifest["metadata"]["sha256"] == METADATA_SHA256
    acceptance_bytes = read(STATIC_ACCEPTANCE)
    assert sha(acceptance_bytes) == STATIC_ACCEPTANCE_SHA256
    acceptance = json.loads(acceptance_bytes)
    assert acceptance["status"] == "accepted"
    assert acceptance["manifest_sha256"] == STATIC_MANIFEST_SHA256
    assert acceptance["inventory_sha256"] == STATIC_INVENTORY_SHA256
    assert acceptance["closed_files"] == 97 and acceptance["commands"] == 24
    assert acceptance["complete_source_runtime_legs"] == 8
    assert acceptance["candidate_cli_sha256"] == CLI_SHA256
    class_facts = {}
    oracle_hashes = []
    for case in manifest["cases"]:
        assert case["success"] and case["compile_success"] and case["runtime_success"]
        assert case["exact_class_set"] and case["runtime_matches_original_raw"]
        assert all(case["proof_checks"].values())
        cls = case["original_class"]
        original_bytes = record_data(STATIC_ROOT, cls)
        identity = f"{case['family']}/{case['jdk_leg']}"
        current = {"bytes": cls["bytes"], "sha256": cls["sha256"]}
        assert identity not in class_facts or class_facts[identity] == current
        class_facts[identity] = current
        oracle = raw_triplet(STATIC_ROOT, case["oracle_raw"])
        replay = raw_triplet(STATIC_ROOT, case["runtime"])
        assert oracle == replay
        oracle_hashes.append({"label": case["label"], "class_sha256": sha(original_bytes),
                              "oracle_stdout_sha256": sha(oracle[1]),
                              "replay_stdout_sha256": sha(replay[1])})
    assert len(class_facts) == 4 and len(oracle_hashes) == 8
    return {"manifest_sha256": STATIC_MANIFEST_SHA256,
            "inventory_sha256": STATIC_INVENTORY_SHA256,
            "closed_files": len(inventory), "accepted_legs": 8,
            "original_class_families": class_facts, "raw_runtime_legs": oracle_hashes,
            "acceptance_sha256": STATIC_ACCEPTANCE_SHA256}


def verify_local_execution(path: Path, expected_sha: str, schema: str,
                            expected_commands: list[list[str]], base: Path) -> tuple[dict, list[str]]:
    raw = read(path)
    assert sha(raw) == expected_sha
    execution = json.loads(raw)
    assert execution["schema"] == schema and len(execution["commands"]) == len(expected_commands)
    outputs, rows = [], []
    for index, (command, argv) in enumerate(zip(execution["commands"], expected_commands)):
        assert command["argv"] == argv and command["cwd"] == str(ROOT)
        assert command["exit_code"] == 0 and command["guard_stop"] is None
        streams = {}
        for name in ("stdout", "stderr"):
            record = command["streams"][name]
            payload = record_data(base, record)
            streams[name] = payload
        outputs.append(streams["stdout"].decode("utf-8", errors="replace"))
        rows.append({"argv": argv, "exit": 0,
                     "stdout_sha256": sha(streams["stdout"]),
                     "stderr_sha256": sha(streams["stderr"])})
    return {"execution_sha256": expected_sha, "commands": rows}, outputs


def verify_local_gates() -> dict:
    instance, instance_text = verify_local_execution(
        SCOPED_RUST, SCOPED_RUST_SHA256, "instance-array-scoped-rust-root-v7",
        [["cargo", "test", "-p", "jarde-java", "--lib", "instance_array", "--", "--nocapture"],
         ["cargo", "test", "-p", "jarde", "--lib", "common_instance_array_initializer_tests", "--", "--nocapture"]],
        ROOT)
    assert re.search(r"test result: ok\. 5 passed; 0 failed; 0 ignored;", instance_text[0])
    assert re.search(r"test result: ok\. 6 passed; 0 failed; 0 ignored;", instance_text[1])
    for test in INSTANCE_UNIT_TESTS:
        assert re.search(r"(?m)^test " + re.escape(test) + r" \.\.\. ok$", instance_text[0]), test
    for test in FACADE_TESTS:
        assert re.search(r"(?m)^test " + re.escape(test) + r" \.\.\. ok$", instance_text[1]), test

    reader, reader_text = verify_local_execution(
        READER_FINGERPRINT, READER_FINGERPRINT_SHA256, "instance-array-reader-fingerprint-root-v1",
        [["cargo", "test", "-p", "jarde-reader", "--lib", "--locked"],
         ["cargo", "test", "-p", "jarde", "--test", "p5_corpus_fingerprint", "--locked"]],
        ROOT)
    assert re.search(r"test result: ok\. 178 passed; 0 failed; 0 ignored;", reader_text[0])
    assert re.search(r"test result: ok\. 5 passed; 0 failed; 1 ignored;", reader_text[1])
    assert "test regenerate_corpus_fingerprint ... ignored," in reader_text[1]

    checkpoint_raw = read(CHECKPOINT)
    assert sha(checkpoint_raw) == CHECKPOINT_SHA256
    checkpoint = json.loads(checkpoint_raw)
    expected = [["cargo", "fmt", "--all", "--", "--check"],
                ["openspec", "validate", "--all", "--strict", "--no-interactive"],
                ["git", "diff", "--check"]]
    assert len(checkpoint["commands"]) == 3
    checkpoint_rows, validation_text = [], ""
    for i, (command, argv) in enumerate(zip(checkpoint["commands"], expected)):
        assert command["argv"] == argv and command["cwd"] == str(ROOT) and command["exit"] == 0
        stdout = record_data(CHECKPOINT.parent, command["streams"]["stdout"])
        stderr = record_data(CHECKPOINT.parent, command["streams"]["stderr"])
        if i == 1:
            validation_text = stdout.decode("utf-8")
        checkpoint_rows.append({"argv": argv, "exit": 0,
                                "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
    assert "Totals: 334 passed, 0 failed (334 items)" in validation_text
    return {"instance_unit_tests": instance["commands"][0],
            "facade_tests": instance["commands"][1],
            "reader_and_fingerprint": reader,
            "fmt_and_strict_validation": {"execution_sha256": CHECKPOINT_SHA256,
                                           "commands": checkpoint_rows,
                                           "openspec_total": "334 passed, 0 failed"}}


def verify_ci(helper) -> dict:
    ci = helper.verify_ci(PRODUCT, RUN_ID)
    assert ci["jobs"] == 4 and ci["step_count"] == 52 and ci["all_jobs_and_steps_success"] is True
    stable_gz = read(CI_EVIDENCE / "ci-stable-job-v1.log.gz")
    stable_raw = gzip.decompress(stable_gz)
    stable_text = helper.strip_terminal(stable_raw)
    groups = helper.command_groups(stable_text.splitlines(), helper.WORKSPACE_COMMAND)
    assert len(groups) == 2
    for _, text in groups:
        for test in INSTANCE_UNIT_TESTS + FACADE_TESTS:
            assert re.search(r"test " + re.escape(test) + r" \.\.\. ok", text), test
    api = read(CI_EVIDENCE / "ci-run-v1.json")
    supply_gz = read(CI_EVIDENCE / "ci-supply-job-v1.log.gz")
    supply_raw = gzip.decompress(supply_gz)
    return {"run_id": RUN_ID, "product_commit": PRODUCT, "jobs": 4, "steps": 52,
            "all_jobs_success": True, "two_seed_instance_tests_passed": True,
            "full_api_sha256": sha(api), "full_api_bytes": len(api),
            "stable_log_gzip_sha256": sha(stable_gz), "stable_log_raw_sha256": sha(stable_raw),
            "supply_log_gzip_sha256": sha(supply_gz), "supply_log_raw_sha256": sha(supply_raw),
            "shared_static_ci_parser_result": ci}


def main() -> None:
    if len(sys.argv) != 1:
        raise SystemExit("This verifier is pinned to the committed instance product and CI run; no arguments are accepted.")
    assert not RESULT.exists(), f"refusing to overwrite {RESULT}"
    assert CI_EVIDENCE.is_dir(), "capture ci-product-v1 after the fixed workflow completes"
    helper, helper_sha = load_static_helper()
    metadata = verify_commit_metadata(helper)
    replay = verify_candidate_replay()
    static = verify_static_eight_legs()
    local = verify_local_gates()
    ci = verify_ci(helper)
    result = {"schema": "instance-array-ci-product-acceptance-root-v3", "status": "accepted",
              "product_commit": PRODUCT, "ci_run_id": RUN_ID,
              "shared_ci_helper_sha256": helper_sha, "commit_metadata": metadata,
              "candidate_replay": replay, "previous_static_eight_legs": static,
              "local_gates": local, "ci": ci,
              "claim_boundary": "Fixed product Git blobs, frozen CLI replay, accepted raw baseline legs, captured local gates, and the fixed full CI run; no current working-tree source bytes gate acceptance."}
    RESULT.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "product_commit": PRODUCT,
                      "ci_run_id": RUN_ID, "jobs": ci["jobs"], "steps": ci["steps"]}))


if __name__ == "__main__":
    main()
