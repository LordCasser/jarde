#!/usr/bin/env python3
"""Verify the frozen static-initializer product CI run and its local gates.

Usage: uv run --no-project --with blake3==1.0.11 python -B verify-ci-product-root-v7.py PRODUCT_SHA RUN_ID
This verifier is prepared for root to run after the workflow evidence is captured.
It independently fingerprints corpus files in Python; it never runs the Rust generator.
"""
import gzip
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import subprocess
import sys

import blake3


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CI_EVIDENCE = HERE / "ci-product-v4"
CANDIDATE_RESULTS = HERE / "candidate-luna-v1"
CANDIDATE_VERIFY = HERE / "candidate-root-verification-v1.json"
METADATA_PATH = HERE / "candidate-cli-v1.json"
METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
PRODUCT_SOURCES = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
TEST_SOURCES = {
    "tests/class_static_initializer_projection.rs",
    "tests/interface_initializer_proof.rs",
    "crates/jarde-java/tests/p3_patterns.rs",
    ".github/workflows/ci.yml",
}
STATIC_TESTS = (
    "nonfinal_runtime_constant_keeps_its_phase_in_an_ordered_static_group",
    "final_runtime_constant_without_constant_value_still_refuses_phase_change",
)
INTERFACE_TESTS = (
    "proves_all_runtime_writes_and_keeps_constantvalue_separate",
    "rejects_unclaimed_effects_duplicate_and_missing_writes_and_branches_as_whole_groups",
    "admits_a_qualified_forward_read_only_when_original_write_time_is_preserved",
    "refuses_exception_edges_and_runtime_constants_that_would_change_initialization_phase",
)
WORKSPACE_COMMAND = "cargo test --workspace --all-targets --all-features --locked"
IGNORED_COMMANDS = {
    "jdk25_instruction_boundaries": "cargo test --test jvm_bytecode_oracle --locked -- --ignored --exact jdk25_instruction_boundaries_match_public_bytecode_inspection",
    "p3_execution": "cargo test --test p3_execution_comparison --locked -- --ignored",
    "functional_constructor": "cargo test -p jarde-cli --test json_cli --locked -- --ignored --exact functional_constructor_arguments_replay_the_complete_class_on_both_jdks",
    "bigdecimal": "cargo test --test p3_bigdecimal_number_widening --locked -- --ignored",
    "nested": "cargo test --test p3_nested_int_array_compound_updates --locked -- --ignored --exact nested_int_array_updates_match_the_frozen_runner_oracle",
    "returned": "cargo test --test p3_nested_int_array_compound_updates --locked -- --ignored --exact returned_int_array_updates_compile_and_execute_as_a_complete_class",
}
LOCAL_GATES = {
    "root-static-tests-v2": ["cargo", "test", "--test", "class_static_initializer_projection", "--locked"],
    "root-interface-tests-v1": ["cargo", "test", "--test", "interface_initializer_proof", "--locked"],
    "root-scoped-clippy-v1": ["cargo", "clippy", "-p", "jarde"],
    "root-cli-build-v1": ["cargo", "build", "-p", "jarde-cli", "--bin", "jarde-cli", "--locked"],
}
VALIDATION_COMMANDS = [
    ["cargo", "fmt", "--all", "--", "--check"],
    ["openspec", "validate", "--all", "--strict", "--no-interactive"],
    ["git", "diff", "--check"],
]
EXPECTED_CLI = "/private/tmp/jarde-nonfinal-static-cli-v1"
EXPECTED_TEMURIN = "25.0.4+7.0.LTS"
EXPECTED_DENY_VERSION = "0.20.2"
PREVIOUS_PRODUCT = "7196a3653f7b8d5b6b285970c3b41b59adf84e55"
OLD_A1_GOLDEN_PRODUCT = "25c5a6fd09ed8603dbbbf3d217469432f27e5c40"
ORDINARY_ENUM_TEST = "enum_constants::tests::ordinary_class_fields_and_static_initializer_keep_the_existing_projection"
ENUM_CONSTANTS_SHA256 = "e08766db22e14e5e7496492de38242af04dacb78c78652ae2de0995dc3aafa6d"
ORDINARY_GATE = HERE / "root-ordinary-static-test-v1" / "result.json"
ORDINARY_TEST_ARGV = ["cargo", "test", "--lib", ORDINARY_ENUM_TEST, "--", "--exact", "--nocapture"]
A1_TEST_SOURCE = "tests/p3_array_slot_retype_locals.rs"
A1_TEST_NAME = "the_dynamic_dimension_control_projects_the_proven_static_field_initializer"
A1_TEST_GATE = HERE / "root-a1-static-tests-v1" / "result.json"
A1_TEST_ARGV = ["cargo", "test", "--test", "p3_array_slot_retype_locals", "--", "--nocapture"]
A1_BASELINE_SOURCE = "tests/fixtures/p3-array-slot-retype-locals/baseline/A1.jarde.java"
A1_EXPECTED_SOURCE = "tests/fixtures/p3-array-slot-retype-locals/expected/A1.static-init.jarde.java"
A1_BASELINE_SHA256 = "ca9109e99a8f8e97dec77739ba66d41f5650699468284f0bd2753363a3aba1a2"
A1_CLI_RESULT = HERE / "a1-static-regression-root-v1" / "result.json"
A1_CLI_OUTPUT = HERE / "a1-static-regression-root-v1" / "A1.current.jarde.java"
A1_CLI_STDOUT = HERE / "a1-static-regression-root-v1" / "stdout"
A1_CLI_STDERR = HERE / "a1-static-regression-root-v1" / "stderr"
A1_INPUT = ROOT / "tests/fixtures/p3-array-slot-retype-locals/v8/A1.class"
FINGERPRINT_PATH = "tests/fixtures/corpus-fingerprint.json"
FINGERPRINT_SCOPE_SOURCE = "tests/p5_corpus_fingerprint.rs"
FINGERPRINT_OLD_PRODUCT = "2daa21c99db7e126d91b5ccbe3d016208fd54aba"
FINGERPRINT_FAILURE_RUN = 38014660202
FINGERPRINT_OLD_SHA256 = "b7c86f74349da1ee12f13fa0d2c1b9e89765e1deef9343eec51d2fa457542f14"
FINGERPRINT_NEW_SHA256 = "6f7eac52f10e231cd45666438a31e9807bd4ba442c8ac4abfb83d56f51a38147"
FINGERPRINT_SCOPE_SHA256 = "a29cc5ca21c3201466c7c4bca62d84e16fe3cfe5b40b05a2bafcf29b2fc3ffaa"
FINGERPRINT_REGISTER_SCRIPT = HERE / "register-a1-fingerprint-luna-v1.py"
FINGERPRINT_REGISTER_SHA256 = "e6480a9dbc0bae6a3ba9d267e566a3422f067d6d02a41138f43c08849aa86b12"
FINGERPRINT_REGISTRATION = HERE / "register-a1-fingerprint-root-v1.json"
FINGERPRINT_EXECUTION_DIR = HERE / "root-a1-fingerprint-registration-execution-v1"
FINGERPRINT_TARGET = "tests/fixtures/p3-array-slot-retype-locals/expected/A1.static-init.jarde.java"
FINGERPRINT_TARGET_SHA256 = "a8b84e8cfd3fa4610a05ecd9ef3f1204eb267a063aa3fcd0d28dcc6d73cc96ca"
FINGERPRINT_TARGET_BLAKE3 = "eaac73c5450923dadab849829f1dc2ca1e9503b0c5c3aa14fbd7ab39df6cc7b4"
FINGERPRINT_ROWS_SHA256 = "9c60e2c918b231f2f789a78354d84d6526342608cd9aaf12c9f8a85ca062f068"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return Path(path).read_bytes()


