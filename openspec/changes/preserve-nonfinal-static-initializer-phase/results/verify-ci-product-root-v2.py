#!/usr/bin/env python3
"""Verify the frozen static-initializer product CI run and its local gates.

Usage: verify-ci-product-root-prepared-v1.py PRODUCT_SHA RUN_ID
This verifier is prepared for root to run after the workflow evidence is captured.
"""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CI_EVIDENCE = HERE / "ci-product-v1"
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


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return Path(path).read_bytes()


def git_blob(product, relative):
    return subprocess.check_output(["git", "show", f"{product}:{relative}"], cwd=ROOT)


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
        for test in STATIC_TESTS + INTERFACE_TESTS:
            assert re.search(r"test " + re.escape(test) + r" \.\.\. ok", text), test
        binaries = {}
        for binary, expected in (("tests/class_static_initializer_projection.rs", 6),
                                 ("tests/interface_initializer_proof.rs", 4),
                                 ("tests/p3_patterns.rs", 85)):
            _, counts = binary_group(text.splitlines(), binary, expected)
            binaries[binary] = {"passed": expected, "failed": 0, "ignored": 0}
        for binary in ("tests/p5_corpus_fingerprint.rs", "tests/p5_bulk_corpus.rs"):
            _, counts = binary_group(text.splitlines(), binary, 5, 1)
            binaries[binary] = {"passed": 5, "failed": 0, "ignored": 1}
        assert "test classfile::tests::repository_class_fixtures_validate_without_false_target_rejections ... ok" in text
        for path in ("crates/jarde-reader/src/classfile.rs", "tests/fixtures/corpus-fingerprint.json"):
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
        raise SystemExit("usage: verify-ci-product-root-prepared-v1.py PRODUCT_SHA RUN_ID")
    product, run_id = sys.argv[1:]
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(run_id)
    ci = verify_ci(product, run_id)
    candidate = verify_candidate(product)
    product_source_map = {**json.loads(read(METADATA_PATH))["candidate_sources"],
                          **json.loads(read(METADATA_PATH))["test_sources"]}
    local = verify_local_gates(product, product_source_map)
    result = {
        "schema": "preserve-nonfinal-static-initializer-ci-product-root-verification-v1",
        "status": "accepted", "product_commit": product, "run_id": run_id,
        "ci": ci, "candidate": candidate, "local_gates": local,
        "current_workspace_scope": "Immutable Git product/test pins and dedicated frozen CLI are required to match the submitted product; current WIP source hashes are reported separately.",
        "open_spec_status": "Exact submitted product CI accepted; task-state and documentation update remain root work.",
    }
    output = HERE / "ci-product-root-verification-v1.json"
    if output.exists():
        raise SystemExit(f"refusing to overwrite {output}")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "product_commit": product, "run_id": run_id,
                      "jobs": ci["jobs"], "candidate_cases": candidate["candidate_case_count"],
                      "local_gates": len(local)}))


if __name__ == "__main__":
    main()
