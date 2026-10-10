#!/usr/bin/env python3
"""Independently verify the prefixed one-arm loop product build and captured CI evidence (v2 summary contract)."""

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
HERE = ROOT / "openspec/changes/recover-prefixed-one-arm-loops/results"
CI_EVIDENCE = HERE / "ci-product-v1"
RESULT = CI_EVIDENCE / "acceptance-prefixed-one-arm-loops-root-v4.json"

STATIC_HELPER = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/verify-ci-product-root-v12.py"
STATIC_HELPER_SHA256 = "806547956c91e1213b3549ea641497018b97970a5a7796c91c06d345f362d55d"
INTEGER_VERIFIER = ROOT / "openspec/changes/recover-int-array-constant-names/results/verify-int-array-ci-product-luna-v2.py"
INTEGER_VERIFIER_SHA256 = "e5534e0b2acc6066ba518a1b55f5d0f79b9e5e25a1c4dc5db768040b6d47b425"
METADATA_PATH = HERE / "candidate-cli-v1.json"
CLI_PATH = Path("/private/tmp/jarde-prefixed-loop-cli-v1")
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
LOOP_TESTS = (
    "cf08_two_gateways_have_one_loop_owner_and_complete_sources",
    "cf08_unproved_gateways_keep_physical_quotes",
    "cf08_gateway_budget_and_cancellation_publish_no_partial_source",
    "one_arm_straight_prefix_loop_and_optional_tail_are_closed",
    "prefixed_one_arm_loop_budget_and_cancellation_publish_no_partial_source",
    "no_prefix_while_latch_keeps_physical_source_and_one_owner",
    "no_prefix_latch_budget_and_cancellation_publish_no_partial_source",
)
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
    helper = load_module("fixed_static_ci_verifier_v12_loop_latch", STATIC_HELPER)
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
            assert {name for name, _ in outcomes} == set(LOOP_TESTS), outcomes
            assert all(result == "ok" for _, result in outcomes), outcomes
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
    integer = load_module("accepted_integer_ci_verifier_v2_loop_latch", INTEGER_VERIFIER)
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
    assert paths, "focused test sources must retain literal fixture includes"
    return paths

def expected_clippy_argv(workflow: str) -> list[str]:
    start = workflow.index("      - name: Run Clippy")
    end = workflow.index("      - name: Run workspace tests", start)
    block = workflow[start:end]
    allowed = re.findall(r"(?m)^\s+-A clippy::([a-z0-9_]+)\s*$", block)
    assert len(allowed) == 29 and len(set(allowed)) == 29
    return ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--",
            *sum((["-A", f"clippy::{name}"] for name in allowed), []), "-D", "warnings"]


