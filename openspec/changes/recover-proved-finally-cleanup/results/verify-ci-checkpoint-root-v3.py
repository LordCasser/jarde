#!/usr/bin/env python3
"""Prepare independent CI acceptance for the CF16 test checkpoint."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
RETURNED_RESULTS = ROOT / "openspec/changes/recover-returned-int-array-compound-updates/results"
CI_EVIDENCE = HERE / "ci-checkpoint-v1"
CF16_TEST_SOURCE = "crates/jarde-java/tests/p3_patterns.rs"
CF16_FIXTURE_FILES = (
    "crates/jarde-java/tests/fixtures/cf16-region-depth/README.md",
    "crates/jarde-java/tests/fixtures/cf16-region-depth/copy-manifest.json",
    "crates/jarde-java/tests/fixtures/cf16-region-depth/source/BranchFinally.java",
    "crates/jarde-java/tests/fixtures/cf16-region-depth/source/Runner.java",
    "crates/jarde-java/tests/fixtures/cf16-region-depth/javac8/BranchFinally.class",
    "crates/jarde-java/tests/fixtures/cf16-region-depth/javac23/BranchFinally.class",
)
CF16_NEW_TESTS = (
    "structured_finally_stop_never_publishes_partial_class_source",
    "cf16_region_depth_stop_never_publishes_class_source",
)
PRODUCT_SOURCES = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
TEST_SOURCES = {
    "tests/p3_nested_int_array_compound_updates.rs", ".github/workflows/ci.yml",
    "tests/recover_lambda_primitive_array_capture.rs",
    "tests/p3_constructed_reference_array_elements.rs", "tests/recover_boxed_number_widening.rs",
}
EXPECTED_JDK = "25.0.4+7.0.LTS"
EXPECTED_JDK_DISTRIBUTION = "temurin"
EXPECTED_PRODUCT = "acd55c213ac7c670c6e76609871281a341d03ad0"
EXPECTED_RUN_ID = 38007755097
WORKSPACE_COMMAND = "cargo test --workspace --all-targets --all-features --locked"
NESTED_IGNORED_COMMAND = (
    "cargo test --test p3_nested_int_array_compound_updates --locked -- --ignored --exact "
    "nested_int_array_updates_match_the_frozen_runner_oracle"
)
RETURNED_IGNORED_COMMAND = (
    "cargo test --test p3_nested_int_array_compound_updates --locked -- --ignored --exact "
    "returned_int_array_updates_compile_and_execute_as_a_complete_class"
)
NESTED_ORDINARY_TESTS = (
    "nested_int_array_boundaries_do_not_claim_unproved_compound_updates",
    "nested_int_array_class_source_stops_do_not_claim_complete_members",
    "nested_int_array_updates_are_structured_and_preserve_lvalue_boundaries",
)
RETURNED_ORDINARY_TESTS = (
    "returned_int_array_updates_are_structured_and_keep_store_return_origins",
    "returned_int_array_updates_use_java_types_beyond_category_one_frames",
    "returned_int_array_updates_reject_extra_consumers_and_non_int_returns",
    "returned_int_array_updates_invalid_copy_category_stops_before_java_publication",
    "returned_int_array_updates_public_budget_and_cancellation_do_not_publish_partial_claims",
)
IMMUTABLE_CURRENT_SOURCES = (
    "crates/jarde-reader/src/classfile.rs",
    "Cargo.lock",
    "tests/fixtures/corpus-fingerprint.json",
    "tests/p3_bigdecimal_number_widening.rs",
)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_current(relative):
    return (ROOT / relative).read_bytes()


def git_blob(product, relative):
    return subprocess.check_output(["git", "show", f"{product}:{relative}"], cwd=ROOT)


def group_text(lines, start):
    end = next((i for i in range(start + 1, len(lines)) if "##[group]Run " in lines[i]), len(lines))
    return "\n".join(lines[start:end])


def command_groups(lines, command):
    needle = f"##[group]Run {command}"
    return [(i, group_text(lines, i)) for i, line in enumerate(lines) if needle in line]


def test_counts(text):
    return [tuple(map(int, row)) for row in re.findall(
        r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]


def binary_group(lines, binary):
    starts = [i for i, line in enumerate(lines) if f"Running {binary}" in line]
    assert len(starts) == 1, (binary, len(starts))
    start = starts[0]
    end = next((i for i in range(start + 1, len(lines))
                if "     Running " in lines[i] or "   Doc-tests " in lines[i]),
               len(lines))
    block = "\n".join(lines[start:end])
    counts = test_counts(block)
    assert counts and all(failed == 0 for _, failed, _ in counts), binary
    assert "test result: FAILED" not in block and "error: test failed" not in block, binary
    return block, counts


def validate_recorded_gate(relative, expected_test):
    path = ROOT / relative
    record = json.loads(path.read_bytes())
    assert record["exit"] == 0 and record.get("runner_exit", 0) == 0, relative
    stdout_meta, stderr_meta = record["stdout"], record["stderr"]
    stdout, stderr = Path(stdout_meta["path"]).read_bytes(), Path(stderr_meta["path"]).read_bytes()
    assert len(stdout) == stdout_meta["bytes"] and sha(stdout) == stdout_meta["sha256"]
    assert len(stderr) == stderr_meta["bytes"] and sha(stderr) == stderr_meta["sha256"]
    output = stdout.decode(errors="replace") + stderr.decode(errors="replace")
    assert re.search(r"test " + re.escape(expected_test) + r" \.\.\. ok", output), relative
    assert re.search(r"test result: ok\. \d+ passed; 0 failed", output), relative
    return {"path": relative, "result_sha256": sha(path.read_bytes()),
            "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr),
            "test": expected_test, "exit": 0}


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: verify-ci-checkpoint-root-v3.py PRODUCT_SHA RUN_ID")
    product, run_id = sys.argv[1:]
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(run_id)
    assert product == EXPECTED_PRODUCT
    assert run_id == EXPECTED_RUN_ID

    raw_path = CI_EVIDENCE / "ci-run-v1.json"
    raw = json.loads(raw_path.read_bytes())
    # Scope the evidence to the immutable product commit and its frozen CLI. Current WIP hashes
    # are reported separately and cannot change whether this historical CI run is accepted.
    assert raw["headSha"] == product
    assert raw["status"] == "completed" and raw["conclusion"] == "success"
    url_match = re.search(r"/actions/runs/(\d+)(?:$|[/?#])", raw["url"])
    assert url_match and int(url_match.group(1)) == run_id

    expected_jobs = {"stable / test and specification", "MSRV 1.88.0", "supply chain", "fuzz smoke"}
    assert {job["name"] for job in raw["jobs"]} == expected_jobs
    for job in raw["jobs"]:
        assert job["status"] == "completed" and job["conclusion"] == "success"
        assert all(step["status"] == "completed" and step["conclusion"] == "success"
                   for step in job["steps"])

    stable = next(job for job in raw["jobs"] if job["name"] == "stable / test and specification")
    required_steps = {
        "Check formatting", "Run Clippy", "Run ignored JDK 25 instruction-boundary oracle",
        "Run the ignored P3 Java 8 compile-and-execute comparison",
        "Run the functional-constructor full-class comparison",
        "Run the BigDecimal Number full-class comparisons",
        "Run the nested int array full-class comparisons",
        "Run the returned int array full-class comparison",
        "Validate OpenSpec strictly", "Ensure tracked files remain unchanged",
    }
    seed_steps = [f"Run workspace tests with two fixed independent seeds (pass {i})" for i in (1, 2)]
    step_names = {step["name"] for step in stable["steps"]}
    assert required_steps.union(seed_steps) <= step_names

    metadata_path = RETURNED_RESULTS / "candidate-cli-v2.json"
    metadata_bytes = metadata_path.read_bytes()
    metadata = json.loads(metadata_bytes)
    assert set(metadata["candidate_sources"]) == PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == TEST_SOURCES
    assert metadata["cli_sha256"] == "71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110"
    source_identity = {}
    for category, paths in (("product", metadata["candidate_sources"]),
                            ("test", metadata["test_sources"])):
        for relative, expected in paths.items():
            frozen_git = git_blob(product, relative)
            current = read_current(relative)
            assert sha(frozen_git) == expected, f"{category} immutable CI source identity changed: {relative}"
            source_identity[relative] = {
                "category": category, "sha256": expected, "current_sha256": sha(current),
                "current_matches_ci_product": sha(current) == expected,
            }

    # Bind the CF16-only checkpoint additions separately from the reused frozen CLI metadata.
    cf16_identity = {}
    for relative in (CF16_TEST_SOURCE, *CF16_FIXTURE_FILES):
        frozen_git, current = git_blob(product, relative), read_current(relative)
        assert frozen_git == current, f"CF16 CI blob differs from current file: {relative}"
        cf16_identity[relative] = {"sha256": sha(frozen_git), "bytes": len(frozen_git),
                                   "git_matches_current": True}
    source_text = read_current(CF16_TEST_SOURCE).decode()
    for name in CF16_NEW_TESTS:
        assert re.search(r"fn\s+" + re.escape(name) + r"\s*\(", source_text), name
    depth_test = source_text.split("fn cf16_region_depth_stop_never_publishes_class_source", 1)[1]
    depth_test = depth_test.split("\n#[test]", 1)[0]
    assert ".stack_size(8 * 1024 * 1024)" in depth_test
    assert "at: Some(450)" in depth_test and 'code == "jre_recursion_bound"' in depth_test
    assert "report.text.is_empty()" in depth_test and "report.source_map.is_empty()" in depth_test
    fixture_manifest = json.loads(read_current(CF16_FIXTURE_FILES[1]))
    assert fixture_manifest["schema"] == "cf16-region-depth-fixture-copy-v1"
    for entry in fixture_manifest["files"]:
        relative = "crates/jarde-java/tests/fixtures/cf16-region-depth/" + entry["path"]
        data = read_current(relative)
        assert len(data) == entry["bytes"] and sha(data) == entry["sha256"], relative
    cli_path = Path(metadata["cli_path"])
    assert cli_path.is_file() and sha(cli_path.read_bytes()) == metadata["cli_sha256"]

    workflow = git_blob(product, ".github/workflows/ci.yml").decode()
    assert sha(workflow.encode()) == metadata["test_sources"][".github/workflows/ci.yml"]
    seeds = re.findall(r'PROPTEST_RNG_SEED: "(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    assert workflow.count(f"run: {WORKSPACE_COMMAND}") == 2
    assert 'java-version: "25.0.4+7.0.LTS"' in workflow
    assert "distribution: temurin" in workflow

    compressed = CI_EVIDENCE / "ci-stable-job-v1.log.gz"
    compressed_bytes = compressed.read_bytes()
    log_bytes = gzip.decompress(compressed_bytes)
    log = re.sub(r"(?:\x1b\[[0-9;]*m|\^\[\[[0-9;]*m)", "", log_bytes.decode())
    # The captured Actions stream provides exact command groups. API step names/status are
    # checked independently above because the compressed stream labels these as UNKNOWN STEP.
    assert "distribution: temurin" in log
    assert f"java-version: {EXPECTED_JDK}" in log
    assert f"Java {EXPECTED_JDK} was downloaded" in log
    assert f"Setting Java {EXPECTED_JDK} as the default" in log
    assert "Temurin-Hotspot_jdk/25.0.4-7.0.LTS" in log

    lines = log.splitlines()
    workspace_groups = command_groups(lines, WORKSPACE_COMMAND)
    assert len(workspace_groups) == 2
    seed_results = []
    for (_, text), step, seed in zip(workspace_groups, seed_steps, seeds):
        assert WORKSPACE_COMMAND in text
        assert seed in text and all(other not in text for other in seeds if other != seed)
        for name in (*NESTED_ORDINARY_TESTS, *RETURNED_ORDINARY_TESTS, *CF16_NEW_TESTS):
            assert re.search(r"test " + re.escape(name) + r" \.\.\. ok", text), name
        binary_summaries = {}
        for binary in ("tests/p3_patterns.rs", "tests/nested_monitor_regions.rs",
                       "tests/p3_java_recovery.rs"):
            _, binary_counts = binary_group(text.splitlines(), binary)
            assert len(binary_counts) == 1, (binary, binary_counts)
            if binary == "tests/p3_patterns.rs":
                assert binary_counts == [(85, 0, 0)], binary_counts
            binary_summaries[binary] = {"passed": sum(row[0] for row in binary_counts),
                                        "failed": sum(row[1] for row in binary_counts)}
        counts = test_counts(text)
        assert counts and all(failed == 0 for _, failed, _ in counts)
        assert not re.search(r"test result: FAILED|error: test failed", text)
        seed_results.append({"step": step, "seed": seed, "test_result_records": len(counts),
                             "passed": sum(row[0] for row in counts),
                             "failed": sum(row[1] for row in counts),
                             "ignored": sum(row[2] for row in counts),
                             "required_suite_binaries": binary_summaries})

    bigdecimal_command = "cargo test --test p3_bigdecimal_number_widening --locked -- --ignored"
    bigdecimal_groups = command_groups(lines, bigdecimal_command)
    assert len(bigdecimal_groups) == 1
    bigdecimal_counts = test_counts(bigdecimal_groups[0][1])
    assert bigdecimal_counts == [(2, 0, 0)]
    assert not re.search(r"test result: FAILED|error: test failed", bigdecimal_groups[0][1])

    nested_groups = command_groups(lines, NESTED_IGNORED_COMMAND)
    assert len(nested_groups) == 1
    nested_counts = test_counts(nested_groups[0][1])
    assert nested_counts == [(1, 0, 0)]
    assert re.search(r"test nested_int_array_updates_match_the_frozen_runner_oracle \.\.\. ok", nested_groups[0][1])

    returned_groups = command_groups(lines, RETURNED_IGNORED_COMMAND)
    assert len(returned_groups) == 1
    returned_counts = test_counts(returned_groups[0][1])
    assert returned_counts == [(1, 0, 0)]
    assert re.search(r"test returned_int_array_updates_compile_and_execute_as_a_complete_class \.\.\. ok", returned_groups[0][1])

    canonical = metadata["canonical_files"]
    assert len(canonical) == 22
    canonical_files = []
    for relative, expected in canonical.items():
        blob = git_blob(product, relative)
        current = read_current(relative)
        assert sha(blob) == expected, f"canonical Git blob differs from metadata: {relative}"
        assert sha(current) == expected, f"canonical current file differs from metadata: {relative}"
        canonical_files.append({"path": relative, "bytes": len(blob), "sha256": expected,
                                "current_matches_ci_product": blob == current})

    immutable_sources = {}
    for relative in IMMUTABLE_CURRENT_SOURCES:
        blob = git_blob(product, relative)
        current = read_current(relative)
        assert blob == current, f"immutable Git source differs from current workspace: {relative}"
        immutable_sources[relative] = {"bytes": len(blob), "sha256": sha(blob), "git_matches_current": True}

    supply_path = CI_EVIDENCE / "ci-supply-job-v1.log.gz"
    supply_compressed = supply_path.read_bytes()
    supply_raw = gzip.decompress(supply_compressed)
    supply_lines = re.sub(r"(?:\x1b\[[0-9;]*m|\^\[\[[0-9;]*m)", "", supply_raw.decode()).splitlines()
    install = command_groups(supply_lines, "cargo install cargo-deny --version 0.20.2 --locked")
    assert len(install) == 1 and "cargo-deny v0.20.2" in install[0][1]
    supply_checks = []
    for manifest in ("./Cargo.toml", "./fuzz/Cargo.toml"):
        command = f"cargo deny --all-features --workspace --locked --config ./deny.toml --manifest-path {manifest} check"
        groups = command_groups(supply_lines, command)
        assert len(groups) == 1
        for policy in ("advisories", "bans", "licenses", "sources"):
            assert f"{policy} ok" in groups[0][1], (manifest, policy)
        supply_checks.append(command)

    # Retain the successful local source-test gates as separately hash-verified evidence.
    # The reader census plus corpus fingerprint pair is the concrete reader/fingerprint record.
    prior_gates = [
        validate_recorded_gate(
            "openspec/changes/recover-proved-finally-cleanup/results/root-patterns-depth-final-v7/result.json",
            "cf16_region_depth_stop_never_publishes_class_source"),
        validate_recorded_gate(
            "openspec/changes/recover-returned-int-array-compound-updates/results/root-reader-census-v2/result.json",
            "classfile::tests::repository_class_fixtures_validate_without_false_target_rejections"),
        validate_recorded_gate(
            "openspec/changes/recover-returned-int-array-compound-updates/results/root-fingerprint-p5-v1/result.json",
            "corpus_files_match_the_recorded_fingerprint"),
    ]

    result = {
        "schema": "cf16-ci-checkpoint-root-acceptance-v1",
        "product_commit": product,
        "run_id": run_id,
        "url": raw["url"],
        "status": "accepted",
        "raw_ci_json_sha256": sha(raw_path.read_bytes()),
        "stable_raw_log_sha256": sha(log_bytes),
        "stable_gzip_log_sha256": sha(compressed_bytes),
        "jobs": len(raw["jobs"]),
        "steps": sum(len(job["steps"]) for job in raw["jobs"]),
        "all_jobs_and_steps_success": True,
        "workspace_seed_runs": seed_results,
        "bigdecimal_ignored_test": {"command": bigdecimal_command, "test_result_records": len(bigdecimal_counts),
                                     "passed": 2, "failed": 0, "ignored": 0},
        "installed_jdk": {"distribution": "Temurin", "version": EXPECTED_JDK,
                           "confirmed_from_stable_log": True},
        "source_identity": source_identity,
        "cf16_checkpoint_identity": cf16_identity,
        "immutable_current_sources": immutable_sources,
        "canonical_input_files": canonical_files,
        "prior_local_gates": prior_gates,
        "nested_ignored_test": {"command": NESTED_IGNORED_COMMAND, "passed": 1, "failed": 0, "ignored": 0},
        "returned_ignored_test": {"command": RETURNED_IGNORED_COMMAND, "passed": 1, "failed": 0, "ignored": 0},
        "supply_cli": {"version": "0.20.2", "commands": supply_checks,
                       "raw_log_sha256": sha(supply_raw), "gzip_log_sha256": sha(supply_compressed)},
        "candidate_cli": {"path": str(cli_path), "sha256": metadata["cli_sha256"],
                          "metadata_sha256": sha(metadata_bytes)},
        "workflow_sha256": sha(workflow.encode()),
        "verifier": "verify-ci-checkpoint-root-v3.py",
        "scope": "Only the immutable product commit and matching frozen CLI are accepted. Current-source hashes are recorded separately.",
        "log_partition": "two exact workspace command groups with distinct declared seeds; each contains CF16 tests and passing p3_patterns/nested_monitor_regions/p3_java_recovery binaries, plus BigDecimal and exact nested/returned ignored-test groups; API step names/status checked separately",
    }
    output = HERE / "ci-checkpoint-root-acceptance-v3.json"
    if output.exists():
        raise SystemExit(f"refusing to overwrite {output}")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("status", "product_commit", "run_id", "jobs", "steps",
                                                   "workspace_seed_runs", "bigdecimal_ignored_test",
                                                   "nested_ignored_test", "returned_ignored_test")}))


if __name__ == "__main__":
    main()
