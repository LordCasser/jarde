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
OUT = RESULTS / "validation-build-root-v8"
CLI_PATH = Path("/private/tmp/jarde-loop-latch-cli-v1")
METADATA_PATH = RESULTS / "candidate-cli-v1.json"
SOURCE_COMMIT_BASE: str | None = None
RESUME_PREFIX: dict | None = None
TARGET_LIMIT = 1024**3
FREE_LIMIT = 5 * 1024**3
ENV_VALUES = {
    "CARGO_BUILD_JOBS": "1",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "RUST_TEST_THREADS": "1",
    "CARGO_TERM_COLOR": "always",
}
STRIPPED_ENV_KEYS = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")

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
    "tests/p3_loop_boolean_exit.rs",
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
    ["cargo", "test", "-p", "jarde", "--test", "p3_loop_boolean_exit", "--locked", "--", "--nocapture"],
    ["cargo", "build", "-p", "jarde-cli", "--locked"],
]
assert len(COMMANDS) == 12

EXPECTED_SUMMARIES = {
    3: [(5, 0, 0)],   # 3 accepted gateway tests + 2 exact noPrefix latch tests from the patch
    4: [(4, 0, 0)],   # root-level p3_loop_arm_join target
    5: [(7, 0, 0)],
    6: [(5, 0, 0)],
    7: [(12, 0, 0)],
    9: [(5, 0, 1)],
    10: [(3, 0, 0)],
}
TEST_COMMANDS = {2, 3, 4, 5, 6, 7, 8, 9, 10}
SUMMARY_RE = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored")
RUNNING_BINARY_RE = re.compile(r"^\s*Running\b[^\r\n]*\(([^()\r\n]+)\)\s*$")
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
REQUIRED_GATEWAY_TESTS = (
    "no_prefix_while_latch_keeps_physical_source_and_one_owner",
    "no_prefix_latch_budget_and_cancellation_publish_no_partial_source",
)
REQUIRED_BOOLEAN_LOOP_TESTS = (
    "proved_header_test_chains_become_short_circuit_loop_conditions",
    "an_independent_effect_between_header_tests_is_never_folded_into_the_condition",
    "recovered_loop_exit_matches_the_java8_class_for_terminating_and_continuing_inputs",
)
JDK23_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK23_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JDK23_TOOL_SHA256 = {
    "java": "b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1",
    "javac": "a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8",
    "javap": "f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e",
}


def sha_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()



def configure_jdk23(env: dict[str, str]) -> dict:
    if sha_file(JDK23_MANIFEST) != JDK23_MANIFEST_SHA256:
        raise RuntimeError(f"JDK 23 controls manifest SHA-256 mismatch: {JDK23_MANIFEST}")
    manifest = json.loads(JDK23_MANIFEST.read_text(encoding="utf-8"))
    leg = next((row for row in manifest.get("legs", []) if row.get("leg") == "javac23"), None)
    if leg is None:
        raise RuntimeError("JDK 23 controls manifest has no javac23 leg")
    tool_paths = {}
    for name, expected_sha in JDK23_TOOL_SHA256.items():
        record = leg.get("jdk_tools", {}).get(name, {})
        path = Path(record.get("path", ""))
        if record.get("sha256") != expected_sha or not path.is_file() or sha_file(path) != expected_sha:
            raise RuntimeError(f"pinned JDK 23 {name} identity/SHA-256 mismatch: {path}")
        tool_paths[name] = path
    java_home = tool_paths["java"].parent.parent
    if any(path.parent != java_home / "bin" for path in tool_paths.values()):
        raise RuntimeError("pinned JDK 23 tools do not share one JAVA_HOME/bin")
    env["JAVA_HOME"] = str(java_home)
    env["PATH"] = str(java_home / "bin") + os.pathsep + env.get("PATH", "")
    return {
        "manifest_path": str(JDK23_MANIFEST),
        "manifest_sha256": JDK23_MANIFEST_SHA256,
        "java_home": str(java_home),
        "tools": {name: {"path": str(path), "sha256": JDK23_TOOL_SHA256[name]}
                  for name, path in tool_paths.items()},
    }


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


def target_deps() -> Path:
    return (ROOT / "target/debug/deps").resolve(strict=True)


