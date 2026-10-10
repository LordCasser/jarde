#!/usr/bin/env python3
"""Root-only guarded validation and candidate CLI freeze for proved if-arm join origins."""

from __future__ import annotations

import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
TASK = Path("/private/tmp/jarde-proved-if-join-validation-v1")
RESULTS = ROOT / "openspec/changes/preserve-proved-if-arm-join-origins/results"
OUT = RESULTS / "validation-build-root-v1"
CLI_PATH = Path("/private/tmp/jarde-proved-if-join-cli-v1")
METADATA_PATH = RESULTS / "candidate-cli-v1.json"
SOURCE_COMMIT_BASE: str | None = None
TEMPLATE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
TEMPLATE_SHA256 = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
EXPECTED = {
    4: [(4, 0, 0)],
    5: [(7, 0, 0)],
    6: [(5, 0, 0)],
    7: [(12, 0, 0)],
    8: [(178, 0, 0)],
    9: [(5, 0, 1)],
    10: [(3, 0, 0)],
}
SUMMARY_RE = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored")
PRIOR_CI_ACCEPTANCE = RESULTS.parents[1] / "preserve-proved-for-latch-origins/results/ci-verification-invocation-root-v2/stdout.raw"
PRIOR_CI_ACCEPTANCE_SHA256 = "cc1eb09c28df54bd2b8974d5b764c45fd778239dfa844afbe191827616b72530"
PRIOR_CI_EXPECTED = {"status": "accepted", "product_commit": "d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c",
                    "run_id": 38062706229, "jobs": 4, "steps": 52,
                    "proved_for_latch_tests_per_seed": 9, "prior_integer_tests_per_seed": 11}