def verify_build_and_candidate(product: str, metadata_sha256: str,
                               build_path_arg: str, build_schema_arg: str) -> dict:
    assert re.fullmatch(r"[0-9a-f]{64}", metadata_sha256), "metadata SHA-256 must be 64 lowercase hex characters"
    metadata_bytes = read(METADATA_PATH)
    assert sha(metadata_bytes) == metadata_sha256
    metadata = json.loads(metadata_bytes)
    assert metadata["schema"] == "recover-prefixed-one-arm-loops-candidate-cli-v1"
    assert metadata["cli_path"] == str(CLI_PATH)
    assert metadata["cli_sha256"] == "472c5f957470671252649da688287895596bb8008342fb5d35e7e51d40d1e18a"
    assert metadata["source_commit_base"] == "404b422e141e200f0eea000f64937becedbad664"
    assert re.fullmatch(r"[0-9a-f]{64}", metadata["cli_sha256"])
    assert re.fullmatch(r"[0-9a-f]{64}", metadata["build_result_sha256"])
    assert metadata["uncommitted_prefixed_loop_product"] is True
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
    assert len(git_pins) == len(PRODUCT_SOURCES | TEST_SOURCES | expected_canonical) == 50

    cli_bytes = read(CLI_PATH)
    cli_mode = stat.S_IMODE(CLI_PATH.stat().st_mode)
    cli_sha256 = metadata["cli_sha256"]
    assert sha(cli_bytes) == cli_sha256 and cli_mode == 0o555

    build_path = Path(build_path_arg)
    if not build_path.is_absolute():
        build_path = ROOT / build_path
    build_path = build_path.resolve()
    try:
        build_path.relative_to(ROOT.resolve())
    except ValueError as error:
        raise AssertionError("build execution path escapes the project root") from error
    assert build_path.name == "execution.json"
    assert build_path.parent.name == "validation-build-root-v5"
    build_bytes = read(build_path)
    build_sha256 = metadata["build_result_sha256"]
    assert sha(build_bytes) == build_sha256
    build = json.loads(build_bytes)
    assert build["schema"] == build_schema_arg
    assert re.fullmatch(r"recover-prefixed-one-arm-loops-validation-build-root-v[0-9]+", build["schema"])
    build_version = build["schema"].rsplit("-v", 1)[1]
    assert build_path.parent.name == f"validation-build-root-v{build_version}"
    assert build["status"] == "validation-passed-cli-frozen"
    assert source_base == "404b422e141e200f0eea000f64937becedbad664"
    assert build["source_commit_base_expected"] == source_base
    assert build["uncommitted_prefixed_loop_product"] is True
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

    template_record = build["guarded_runner_template"]
    template_path = Path(template_record["path"])
    template_bytes = read(template_path)
    template_sha = sha(template_bytes)
    assert template_record["sha256"] == template_sha
    assert template_sha == "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
    assert runner_path.name == "run-validation-build-root-v5.py"
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
        2: [(335, 0, 0)], 3: [(7, 0, 0)], 4: [(4, 0, 0)], 5: [(7, 0, 0)],
        6: [(5, 0, 0)], 7: [(12, 0, 0)], 8: [(178, 0, 0)], 9: [(5, 0, 1)], 10: [(3, 0, 0)],
    }
    required_test_commands = set(expected_summary)
    expected_required_names = {2: list(REGION_TESTS), 3: list(LOOP_TESTS[3:])}
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
            assert check["required_test_names"] == expected_required_names.get(index, []), (index, check["required_test_names"])
            assert check["required_test_names_present"] is True
            if check["expected"] is not None:
                recorded_expected = [tuple(item) for item in check["expected"]]
                assert recorded_expected == expected_summary[index], (index, recorded_expected, expected_summary[index])
            else:
                recorded_expected = None
            assert "stdout" in captured_streams, index
            stdout_text = captured_streams["stdout"].decode("utf-8", errors="strict")
            actual = [tuple(map(int, match)) for match in SUMMARY_RE.findall(stdout_text)]
            independently_required = {2: REGION_TESTS, 3: LOOP_TESTS}.get(index, ())
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
    # V5 records the CLI freeze without a built_binary metadata key.
    assert expected_commands[-1] == ["cargo", "build", "-p", "jarde-cli", "--locked"]
    assert freeze["cli_path"] == str(CLI_PATH) and freeze["cli_sha256"] == cli_sha256
    assert freeze["cli_mode"] == "0o555" and freeze["metadata_path"] == str(METADATA_PATH)
    assert freeze["source_commit_base"] == source_base
    assert freeze["uncommitted_prefixed_loop_product"] is True
    assert freeze["product_path_sets"] == {
        "candidate_sources": sorted(PRODUCT_SOURCES),
        "test_sources": sorted(TEST_SOURCES),
        "canonical_files": sorted(expected_canonical),
    }
    return {
        "candidate_metadata_sha256": metadata_sha256,
        "candidate_cli": {"path": str(CLI_PATH), "sha256": cli_sha256, "mode": oct(cli_mode)},
        "candidate_build_sha256": build_sha256,
        "validation_runner": {"path": str(runner_path), "sha256": runner_sha},
        "guarded_runner_template": {"path": str(template_path), "sha256": template_sha},
        "product_commit_git_blob_pins_verified": len(git_pins),
        "current_files_match_build_pins": len(current_pins),
        "pin_counts": {"candidate_sources": len(PRODUCT_SOURCES),
                        "test_sources": len(TEST_SOURCES), "canonical_files": len(expected_canonical)},
        "source_commit_base": source_base,
        "uncommitted_prefixed_loop_product": True,
        "build_commands": len(commands), "build_streams_verified": streams_verified,
        "build_test_result_records_recomputed_from_stdout": recomputed_summaries,
        "all_build_commands_exit_zero_no_guard_stop": True,
    }