def checked_deps_file(path: Path, deps: Path, *, executable: bool = False) -> tuple[Path, os.stat_result]:
    if path.parent.resolve() != deps or path.is_symlink():
        raise RuntimeError(f"refusing to clean out-of-scope or linked target file: {path}")
    resolved = path.resolve(strict=True)
    info = resolved.lstat()
    if resolved.parent != deps or not stat.S_ISREG(info.st_mode):
        raise RuntimeError(f"refusing to clean non-regular target file: {path}")
    if executable and not info.st_mode & 0o111:
        raise RuntimeError(f"refusing to clean non-executable test binary: {path}")
    return resolved, info


def clean_successful_test_binary(index: int, removed: list[dict], stderr_path: Path | None = None) -> None:
    stderr_path = stderr_path or (OUT / f"{index}.stderr.raw")
    stderr = stderr_path.read_text(encoding="utf-8", errors="replace")
    stderr = ANSI_RE.sub("", stderr)
    matches = [match.group(1).strip() for line in stderr.splitlines()
               if (match := RUNNING_BINARY_RE.match(line))]
    paths = [ROOT / value if not Path(value).is_absolute() else Path(value)
             for value in matches if "target/debug/deps" in Path(value).as_posix()]
    deps = target_deps()
    if len(paths) != 1:
        raise RuntimeError(f"test command {index} must report exactly one target/debug/deps binary; got {len(paths)}")
    binary, info = checked_deps_file(paths[0], deps, executable=True)
    dsym = binary.with_name(binary.name + ".dSYM")
    dsym_record = None
    if dsym.exists() or dsym.is_symlink():
        if dsym.parent.resolve() != deps or dsym.is_symlink() or not dsym.is_dir():
            raise RuntimeError(f"refusing to clean out-of-scope dSYM: {dsym}")
        entries = list(dsym.rglob("*"))
        if any(path.is_symlink() for path in entries):
            raise RuntimeError(f"refusing to clean dSYM containing a symlink: {dsym}")
        byte_count = sum(path.stat().st_size for path in entries if path.is_file())
        dsym_record = {"path": str(dsym.relative_to(ROOT)), "bytes": byte_count, "kind": "dSYM"}
    binary.unlink()
    removed.append({"path": str(binary.relative_to(ROOT)), "bytes": info.st_size, "kind": "test-binary"})
    if dsym_record is not None:
        shutil.rmtree(dsym)
        removed.append(dsym_record)


def clean_check_only_metadata(removed: list[dict]) -> None:
    deps = target_deps()
    for metadata in sorted(deps.glob("*.rmeta")):
        if metadata.with_suffix(".rlib").exists():
            continue
        candidate, info = checked_deps_file(metadata, deps)
        metadata.unlink()
        removed.append({"path": str(candidate.relative_to(ROOT)), "bytes": info.st_size,
                        "kind": "check-only-rmeta"})

def expected_test_summaries(index: int, stdout: bytes) -> dict:
    text = stdout.decode("utf-8", errors="replace")
    actual = [tuple(map(int, match)) for match in SUMMARY_RE.findall(text)]
    expected = EXPECTED_SUMMARIES.get(index)
    required = index in TEST_COMMANDS
    nonzero_ok = any(sum(summary) > 0 for summary in actual)
    names_ok = (index != 3 or all(re.search(rf"(?m)^test {re.escape(name)} \.\.\. ok$", text)
                                  for name in REQUIRED_GATEWAY_TESTS))
    if index == 10:
        names_ok = names_ok and all(
            re.search(rf"(?m)^test {re.escape(name)} \.\.\. ok$", text)
            for name in REQUIRED_BOOLEAN_LOOP_TESTS
        )
    ok = (expected is None or actual == expected) and (not required or nonzero_ok) and names_ok
    reason = None
    if expected is not None and actual != expected:
        reason = f"expected summaries {expected}, got {actual}"
    elif required and not nonzero_ok:
        reason = "required successful test summary was absent or empty"
    elif not names_ok:
        reason = ("one or more exact noPrefix latch test names were absent from --nocapture output"
                  if index == 3 else "one or more required boolean-loop test names were absent from --nocapture output")
    return {"expected": expected, "actual": actual, "required": required,
            "required_gateway_tests": list(REQUIRED_GATEWAY_TESTS) if index == 3 else [],
            "required_gateway_tests_present": names_ok if index == 3 else None,
            "required_boolean_loop_tests": list(REQUIRED_BOOLEAN_LOOP_TESTS) if index == 10 else [],
            "required_boolean_loop_tests_present": names_ok if index == 10 else None,
            "ok": ok, "failure_reason": reason}


