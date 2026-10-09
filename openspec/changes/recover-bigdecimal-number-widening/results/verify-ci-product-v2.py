#!/usr/bin/env python3
"""Independently verify the exact BigDecimal product CI record and ignored JDK test."""
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
    "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs",
    "src/class_source.rs",
    "Cargo.lock",
}
TEST_SOURCES = {"tests/p3_bigdecimal_number_widening.rs", ".github/workflows/ci.yml"}
EXPECTED_JDK = "25.0.4+7.0.LTS"
EXPECTED_JDK_DISTRIBUTION = "temurin"


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
        raise SystemExit("usage: verify-ci-product-v1.py PRODUCT_SHA RUN_ID")
    product, run_id = sys.argv[1:]
    if not re.fullmatch(r"[0-9a-f]{40}", product):
        raise SystemExit("PRODUCT_SHA must be exactly 40 lowercase hexadecimal characters")
    if not re.fullmatch(r"[0-9]+", run_id):
        raise SystemExit("RUN_ID must be a decimal identifier")
    run_id = int(run_id)

    raw_path = HERE / "ci-run-v2.json"
    raw = json.loads(raw_path.read_bytes())
    # Verification is scoped to the immutable commit and its frozen CLI. Current WIP may advance;
    # its separate hashes are recorded and are never accepted by this historical CI.
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
        "Validate OpenSpec strictly", "Ensure tracked files remain unchanged",
    }
    seed_steps = [f"Run workspace tests with two fixed independent seeds (pass {i})" for i in (1, 2)]
    step_names = {step["name"] for step in stable["steps"]}
    assert required_steps.union(seed_steps) <= step_names

    metadata_path = HERE / "candidate-cli-v1.json"
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
            assert sha(frozen_git) == expected, f"{category} immutable CI product source identity changed: {relative}"
            source_identity[relative] = {"category": category, "sha256": expected, "current_sha256": sha(current), "current_matches_ci_product": sha(current) == expected}
    stale_tests = json.loads((HERE / "ci-stale-boundary-root-verification-v1.json").read_bytes())["changed_test_sources"]
    for relative, expected in stale_tests.items():
        assert sha(git_blob(product, relative)) == expected, relative
        source_identity[relative] = {"category": "ci_stale_boundary_test", "sha256": expected, "current_sha256": sha(read_current(relative)), "current_matches_ci_product": sha(read_current(relative)) == expected}
    cli_path = Path(metadata["cli_path"])
    assert cli_path.is_file() and sha(cli_path.read_bytes()) == metadata["cli_sha256"]

    workflow = git_blob(product, ".github/workflows/ci.yml").decode()
    assert sha(workflow.encode()) == metadata["test_sources"][".github/workflows/ci.yml"]
    seeds = re.findall(r'PROPTEST_RNG_SEED: "(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    workspace_command = "cargo test --workspace --all-targets --all-features --locked"
    assert workflow.count(f"run: {workspace_command}") == 2
    assert 'java-version: "25.0.4+7.0.LTS"' in workflow
    assert "distribution: temurin" in workflow

    compressed = HERE / "ci-stable-job-v2.log.gz"
    compressed_bytes = compressed.read_bytes()
    log_bytes = gzip.decompress(compressed_bytes)
    log = re.sub(r"\x1b\[[0-9;]*m", "", log_bytes.decode())
    # GitHub's captured job stream reports UNKNOWN STEP labels; exact command groups provide
    # the stable boundaries while the API record independently verifies named step success.
    assert "distribution: temurin" in log
    assert f"java-version: {EXPECTED_JDK}" in log
    assert f"Java {EXPECTED_JDK} was downloaded" in log
    assert f"Setting Java {EXPECTED_JDK} as the default" in log
    assert "Temurin-Hotspot_jdk/25.0.4-7.0.LTS" in log

    lines = log.splitlines()
    workspace_groups = command_groups(lines, workspace_command)
    assert len(workspace_groups) == 2
    seed_results = []
    for (_, text), step, seed in zip(workspace_groups, seed_steps, seeds):
        assert workspace_command in text
        assert seed in text and all(other not in text for other in seeds if other != seed)
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
    _, bigdecimal_text = bigdecimal_groups[0]
    bigdecimal_counts = test_counts(bigdecimal_text)
    assert bigdecimal_counts == [(2, 0, 0)]
    assert not re.search(r"test result: FAILED|error: test failed", bigdecimal_text)

    result = {
        "schema": "bigdecimal-ci-product-root-acceptance-v2",
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
        "candidate_cli": {"path": str(cli_path), "sha256": metadata["cli_sha256"],
                          "metadata_sha256": sha(metadata_bytes)},
        "workflow_sha256": sha(workflow.encode()),
        "verifier": "verify-ci-product-v2.py",
        "scope": "Only the immutable product commit and matching frozen CLI are accepted. Different current-source hashes are recorded but not validated by this historical CI.",
        "log_partition": "two exact workspace command groups with distinct declared seeds plus one exact BigDecimal ignored-test group; API step names/status checked separately",
    }
    output = HERE / "ci-product-root-acceptance-v2.json"
    if output.exists():
        raise SystemExit(f"refusing to overwrite {output}")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("status", "product_commit", "run_id", "jobs", "steps",
                                                   "workspace_seed_runs", "bigdecimal_ignored_test")}))


if __name__ == "__main__":
    main()
