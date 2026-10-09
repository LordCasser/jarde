"""Accept the exact frozen product's completed CI, never a parent/docs-only run."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
PRODUCT = "45b4848c558f4f5d317720535cea32fe431288bf"
RUN = 37981309004


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    raw = HERE / "ci-run-v1.json"
    run = json.loads(raw.read_text())
    assert run["headSha"] == PRODUCT
    assert run["status"] == "completed" and run["conclusion"] == "success"
    expected_jobs = {"stable / test and specification", "MSRV 1.88.0", "supply chain", "fuzz smoke"}
    assert {job["name"] for job in run["jobs"]} == expected_jobs
    for job in run["jobs"]:
        assert job["status"] == "completed" and job["conclusion"] == "success"
        assert all(step["status"] == "completed" and step["conclusion"] == "success" for step in job["steps"])
    stable = next(job for job in run["jobs"] if job["name"] == "stable / test and specification")
    required_steps = {
        "Check formatting", "Run Clippy", "Run ignored JDK 25 instruction-boundary oracle",
        "Run the ignored P3 Java 8 compile-and-execute comparison", "Run the functional-constructor full-class comparison",
        "Validate OpenSpec strictly", "Ensure tracked files remain unchanged",
    }
    seed_steps = [f"Run workspace tests with two fixed independent seeds (pass {i})" for i in (1, 2)]
    assert required_steps.union(seed_steps).issubset({step["name"] for step in stable["steps"]})
    metadata = json.loads((HERE / "candidate-cli-v1.json").read_text())
    sources = {}
    for path, expected in metadata["candidate_sources"].items():
        blob = subprocess.check_output(["git", "show", f"{PRODUCT}:{path}"], cwd=ROOT)
        assert sha(blob) == expected == sha((ROOT / path).read_bytes())
        sources[path] = expected
    assert sha(Path(metadata["cli_path"]).read_bytes()) == metadata["cli_sha256"]
    workflow = subprocess.check_output(["git", "show", f"{PRODUCT}:.github/workflows/ci.yml"], cwd=ROOT).decode()
    seeds = re.findall(r'PROPTEST_RNG_SEED: "(\d+)"', workflow)
    assert seeds == ["5350648285461741569", "5350648285461741570"]
    assert workflow.count("run: cargo test --workspace --all-targets --all-features --locked") == 2
    compressed = HERE / "ci-stable-job-v1.log.gz"
    log_bytes = gzip.decompress(compressed.read_bytes())
    log = re.sub(r"\x1b\[[0-9;]*m", "", log_bytes.decode())
    # gh labels this runner's log records UNKNOWN STEP; use exact command groups,
    # with distinct seeds and the separately verified completed step identities.
    lines = log.splitlines()
    starts = [i for i, line in enumerate(lines) if "##[group]Run cargo test --workspace --all-targets --all-features --locked" in line]
    assert len(starts) == 2
    seed_results = []
    for start, step, seed in zip(starts, seed_steps, seeds):
        end = next((i for i in range(start + 1, len(lines)) if "##[group]Run " in lines[i]), len(lines))
        text = "\n".join(lines[start:end])
        assert "cargo test --workspace --all-targets --all-features --locked" in text
        assert seed in text and all(other not in text for other in seeds if other != seed)
        counts = [tuple(map(int, match)) for match in re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
        assert counts and all(failed == 0 for _, failed, _ in counts)
        assert not re.search(r"test result: FAILED|error: test failed", text)
        seed_results.append({"step": step, "seed": seed, "test_result_records": len(counts),
                             "passed": sum(row[0] for row in counts), "failed": sum(row[1] for row in counts), "ignored": sum(row[2] for row in counts)})
    result = {
        "product_commit": PRODUCT, "run_id": RUN, "url": run["url"], "status": "accepted",
        "raw_ci_json_sha256": sha(raw.read_bytes()), "stable_raw_log_sha256": sha(log_bytes),
        "stable_gzip_log_sha256": sha(compressed.read_bytes()), "jobs": len(run["jobs"]),
        "steps": sum(len(job["steps"]) for job in run["jobs"]), "all_jobs_and_steps_success": True,
        "seeds": seed_results, "source_identity": sources,
        "frozen_cli_sha256": metadata["cli_sha256"], "workflow_sha256": sha(workflow.encode()),
        "verifier": "verify-ci-product-v2.py",
        "log_partition": "exact Run cargo test workspace command groups, unique declared seeds; gh UNKNOWN STEP labels are not used",
        "scope": "exact product CI plus unchanged frozen CLI sources; no claim about a later docs-only head CI",
    }
    (HERE / "ci-product-root-acceptance-v1.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("status", "product_commit", "jobs", "steps", "seeds")}))


if __name__ == "__main__":
    main()
