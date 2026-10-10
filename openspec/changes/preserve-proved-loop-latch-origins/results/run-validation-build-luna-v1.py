#!/usr/bin/env python3
"""Run guarded loop-latch validation and freeze its candidate CLI v1."""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import stat
import subprocess
import sys
import time


HERE = Path(__file__).resolve().parent
ROOT = Path(__file__).resolve().parents[4]
RESULTS = HERE
OUT = RESULTS / "validation-build-root-v1"
CLI_PATH = Path("/private/tmp/jarde-loop-latch-cli-v1")
METADATA_PATH = RESULTS / "candidate-cli-v1.json"
SOURCE_COMMIT_BASE: str | None = None
TARGET_LIMIT = 1024**3
FREE_LIMIT = 20 * 1024**3
ENV_VALUES = {
    "CARGO_BUILD_JOBS": "1",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "RUST_TEST_THREADS": "1",
    "CARGO_TERM_COLOR": "always",
}

# These files cover the proof, origin fold, report/emitter, and CLI-facing recovery path.
PRODUCT_PATHS = {
    "Cargo.toml",
    "Cargo.lock",
    "crates/jarde-jvm/Cargo.toml",
    "crates/jarde-reader/Cargo.toml",
    "crates/jarde-query/Cargo.toml",
    "crates/jarde-java/Cargo.toml",
    "crates/jarde-cli/Cargo.toml",
    "crates/jarde-java/src/region.rs",
    "crates/jarde-java/src/build.rs",
    "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/lib.rs",
    "src/class_source.rs",
    "src/facade.rs",
    "src/lib.rs",
    "crates/jarde-cli/src/main.rs",
    "crates/jarde-cli/src/task.rs",
}
TEST_PATHS = {
    ".github/workflows/ci.yml",
    "crates/jarde-java/tests/p3_loop_exit_gateways.rs",
    "crates/jarde-java/tests/p3_loop_body_double_jumps.rs",
    "crates/jarde-java/tests/p3_loop_terminal_return.rs",
    "crates/jarde-java/tests/p3_effectful_exits.rs",
    "tests/p3_loop_arm_join.rs",
    "tests/p5_corpus_fingerprint.rs",
}

# Every literal include_bytes!/include_str! in the focused tests is a canonical input.
INCLUDE_RE = re.compile(r"(?:include_bytes|include_str)!\s*\(\s*\"([^\"]+)\"\s*,?\s*\)")
CANONICAL_PATHS: set[str] = set()
for test_source in sorted(TEST_PATHS):
    if test_source.endswith(".rs"):
        text = (ROOT / test_source).read_text(encoding="utf-8")
        for included in INCLUDE_RE.findall(text):
            resolved = (ROOT / test_source).parent.joinpath(included).resolve()
            CANONICAL_PATHS.add(resolved.relative_to(ROOT).as_posix())

CI_CLIPPY_ALLOW_LINTS = (
    "too_many_arguments", "cloned_ref_to_slice_refs", "collapsible_if", "type_complexity",
    "len_zero", "needless_option_as_deref", "needless_borrow", "useless_conversion",
    "large_enum_variant", "question_mark", "comparison_to_empty", "op_ref",
    "manual_range_patterns", "if_same_then_else", "filter_map_bool_then", "filter_next",
    "unneeded_struct_pattern", "redundant_guards", "map_identity", "redundant_slicing",
    "unnecessary_get_then_check", "unnecessary_unwrap", "redundant_locals", "replace_box",
    "map_clone", "unnecessary_mut_passed", "single_element_loop", "unnecessary_to_owned",
    "needless_lifetimes",
)