def write_execution(status: str, rows: list[dict], preflight: dict, freeze: dict | None = None) -> None:
    write_json(OUT / "execution.json", {
        "schema": "preserve-proved-loop-latch-validation-build-root-v8",
        "validation_runner": {"path": str(Path(__file__).resolve()),
                              "sha256": sha_file(Path(__file__).resolve())},
        "status": status,
        "source_commit_base_expected": SOURCE_COMMIT_BASE,
        "uncommitted_loop_latch_product": True,
        "environment_overrides": ENV_VALUES,
        "guards": {"minimum_free_bytes": FREE_LIMIT, "maximum_target_bytes": TARGET_LIMIT},
        "preflight": preflight,
        "resume_prefix": RESUME_PREFIX,
        "commands": rows,
        "freeze": freeze,
    })


def run_command(index: int, argv: list[str], env: dict[str, str]) -> dict:
    free_start = shutil.disk_usage(ROOT).free
    size_start = target_bytes()
    if free_start < FREE_LIMIT:
        raise RuntimeError(f"5 GiB free-space guard before command {index}: {free_start}")
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


def verified_stream(stream: dict, evidence_root: Path) -> bytes:
    path = Path(stream["path"])
    if not path.is_absolute():
        path = ROOT / path
    path = path.resolve(strict=True)
    try:
        path.relative_to(evidence_root.resolve())
    except ValueError as error:
        raise RuntimeError(f"recorded stream escapes its evidence directory: {path}") from error
    raw = path.read_bytes()
    if len(raw) != stream["bytes"] or sha(raw) != stream["sha256"]:
        raise RuntimeError(f"recorded stream bytes/hash mismatch: {path}")
    return raw


