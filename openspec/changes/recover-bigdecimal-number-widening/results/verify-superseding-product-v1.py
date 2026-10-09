#!/usr/bin/env python3
"""Accept BigDecimal in the separately verified composed product, never the failed old run."""
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
NEXT = HERE.parents[1] / "recover-nested-int-array-compound-updates" / "results"
PRODUCT = "561de209531c021a9d8adb7c979bae5a57d6c3fb"

def sha(data):
    return hashlib.sha256(data).hexdigest()

def load(path):
    return json.loads(path.read_bytes())

def blob(path):
    return subprocess.check_output(["git", "show", f"{PRODUCT}:{path}"], cwd=ROOT)

def main():
    ci_path = NEXT / "ci-product-root-acceptance-v1.json"
    ci = load(ci_path)
    assert ci["status"] == "accepted" and ci["product_commit"] == PRODUCT
    assert ci["run_id"] == 37994276707 and ci["all_jobs_and_steps_success"]
    assert ci["jobs"] == 4
    assert len(ci["workspace_seed_runs"]) == 2
    assert all(r["passed"] > 0 and r["failed"] == 0 for r in ci["workspace_seed_runs"])
    assert ci["bigdecimal_ignored_test"]["passed"] == 2
    assert ci["installed_jdk"]["confirmed_from_stable_log"]
    raw_ci = NEXT / "ci-run-v1.json"
    assert sha(raw_ci.read_bytes()) == ci["raw_ci_json_sha256"]
    metadata_path = NEXT / "candidate-cli-v1.json"
    metadata = load(metadata_path)
    assert sha(metadata_path.read_bytes()) == ci["candidate_cli"]["metadata_sha256"]
    assert sha(Path(metadata["cli_path"]).read_bytes()) == metadata["cli_sha256"]
    historical = load(HERE / "candidate-cli-v1.json")
    identities = {}
    for path, expected in metadata["candidate_sources"].items():
        assert sha(blob(path)) == expected
        identities[path] = {"sha256": expected, "matches_historical_bigdecimal_source": historical["candidate_sources"].get(path) == expected}
    assert all(row["matches_historical_bigdecimal_source"] for path, row in identities.items() if path != "crates/jarde-java/src/build.rs")
    assert not identities["crates/jarde-java/src/build.rs"]["matches_historical_bigdecimal_source"]
    test = "tests/p3_bigdecimal_number_widening.rs"
    assert sha(blob(test)) == historical["test_sources"][test] == ci["extra_ci_sources"][test]
    replay_path = NEXT / "legacy24-root-verification-v1.json"
    replay = load(replay_path)
    assert replay["checks"] == 1226 and not replay["errors"] and replay["task_3_1_complete"]
    assert replay["cli_sha256"] == metadata["cli_sha256"]
    assert len(replay["cases"]) == 24 and all(row["success"] for row in replay["cases"])
    positives = [r for r in replay["cases"] if r["family"] == "bigdecimal-control-corrected"]
    assert len(positives) == 2 and all(row["bigdecimal"] for row in positives)
    manifest = NEXT / "legacy24-candidate-v1" / "manifest.json"
    assert sha(manifest.read_bytes()) == replay["manifest_sha256"]
    assert replay["commands"] == 102 and replay["files"] == 384
    old_failures = [load(HERE / f"ci-run-v{i}.json") for i in (1, 2, 3)]
    assert all(r["conclusion"] == "failure" for r in old_failures)
    result = {
        "status": "accepted_in_superseding_composed_product", "product_commit": PRODUCT,
        "run_id": ci["run_id"], "ci_acceptance_sha256": sha(ci_path.read_bytes()),
        "raw_ci_json_sha256": sha(raw_ci.read_bytes()), "source_identity": identities,
        "candidate_cli": ci["candidate_cli"], "bigdecimal_test_sha256": sha(blob(test)),
        "fresh_candidate_replay": {"success": "24/24", "checks": 1226, "commands": 102, "files": 384,
            "verifier_record_sha256": sha(replay_path.read_bytes()), "manifest_sha256": replay["manifest_sha256"]},
        "workspace_seed_runs": ci["workspace_seed_runs"], "installed_jdk": ci["installed_jdk"],
        "bigdecimal_ignored_test": ci["bigdecimal_ignored_test"],
        "old_runs_remain_failures": True,
        "scope": "Compose prior local/semantic acceptance with the independently accepted exact 561 product CI and its frozen CLI replay. No acceptance of failed 6997/8cd CI; no fresh original/JADX execution claimed for legacy controls.",
        "verifier_sha256": sha(Path(__file__).read_bytes()),
    }
    output = HERE / "superseding-product-root-acceptance-v1.json"
    if output.exists():
        raise SystemExit(f"refusing to overwrite {output}")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("status", "product_commit", "run_id")}))

if __name__ == "__main__":
    main()