def verify_loop_workspace_tests(helper, product: str, integer_verifier) -> dict:
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
        block, counts = helper.binary_group(lines, "tests/p3_loop_exit_gateways.rs", 7)
        for test_name in LOOP_TESTS:
            assert re.search(r"test " + re.escape(test_name) + r" \.\.\. ok", block), (step, test_name)
        region_lines = [line for line in lines if any(name in line for name in REGION_TESTS)]
        assert len(region_lines) == len(REGION_TESTS), (step, region_lines)
        assert all(re.search(r"test " + re.escape(name) + r" \.\.\. ok", "\n".join(region_lines)) for name in REGION_TESTS), (step, region_lines)
        assert counts == [(7, 0, 0)]
        totals = helper.test_counts(text)
        assert len(totals) == 354
        assert (sum(row[0] for row in totals), sum(row[1] for row in totals),
                sum(row[2] for row in totals)) == (3383, 0, 97)
        assert all(failed == 0 for _, failed, _ in totals)
        seed_runs.append({
            "step": step, "seed": seed, "command_group_index": group_index,
            "focused_binary": "tests/p3_loop_exit_gateways.rs",
            "focused_test_counts": [list(row) for row in counts],
            "region_test_names": list(REGION_TESTS),
            "region_test_results": {name: "ok" for name in REGION_TESTS},
            "prefixed_one_arm_tests": list(LOOP_TESTS),
            "prefixed_one_arm_test_results": {name: "ok" for name in LOOP_TESTS},
            "workspace_result_records": len(totals),
            "workspace_expected_totals": [3383, 0, 97],
            "workspace_passed": sum(row[0] for row in totals),
            "workspace_failed": sum(row[1] for row in totals),
            "workspace_ignored": sum(row[2] for row in totals),
        })
    assert len(seed_runs) == 2

    integer_verifier.CI_EVIDENCE = CI_EVIDENCE
    integer_runs = integer_verifier.verify_integer_tests_in_both_seeds(helper, product)
    assert len(integer_verifier.WORKSPACE_TESTS) == 11 and len(integer_runs) == 2
    return {"prefixed_one_arm_workspace_tests": seed_runs,
            "prior_integer_array_tests": integer_runs,
            "prior_integer_array_test_count": len(integer_verifier.WORKSPACE_TESTS)}


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("product_sha", help="submitted product commit SHA")
    parser.add_argument("run_id", help="captured GitHub Actions run ID")
    parser.add_argument("--metadata-sha256", required=True)
    parser.add_argument("--build-path", required=True, help="frozen execution.json path, relative to the project root or absolute")
    parser.add_argument("--build-schema", required=True, help="exact schema recorded by the frozen build")
    args = parser.parse_args()
    product = args.product_sha
    if product != "ca43ac74767c4609284f13b251de9d6c349c67a4":
        raise SystemExit("PRODUCT_SHA does not match the independently pinned product commit")
    if args.run_id != "38048944005":
        raise SystemExit("RUN_ID does not match this product's captured CI run")
    if args.build_path != "openspec/changes/recover-prefixed-one-arm-loops/results/validation-build-root-v5/execution.json":
        raise SystemExit("build path does not match the pinned v5 execution")
    if args.build_schema != "recover-prefixed-one-arm-loops-validation-build-root-v5":
        raise SystemExit("build schema does not match the pinned v5 execution")
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
                                       args.build_path, args.build_schema)
    ci = helper.verify_ci(product, run_id)
    assert ci["jobs"] == 4 and ci["step_count"] == 52
    assert ci["all_jobs_and_steps_success"] is True
    workspace = verify_loop_workspace_tests(helper, product, integer_verifier)

    result = {
        "schema": "recover-prefixed-one-arm-loops-ci-product-root-acceptance-v4",
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
            "method": "recheck exact gateway binary summary and all seven gateway test names in each seed command group",
            "all_test_names_and_exact_summary_rechecked": True,
        },
        "ci": ci,
        "workspace_regressions": workspace,
        "frozen_cli_and_build": build,
        "acceptance_scope": "this exact submitted product commit, current source and fixture pins, frozen candidate CLI/build, four successful CI jobs and all recorded steps, exact workspace result totals and required prefixed-loop/latch/integer regression names from both fixed seeds; does not establish the separate full-class allBCI gate",
    }
    with RESULT.open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "product_commit": product,
                      "run_id": run_id, "jobs": ci["jobs"], "steps": ci["step_count"],
                      "prefixed_one_arm_tests_per_seed": len(LOOP_TESTS),
                      "prior_integer_tests_per_seed": workspace["prior_integer_array_test_count"]}))


if __name__ == "__main__":
    main()
