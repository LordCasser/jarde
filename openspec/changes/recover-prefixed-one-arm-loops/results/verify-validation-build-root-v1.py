#!/usr/bin/env python3
"""Independently accept the exact frozen prefixed-loop validation build."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import stat


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
RESULTS = ROOT / "openspec/changes/recover-prefixed-one-arm-loops/results"
BUILD_DIR = RESULTS / "validation-build-root-v5"
EXECUTION = BUILD_DIR / "execution.json"
METADATA_PATH = RESULTS / "candidate-cli-v1.json"
ACCEPTANCE = RESULTS / "validation-acceptance-root-v1.json"
CLI_PATH = Path("/private/tmp/jarde-prefixed-loop-cli-v1")
BASE = "404b422e141e200f0eea000f64937becedbad664"
TEMPLATE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
TEMPLATE_SHA = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
RUNNER = RESULTS / "run-validation-build-root-v5.py"
SUMMARY_RE = re.compile(rb"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored")
INCLUDE_RE = re.compile(r"(?:include_bytes|include_str)!\s*\(\s*\"([^\"]+)\"\s*,?\s*\)")
PRODUCT = {
    "Cargo.toml", "Cargo.lock", "crates/jarde-jvm/Cargo.toml", "crates/jarde-reader/Cargo.toml",
    "crates/jarde-query/Cargo.toml", "crates/jarde-java/Cargo.toml", "crates/jarde-cli/Cargo.toml",
    "crates/jarde-java/src/region.rs", "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/report.rs", "crates/jarde-java/src/lib.rs", "src/class_source.rs",
    "src/facade.rs", "src/lib.rs", "crates/jarde-cli/src/main.rs", "crates/jarde-cli/src/task.rs",
}
TESTS = {
    ".github/workflows/ci.yml", "crates/jarde-java/tests/p3_loop_exit_gateways.rs",
    "crates/jarde-java/tests/p3_loop_body_double_jumps.rs", "crates/jarde-java/tests/p3_loop_terminal_return.rs",
    "crates/jarde-java/tests/p3_effectful_exits.rs", "tests/p3_loop_arm_join.rs",
    "tests/p3_loop_boolean_exit.rs", "tests/p5_corpus_fingerprint.rs",
}
EXPECTED = {2: [(335, 0, 0)], 3: [(7, 0, 0)], 4: [(4, 0, 0)], 5: [(7, 0, 0)],
            6: [(5, 0, 0)], 7: [(12, 0, 0)], 8: [(178, 0, 0)],
            9: [(5, 0, 1)], 10: [(3, 0, 0)]}
NEW_TESTS = (
    "region::tests::prefixed_loop_chain_certificate_rejects_extra_entries_exits_and_owners",
    "region::tests::prefixed_loop_header_cannot_reopen_claimed_outer_target_or_parent_scope",
    "one_arm_straight_prefix_loop_and_optional_tail_are_closed",
    "prefixed_one_arm_loop_budget_and_cancellation_publish_no_partial_source",
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read(path: Path) -> bytes:
    return path.read_bytes()


def pins(paths: set[str]) -> dict[str, str]:
    return {p: sha(read(ROOT / p)) for p in sorted(paths)}


def main() -> None:
    assert not ACCEPTANCE.exists(), f"refusing to overwrite {ACCEPTANCE}"
    assert sha(read(TEMPLATE)) == TEMPLATE_SHA, "pinned v9 constants/template changed"
    spec = importlib.util.spec_from_file_location("pinned_loop_validation_v9", TEMPLATE)
    assert spec and spec.loader
    v9 = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(v9)
    data = read(EXECUTION)
    execution = json.loads(data)
    assert execution["schema"] == "recover-prefixed-one-arm-loops-validation-build-root-v5"
    assert execution["status"] == "validation-passed-cli-frozen"
    assert execution["source_commit_base_expected"] == BASE
    assert execution["uncommitted_prefixed_loop_product"] is True
    assert execution["guards"] == {"minimum_free_bytes": 5 * 1024**3, "maximum_target_bytes": 1024**3}
    assert execution["environment_overrides"] == v9.ENV_VALUES
    assert execution["guarded_runner_template"] == {"path": str(TEMPLATE), "sha256": TEMPLATE_SHA}
    assert len(v9.CI_CLIPPY_ALLOW_LINTS) == 29
    assert execution["validation_runner"]["path"] == str(RUNNER.resolve())
    assert sha(read(RUNNER)) == execution["validation_runner"]["sha256"]
    head = execution["preflight"]["git_head"]
    assert head["argv"] == ["git", "rev-parse", "HEAD"] and head["exit_code"] == 0
    assert head["value"] == BASE and head["matches_expected"] is True
    for stream in (head["stdout"], head["stderr"]):
        path = Path(stream["path"])
        path = path if path.is_absolute() else ROOT / path
        path.resolve().relative_to(ROOT.resolve())
        raw = read(path)
        assert len(raw) == stream["bytes"] and sha(raw) == stream["sha256"]
    head_stdout = Path(head["stdout"]["path"])
    head_stdout = head_stdout if head_stdout.is_absolute() else ROOT / head_stdout
    assert read(head_stdout).decode("ascii").strip() == BASE

    canonical = set()
    for test in TESTS:
        if test.endswith(".rs"):
            source = read(ROOT / test).decode("utf-8")
            canonical.update((ROOT / test).parent.joinpath(p).resolve().relative_to(ROOT).as_posix()
                             for p in INCLUDE_RE.findall(source))
    assert (len(PRODUCT), len(TESTS), len(canonical)) == (17, 8, 25)
    assert len(PRODUCT | TESTS | canonical) == 50
    live = {"candidate_sources": pins(PRODUCT), "test_sources": pins(TESTS), "canonical_files": pins(canonical)}
    before = execution["preflight"]["source_pins_before"]
    after = execution["preflight"]["source_pins_after"]
    assert before == after == live

    metadata_bytes = read(METADATA_PATH)
    metadata = json.loads(metadata_bytes)
    freeze = execution["freeze"]
    cli = read(CLI_PATH)
    assert metadata["schema"] == "recover-prefixed-one-arm-loops-candidate-cli-v1"
    assert metadata["cli_path"] == freeze["cli_path"] == str(CLI_PATH)
    assert metadata["cli_sha256"] == freeze["cli_sha256"] == sha(cli)
    assert stat.S_IMODE(CLI_PATH.stat().st_mode) == 0o555 and freeze["cli_mode"] == "0o555"
    assert metadata["metadata_path"] == freeze["metadata_path"] == str(METADATA_PATH)
    assert metadata["source_commit_base"] == freeze["source_commit_base"] == BASE
    assert metadata["guarded_runner_template"] == execution["guarded_runner_template"]
    assert metadata["build_result_sha256"] == sha(data)
    assert {k: metadata[k] for k in live} == live
    assert freeze["product_path_sets"] == {"candidate_sources": sorted(PRODUCT),
                                           "test_sources": sorted(TESTS),
                                           "canonical_files": sorted(canonical)}

    commands = execution["commands"]
    assert len(commands) == len(v9.COMMANDS) == 12
    stream_count = 0
    recomputed = {}
    for i, (row, argv) in enumerate(zip(commands, v9.COMMANDS)):
        assert row["index"] == i and row["argv"] == argv
        assert row["cwd"] == str(ROOT)
        assert row["exit_code"] == 0 and row["guard_stop"] is None
        assert row["env_overrides"] == v9.ENV_VALUES
        assert row["peak_target_bytes"] <= 1024**3 and row["free_bytes_after"] >= 5 * 1024**3
        for stream_name, stream in row["streams"].items():
            path = Path(stream["path"])
            path = path if path.is_absolute() else ROOT / path
            path.resolve().relative_to(BUILD_DIR.resolve())
            raw = read(path)
            assert len(raw) == stream["bytes"] and sha(raw) == stream["sha256"]
            if stream_name == "stdout":
                stdout = raw
            stream_count += 1
        if i in EXPECTED:
            actual = [tuple(map(int, m)) for m in SUMMARY_RE.findall(stdout)]
            assert actual == EXPECTED[i], (i, actual)
            check = row["test_summary_check"]
            assert check["required"] and check["ok"]
            if check["expected"] is not None:
                assert [tuple(x) for x in check["expected"]] == EXPECTED[i]
            assert [tuple(x) for x in check["actual"]] == actual
            if i in (2, 3):
                text = stdout.decode("utf-8")
                for name in NEW_TESTS[:2] if i == 2 else NEW_TESTS[2:]:
                    assert re.search(r"(?m)^test " + re.escape(name) + r" \.\.\. ok$", text), name
            recomputed[str(i)] = [list(x) for x in actual]
    assert stream_count == 24
    acceptance = {
        "schema": "recover-prefixed-one-arm-loops-validation-acceptance-root-v1",
        "status": "accepted", "execution": str(EXECUTION), "execution_sha256": sha(data),
        "acceptance_runner": {"path": str(Path(__file__).resolve()), "sha256": sha(read(Path(__file__)))},
        "source_commit_base": BASE, "pin_counts": {"product": 17, "tests": 8, "canonical": 25, "total": 50},
        "candidate_cli": {"path": str(CLI_PATH), "sha256": sha(cli), "mode": "0o555"},
        "metadata_sha256": sha(metadata_bytes), "commands": 12, "raw_streams_recomputed": stream_count,
        "test_results_recomputed_from_raw_stdout": recomputed, "new_tests_present": list(NEW_TESTS),
    }
    with ACCEPTANCE.open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(acceptance, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