def sha_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def load_runner():
    if sha_file(TEMPLATE) != TEMPLATE_SHA256:
        raise RuntimeError(f"guarded runner template SHA-256 mismatch: {TEMPLATE}")
    spec = importlib.util.spec_from_file_location("loop_validation_v9", TEMPLATE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load guarded runner template: {TEMPLATE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


REQUIRED_LIB_TESTS: tuple[str, ...] = ()
REQUIRED_GATEWAY_TESTS: tuple[str, ...] = ()


def summary_check(index: int, stdout: bytes) -> dict:
    text = stdout.decode("utf-8", errors="replace")
    actual = [tuple(map(int, found)) for found in SUMMARY_RE.findall(text)]
    expected = EXPECTED.get(index)
    if index == 2:
        expected = [(EXPECTED_LIB_COUNT, 0, 0)]
    elif index == 3:
        expected = [(EXPECTED_GATEWAY_COUNT, 0, 0)]
    required = index in range(2, 11)
    nonempty_success = bool(actual) and all(failed == 0 for _, failed, _ in actual)
    has_tests = any(passed + failed + ignored > 0 for passed, failed, ignored in actual)
    ok = (expected is None or actual == expected) and (not required or (nonempty_success and has_tests))
    required_names = REQUIRED_LIB_TESTS if index == 2 else REQUIRED_GATEWAY_TESTS if index == 3 else ()
    names_ok = all(re.search(r"(?m)^test " + re.escape(name) + r" \.\.\. ok$", text) for name in required_names)
    ok = ok and names_ok
    reason = None
    if expected is not None and actual != expected:
        reason = f"expected summaries {expected}, got {actual}"
    elif required and (not nonempty_success or not has_tests):
        reason = "required successful non-empty test summary was absent or failed"
    return {"expected": expected, "actual": actual, "required": required,
            "required_test_names": list(required_names), "required_test_names_present": names_ok, "ok": ok, "failure_reason": reason}


EXPECTED_LIB_COUNT = 0
EXPECTED_GATEWAY_COUNT = 0


def main() -> int:
    global SOURCE_COMMIT_BASE, EXPECTED_LIB_COUNT, EXPECTED_GATEWAY_COUNT
    global REQUIRED_LIB_TESTS, REQUIRED_GATEWAY_TESTS
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-base", required=True)
    parser.add_argument("--expected-lib-count", required=True, type=int)
    parser.add_argument("--expected-gateway-count", required=True, type=int)
    parser.add_argument("--required-lib-test", action="append", required=True)
    parser.add_argument("--required-gateway-test", action="append", required=True)
    args = parser.parse_args()
    if len(args.source_base) != 40 or any(c not in "0123456789abcdef" for c in args.source_base):
        raise ValueError("--source-base must be a lowercase 40-hex commit")
    if args.expected_lib_count <= 0 or args.expected_gateway_count <= 0:
        raise ValueError("expected test counts must be positive")
    SOURCE_COMMIT_BASE = args.source_base
    EXPECTED_LIB_COUNT = args.expected_lib_count
    EXPECTED_GATEWAY_COUNT = args.expected_gateway_count
    REQUIRED_LIB_TESTS = tuple(args.required_lib_test)
    REQUIRED_GATEWAY_TESTS = tuple(args.required_gateway_test)
    runner = load_runner()
    # Reuse the already exercised guard, JDK identity, source-pin, and cleanup code.
    runner.ROOT = ROOT
    runner.HERE = TASK
    runner.RESULTS = RESULTS
    runner.OUT = OUT
    runner.CLI_PATH = CLI_PATH
    runner.METADATA_PATH = METADATA_PATH
    runner.SOURCE_COMMIT_BASE = SOURCE_COMMIT_BASE
    runner.EXPECTED_SUMMARIES = EXPECTED
    runner.TEST_COMMANDS = set(range(2, 11))
    runner.expected_test_summaries = summary_check
    runner.command_stream = lambda path: {
        "path": str(path), "bytes": path.stat().st_size, "sha256": sha_file(path)
    }
    runner.PRODUCT_PATHS = {
        "Cargo.toml", "Cargo.lock", "crates/jarde-jvm/Cargo.toml",
        "crates/jarde-reader/Cargo.toml", "crates/jarde-query/Cargo.toml",
        "crates/jarde-java/Cargo.toml", "crates/jarde-cli/Cargo.toml",
        "crates/jarde-java/src/region.rs", "crates/jarde-java/src/build.rs",
        "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/report.rs",
        "crates/jarde-java/src/lib.rs", "src/class_source.rs", "src/facade.rs",
        "src/lib.rs", "crates/jarde-cli/src/main.rs", "crates/jarde-cli/src/task.rs",
    }
    runner.TEST_PATHS = {
        ".github/workflows/ci.yml", "crates/jarde-java/tests/p3_loop_exit_gateways.rs",
        "crates/jarde-java/tests/p3_loop_body_double_jumps.rs",
        "crates/jarde-java/tests/p3_loop_terminal_return.rs",
        "crates/jarde-java/tests/p3_effectful_exits.rs", "tests/p3_loop_arm_join.rs",
        "tests/p3_loop_boolean_exit.rs", "tests/p5_corpus_fingerprint.rs",
    }
    canonical: set[str] = set()
    for relative in sorted(runner.TEST_PATHS):
        if relative.endswith(".rs"):
            source = (ROOT / relative).read_text(encoding="utf-8")
            for included in runner.INCLUDE_RE.findall(source):
                canonical.add((ROOT / relative).parent.joinpath(included).resolve()
                              .relative_to(ROOT).as_posix())
    canonical.add("openspec/changes/preserve-proved-if-arm-join-origins/results/exception-join-case-root-v1/classes/ifjoin/ExceptionIfJoin.class")
    runner.CANONICAL_PATHS = canonical
    runner.COMMANDS = [
        ["cargo", "fmt", "--all", "--", "--check"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--",
         *sum((["-A", f"clippy::{name}"] for name in runner.CI_CLIPPY_ALLOW_LINTS), []), "-D", "warnings"],
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
    if len(runner.COMMANDS) != 12:
        raise RuntimeError(f"expected exactly 12 validation commands, got {len(runner.COMMANDS)}")

    if not ROOT.is_dir():
        raise RuntimeError(f"workspace root is unavailable: {ROOT}")
    if OUT.exists() or CLI_PATH.exists() or METADATA_PATH.exists():
        raise FileExistsError("refusing to overwrite validation output, candidate CLI, or metadata")
    if shutil.disk_usage(ROOT).free < runner.FREE_LIMIT:
        raise RuntimeError("5 GiB free-space guard failed before output creation")
    if runner.target_bytes() > runner.TARGET_LIMIT:
        raise RuntimeError("1 GiB target-size guard failed before output creation")
    OUT.mkdir(parents=True)

    prior_ci_raw = PRIOR_CI_ACCEPTANCE.read_bytes()
    if sha_file(PRIOR_CI_ACCEPTANCE) != PRIOR_CI_ACCEPTANCE_SHA256:
        raise RuntimeError("prior d714/38062706229 accepted-v2 artifact SHA mismatch")
    prior_ci = json.loads(prior_ci_raw)
    if prior_ci != PRIOR_CI_EXPECTED:
        raise RuntimeError("prior d714/38062706229 accepted-v2 artifact contents mismatch")
    rows = []
    pins_before = {"candidate_sources": runner.source_pins(runner.PRODUCT_PATHS),
                   "test_sources": runner.source_pins(runner.TEST_PATHS),
                   "canonical_files": runner.source_pins(runner.CANONICAL_PATHS)}
    preflight = {"git_head": None, "source_pins_before": pins_before, "source_pins_after": None,
                 "uncommitted_if_arm_join_product": True,
                 "prior_accepted_ci": {"path": str(PRIOR_CI_ACCEPTANCE),
                                       "sha256": PRIOR_CI_ACCEPTANCE_SHA256,
                                       "record": prior_ci}}
    env = os.environ.copy()
    stripped = []
    for key in runner.STRIPPED_ENV_KEYS:
        if key in env:
            env.pop(key)
            stripped.append(key)
    env.update(runner.ENV_VALUES)
    try:
        preflight["stripped_environment_keys"] = stripped
        preflight["jdk23"] = runner.configure_jdk23(env)
        head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, check=False)
        head_out, head_err = OUT / "git-head.stdout.raw", OUT / "git-head.stderr.raw"
        head_out.write_bytes(head.stdout)
        head_err.write_bytes(head.stderr)
        value = head.stdout.decode("ascii", errors="replace").strip()
        preflight["git_head"] = {"argv": ["git", "rev-parse", "HEAD"], "exit_code": head.returncode,
                                  "value": value, "matches_expected": head.returncode == 0 and value == SOURCE_COMMIT_BASE,
                                  "stdout": runner.command_stream(head_out), "stderr": runner.command_stream(head_err)}
        if not preflight["git_head"]["matches_expected"]:
            raise RuntimeError("HEAD does not exactly match the caller-supplied product base")

        cleanup = []
        runner.clean_check_only_metadata(cleanup)
        preflight["check_only_metadata_cleanup"] = cleanup
        write_execution(runner, rows, preflight, "running")
        if shutil.disk_usage(ROOT).free < runner.FREE_LIMIT or runner.target_bytes() > runner.TARGET_LIMIT:
            raise RuntimeError("disk guard failed after check-only metadata cleanup")

        for index, argv in enumerate(runner.COMMANDS):
            row = runner.run_command(index, argv, env)
            rows.append(row)
            write_execution(runner, rows, preflight, "running")
            print(json.dumps(row, ensure_ascii=False), flush=True)
            is_test = argv[:2] == ["cargo", "test"]
            if row["exit_code"] != 0 or row["guard_stop"] is not None or (is_test and not row["test_summary_check"]["ok"]):
                raise RuntimeError(f"validation command {index} failed exit, summary, or disk guard")
            row["cleanup"] = {"test_binaries": [], "check_only_metadata": []}
            if index == 1:
                runner.clean_check_only_metadata(row["cleanup"]["check_only_metadata"])
            if is_test:
                runner.clean_successful_test_binary(index, row["cleanup"]["test_binaries"])
            write_execution(runner, rows, preflight, "running")

        pins_after = {"candidate_sources": runner.source_pins(runner.PRODUCT_PATHS),
                      "test_sources": runner.source_pins(runner.TEST_PATHS),
                      "canonical_files": runner.source_pins(runner.CANONICAL_PATHS)}
        preflight["source_pins_after"] = pins_after
        if pins_after != pins_before:
            raise RuntimeError("source pins changed during validation")
        binary = ROOT / "target/debug/jarde-cli"
        if not binary.is_file():
            raise FileNotFoundError(f"successful build did not produce {binary}")
        CLI_PATH.parent.mkdir(parents=True, exist_ok=True)
        temporary = CLI_PATH.with_name(f".{CLI_PATH.name}.tmp-{os.getpid()}")
        shutil.copyfile(binary, temporary)
        os.chmod(temporary, 0o555)
        os.replace(temporary, CLI_PATH)
        own_runner = {"path": str(Path(__file__).resolve()), "sha256": sha_file(Path(__file__).resolve())}
        guarded_template = {"path": str(TEMPLATE), "sha256": TEMPLATE_SHA256}
        freeze = {"cli_path": str(CLI_PATH), "cli_sha256": sha_file(CLI_PATH),
                  "cli_mode": oct(stat.S_IMODE(CLI_PATH.stat().st_mode)),
                  "metadata_path": str(METADATA_PATH), "source_commit_base": SOURCE_COMMIT_BASE,
                  "uncommitted_if_arm_join_product": True,
                  "validation_runner": own_runner, "guarded_runner_template": guarded_template,
                  "product_path_sets": {"candidate_sources": sorted(runner.PRODUCT_PATHS),
                                        "test_sources": sorted(runner.TEST_PATHS),
                                        "canonical_files": sorted(runner.CANONICAL_PATHS)}}
        write_execution(runner, rows, preflight, "validation-passed-cli-frozen", freeze)
        metadata = {"schema": "preserve-proved-if-arm-join-origins-candidate-cli-v1",
                    **freeze, "candidate_sources": pins_after["candidate_sources"],
                    "test_sources": pins_after["test_sources"], "canonical_files": pins_after["canonical_files"],
                    "build_result_sha256": sha_file(OUT / "execution.json")}
        METADATA_PATH.parent.mkdir(parents=True, exist_ok=True)
        with METADATA_PATH.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(metadata, ensure_ascii=False, indent=2) + "\n")
        return 0
    except Exception as error:
        write_execution(runner, rows, {**preflight, "failure": f"{type(error).__name__}: {error}"}, "failed")
        print(f"validation stopped: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


def write_execution(runner, rows: list[dict], preflight: dict, status: str,
                    freeze: dict | None = None) -> None:
    payload = {
        "schema": "preserve-proved-if-arm-join-origins-validation-build-root-v1",
        "validation_runner": {"path": str(Path(__file__).resolve()), "sha256": sha_file(Path(__file__).resolve())},
        "guarded_runner_template": {"path": str(TEMPLATE), "sha256": TEMPLATE_SHA256},
        "status": status, "source_commit_base_expected": SOURCE_COMMIT_BASE,
        "uncommitted_if_arm_join_product": True, "environment_overrides": runner.ENV_VALUES,
        "guards": {"minimum_free_bytes": runner.FREE_LIMIT, "maximum_target_bytes": runner.TARGET_LIMIT},
        "preflight": preflight, "commands": rows, "freeze": freeze,
    }
    runner.write_json(OUT / "execution.json", payload)


if __name__ == "__main__":
    raise SystemExit(main())