def git_blob(product, relative):
    return subprocess.check_output(["git", "show", f"{product}:{relative}"], cwd=ROOT)


def strip_ordinary_enum_test(source):
    start_marker = ("    #[test]\n    fn ordinary_class_fields_and_static_initializer_keep_the_existing_projection() {\n").encode()
    next_marker = ("    #[test]\n    fn counted_side_effectful_suffix_executes_once_in_the_projected_source() {\n").encode()
    assert source.count(start_marker) == 1, "ordinary enum regression test marker"
    assert source.count(next_marker) == 1, "following enum test marker"
    start = source.index(start_marker)
    end = source.index(next_marker, start + len(start_marker))
    assert start < end
    return source[:start] + b"<ordinary-static-regression-test>\n" + source[end:]


def verify_ordinary_enum_regression(product):
    relative = "src/enum_constants.rs"
    current = read(ROOT / relative)
    submitted = git_blob(product, relative)
    previous = git_blob(PREVIOUS_PRODUCT, relative)
    assert sha(current) == ENUM_CONSTANTS_SHA256
    assert sha(submitted) == ENUM_CONSTANTS_SHA256
    assert strip_ordinary_enum_test(previous) == strip_ordinary_enum_test(submitted), (
        "submitted enum_constants.rs may differ from the prior product only inside the target test"
    )
    previous_test = previous[previous.index(
        b"    fn ordinary_class_fields_and_static_initializer_keep_the_existing_projection() {"):]
    submitted_test = submitted[submitted.index(
        b"    fn ordinary_class_fields_and_static_initializer_keep_the_existing_projection() {"):]
    assert previous_test != submitted_test, "the target test body changed in the new product"

    record = json.loads(read(ORDINARY_GATE))
    assert record["argv"] == ORDINARY_TEST_ARGV
    assert record["exit"] == 0 and record.get("runner_exit", 0) == 0
    assert record.get("disk_stop_reason") is None
    expected_source_path = (ROOT / relative).resolve()
    before = [entry for entry in record.get("before", [])
              if Path(entry["path"]).resolve() == expected_source_path]
    assert len(before) == 1
    assert before[0]["bytes"] == len(current) and before[0]["sha256"] == ENUM_CONSTANTS_SHA256
    stdout, stderr = verify_raw_record(record, ORDINARY_GATE.parent)
    output = strip_terminal(stdout + stderr)
    assert test_counts(output) == [(1, 0, 0)]
    assert re.search(r"test " + re.escape(ORDINARY_ENUM_TEST) + r" \.\.\. ok", output)
    return {
        "product_blob_sha256": sha(submitted), "current_source_sha256": sha(current),
        "previous_product": PREVIOUS_PRODUCT, "change_confined_to_target_test": True,
        "local_gate": {"path": str(ORDINARY_GATE.relative_to(ROOT)),
                       "result_sha256": sha(read(ORDINARY_GATE)), "argv": record["argv"],
                       "exit": record["exit"], "disk_stop_reason": None,
                       "test_counts": [1, 0, 0], "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)},
    }


def without_a1_baseline_const(source):
    declaration = (
        b'const A1_BASELINE: &str =\n'
        b'    include_str!("fixtures/p3-array-slot-retype-locals/baseline/A1.jarde.java");\n'
    )
    assert source.count(declaration) == 1, "old A1 baseline include is present exactly once"
    return source.replace(declaration, b"", 1)


def normalize_final_a1_test(source, expected_name):
    start = source.rfind(b"#[test]")
    assert start >= 0, "final test marker exists"
    tail = source[start:]
    assert expected_name.encode() in tail, (expected_name, tail[-240:])
    assert tail.rstrip().endswith(b"}"), "the A1 regression test is the final source item"
    assert tail.count(b"#[test]") == 1, "only the final A1 regression test is normalized"
    return source[:start] + b"#[test]\n<A1 regression test body>\n"


def verify_a1_static_regression(product):
    test_current = read(ROOT / A1_TEST_SOURCE)
    test_submitted = git_blob(product, A1_TEST_SOURCE)
    test_25c5 = git_blob(OLD_A1_GOLDEN_PRODUCT, A1_TEST_SOURCE)
    assert test_current == test_submitted, "current A1 test source equals the submitted Git blob"
    old_without_const = without_a1_baseline_const(test_25c5)
    assert b"A1_BASELINE" not in test_submitted, "submitted A1 test no longer includes the stale golden"
    assert test_submitted.count(A1_TEST_NAME.encode()) == 1
    assert b"the_dynamic_dimension_control_keeps_its_text_verbatim" not in test_submitted
    assert normalize_final_a1_test(old_without_const,
                                   "the_dynamic_dimension_control_keeps_its_text_verbatim") == \
        normalize_final_a1_test(test_submitted, A1_TEST_NAME), (
            "against 25c5, only A1_BASELINE and the final A1 test may change"
        )

    baseline_current = read(ROOT / A1_BASELINE_SOURCE)
    baseline_submitted = git_blob(product, A1_BASELINE_SOURCE)
    baseline_25c5 = git_blob(OLD_A1_GOLDEN_PRODUCT, A1_BASELINE_SOURCE)
    assert baseline_current == baseline_submitted == baseline_25c5, (
        "the old A1 golden remains byte-for-byte identical to 25c5"
    )
    assert sha(baseline_current) == A1_BASELINE_SHA256

    expected = read(ROOT / A1_EXPECTED_SOURCE)
    expected_submitted = git_blob(product, A1_EXPECTED_SOURCE)
    actual = read(A1_CLI_OUTPUT)
    cli_record = json.loads(read(A1_CLI_RESULT))
    result_stdout = read(A1_CLI_STDOUT)
    result_stderr = read(A1_CLI_STDERR)
    assert cli_record["argv"] == [
        EXPECTED_CLI, "class-source", "--input", str(A1_INPUT), "--class", "A1",
        "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all",
    ]
    assert cli_record["exit"] == 0
    assert cli_record["cli"]["path"] == EXPECTED_CLI
    assert cli_record["cli"]["sha256"] == CLI_SHA256
    assert cli_record["cli"]["bytes"] > 0
    assert cli_record["input"]["path"] == str(A1_INPUT)
    assert cli_record["input"]["bytes"] == len(read(A1_INPUT))
    assert cli_record["input"]["sha256"] == sha(read(A1_INPUT))
    stdout_meta = cli_record["stdout"]
    assert len(result_stdout) == stdout_meta["bytes"] and sha(result_stdout) == stdout_meta["sha256"]
    stderr_meta = cli_record["stderr"]
    assert len(result_stderr) == stderr_meta["bytes"] and sha(result_stderr) == stderr_meta["sha256"]
    document = json.loads(result_stdout)
    assert document["text"].encode("utf-8") == actual
    assert expected == expected_submitted == actual, (
        "new A1 expected source equals the submitted file and frozen CLI's actual text"
    )
    assert sha(expected) == sha(actual)

    gate = json.loads(read(A1_TEST_GATE))
    assert gate["argv"] == A1_TEST_ARGV
    assert gate["exit"] == 0 and gate.get("runner_exit", 0) == 0
    assert gate.get("disk_stop_reason") is None
    gate_stdout, gate_stderr = verify_raw_record(gate, A1_TEST_GATE.parent)
    gate_output = strip_terminal(gate_stdout + gate_stderr)
    assert test_counts(gate_output) == [(6, 0, 0)]
    assert re.search(r"test " + re.escape(A1_TEST_NAME) + r" \.\.\. ok", gate_output)
    before = {}
    for entry in gate.get("before", []):
        path = Path(entry["path"])
        relative = str(path.relative_to(ROOT)) if path.is_absolute() else str(path)
        current = read(ROOT / relative)
        submitted = git_blob(product, relative)
        assert len(current) == entry["bytes"] and sha(current) == entry["sha256"], relative
        assert len(submitted) == entry["bytes"] and sha(submitted) == entry["sha256"], relative
        before[relative] = entry["sha256"]
    assert before.get(A1_TEST_SOURCE) == sha(test_submitted)
    assert before.get(A1_EXPECTED_SOURCE) == sha(expected)
    return {
        "test_source": {"path": A1_TEST_SOURCE, "current_sha256": sha(test_current),
                        "submitted_blob_sha256": sha(test_submitted),
                        "current_equals_submitted": True,
                        "change_from_25c5": "removed A1_BASELINE constant and replaced only final A1 test"},
        "old_golden": {"path": A1_BASELINE_SOURCE, "sha256": sha(baseline_current),
                       "byte_identical_to_25c5": True},
        "expected_and_cli_output": {"expected_path": A1_EXPECTED_SOURCE,
                                    "expected_sha256": sha(expected),
                                    "cli_output_path": str(A1_CLI_OUTPUT.relative_to(ROOT)),
                                    "cli_output_sha256": sha(actual), "byte_equal": True,
                                    "cli_result_sha256": sha(read(A1_CLI_RESULT)),
                                    "cli_stdout_sha256": sha(result_stdout),
                                    "cli_stderr_sha256": sha(result_stderr),
                                    "submitted_expected_sha256": sha(expected_submitted)},
        "local_gate": {"path": str(A1_TEST_GATE.relative_to(ROOT)),
                       "result_sha256": sha(read(A1_TEST_GATE)), "argv": gate["argv"],
                       "exit": gate["exit"], "disk_stop_reason": None,
                       "test_counts": [6, 0, 0], "target_test": A1_TEST_NAME,
                       "stdout_sha256": sha(gate_stdout), "stderr_sha256": sha(gate_stderr),
                       "before_source_count": len(before), "before_sources_match_submitted": True},
    }


def corpus_collect(paths, excluded_dirs, excluded_names, excluded_extensions):
    found = []

    def visit(directory):
        with os.scandir(directory) as iterator:
            entries = sorted(iterator, key=lambda entry: entry.name)
        for entry in entries:
            path = Path(entry.path)
            if entry.is_dir(follow_symlinks=False):
                if entry.name not in excluded_dirs:
                    visit(path)
                continue
            if not entry.is_file(follow_symlinks=False) or entry.name in excluded_names:
                continue
            extension = path.suffix[1:] if path.suffix else ""
            if extension not in excluded_extensions:
                found.append(path.relative_to(ROOT).as_posix())

    for relative in paths:
        directory = ROOT / relative
        assert directory.is_dir() and not directory.is_symlink(), relative
        visit(directory)
    return sorted(found)


def corpus_fingerprint(paths):
    rows = []
    for relative in paths:
        data = read(ROOT / relative)
        rows.append({"blake3": blake3.blake3(data).hexdigest(),
                     "bytes": len(data), "path": relative})
    return rows


def verify_previous_fingerprint_failure():
    folder = HERE / "ci-2daa-failure-v1"
    capture = json.loads(read(folder / "capture.json"))
    assert len(capture) == 2
    api_capture, stable_capture = capture
    assert api_capture["argv"] == ["gh", "run", "view", str(FINGERPRINT_FAILURE_RUN),
                                   "--json", "headSha,status,conclusion,url,jobs"]
    assert stable_capture["argv"] == ["gh", "run", "view", str(FINGERPRINT_FAILURE_RUN),
                                      "--job", "114102160125", "--log"]
    assert api_capture["exit"] == stable_capture["exit"] == 0
    api_path = folder / api_capture["output"]
    api_raw = read(api_path)
    assert len(api_raw) == api_capture["raw_stdout_bytes"]
    assert sha(api_raw) == api_capture["raw_stdout_sha256"] == api_capture["stored_sha256"]
    api_stderr = read(folder / "ci-run.json.stderr")
    assert len(api_stderr) == api_capture["stderr_bytes"]
    assert sha(api_stderr) == api_capture["stderr_sha256"]
    api = json.loads(api_raw)
    assert api["headSha"] == FINGERPRINT_OLD_PRODUCT
    assert api["status"] == "completed" and api["conclusion"] == "failure"
    assert api["url"].rstrip("/").endswith(f"/actions/runs/{FINGERPRINT_FAILURE_RUN}")
    jobs = {job["name"]: job for job in api["jobs"]}
    stable_job = jobs["stable / test and specification"]
    assert stable_job["conclusion"] == "failure"
    failed_steps = [step for step in stable_job["steps"] if step["conclusion"] == "failure"]
    assert [step["name"] for step in failed_steps] == [
        "Run workspace tests with two fixed independent seeds (pass 1)"]
    assert all(job["conclusion"] == "success" for name, job in jobs.items()
               if name != "stable / test and specification")

    stable_path = folder / stable_capture["output"]
    stable_gzip = read(stable_path)
    assert sha(stable_gzip) == stable_capture["stored_sha256"]
    stable_raw = gzip.decompress(stable_gzip)
    assert len(stable_raw) == stable_capture["raw_stdout_bytes"]
    assert sha(stable_raw) == stable_capture["raw_stdout_sha256"]
    stable_stderr = read(folder / "stable.log.gz.stderr")
    assert len(stable_stderr) == stable_capture["stderr_bytes"]
    assert sha(stable_stderr) == stable_capture["stderr_sha256"]
    lines = strip_terminal(stable_raw).splitlines()
    work = command_groups(lines, WORKSPACE_COMMAND)
    assert len(work) == 1
    workspace_text = work[0][1]
    assert "5350648285461741569" in workspace_text
    assert "5350648285461741570" not in workspace_text
    workspace_failed_tests = re.findall(
        r"test ([A-Za-z0-9_:]+) \.\.\. FAILED", workspace_text)
    assert workspace_failed_tests == ["corpus_files_match_the_recorded_fingerprint"]
    workspace_failed_summaries = re.findall(r"test result: FAILED\. [^\n]+", workspace_text)
    assert len(workspace_failed_summaries) == 1
    p5_starts = [i for i, line in enumerate(work[0][1].splitlines())
                 if "Running tests/p5_corpus_fingerprint.rs" in line]
    assert len(p5_starts) == 1
    work_lines = work[0][1].splitlines()
    start = p5_starts[0]
    end = next((i for i in range(start + 1, len(work_lines))
                if "     Running " in work_lines[i] or "   Doc-tests " in work_lines[i]), len(work_lines))
    p5_block = "\n".join(work_lines[start:end])
    summary = re.search(r"test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored;", p5_block)
    assert summary and tuple(map(int, summary.groups())) == (4, 1, 1)
    failed_tests = re.findall(
        r"test (corpus_files_match_the_recorded_fingerprint) \.\.\. FAILED", p5_block)
    assert failed_tests == ["corpus_files_match_the_recorded_fingerprint"]
    assert ("unlisted: " + FINGERPRINT_TARGET +
            " (2210, blake3 " + FINGERPRINT_TARGET_BLAKE3 +
            ") is in the corpus but not in the manifest") in p5_block
    assert "missing:" not in p5_block
    return {
        "run_id": FINGERPRINT_FAILURE_RUN, "head_sha": api["headSha"],
        "api_capture_sha256": sha(api_raw), "stable_gzip_sha256": sha(stable_gzip),
        "stable_raw_sha256": sha(stable_raw), "failed_seed_step": failed_steps[0]["name"],
        "p5_fingerprint_tests": {"passed": 4, "failed": 1, "ignored": 1,
                                  "only_failed_test": failed_tests[0],
                                  "only_unlisted_file": FINGERPRINT_TARGET},
    }


def verify_a1_fingerprint_registration(product):
    assert importlib.metadata.version("blake3") == "1.0.11"
    registration = json.loads(read(FINGERPRINT_REGISTRATION))
    execution_path = FINGERPRINT_EXECUTION_DIR / "execution.json"
    execution = json.loads(read(execution_path))
    script_bytes = read(FINGERPRINT_REGISTER_SCRIPT)
    assert sha(script_bytes) == FINGERPRINT_REGISTER_SHA256
    expected_argv = ["uv", "run", "--no-project", "--with", "blake3", "python", "-B",
                     str(FINGERPRINT_REGISTER_SCRIPT)]
    assert execution["argv"] == expected_argv and execution["cwd"] == str(ROOT)
    assert execution["exit"] == 0 and execution["script_sha256"] == FINGERPRINT_REGISTER_SHA256
    for name in ("stdout", "stderr"):
        expected = execution["streams"][name]
        raw = read(FINGERPRINT_EXECUTION_DIR / name)
        assert len(raw) == expected["bytes"] and sha(raw) == expected["sha256"]
    register_stdout = read(FINGERPRINT_EXECUTION_DIR / "stdout")
    register_stderr = read(FINGERPRINT_EXECUTION_DIR / "stderr")
    assert json.loads(register_stdout) == registration
    assert register_stderr.decode("utf-8") == "WARN `--no-project` was provided, but no project was found\n"

    rust_current = read(ROOT / FINGERPRINT_SCOPE_SOURCE)
    rust_submitted = git_blob(product, FINGERPRINT_SCOPE_SOURCE)
    assert sha(rust_current) == sha(rust_submitted) == FINGERPRINT_SCOPE_SHA256
    assert registration["schema"] == "jarde-a1-corpus-registration/1"
    assert registration["status"] == "registered"
    assert registration["manifest"] == FINGERPRINT_PATH
    assert registration["rust_scope_source_sha256"] == FINGERPRINT_SCOPE_SHA256
    assert registration["blake3_package_version"] == "1.0.11"
    rust_generator = registration["rust_generator"]
    assert rust_generator["command"] == (
        "cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint")
    assert rust_generator["status"] == "not_executed_by_this_script"
    assert rust_generator["stdout"] is None and rust_generator["stderr"] is None
    assert "The Python registrar does not invoke the Rust generator" in rust_generator["note"]

    old_bytes = git_blob(FINGERPRINT_OLD_PRODUCT, FINGERPRINT_PATH)
    submitted_bytes = git_blob(product, FINGERPRINT_PATH)
    current_bytes = read(ROOT / FINGERPRINT_PATH)
    assert sha(old_bytes) == registration["manifest_sha256_before"] == FINGERPRINT_OLD_SHA256
    assert sha(submitted_bytes) == sha(current_bytes) == registration["manifest_sha256_after"] == FINGERPRINT_NEW_SHA256
    old, updated, current = map(json.loads, (old_bytes, submitted_bytes, current_bytes))
    assert updated == current
    assert {key: value for key, value in old.items() if key != "files"} == \
        {key: value for key, value in updated.items() if key != "files"}, \
        "corpus classification or other non-files metadata changed"
    comparison = registration["corpus_comparison"]
    old_files, new_files = old["files"], updated["files"]
    added = {"blake3": FINGERPRINT_TARGET_BLAKE3, "bytes": 2210, "path": FINGERPRINT_TARGET}
    assert len(old_files) == comparison["manifest_files_before"] == 2061
    assert len(new_files) == comparison["manifest_files_after"] == 2062
    assert comparison["scanned_files"] == 2062 and comparison["unchanged_existing_entries"] == 2061
    assert comparison["added_only"] == added
    assert comparison["unlisted_before"] == [FINGERPRINT_TARGET]
    assert comparison["missing_before"] == [] and comparison["existing_pin_mismatches"] == []
    assert FINGERPRINT_TARGET not in {row["path"] for row in old_files}
    assert new_files == sorted([*old_files, added], key=lambda row: row["path"])
    assert sha((json.dumps(new_files, ensure_ascii=False, indent=2) + "\n").encode()) == FINGERPRINT_ROWS_SHA256
    assert comparison["scanned_entries_sha256"] == comparison["updated_files_array_sha256"] == FINGERPRINT_ROWS_SHA256

    # Recompute the complete scoped corpus fingerprint in Python; this is not the Rust test/generator.
    roots = ["tests/fixtures", "fuzz/corpus"]
    excluded_dirs = ["target", "out", "artifacts", "__pycache__"]
    excluded_names = ["corpus-fingerprint.json"]
    excluded_extensions = ["md", "py"]
    assert b'const ROOTS: &[&str] = &["tests/fixtures", "fuzz/corpus"];' in rust_current
    assert b'const EXCLUDED_DIRECTORIES: &[&str] = &["target", "out", "artifacts", "__pycache__"];' in rust_current
    assert b'const EXCLUDED_FILE_NAMES: &[&str] = &["corpus-fingerprint.json"];' in rust_current
    assert b'const EXCLUDED_EXTENSIONS: &[&str] = &["md", "py"];' in rust_current
    corpus_paths = corpus_collect(roots, excluded_dirs, excluded_names, excluded_extensions)
    scanned = corpus_fingerprint(corpus_paths)
    assert len(scanned) == 2062 and scanned == new_files
    added_file = read(ROOT / FINGERPRINT_TARGET)
    assert len(added_file) == 2210 and sha(added_file) == FINGERPRINT_TARGET_SHA256
    assert blake3.blake3(added_file).hexdigest() == FINGERPRINT_TARGET_BLAKE3
    assert registration["corpus_comparison"]["scanned_entries_sha256"] == sha(
        (json.dumps(scanned, ensure_ascii=False, indent=2) + "\n").encode())

    return {
        "old_failure": verify_previous_fingerprint_failure(),
        "registration": {"path": str(FINGERPRINT_REGISTRATION.relative_to(ROOT)),
                         "sha256": sha(read(FINGERPRINT_REGISTRATION)),
                         "execution_path": str(execution_path.relative_to(ROOT)),
                         "execution_sha256": sha(read(execution_path)),
                         "registrar_sha256": FINGERPRINT_REGISTER_SHA256,
                         "registration_stdout_sha256": sha(register_stdout),
                         "registration_stderr_sha256": sha(register_stderr)},
        "manifest_transition": {"old_sha256": sha(old_bytes), "new_sha256": sha(submitted_bytes),
                                "old_files": len(old_files), "new_files": len(new_files),
                                "existing_2061_rows_byte_identical": True,
                                "only_added_row": added, "non_files_classification_identical": True},
        "independent_python_b3_scan": {"files": len(scanned),
                                       "rows_sha256": FINGERPRINT_ROWS_SHA256,
                                       "matches_submitted_manifest": True,
                                       "rust_generator_executed": False},
        "new_fixture": {"path": FINGERPRINT_TARGET, "bytes": len(added_file),
                        "sha256": sha(added_file), "blake3": blake3.blake3(added_file).hexdigest()},
    }


def test_counts(text):
    return [tuple(map(int, row)) for row in re.findall(
        r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]


def strip_terminal(raw):
    text = raw.decode("utf-8", errors="replace")
    text = re.sub(r"(?:\x1b\[[0-9;]*m|\^\[\[[0-9;]*m)", "", text)
    # Actions log downloads can prepend ISO timestamps to individual lines.
    return re.sub(r"(?m)^\d{4}-\d\d-\d\d[T ][^ ]+Z\s+", "", text)


def group_text(lines, start):
    end = next((i for i in range(start + 1, len(lines)) if "##[group]Run " in lines[i]), len(lines))
    return "\n".join(lines[start:end])


def command_groups(lines, command):
    needle = f"##[group]Run {command}"
    return [(i, group_text(lines, i)) for i, line in enumerate(lines) if needle in line]


def binary_group(lines, binary, expected_count, expected_ignored=0):
    starts = [i for i, line in enumerate(lines) if f"Running {binary}" in line]
    assert len(starts) == 1, (binary, len(starts))
    start = starts[0]
    end = next((i for i in range(start + 1, len(lines))
                if "     Running " in lines[i] or "   Doc-tests " in lines[i]), len(lines))
    block = "\n".join(lines[start:end])
    counts = test_counts(block)
    assert counts == [(expected_count, 0, expected_ignored)], (binary, counts)
    assert "test result: FAILED" not in block and "error: test failed" not in block, binary
    return block, counts


def verify_raw_record(record, base_dir):
    stdout_meta, stderr_meta = record["stdout"], record["stderr"]
    stdout_path, stderr_path = Path(stdout_meta["path"]), Path(stderr_meta["path"])
    if not stdout_path.is_absolute():
        stdout_path = base_dir / stdout_path
    if not stderr_path.is_absolute():
        stderr_path = base_dir / stderr_path
    stdout, stderr = read(stdout_path), read(stderr_path)
    assert len(stdout) == stdout_meta["bytes"] and sha(stdout) == stdout_meta["sha256"]
    assert len(stderr) == stderr_meta["bytes"] and sha(stderr) == stderr_meta["sha256"]
    return stdout, stderr


def verify_gate(name, expected_argv, product, product_source_map):
    result_path = HERE / name / "result.json"
    record = json.loads(read(result_path))
    assert record["argv"][:len(expected_argv)] == expected_argv, (name, record["argv"])
    assert record["exit"] == 0 and record.get("runner_exit", 0) == 0, name
    assert record.get("disk_stop_reason") is None, name
    stdout, stderr = verify_raw_record(record, result_path.parent)
    before = {}
    for entry in record.get("before", []):
        path = Path(entry["path"])
        relative = str(path.relative_to(ROOT)) if path.is_absolute() else str(path)
        blob = git_blob(product, relative)
        assert len(blob) == entry["bytes"] and sha(blob) == entry["sha256"], (name, relative)
        before[relative] = entry["sha256"]
    for relative, expected in product_source_map.items():
        if relative in before:
            assert before[relative] == expected, (name, relative)
    return {"name": name, "path": str(result_path.relative_to(ROOT)),
            "result_sha256": sha(read(result_path)), "argv": record["argv"],
            "exit": record["exit"], "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr),
            "source_before_count": len(before), "source_before_matches_product": True}


def verify_validation():
    folder = HERE / "root-validation-v1"
    result_path = folder / "result.json"
    records = json.loads(read(result_path))
    assert len(records) == len(VALIDATION_COMMANDS)
    out = []
    for index, (record, expected_argv) in enumerate(zip(records, VALIDATION_COMMANDS)):
        assert record["argv"] == expected_argv and record["exit"] == 0
        stdout, stderr = read(folder / f"{index}.stdout"), read(folder / f"{index}.stderr")
        assert sha(stdout) == record["stdout_sha256"] and sha(stderr) == record["stderr_sha256"]
        out.append({"argv": record["argv"], "exit": record["exit"],
                    "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
    return {"result_sha256": sha(read(result_path)), "commands": out}


def verify_local_gates(product, product_source_map):
    out = []
    gate_base = HERE
    for name, argv in LOCAL_GATES.items():
        result_path = gate_base / name / "result.json"
        record = json.loads(read(result_path))
        assert record["argv"][:len(argv)] == argv, (name, record["argv"])
        stdout, stderr = verify_raw_record(record, result_path.parent)
        item = verify_gate(name, argv, product, product_source_map)
        if name == "root-static-tests-v2":
            text = (stdout + stderr).decode("utf-8", errors="replace")
            assert test_counts(text) == [(6, 0, 0)]
            for test in STATIC_TESTS:
                assert re.search(r"test " + re.escape(test) + r" \.\.\. ok", text), test
            item["test_counts"] = {"class_static_initializer_projection": [6, 0, 0]}
        elif name == "root-interface-tests-v1":
            text = (stdout + stderr).decode("utf-8", errors="replace")
            assert test_counts(text) == [(4, 0, 0)]
            for test in INTERFACE_TESTS:
                assert re.search(r"test " + re.escape(test) + r" \.\.\. ok", text), test
            item["test_counts"] = {"interface_initializer_proof": [4, 0, 0]}
        out.append(item)

    out.append({"name": "root-ordinary-static-test-v1", **verify_ordinary_enum_regression(product)})
    out.append({"name": "root-a1-static-tests-v1", **verify_a1_static_regression(product)})

    validation = verify_validation()
    out.append({"name": "root-validation-v1", **validation})

    clean_path = HERE / "root-clean-v1/result.json"
    clean = json.loads(read(clean_path))
    assert clean["argv"] == ["cargo", "clean", "--target-dir", str(ROOT / "target")]
    assert clean["exit"] == 0 and clean["target_exists_after"] is False
    assert clean["cli_sha256"] == CLI_SHA256 and clean["metadata_sha256"] == METADATA_SHA256
    assert clean["frozen_sources_unchanged"] is True
    stdout, stderr = read(clean_path.parent / "stdout"), read(clean_path.parent / "stderr")
    assert sha(stdout) == clean["stdout_sha256"] and sha(stderr) == clean["stderr_sha256"]
    out.append({"name": "root-clean-v1", "result_sha256": sha(read(clean_path)),
                "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr), "exit": clean["exit"]})
    return out


def verify_candidate(product):
    assert sha(read(EXPECTED_CLI)) == CLI_SHA256
    metadata_bytes = read(METADATA_PATH)
    assert sha(metadata_bytes) == METADATA_SHA256
    metadata = json.loads(metadata_bytes)
    assert metadata["cli_path"] == EXPECTED_CLI and metadata["cli_sha256"] == CLI_SHA256
    assert set(metadata["candidate_sources"]) == PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == TEST_SOURCES
    assert len(metadata["canonical_files"]) == 16
    product_sources, test_sources = {}, {}
    current_frozen = {}
    for category, pins, destination in (
        ("product", metadata["candidate_sources"], product_sources),
        ("test", metadata["test_sources"], test_sources),
    ):
        for relative, expected in pins.items():
            blob, current = git_blob(product, relative), read(ROOT / relative)
            assert sha(blob) == expected, f"{category} Git blob changed: {relative}"
            destination[relative] = expected
            current_frozen[relative] = {"sha256": expected, "git_matches_pin": True,
                                        "current_matches_pin": sha(current) == expected, "current_sha256": sha(current)}

    canonical = []
    for relative, expected in metadata["canonical_files"].items():
        blob = git_blob(product, relative)
        assert sha(blob) == expected, f"canonical input Git blob changed: {relative}"
        current = read(ROOT / relative)
        canonical.append({"path": relative, "bytes": len(blob), "sha256": expected,
                          "current_sha256": sha(current), "current_matches_product": current == blob})

    manifest_path = CANDIDATE_RESULTS / "manifest.json"
    inventory_path = CANDIDATE_RESULTS / "file-inventory.json"
    manifest_bytes, inventory_bytes = read(manifest_path), read(inventory_path)
    manifest, inventory = json.loads(manifest_bytes), json.loads(inventory_bytes)
    manifest_sha, inventory_sha = sha(manifest_bytes), sha(inventory_bytes)
    assert manifest["status"] == "completed"
    assert manifest["candidate_cli"]["sha256"] == CLI_SHA256
    assert manifest["metadata"]["sha256"] == METADATA_SHA256
    assert manifest["file_inventory"]["path"] == "file-inventory.json"
    listed = set()
    for entry in inventory:
        relative = entry["path"]
        assert relative not in listed and relative != "file-inventory.json"
        listed.add(relative)
        path = CANDIDATE_RESULTS / relative
        data = read(path)
        assert len(data) == entry["bytes"] and sha(data) == entry["sha256"], relative
    actual_files = {str(path.relative_to(CANDIDATE_RESULTS)) for path in CANDIDATE_RESULTS.rglob("*")
                    if path.is_file() and path != inventory_path}
    assert listed == actual_files
    assert len(manifest["cases"]) == 4 and all(case["success"] for case in manifest["cases"])
    assert manifest["case_counts"] == {"candidate": 4} and manifest["success_counts"] == {"candidate": 4}
    assert manifest["status"] == "completed"

    verification_bytes = read(CANDIDATE_VERIFY)
    candidate_verify = json.loads(verification_bytes)
    assert candidate_verify["status"] in ("passed", "accepted")
    assert candidate_verify.get("errors", []) == []
    assert candidate_verify["candidate_manifest_sha256"] == manifest_sha
    assert len(candidate_verify["candidate_legs"]) == 4
    assert all(leg["raw_matches_original_and_baseline_jarde"] for leg in candidate_verify["candidate_legs"])
    assert candidate_verify["candidate_inventory_sha256"] == inventory_sha
    assert candidate_verify["candidate_cli"]["sha256"] == CLI_SHA256
    assert candidate_verify["candidate_cli"]["metadata_sha256"] == METADATA_SHA256
    assert candidate_verify["checks"] == 2245 and candidate_verify["command_count"] == 16
    return {
        "metadata_sha256": sha(metadata_bytes), "candidate_cli_sha256": CLI_SHA256,
        "current_frozen_source_identity": current_frozen, "canonical_git_blobs": canonical,
        "candidate_manifest_sha256": manifest_sha, "candidate_inventory_sha256": inventory_sha,
        "candidate_file_count": len(inventory), "candidate_case_count": len(manifest["cases"]),
        "candidate_root_verification": {"path": str(CANDIDATE_VERIFY.relative_to(ROOT)),
                                         "sha256": sha(verification_bytes),
                                         "manifest_sha256": manifest_sha,
                                         "inventory_sha256": inventory_sha},
    }


def verify_ci(product, run_id):
    raw_path = CI_EVIDENCE / "ci-run-v1.json"
    raw_bytes = read(raw_path)
    raw = json.loads(raw_bytes)
    assert raw["headSha"] == product
    assert raw["status"] == "completed" and raw["conclusion"] == "success"
    match = re.search(r"/actions/runs/(\d+)(?:$|[/?#])", raw["url"])
    assert match and int(match.group(1)) == run_id
    expected_jobs = {"stable / test and specification", "MSRV 1.88.0", "supply chain", "fuzz smoke"}
    assert {job["name"] for job in raw["jobs"]} == expected_jobs
    step_count = sum(len(job["steps"]) for job in raw["jobs"])
    assert step_count == 52, step_count
    for job in raw["jobs"]:
        assert job["status"] == "completed" and job["conclusion"] == "success"
        assert all(step["status"] == "completed" and step["conclusion"] == "success"
                   for step in job["steps"]), job["name"]

    jobs = {job["name"]: job for job in raw["jobs"]}
    stable = jobs["stable / test and specification"]
    stable_names = {step["name"] for step in stable["steps"]}
    required_stable = {
        "Check formatting", "Run Clippy", "Run workspace tests with two fixed independent seeds (pass 1)",
        "Run workspace tests with two fixed independent seeds (pass 2)",
        "Run ignored JDK 25 instruction-boundary oracle",
        "Run the ignored P3 Java 8 compile-and-execute comparison",
        "Run the functional-constructor full-class comparison",
        "Run the BigDecimal Number full-class comparisons",
        "Run the nested int array full-class comparisons",
        "Run the returned int array full-class comparison", "Run public API example",
        "Check resolved feature tree", "Check normal dependency tree boundary",
        "Check layered crate dependency closure", "Validate OpenSpec strictly",
        "Ensure tracked files remain unchanged",
    }
    assert required_stable <= stable_names
    assert {"Install Temurin 25.0.4+7"} <= stable_names
    assert {"Check workspace on MSRV"} <= {step["name"] for step in jobs["MSRV 1.88.0"]["steps"]}
    assert {"Install cargo-deny 0.20.2", "Check root workspace advisories, licenses, bans, and sources",
            "Check fuzz workspace advisories, licenses, bans, and sources"} <= {
                step["name"] for step in jobs["supply chain"]["steps"]}
    assert {"Run the bounded query smoke", "Run the bounded artifact-tree smoke",
            "Run the bounded method-analysis smoke", "Ensure the smoke runs left the tracked tree unchanged"} <= {
                step["name"] for step in jobs["fuzz smoke"]["steps"]}

    metadata = json.loads(read(METADATA_PATH))
    workflow = git_blob(product, ".github/workflows/ci.yml").decode()
    seeds = re.findall(r'PROPTEST_RNG_SEED:\s*"(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    assert workflow.count(f"run: {WORKSPACE_COMMAND}") == 2
    assert 'java-version: "25.0.4+7.0.LTS"' in workflow and "distribution: temurin" in workflow

    stable_gzip_path = CI_EVIDENCE / "ci-stable-job-v1.log.gz"
    stable_gzip = read(stable_gzip_path)
    stable_raw = gzip.decompress(stable_gzip)
    stable_text = strip_terminal(stable_raw)
    assert "distribution: temurin" in stable_text
    assert f"java-version: {EXPECTED_TEMURIN}" in stable_text
    assert f"Java {EXPECTED_TEMURIN} was downloaded" in stable_text
    assert f"Setting Java {EXPECTED_TEMURIN} as the default" in stable_text
    assert "Temurin-Hotspot_jdk/25.0.4-7.0.LTS" in stable_text

    lines = stable_text.splitlines()
    work_groups = command_groups(lines, WORKSPACE_COMMAND)
    assert len(work_groups) == 2
    seeds_found = []
    workspace = []
    for (_, text), step, seed in zip(work_groups, (
        "Run workspace tests with two fixed independent seeds (pass 1)",
        "Run workspace tests with two fixed independent seeds (pass 2)"), seeds):
        assert seed in text and all(other not in text for other in seeds if other != seed)
        for test in STATIC_TESTS + INTERFACE_TESTS + (ORDINARY_ENUM_TEST,):
            assert re.search(r"test " + re.escape(test) + r" \.\.\. ok", text), test
        assert re.search(r"test " + re.escape(A1_TEST_NAME) + r" \.\.\. ok", text), A1_TEST_NAME
        binaries = {}
        for binary, expected in (("tests/class_static_initializer_projection.rs", 6),
                                 ("tests/interface_initializer_proof.rs", 4),
                                 ("tests/p3_patterns.rs", 85),
                                 ("tests/p3_array_slot_retype_locals.rs", 6)):
            _, counts = binary_group(text.splitlines(), binary, expected)
            binaries[binary] = {"passed": expected, "failed": 0, "ignored": 0}
        for binary in ("tests/p5_corpus_fingerprint.rs", "tests/p5_bulk_corpus.rs"):
            _, counts = binary_group(text.splitlines(), binary, 5, 1)
            binaries[binary] = {"passed": 5, "failed": 0, "ignored": 1}
        assert "test classfile::tests::repository_class_fixtures_validate_without_false_target_rejections ... ok" in text
        for path in ("crates/jarde-reader/src/classfile.rs",):
            assert git_blob(product, path) == git_blob("acd55c213ac7c670c6e76609871281a341d03ad0", path)
        totals = test_counts(text)
        assert totals and all(failed == 0 for _, failed, _ in totals)
        assert not re.search(r"test result: FAILED|error: test failed", text)
        workspace.append({"step": step, "seed": seed, "binaries": binaries,
                          "test_result_records": len(totals), "passed": sum(x[0] for x in totals),
                          "failed": sum(x[1] for x in totals), "ignored": sum(x[2] for x in totals)})
        seeds_found.append(seed)

    ignored_results = {}
    for name, command in IGNORED_COMMANDS.items():
        groups = command_groups(lines, command)
        assert len(groups) == 1, (name, len(groups))
        counts = test_counts(groups[0][1])
        assert counts and all(failed == 0 for _, failed, _ in counts)
        assert not re.search(r"test result: FAILED|error: test failed", groups[0][1])
        if name == "bigdecimal":
            assert counts == [(2, 0, 0)]
        if name in ("nested", "returned", "jdk25_instruction_boundaries"):
            assert sum(row[0] for row in counts) == 1
        ignored_results[name] = {"command": command, "records": len(counts),
                                 "passed": sum(row[0] for row in counts),
                                 "failed": sum(row[1] for row in counts),
                                 "ignored": sum(row[2] for row in counts)}

    supply_gzip_path = CI_EVIDENCE / "ci-supply-job-v1.log.gz"
    supply_gzip = read(supply_gzip_path)
    supply_raw = gzip.decompress(supply_gzip)
    supply_lines = strip_terminal(supply_raw).splitlines()
    install = command_groups(supply_lines, "cargo install cargo-deny --version 0.20.2 --locked")
    assert len(install) == 1 and f"cargo-deny v{EXPECTED_DENY_VERSION}" in install[0][1]
    supply_commands = []
    for manifest in ("./Cargo.toml", "./fuzz/Cargo.toml"):
        command = (f"cargo deny --all-features --workspace --locked --config ./deny.toml "
                   f"--manifest-path {manifest} check")
        groups = command_groups(supply_lines, command)
        assert len(groups) == 1
        for policy in ("advisories", "bans", "licenses", "sources"):
            assert f"{policy} ok" in groups[0][1], (manifest, policy)
        supply_commands.append(command)

    return {
        "ci_json_sha256": sha(raw_bytes), "stable_gzip_sha256": sha(stable_gzip),
        "stable_raw_sha256": sha(stable_raw), "supply_gzip_sha256": sha(supply_gzip),
        "supply_raw_sha256": sha(supply_raw), "jobs": len(raw["jobs"]),
        "step_count": sum(len(job["steps"]) for job in raw["jobs"]),
        "all_jobs_and_steps_success": True, "installed_jdk": EXPECTED_TEMURIN,
        "workspace_seed_runs": workspace, "workspace_seeds": seeds_found,
        "ignored_comparisons": ignored_results,
        "cargo_deny": {"version": EXPECTED_DENY_VERSION, "commands": supply_commands},
        "workflow_sha256": sha(workflow.encode()),
    }


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: verify-ci-product-root-v7.py PRODUCT_SHA RUN_ID")
    product, run_id = sys.argv[1:]
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(run_id)
    ci = verify_ci(product, run_id)
    candidate = verify_candidate(product)
    fingerprint_registration = verify_a1_fingerprint_registration(product)
    product_source_map = {**json.loads(read(METADATA_PATH))["candidate_sources"],
                          **json.loads(read(METADATA_PATH))["test_sources"]}
    local = verify_local_gates(product, product_source_map)
    enum_regression = next(item for item in local if item["name"] == "root-ordinary-static-test-v1")
    result = {
        "schema": "preserve-nonfinal-static-initializer-ci-product-root-verification-v7",
        "status": "accepted", "product_commit": product, "run_id": run_id,
        "ci": ci, "candidate": candidate, "ordinary_enum_regression": enum_regression,
        "a1_static_regression": next(item for item in local if item["name"] == "root-a1-static-tests-v1"),
        "a1_fingerprint_registration": fingerprint_registration,
        "local_gates": local,
        "local_validation_scope": {
            "no_new_rust_gate_added": True,
            "cargo_below_20_gib_run": False,
            "rust_corpus_generator_executed": False,
            "corpus_fingerprint_verification": "independent Python BLAKE3 full-scope scan only",
        },
        "current_workspace_scope": "Immutable Git product/test pins and dedicated frozen CLI are required to match the submitted product; current WIP source hashes are reported separately.",
        "open_spec_status": "Exact submitted product CI accepted; task-state and documentation update remain root work.",
    }
    output = CI_EVIDENCE / "acceptance-v7.json"
    if output.exists():
        raise SystemExit(f"refusing to overwrite {output}")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "product_commit": product, "run_id": run_id,
                      "jobs": ci["jobs"], "candidate_cases": candidate["candidate_case_count"],
                      "local_gates": len(local)}))


if __name__ == "__main__":
    main()