COMMANDS = [
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--",
     *sum((["-A", f"clippy::{name}"] for name in CI_CLIPPY_ALLOW_LINTS), []), "-D", "warnings"],
    ["cargo", "test", "-p", "jarde-java", "--lib", "--locked"],
    ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_exit_gateways", "--locked", "--", "--nocapture"],
    ["cargo", "test", "-p", "jarde", "--test", "p3_loop_arm_join", "--locked", "--", "--nocapture"],
    ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_body_double_jumps", "--locked"],
    ["cargo", "test", "-p", "jarde-java", "--test", "p3_loop_terminal_return", "--locked"],
    ["cargo", "test", "-p", "jarde-java", "--test", "p3_effectful_exits", "--locked"],
    ["cargo", "test", "-p", "jarde-reader", "--lib", "--locked"],
    ["cargo", "test", "-p", "jarde", "--test", "p5_corpus_fingerprint", "--locked"],
    ["cargo", "build", "-p", "jarde-cli", "--locked"],
]
assert len(COMMANDS) == 11

EXPECTED_SUMMARIES = {
    3: [(5, 0, 0)],   # 3 accepted gateway tests + 2 exact noPrefix latch tests from the patch
    4: [(4, 0, 0)],   # root-level p3_loop_arm_join target
    5: [(7, 0, 0)],
    6: [(5, 0, 0)],
    7: [(12, 0, 0)],
    9: [(5, 0, 1)],
}
TEST_COMMANDS = {2, 3, 4, 5, 6, 7, 8, 9}
SUMMARY_RE = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored")
REQUIRED_GATEWAY_TESTS = (
    "no_prefix_while_latch_keeps_physical_source_and_one_owner",
    "no_prefix_latch_budget_and_cancellation_publish_no_partial_source",
)


def sha_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def target_bytes() -> int:
    target = ROOT / "target"
    return sum(path.stat().st_size for path in target.rglob("*") if path.is_file()) if target.exists() else 0


def source_pins(paths: set[str]) -> dict[str, str]:
    result = {}
    for relative in sorted(paths):
        path = ROOT / relative
        if not path.is_file():
            raise FileNotFoundError(f"required pin input missing: {relative}")
        result[relative] = sha_file(path)
    return result


def command_stream(path: Path) -> dict:
    return {"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": sha_file(path)}


def expected_test_summaries(index: int, stdout: bytes) -> dict:
    text = stdout.decode("utf-8", errors="replace")
    actual = [tuple(map(int, match)) for match in SUMMARY_RE.findall(text)]
    expected = EXPECTED_SUMMARIES.get(index)
    required = index in TEST_COMMANDS
    nonzero_ok = any(sum(summary) > 0 for summary in actual)
    names_ok = (index != 3 or all(re.search(rf"(?m)^test {re.escape(name)} \.\.\. ok$", text)
                                  for name in REQUIRED_GATEWAY_TESTS))
    ok = (expected is None or actual == expected) and (not required or nonzero_ok) and names_ok
    reason = None
    if expected is not None and actual != expected:
        reason = f"expected summaries {expected}, got {actual}"
    elif required and not nonzero_ok:
        reason = "required successful test summary was absent or empty"
    elif not names_ok:
        reason = "one or more exact noPrefix latch test names were absent from --nocapture output"
    return {"expected": expected, "actual": actual, "required": required,
            "required_gateway_tests": list(REQUIRED_GATEWAY_TESTS) if index == 3 else [],
            "required_gateway_tests_present": names_ok if index == 3 else None,
            "ok": ok, "failure_reason": reason}


def write_execution(status: str, rows: list[dict], preflight: dict, freeze: dict | None = None) -> None:
    write_json(OUT / "execution.json", {
        "schema": "preserve-proved-loop-latch-validation-build-root-v1",
        "validation_runner": {"path": str(Path(__file__).resolve()),
                              "sha256": sha_file(Path(__file__).resolve())},
        "status": status,
        "source_commit_base_expected": SOURCE_COMMIT_BASE,
        "uncommitted_loop_latch_product": True,
        "environment_overrides": ENV_VALUES,
        "guards": {"minimum_free_bytes": FREE_LIMIT, "maximum_target_bytes": TARGET_LIMIT},
        "preflight": preflight,
        "commands": rows,
        "freeze": freeze,
    })


