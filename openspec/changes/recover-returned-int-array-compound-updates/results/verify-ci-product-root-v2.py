#!/usr/bin/env python3
"""Independently verify immutable CI for returned int-array compound updates."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
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


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: verify-ci-product-root-v2.py PRODUCT_SHA RUN_ID")
    product, run_id = sys.argv[1:]
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(run_id)

    raw_path = HERE / "ci-run-v1.json"
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

    metadata_path = HERE / "candidate-cli-v2.json"
    metadata_bytes = metadata_path.read_bytes()
    metadata = json.loads(metadata_bytes)
    assert set(metadata["candidate_sources"]) == PRODUCT_SOURCES
    assert set(metadata["test_sources"]) == TEST_SOURCES
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
    cli_path = Path(metadata["cli_path"])
    assert cli_path.is_file() and sha(cli_path.read_bytes()) == metadata["cli_sha256"]

    workflow = git_blob(product, ".github/workflows/ci.yml").decode()
    assert sha(workflow.encode()) == metadata["test_sources"][".github/workflows/ci.yml"]
    seeds = re.findall(r'PROPTEST_RNG_SEED: "(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    assert workflow.count(f"run: {WORKSPACE_COMMAND}") == 2
    assert 'java-version: "25.0.4+7.0.LTS"' in workflow
    assert "distribution: temurin" in workflow

    compressed = HERE / "ci-stable-job-v1.log.gz"
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
        for name in (*NESTED_ORDINARY_TESTS, *RETURNED_ORDINARY_TESTS):
            assert re.search(r"test " + re.escape(name) + r" \.\.\. ok", text), name
        counts = test_counts(text)
        assert counts and all(failed == 0 for _, failed, _ in counts)
        assert not re.search(r"test result: FAILED|error: test failed", text)
        seed_results.append({"step": step, "seed": seed, "test_result_records": len(counts),
                             "passed": sum(row[0] for row in counts),
                             "failed": sum(row[1] for row in counts),
                             "ignored": sum(row[2] for row in counts)})

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

    supply_path = HERE / "ci-supply-job-v1.log.gz"
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

    result = {
        "schema": "returned-int-array-ci-product-root-acceptance-v1",
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
        "immutable_current_sources": immutable_sources,
        "canonical_input_files": canonical_files,
        "nested_ignored_test": {"command": NESTED_IGNORED_COMMAND, "passed": 1, "failed": 0, "ignored": 0},
        "returned_ignored_test": {"command": RETURNED_IGNORED_COMMAND, "passed": 1, "failed": 0, "ignored": 0},
        "supply_cli": {"version": "0.20.2", "commands": supply_checks,
                       "raw_log_sha256": sha(supply_raw), "gzip_log_sha256": sha(supply_compressed)},
        "candidate_cli": {"path": str(cli_path), "sha256": metadata["cli_sha256"],
                          "metadata_sha256": sha(metadata_bytes)},
        "workflow_sha256": sha(workflow.encode()),
        "verifier": "verify-ci-product-root-v2.py",
        "scope": "Only the immutable product commit and matching frozen CLI are accepted. Current-source hashes are recorded separately.",
        "log_partition": "two exact workspace command groups with distinct declared seeds, one BigDecimal ignored-test group, and exact nested/returned ignored-test groups; API step names/status checked separately",
    }
    output = HERE / "ci-product-root-acceptance-v2.json"
    if output.exists():
        raise SystemExit(f"refusing to overwrite {output}")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("status", "product_commit", "run_id", "jobs", "steps",
                                                   "workspace_seed_runs", "bigdecimal_ignored_test",
                                                   "nested_ignored_test", "returned_ignored_test")}))


if __name__ == "__main__":
    main()
