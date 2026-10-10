#!/usr/bin/env python3
"""Independently verify the proved-If-join product build and captured CI evidence."""

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


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
HERE = ROOT / "openspec/changes/preserve-proved-if-arm-join-origins/results"
CI_EVIDENCE = HERE / "ci-product-v2"
RESULT = CI_EVIDENCE / "acceptance-proved-if-arm-join-ci-root-v2.json"

STATIC_HELPER = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/verify-ci-product-root-v12.py"
STATIC_HELPER_SHA256 = "806547956c91e1213b3549ea641497018b97970a5a7796c91c06d345f362d55d"
INTEGER_VERIFIER = ROOT / "openspec/changes/recover-int-array-constant-names/results/verify-int-array-ci-product-luna-v2.py"
INTEGER_VERIFIER_SHA256 = "e5534e0b2acc6066ba518a1b55f5d0f79b9e5e25a1c4dc5db768040b6d47b425"
METADATA_PATH = HERE / "candidate-cli-v1.json"
CLI_PATH = Path("/private/tmp/jarde-proved-if-join-cli-v1")
METADATA_SHA256 = "6f785a03e50565bc5d90bd6a6ae85f1bb789e31647811421e5f21d41ee7030ca"
CLI_SHA256 = "7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6"
SOURCE_BASE = "d9855c2764860c7c2e20e40e700abeaa59f00a55"
BUILD_PATH = HERE / "validation-build-root-v1/execution.json"
BUILD_SCHEMA = "preserve-proved-if-arm-join-origins-validation-build-root-v1"
BUILD_SHA256 = "7f0ac81bd57faa67082bdc1f0231650a96c9d2658d00690f086c4039c49d6458"
BUILD_RUNNER_SHA256 = "f139f117354f4b96762737190c03a025f3a02d3efc078062b715c63d83518f6b"
REQUIRED_LIB_TEST_NAME = "build::tests::exception_if_join_transfer_rejects_a_canonical_exception_edge"
PRODUCT_SOURCES = {
    "Cargo.toml", "Cargo.lock",
    "crates/jarde-jvm/Cargo.toml", "crates/jarde-reader/Cargo.toml",
    "crates/jarde-query/Cargo.toml", "crates/jarde-java/Cargo.toml",
    "crates/jarde-cli/Cargo.toml", "crates/jarde-java/src/region.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/report.rs", "crates/jarde-java/src/lib.rs",
    "src/class_source.rs", "src/facade.rs", "src/lib.rs",
    "crates/jarde-cli/src/main.rs", "crates/jarde-cli/src/task.rs",
}
TEST_SOURCES = {
    ".github/workflows/ci.yml",
    "crates/jarde-java/tests/p3_loop_exit_gateways.rs",
    "crates/jarde-java/tests/p3_loop_body_double_jumps.rs",
    "crates/jarde-java/tests/p3_loop_terminal_return.rs",
    "crates/jarde-java/tests/p3_effectful_exits.rs",
    "tests/p3_loop_arm_join.rs", "tests/p3_loop_boolean_exit.rs",
    "tests/p5_corpus_fingerprint.rs",
}
INCLUDE_RE = re.compile(r"(?:include_bytes|include_str)!\s*\(\s*\"([^\"]+)\"")
WORKSPACE_COMMAND = "cargo test --workspace --all-targets --all-features --locked"
KNOWN_IF_GATEWAY_TESTS = {
    "cf07_nonempty_if_join_goto_is_derived_from_complete_if",
    "cf07_if_join_origin_rejects_nonjoin_and_nontransfer_terminals",
    "cf07_if_join_proof_budget_and_cancellation_publish_no_partial_source",
}
REGION_TESTS = (
    "region::tests::prefixed_loop_chain_certificate_rejects_extra_entries_exits_and_owners",
    "region::tests::prefixed_loop_header_cannot_reopen_claimed_outer_target_or_parent_scope",
)
ENV_VALUES = {
    "CARGO_BUILD_JOBS": "1", "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0",
    "RUST_TEST_THREADS": "1", "CARGO_TERM_COLOR": "always",
}
TARGET_LIMIT = 1024**3
FREE_LIMIT = 5 * 1024**3
SUMMARY_RE = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;")
GATEWAY_TEST_NAMES: set[str] = set()


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
    helper = load_module("fixed_static_ci_verifier_v12_if_join", STATIC_HELPER)
    assert helper.ROOT == ROOT
    helper.CI_EVIDENCE = CI_EVIDENCE
    helper.METADATA_PATH = METADATA_PATH
    original_binary_group = helper.binary_group

    def binary_group(lines, binary, expected_count, expected_ignored=0):
        if binary == "tests/p3_loop_exit_gateways.rs":
            starts = [i for i, line in enumerate(lines) if f"Running {binary}" in line]
            assert len(starts) == 1, (binary, len(starts))
            markers = [(i, re.search(r"\brunning (\d+) tests?\b", line))
                       for i, line in enumerate(lines[starts[0] + 1:], starts[0] + 1)
                       if re.search(r"\brunning (\d+) tests?\b", line)]
            assert markers and int(markers[0][1].group(1)) == expected_count + expected_ignored
            run_start = markers[0][0]
            summaries = [(i, helper.test_counts(line)) for i, line in enumerate(lines[run_start + 1:], run_start + 1)
                         if "test result:" in line]
            assert summaries, (binary, "missing summary")
            summary_index, summary = summaries[0]
            block = "\n".join(lines[run_start:summary_index + 1])
            outcomes = re.findall(r"\btest ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)", block)
            assert summary == [(expected_count, 0, expected_ignored)]
            assert len(outcomes) == expected_count + expected_ignored
            assert {name for name, _ in outcomes} == GATEWAY_TEST_NAMES, outcomes
            assert all(result == "ok" for _, result in outcomes), outcomes
            return block, summary
        if binary == "tests/p5_corpus_fingerprint.rs":
            starts = [i for i, line in enumerate(lines) if f"Running {binary}" in line]
            assert len(starts) == 1, (binary, len(starts))
            markers = [(i, re.search(r"\brunning (\d+) tests?\b", line))
                       for i, line in enumerate(lines[starts[0] + 1:], starts[0] + 1)
                       if re.search(r"\brunning (\d+) tests?\b", line)]
            assert markers and int(markers[0][1].group(1)) == expected_count + expected_ignored
            run_start = markers[0][0]
            summaries = [(i, helper.test_counts(line)) for i, line in enumerate(lines[run_start + 1:], run_start + 1)
                         if "test result:" in line]
            assert summaries, (binary, "missing stdout summary")
            summary_index, summary = summaries[0]
            assert not any(re.search(r"\brunning \d+ tests?\b", line) for line in lines[run_start + 1:summary_index])
            block = "\n".join(lines[run_start:summary_index + 1])
            expected_names = {
                "corpus_files_match_the_recorded_fingerprint": "ok",
                "every_acceptance_row_is_indexed_against_existing_corpus": "ok",
                "every_dimension_is_carried_by_existing_corpus": "ok",
                "regenerate_corpus_fingerprint": "ignored",
                "the_indexed_rows_are_the_rows_the_acceptance_table_names": "ok",
                "the_manifest_matches_its_rendered_classification": "ok",
            }
            outcomes = re.findall(r"\btest ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)", block)
            assert expected_count == 5 and expected_ignored == 1
            assert summary == [(5, 0, 1)] and len(outcomes) == 6
            assert dict(outcomes) == expected_names and len(set(name for name, _ in outcomes)) == 6
            return block, summary
        if binary != "tests/interface_initializer_proof.rs":
            return original_binary_group(lines, binary, expected_count, expected_ignored)
        starts = [i for i, line in enumerate(lines) if f"Running {binary}" in line]
        assert len(starts) == 1, (binary, len(starts))
        start = starts[0]
        run_markers = [(i, re.search(r"\brunning (\d+) tests?\b", line))
                       for i, line in enumerate(lines[start + 1:], start + 1)
                       if re.search(r"\brunning (\d+) tests?\b", line)]
        assert run_markers, (binary, "missing stdout running marker")
        run_start, run_match = run_markers[0]
        assert int(run_match.group(1)) == expected_count + expected_ignored, (binary, run_match.group(0))
        summaries = [(i, helper.test_counts(line)) for i, line in enumerate(lines[run_start + 1:], run_start + 1)
                     if "test result:" in line]
        assert summaries, (binary, "missing summary after stdout test block")
        summary_index, counts = summaries[0]
        assert not any(re.search(r"\brunning \d+ tests?\b", line)
                       for line in lines[run_start + 1:summary_index]), (binary, "interleaved stdout run block")
        block = "\n".join(lines[run_start:summary_index + 1])
        assert helper.test_counts(block) == [(expected_count, 0, expected_ignored)], (binary, counts)
        outcomes = re.findall(r"\btest ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)", block)
        assert len(outcomes) == expected_count + expected_ignored, (binary, outcomes)
        assert {name for name, _ in outcomes} == set(helper.INTERFACE_TESTS), (binary, outcomes)
        assert all(result == "ok" for _, result in outcomes), (binary, outcomes)
        return block, [(expected_count, 0, expected_ignored)]

    helper.binary_group = binary_group

    integer_bytes = read(INTEGER_VERIFIER)
    assert sha(integer_bytes) == INTEGER_VERIFIER_SHA256, "accepted integer verifier bytes changed"
    integer = load_module("accepted_integer_ci_verifier_v2_if_join", INTEGER_VERIFIER)
    integer.CI_EVIDENCE = CI_EVIDENCE
    return helper, integer