def run_command(index: int, argv: list[str], env: dict[str, str]) -> dict:
    free_start = shutil.disk_usage(ROOT).free
    size_start = target_bytes()
    if free_start < FREE_LIMIT:
        raise RuntimeError(f"20 GiB free-space guard before command {index}: {free_start}")
    if size_start > TARGET_LIMIT:
        raise RuntimeError(f"1 GiB target-size guard before command {index}: {size_start}")
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    began = time.monotonic()
    stdout_path = OUT / f"{index}.stdout.raw"
    stderr_path = OUT / f"{index}.stderr.raw"
    guard_stop = None
    peak_target = size_start
    with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr,
                                 start_new_session=True)
        while child.poll() is None:
            size = target_bytes()
            free = shutil.disk_usage(ROOT).free
            peak_target = max(peak_target, size)
            if size > TARGET_LIMIT or free < FREE_LIMIT:
                guard_stop = {"target_bytes": size, "free_bytes": free}
                os.killpg(child.pid, signal.SIGTERM)
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                break
            time.sleep(1)
        exit_code = child.wait()
    size_final = target_bytes()
    free_final = shutil.disk_usage(ROOT).free
    peak_target = max(peak_target, size_final)
    if guard_stop is None and (size_final > TARGET_LIMIT or free_final < FREE_LIMIT):
        guard_stop = {"target_bytes": size_final, "free_bytes": free_final,
                      "observed_after_exit": True}
    stdout_bytes = stdout_path.read_bytes()
    summary = expected_test_summaries(index, stdout_bytes) if exit_code == 0 else None
    return {
        "index": index, "argv": argv, "cwd": str(ROOT), "started_at": started,
        "duration_seconds": time.monotonic() - began, "exit_code": exit_code,
        "guard_stop": guard_stop, "peak_target_bytes": peak_target,
        "free_bytes_after": free_final, "env_overrides": ENV_VALUES,
        "test_summary_check": summary,
        "streams": {"stdout": command_stream(stdout_path), "stderr": command_stream(stderr_path)},
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-base", required=True,
                        help="exact expected HEAD commit after the root documentation checkpoint")
    args = parser.parse_args()
    global SOURCE_COMMIT_BASE
    SOURCE_COMMIT_BASE = args.source_base.lower()
    if not re.fullmatch(r"[0-9a-f]{40}", SOURCE_COMMIT_BASE):
        raise SystemExit("--source-base must be an exact 40-character commit SHA")
    if shutil.disk_usage(ROOT).free < FREE_LIMIT:
        raise SystemExit("20 GiB free-space preflight failed before creating validation output; free space and retry")
    if target_bytes() > TARGET_LIMIT:
        raise SystemExit("1 GiB target-size preflight failed before creating validation output; reduce target and retry")
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite validation evidence: {OUT}")
    if CLI_PATH.exists() or METADATA_PATH.exists():
        raise SystemExit("refusing to overwrite frozen CLI or metadata output")
    OUT.mkdir()

    preflight = {"git_head": None, "source_pins_before": None, "source_pins_after": None}
    rows: list[dict] = []
    env = os.environ.copy()
    env.update(ENV_VALUES)
    try:
        git_result = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                    capture_output=True, check=False)
        git_stdout_path = OUT / "git-head.stdout.raw"
        git_stderr_path = OUT / "git-head.stderr.raw"
        git_stdout_path.write_bytes(git_result.stdout)
        git_stderr_path.write_bytes(git_result.stderr)
        head = git_result.stdout.decode("ascii", errors="replace").strip()
        preflight["git_head"] = {
            "argv": ["git", "rev-parse", "HEAD"], "exit_code": git_result.returncode,
            "value": head, "stdout": command_stream(git_stdout_path),
            "stderr": command_stream(git_stderr_path),
            "matches_expected": git_result.returncode == 0 and head == SOURCE_COMMIT_BASE,
        }
        if not preflight["git_head"]["matches_expected"]:
            raise RuntimeError("HEAD does not exactly match the supplied --source-base documentation checkpoint")
        preflight["source_pins_before"] = {
            "candidate_sources": source_pins(PRODUCT_PATHS),
            "test_sources": source_pins(TEST_PATHS),
            "canonical_files": source_pins(CANONICAL_PATHS),
        }
        if shutil.disk_usage(ROOT).free < FREE_LIMIT:
            raise RuntimeError("20 GiB free-space guard preflight failed")
        if target_bytes() > TARGET_LIMIT:
            raise RuntimeError("1 GiB target-size guard preflight failed")
        write_execution("running", rows, preflight)

        for index, argv in enumerate(COMMANDS):
            row = run_command(index, argv, env)
            rows.append(row)
            write_execution("running", rows, preflight)
            print(json.dumps(row, ensure_ascii=False), flush=True)
            if (row["exit_code"] != 0 or row["guard_stop"] is not None
                    or (row["test_summary_check"] is not None
                        and not row["test_summary_check"]["ok"])):
                raise RuntimeError(f"validation command {index} failed its exit, summary, or disk guard")

        if len(rows) != len(COMMANDS) or any(row["exit_code"] != 0 for row in rows):
            raise RuntimeError("not all required validation commands passed")
        preflight["source_pins_after"] = {
            "candidate_sources": source_pins(PRODUCT_PATHS),
            "test_sources": source_pins(TEST_PATHS),
            "canonical_files": source_pins(CANONICAL_PATHS),
        }
        if preflight["source_pins_after"] != preflight["source_pins_before"]:
            raise RuntimeError("pinned source/canonical files changed during validation")

        built_binary = ROOT / "target/debug/jarde-cli"
        if not built_binary.is_file():
            raise FileNotFoundError(f"successful build did not produce {built_binary}")
        CLI_PATH.parent.mkdir(parents=True, exist_ok=True)
        temp_cli = CLI_PATH.with_name(f".{CLI_PATH.name}.tmp-{os.getpid()}")
        if temp_cli.exists() or CLI_PATH.exists():
            raise FileExistsError("refusing to overwrite candidate CLI destination/temp")
        shutil.copyfile(built_binary, temp_cli)
        os.chmod(temp_cli, 0o555)
        os.replace(temp_cli, CLI_PATH)
        cli_sha256 = sha_file(CLI_PATH)
        cli_mode = stat.S_IMODE(CLI_PATH.stat().st_mode)
        if cli_mode != 0o555:
            raise RuntimeError(f"candidate CLI is not frozen read/execute-only: {oct(cli_mode)}")

        freeze = {
            "built_binary": str(built_binary), "cli_path": str(CLI_PATH),
            "cli_sha256": cli_sha256, "cli_mode": oct(cli_mode),
            "metadata_path": str(METADATA_PATH),
            "source_commit_base": SOURCE_COMMIT_BASE,
            "uncommitted_loop_latch_product": True,
            "product_path_sets": {"candidate_sources": sorted(PRODUCT_PATHS),
                                  "test_sources": sorted(TEST_PATHS),
                                  "canonical_files": sorted(CANONICAL_PATHS)},
        }
        write_execution("validation-passed-cli-frozen", rows, preflight, freeze)
        execution_sha256 = sha_file(OUT / "execution.json")
        metadata = {
            "cli_path": str(CLI_PATH),
            "cli_sha256": cli_sha256,
            "candidate_sources": preflight["source_pins_after"]["candidate_sources"],
            "test_sources": preflight["source_pins_after"]["test_sources"],
            "build_result_sha256": execution_sha256,
            "canonical_files": preflight["source_pins_after"]["canonical_files"],
            "source_commit_base": SOURCE_COMMIT_BASE,
            "uncommitted_loop_latch_product": True,
        }
        with METADATA_PATH.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(metadata, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"status": "frozen", "cli_path": str(CLI_PATH),
                          "cli_sha256": cli_sha256, "metadata_path": str(METADATA_PATH),
                          "build_result_sha256": execution_sha256}, ensure_ascii=False, indent=2), flush=True)
        return 0
    except Exception as error:
        write_execution("failed", rows, {**preflight, "failure": f"{type(error).__name__}: {error}"})
        print(f"validation/build/freeze stopped: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