def load_verified_prefix(source_base: str, current_pins: dict) -> tuple[list[dict], dict, dict]:
    old_path = RESULTS / "validation-build-root-v6" / "execution.json"
    old_bytes = old_path.read_bytes()
    old = json.loads(old_bytes)
    old_root = old_path.parent
    runner_path = RESULTS / "run-validation-build-root-v6.py"
    runner = old["validation_runner"]
    if (old["schema"] != "preserve-proved-loop-latch-validation-build-root-v6"
            or old["status"] != "failed" or old["source_commit_base_expected"] != source_base
            or runner != {"path": str(runner_path.resolve()), "sha256": sha_file(runner_path)}
            or old["environment_overrides"] != ENV_VALUES
            or old["guards"] != {"minimum_free_bytes": FREE_LIMIT, "maximum_target_bytes": TARGET_LIMIT}):
        raise RuntimeError("root-v6 execution identity, source base, environment, or guards do not match")
    preflight = old["preflight"]
    if (preflight["source_pins_before"] != current_pins or preflight["source_pins_after"] is not None):
        raise RuntimeError("root-v6 before-pins differ from current sources or unexpectedly completed")
    head = preflight["git_head"]
    if (head["argv"] != ["git", "rev-parse", "HEAD"] or head["exit_code"] != 0
            or not head["matches_expected"] or head["value"] != source_base):
        raise RuntimeError("root-v6 captured HEAD does not match the supplied source base")
    for name in ("stdout", "stderr"):
        raw = verified_stream(head[name], old_root)
        if name == "stdout" and raw.decode("ascii").strip() != source_base:
            raise RuntimeError("root-v6 captured HEAD stdout does not match source base")

    old_rows = old["commands"]
    if len(old_rows) != 5 or old_rows[4]["index"] != 4:
        raise RuntimeError("root-v6 command prefix is not the expected four passes plus failed command 4")
    failed = old_rows[4]
    if failed["argv"] != COMMANDS[4] or failed["exit_code"] == 0 or failed["guard_stop"] is None:
        raise RuntimeError("root-v6 command 4 is not the expected resource-stopped arm-join test")
    prefix = []
    for index, row in enumerate(old_rows[:4]):
        if (row["index"] != index or row["argv"] != COMMANDS[index] or row["cwd"] != str(ROOT)
                or row["exit_code"] != 0 or row["guard_stop"] is not None
                or row["env_overrides"] != ENV_VALUES or row["peak_target_bytes"] > TARGET_LIMIT
                or row["free_bytes_after"] < FREE_LIMIT):
            raise RuntimeError(f"root-v6 command {index} is not a successful reusable prefix row")
        captured = {name: verified_stream(stream, old_root)
                    for name, stream in row["streams"].items()}
        stdout = captured.get("stdout")
        if stdout is None or "stderr" not in captured:
            raise RuntimeError(f"root-v6 command {index} is missing raw streams")
        expected_summary = json.loads(json.dumps(expected_test_summaries(index, stdout)))
        if row["test_summary_check"] != expected_summary or not expected_summary["ok"]:
            raise RuntimeError(f"root-v6 command {index} summary does not recompute as passed")
        prefix.append({**row, "cleanup": {"test_binaries": [], "check_only_metadata": []}})
    return prefix, preflight, {"path": str(old_path.relative_to(ROOT)), "sha256": sha(old_bytes),
                               "commands_reused": 4, "reused_indices": [0, 1, 2, 3],
                               "failed_index_excluded": 4}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-base", required=True,
                        help="exact expected HEAD commit after the root documentation checkpoint")
    args = parser.parse_args()
    global SOURCE_COMMIT_BASE, RESUME_PREFIX
    SOURCE_COMMIT_BASE = args.source_base.lower()
    if not re.fullmatch(r"[0-9a-f]{40}", SOURCE_COMMIT_BASE):
        raise SystemExit("--source-base must be an exact 40-character commit SHA")
    if target_bytes() > TARGET_LIMIT:
        raise SystemExit("1 GiB target-size guard failed before creating validation output; reduce target and retry")
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite validation evidence: {OUT}")
    if CLI_PATH.exists() or METADATA_PATH.exists():
        raise SystemExit("refusing to overwrite frozen CLI or metadata output")
    OUT.mkdir()

    current_pins = {
        "candidate_sources": source_pins(PRODUCT_PATHS),
        "test_sources": source_pins(TEST_PATHS),
        "canonical_files": source_pins(CANONICAL_PATHS),
    }
    rows: list[dict] = []
    preflight = {"git_head": None, "source_pins_before": current_pins, "source_pins_after": None}
    env = os.environ.copy()
    stripped_keys = []
    for key in STRIPPED_ENV_KEYS:
        if key in env:
            env.pop(key)
            stripped_keys.append(key)
    env.update(ENV_VALUES)
    try:
        preflight["stripped_environment_keys"] = stripped_keys
        preflight["jdk23"] = configure_jdk23(env)
        head_result = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                     capture_output=True, check=False)
        head_stdout_path, head_stderr_path = OUT / "git-head.stdout.raw", OUT / "git-head.stderr.raw"
        head_stdout_path.write_bytes(head_result.stdout)
        head_stderr_path.write_bytes(head_result.stderr)
        head_value = head_result.stdout.decode("ascii", errors="replace").strip()
        preflight["git_head"] = {
            "argv": ["git", "rev-parse", "HEAD"], "exit_code": head_result.returncode,
            "value": head_value, "stdout": command_stream(head_stdout_path),
            "stderr": command_stream(head_stderr_path),
            "matches_expected": head_result.returncode == 0 and head_value == SOURCE_COMMIT_BASE,
        }
        if not preflight["git_head"]["matches_expected"]:
            raise RuntimeError("HEAD does not exactly match the supplied --source-base")

        rows, resumed_preflight, resume_record = load_verified_prefix(SOURCE_COMMIT_BASE, current_pins)
        preflight["resumed_root_v6_git_head"] = resumed_preflight["git_head"]
        RESUME_PREFIX = {**resume_record, "cleanup": {"test_binaries": [], "check_only_metadata": []}}
        clean_check_only_metadata(RESUME_PREFIX["cleanup"]["check_only_metadata"])
        preflight["resume_prefix_cleanup"] = RESUME_PREFIX["cleanup"]
        write_execution("running", rows, preflight)
        if shutil.disk_usage(ROOT).free < FREE_LIMIT:
            raise RuntimeError("5 GiB free-space guard remains below threshold after resume-prefix cleanup")
        if target_bytes() > TARGET_LIMIT:
            raise RuntimeError("1 GiB target-size guard failed after resume-prefix cleanup")

        for index in range(4, len(COMMANDS)):
            argv = COMMANDS[index]
            row = run_command(index, argv, env)
            rows.append(row)
            write_execution("running", rows, preflight)
            print(json.dumps(row, ensure_ascii=False), flush=True)
            is_test_command = argv[0] == "cargo" and argv[1] == "test"
            if (row["exit_code"] != 0 or row["guard_stop"] is not None
                    or (is_test_command and (row["test_summary_check"] is None
                                             or not row["test_summary_check"]["ok"]))
                    or (row["test_summary_check"] is not None
                        and not row["test_summary_check"]["ok"])):
                raise RuntimeError(f"validation command {index} failed its exit, summary, or disk guard")
            row["cleanup"] = {"test_binaries": [], "check_only_metadata": []}
            if is_test_command:
                clean_successful_test_binary(index, row["cleanup"]["test_binaries"])
            write_execution("running", rows, preflight)

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