def expected_canonical_files(product: str) -> set[str]:
    paths = set()
    for test_path in sorted(TEST_SOURCES):
        source = git_blob(product, test_path).decode("utf-8")
        included = INCLUDE_RE.findall(source)
        for value in included:
            relative = posixpath.normpath(posixpath.join(posixpath.dirname(test_path), value))
            assert relative != ".." and not relative.startswith("../"), (test_path, value)
            paths.add(relative)
    paths.add("openspec/changes/preserve-proved-if-arm-join-origins/results/exception-join-case-root-v1/classes/ifjoin/ExceptionIfJoin.class")
    assert paths, "focused test sources must retain literal fixture includes"
    assert len(paths) == 27, f"If product must pin 27 exact canonical class inputs, got {len(paths)}"
    return paths

def gateway_tests_from_product(product: str) -> set[str]:
    source = git_blob(product, "crates/jarde-java/tests/p3_loop_exit_gateways.rs").decode("utf-8")
    names = set(re.findall(
        r"(?ms)^\s*#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        source))
    assert len(names) == 12 and KNOWN_IF_GATEWAY_TESTS <= names, (
        "gateway test source must contain 12 functions and all known If-join tests", names)
    return names


def expected_clippy_argv(workflow: str) -> list[str]:
    start = workflow.index("      - name: Run Clippy")
    end = workflow.index("      - name: Run workspace tests", start)
    block = workflow[start:end]
    allowed = re.findall(r"(?m)^\s+-A clippy::([a-z0-9_]+)\s*$", block)
    assert len(allowed) == 29 and len(set(allowed)) == 29
    return ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--",
            *sum((["-A", f"clippy::{name}"] for name in allowed), []), "-D", "warnings"]


def verify_build_and_candidate(product: str, expected_lib_count: int,
                               expected_gateway_count: int) -> dict:
    metadata_bytes = read(METADATA_PATH)
    assert sha(metadata_bytes) == METADATA_SHA256
    metadata = json.loads(metadata_bytes)
    assert metadata["schema"] == "preserve-proved-if-arm-join-origins-candidate-cli-v1"
    assert metadata["cli_path"] == str(CLI_PATH)
    assert metadata["cli_sha256"] == CLI_SHA256
    assert re.fullmatch(r"[0-9a-f]{64}", metadata["cli_sha256"])
    assert re.fullmatch(r"[0-9a-f]{64}", metadata["build_result_sha256"])
    assert metadata["uncommitted_if_arm_join_product"] is True
    assert metadata["source_commit_base"] == SOURCE_BASE
    assert metadata["validation_runner"] == {"path": str((HERE / "run-validation-build-root-v1.py").resolve()),
                                             "sha256": BUILD_RUNNER_SHA256}
    assert metadata["guarded_runner_template"] == {"path": str(ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"),
                                                   "sha256": "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"}
    source_base = metadata["source_commit_base"]
    assert re.fullmatch(r"[0-9a-f]{40}", source_base)

    expected_canonical = expected_canonical_files(product)
    assert set(metadata["candidate_sources"]) == PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == TEST_SOURCES
    assert set(metadata["canonical_files"]) == expected_canonical

    git_pins = {}
    current_pins = {}
    for category, expected_paths in (("candidate_sources", PRODUCT_SOURCES),
                                     ("test_sources", TEST_SOURCES),
                                     ("canonical_files", expected_canonical)):
        pins = metadata[category]
        assert set(pins) == expected_paths, category
        for relative, expected_sha in pins.items():
            product_blob = git_blob(product, relative)
            current = read(ROOT / relative)
            assert sha(product_blob) == expected_sha, (category, relative, "product blob")
            assert sha(current) == expected_sha, (category, relative, "current file")
            git_pins[relative] = {"category": category, "bytes": len(product_blob), "sha256": expected_sha}
            current_pins[relative] = expected_sha
    assert len(git_pins) == len(PRODUCT_SOURCES | TEST_SOURCES | expected_canonical) == 52

    cli_bytes = read(CLI_PATH)
    cli_mode = stat.S_IMODE(CLI_PATH.stat().st_mode)
    cli_sha256 = metadata["cli_sha256"]
    assert sha(cli_bytes) == cli_sha256 and cli_mode == 0o555

    build_path = BUILD_PATH.resolve(strict=True)
    build_bytes = read(build_path)
    build_sha256 = sha(build_bytes)
    assert build_sha256 == BUILD_SHA256 == metadata["build_result_sha256"]
    build = json.loads(build_bytes)
    assert build["schema"] == BUILD_SCHEMA
    assert build_path.parent.name == "validation-build-root-v1"
    build_version = "1"
    assert build["status"] == "validation-passed-cli-frozen"
    assert build["source_commit_base_expected"] == source_base
    assert build["uncommitted_if_arm_join_product"] is True
    assert build["validation_runner"] == metadata["validation_runner"]
    assert build["guarded_runner_template"] == metadata["guarded_runner_template"]
    assert build["environment_overrides"] == ENV_VALUES
    assert build["guards"] == {"minimum_free_bytes": FREE_LIMIT, "maximum_target_bytes": TARGET_LIMIT}

    runner_record = build["validation_runner"]
    runner_path = Path(runner_record["path"])
    if not runner_path.is_absolute():
        runner_path = ROOT / runner_path
    runner_path = runner_path.resolve()
    try:
        runner_path.relative_to(HERE.resolve())
    except ValueError as error:
        raise AssertionError("validation runner path escapes this change's results directory") from error
    runner_bytes = read(runner_path)
    runner_sha = sha(runner_bytes)
    assert runner_record == {"path": str(runner_path), "sha256": runner_sha}
    assert runner_sha == BUILD_RUNNER_SHA256

    template_record = build["guarded_runner_template"]
    template_path = Path(template_record["path"])
    template_bytes = read(template_path)
    template_sha = sha(template_bytes)
    assert template_record["sha256"] == template_sha
    assert template_sha == "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
    assert runner_path.name == f"run-validation-build-root-v{build_version}.py"
    before = build["preflight"]["source_pins_before"]
    after = build["preflight"]["source_pins_after"]
    metadata_pins = {"candidate_sources": metadata["candidate_sources"],
                     "test_sources": metadata["test_sources"],
                     "canonical_files": metadata["canonical_files"]}
    assert before == after == metadata_pins
    head_record = build["preflight"]["git_head"]
    assert head_record["matches_expected"] is True and head_record["value"] == source_base
    assert head_record["argv"] == ["git", "rev-parse", "HEAD"] and head_record["exit_code"] == 0
    for stream_name, stream in (("git-head stdout", head_record["stdout"]),
                                ("git-head stderr", head_record["stderr"])):
        raw_path = Path(stream["path"])
        if not raw_path.is_absolute():
            raw_path = ROOT / raw_path
        try:
            raw_path.resolve().relative_to(ROOT.resolve())
        except ValueError as error:
            raise AssertionError(f"{stream_name} path escapes the project root") from error
        raw = read(raw_path)
        assert len(raw) == stream["bytes"] and sha(raw) == stream["sha256"]
        if stream_name == "git-head stdout":
            assert raw.decode("ascii").strip() == source_base

    expected_commands = [
        ["cargo", "fmt", "--all", "--", "--check"],
        expected_clippy_argv(git_blob(product, ".github/workflows/ci.yml").decode("utf-8")),
        ["cargo", "test", "-p", "jarde-java", "--lib", "--locked"],
        ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_exit_gateways", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_loop_arm_join", "--locked", "--", "--nocapture"],
        ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_body_double_jumps", "--locked"],
        ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_terminal_return", "--locked"],
        ["cargo", "test", "-p", "jarde-java", "--test", "p3_effectful_exits", "--locked"],
        ["cargo", "test", "-p", "jarde-reader", "--lib", "--locked"],
        ["cargo", "test", "-p", "jarde", "--test", "p5_corpus_fingerprint", "--locked"],
        ["cargo", "test", "-p", "jarde", "--test", "p3_loop_boolean_exit", "--locked", "--", "--nocapture"],
        ["cargo", "build", "-p", "jarde-cli", "--locked"],
    ]
    commands = build["commands"]
    assert len(commands) == len(expected_commands) == 12
    expected_summary = {
        2: [(expected_lib_count, 0, 0)], 3: [(expected_gateway_count, 0, 0)], 4: [(4, 0, 0)], 5: [(7, 0, 0)],
        6: [(5, 0, 0)], 7: [(12, 0, 0)], 8: [(178, 0, 0)], 9: [(5, 0, 1)], 10: [(3, 0, 0)],
    }
    required_test_commands = set(expected_summary)
    required_lib_test_name = REQUIRED_LIB_TEST_NAME
    if not required_lib_test_name.startswith("build::tests::"):
        raise AssertionError("frozen If unit test must be a fully qualified jarde-java build::tests name")
    added_lib_name = required_lib_test_name.removeprefix("build::tests::")
    assert re.search(r"(?m)^\s*fn\s+" + re.escape(added_lib_name) + r"\s*\(",
                     git_blob(product, "crates/jarde-java/src/build.rs").decode("utf-8")), (
        "required If unit test name is absent from pinned build.rs", required_lib_test_name)
    expected_required_names = {2: list(REGION_TESTS) + [required_lib_test_name],
                               3: sorted(KNOWN_IF_GATEWAY_TESTS)}
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
        if index in required_test_commands:
            check = row["test_summary_check"]
            assert check is not None and check["ok"] is True and check["required"] is True, index
            wanted = expected_required_names.get(index, [])
            assert set(check["required_test_names"]) == set(wanted)
            assert len(check["required_test_names"]) == len(wanted), (index, check["required_test_names"])
            assert check["required_test_names_present"] is True
            if check["expected"] is not None:
                recorded_expected = [tuple(item) for item in check["expected"]]
                assert recorded_expected == expected_summary[index], (index, recorded_expected, expected_summary[index])
            else:
                recorded_expected = None
            assert "stdout" in captured_streams, index
            stdout_text = captured_streams["stdout"].decode("utf-8", errors="strict")
            actual = [tuple(map(int, match)) for match in SUMMARY_RE.findall(stdout_text)]
            independently_required = {2: tuple(REGION_TESTS) + (required_lib_test_name,),
                                      3: tuple(GATEWAY_TEST_NAMES)}.get(index, ())
            for name in independently_required:
                assert re.search(r"(?m)^test " + re.escape(name) + r" \.\.\. ok$", stdout_text), (index, name)
            recorded = [tuple(item) for item in check["actual"]]
            assert actual and all(failed == 0 for _, failed, _ in actual), (index, actual)
            assert actual == recorded, (index, actual, recorded)
            if expected_summary[index] is not None:
                assert actual == expected_summary[index], (index, actual, expected_summary[index])
            recomputed_summaries[str(index)] = [list(item) for item in actual]
        else:
            check = row["test_summary_check"]
            assert check["expected"] is None and check["actual"] == []
            assert check["required"] is False and check["ok"] is True
            assert check["failure_reason"] is None
            assert check["required_test_names"] == []
            assert check["required_test_names_present"] is True
            assert "required_gateway_tests" not in check
            assert "required_boolean_loop_tests" not in check
    streams_verified = sum(len(row["streams"]) for row in commands)
    assert streams_verified == 24

    freeze = build["freeze"]
    # This build records the CLI freeze without a built_binary metadata key.
    assert expected_commands[-1] == ["cargo", "build", "-p", "jarde-cli", "--locked"]
    assert freeze["cli_path"] == str(CLI_PATH) and freeze["cli_sha256"] == cli_sha256
    assert freeze["cli_mode"] == "0o555" and freeze["metadata_path"] == str(METADATA_PATH)
    assert freeze["source_commit_base"] == source_base
    assert freeze["uncommitted_if_arm_join_product"] is True
    assert freeze["product_path_sets"] == {
        "candidate_sources": sorted(PRODUCT_SOURCES),
        "test_sources": sorted(TEST_SOURCES),
        "canonical_files": sorted(expected_canonical),
    }
    return {
        "candidate_metadata_sha256": METADATA_SHA256,
        "candidate_cli": {"path": str(CLI_PATH), "sha256": cli_sha256, "mode": oct(cli_mode)},
        "candidate_build_sha256": build_sha256,
        "validation_runner": {"path": str(runner_path), "sha256": runner_sha},
        "guarded_runner_template": {"path": str(template_path), "sha256": template_sha},
        "product_commit_git_blob_pins_verified": len(git_pins),
        "current_files_match_build_pins": len(current_pins),
        "pin_counts": {"candidate_sources": len(PRODUCT_SOURCES),
                        "test_sources": len(TEST_SOURCES), "canonical_files": len(expected_canonical)},
        "source_commit_base": source_base,
        "uncommitted_if_arm_join_product": True,
        "build_commands": len(commands), "build_streams_verified": streams_verified,
        "build_test_result_records_recomputed_from_stdout": recomputed_summaries,
        "all_build_commands_exit_zero_no_guard_stop": True,
    }

def verify_loop_workspace_tests(helper, product: str, integer_verifier,
                                expected_gateway_count: int, expected_total_passed: int,
                                required_lib_test_name: str) -> dict:
    workflow = git_blob(product, ".github/workflows/ci.yml").decode("utf-8")
    seeds = re.findall(r'PROPTEST_RNG_SEED:\s*"(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    assert workflow.count(f"run: {WORKSPACE_COMMAND}") == 2
    GATEWAY_TEST_NAMES.update(gateway_tests_from_product(product))
    assert expected_gateway_count == len(GATEWAY_TEST_NAMES)
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
        block, counts = helper.binary_group(lines, "tests/p3_loop_exit_gateways.rs", expected_gateway_count)
        for test_name in sorted(GATEWAY_TEST_NAMES):
            assert re.search(r"(?m)^test " + re.escape(test_name) + r" \.\.\. ok$", block), (step, test_name)
        internal_rows = [line for line in lines if line.strip() == f"test {required_lib_test_name} ... ok"]
        assert len(internal_rows) == 1, (step, internal_rows)
        region_lines = [line for line in lines if any(name in line for name in REGION_TESTS)]
        assert len(region_lines) == len(REGION_TESTS), (step, region_lines)
        assert all(re.search(r"test " + re.escape(name) + r" \.\.\. ok", "\n".join(region_lines)) for name in REGION_TESTS), (step, region_lines)
        assert counts == [(expected_gateway_count, 0, 0)]
        totals = helper.test_counts(text)
        assert len(totals) == 354
        assert (sum(row[0] for row in totals), sum(row[1] for row in totals),
                sum(row[2] for row in totals)) == (expected_total_passed, 0, 97)
        assert all(failed == 0 for _, failed, _ in totals)
        seed_runs.append({
            "step": step, "seed": seed, "command_group_index": group_index,
            "focused_binary": "tests/p3_loop_exit_gateways.rs",
            "focused_test_counts": [list(row) for row in counts],
            "gateway_test_names": sorted(GATEWAY_TEST_NAMES),
            "gateway_test_results": {name: "ok" for name in sorted(GATEWAY_TEST_NAMES)},
            "region_test_names": list(REGION_TESTS),
            "region_test_results": {name: "ok" for name in REGION_TESTS},
            "proved_if_arm_join_tests": sorted(KNOWN_IF_GATEWAY_TESTS),
            "proved_if_arm_join_test_results": {name: "ok" for name in sorted(KNOWN_IF_GATEWAY_TESTS)},
            "workspace_result_records": len(totals),
            "workspace_expected_totals": [expected_total_passed, 0, 97],
            "workspace_passed": sum(row[0] for row in totals),
            "workspace_failed": sum(row[1] for row in totals),
            "workspace_ignored": sum(row[2] for row in totals),
        })
    assert len(seed_runs) == 2

    integer_verifier.CI_EVIDENCE = CI_EVIDENCE
    integer_runs = integer_verifier.verify_integer_tests_in_both_seeds(helper, product)
    assert len(integer_verifier.WORKSPACE_TESTS) == 11 and len(integer_runs) == 2
    return {"proved_if_arm_join_workspace_tests": seed_runs,
            "prior_integer_array_tests": integer_runs,
            "prior_integer_array_test_count": len(integer_verifier.WORKSPACE_TESTS)}


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-commit", required=True, help="submitted product commit SHA")
    parser.add_argument("--run-id", required=True, help="captured GitHub Actions run ID")
    parser.add_argument("--expected-lib-count", type=int, required=True)
    parser.add_argument("--expected-gateway-count", type=int, required=True)
    parser.add_argument("--expected-total-passed", type=int, required=True)
    args = parser.parse_args()
    product = args.product_commit
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("--product-commit must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", args.run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    if min(args.expected_lib_count, args.expected_gateway_count, args.expected_total_passed) < 1:
        raise SystemExit("expected test counts must be positive")
    run_id = int(args.run_id)
    if RESULT.exists():
        raise SystemExit(f"refusing to overwrite {RESULT}")
    if not CI_EVIDENCE.is_dir():
        raise SystemExit(f"missing captured CI evidence directory: {CI_EVIDENCE}")

    GATEWAY_TEST_NAMES.update(gateway_tests_from_product(product))
    helper, integer_verifier = load_fixed_helpers()
    build = verify_build_and_candidate(product, args.expected_lib_count,
                                       args.expected_gateway_count)
    ci = helper.verify_ci(product, run_id)
    assert ci["jobs"] == 4 and ci["step_count"] == 52
    assert ci["all_jobs_and_steps_success"] is True
    capture = json.loads(read(CI_EVIDENCE / "capture-execution-root-v2.json"))
    assert capture["schema"] == "preserve-proved-if-arm-join-origins-ci-capture-root-v2"
    assert capture["status"] == "captured" and capture["run_id"] == args.run_id
    assert capture["expected_head_sha"] == product
    assert capture["expected_test_counts"] == {
        "lib": args.expected_lib_count,
        "gateway": args.expected_gateway_count,
        "workspace_passed": args.expected_total_passed,
    }
    workspace = verify_loop_workspace_tests(helper, product, integer_verifier,
                                            args.expected_gateway_count,
                                            args.expected_total_passed,
                                            REQUIRED_LIB_TEST_NAME)

    result = {
        "schema": "preserve-proved-if-arm-join-origins-ci-product-root-acceptance-v2",
        "status": "accepted",
        "product_commit": product,
        "run_id": run_id,
        "source_commit_base": build["source_commit_base"],
        "shared_ci_verification": {
            "static_helper": {"path": str(STATIC_HELPER.relative_to(ROOT)),
                              "sha256": STATIC_HELPER_SHA256,
                              "verify_ci_called": True, "main_called": False},
            "accepted_integer_verifier": {"path": str(INTEGER_VERIFIER.relative_to(ROOT)),
                                          "sha256": INTEGER_VERIFIER_SHA256,
                                          "integer_test_helper_called": True},
        },
        "binary_group_adaptation": {
            "target": "tests/p3_loop_exit_gateways.rs",
            "method": "derive all 12 gateway names from the product Git blob, require all 12 raw gateway names, with only the three new If-join tests required by build summary, and retain exact fingerprint output checks",
            "all_test_names_and_exact_summary_rechecked": True,
        },
        "ci": ci,
        "workspace_regressions": workspace,
        "frozen_cli_and_build": build,
        "acceptance_scope": "the submitted product commit supplied at invocation, its Git blobs and current 52 source/test/include pins, frozen If candidate CLI/build, four successful CI jobs and all recorded steps, caller-supplied exact test totals and proved-If plus prior integer regression names from both fixed seeds",
    }
    with RESULT.open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "product_commit": product,
                      "run_id": run_id, "jobs": ci["jobs"], "steps": ci["step_count"],
                      "proved_if_arm_join_tests_per_seed": len(KNOWN_IF_GATEWAY_TESTS),
                      "prior_integer_tests_per_seed": workspace["prior_integer_array_test_count"]}))


if __name__ == "__main__":
    main()
