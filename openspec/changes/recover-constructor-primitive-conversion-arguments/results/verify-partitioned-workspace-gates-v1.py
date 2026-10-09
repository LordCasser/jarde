#!/usr/bin/env python3
"""Independently verify saved partitioned workspace Cargo-gate evidence.

This reads the runner's manifest and raw artifacts. It deliberately does not import or execute
run-partitioned-workspace-gates-v1.py and does not launch Cargo.
"""
from __future__ import annotations

import ast
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
OUT = RESULTS / "partitioned-workspace-gates-v1"
MANIFEST_PATH = OUT / "manifest.json"
OUTPUT_PATH = RESULTS / "partitioned-workspace-root-verification-v1.json"
RUNNER_PATH = RESULTS / "run-partitioned-workspace-gates-v1.py"
GATE_RUNNER_PATH = RESULTS / "run-root-gate-v2.py"
INHERITED_ENV_PATH = RESULTS / "root-inherited-build-environment-v1.json"
SEEDS = ("5350648285461741569", "5350648285461741570")
KINDS = {"lib", "bin", "example", "test", "bench"}
CHUNK_SIZE = 10
TEST_RESULT = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in .+$"
)


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def read_json(path: Path, errors: list[str], label: str):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as error:
        fail(errors, f"{label}: cannot read JSON {path}: {type(error).__name__}: {error}")
        return None


def file_identity(path: Path) -> dict:
    data = path.read_bytes()
    return {"bytes": len(data), "sha256": sha_bytes(data)}


def resolve_manifest_file(relative: str) -> Path:
    path = (OUT / relative).resolve()
    if OUT.resolve() not in path.parents:
        raise ValueError(f"artifact path escapes evidence directory: {relative}")
    return path


def verify_recorded_file(record: dict, expected: Path, errors: list[str], label: str) -> bool:
    if not isinstance(record, dict):
        fail(errors, f"{label}: missing file record")
        return False
    try:
        actual = file_identity(expected)
    except OSError as error:
        fail(errors, f"{label}: missing/unreadable file {expected}: {error}")
        return False
    ok = True
    if record.get("bytes") != actual["bytes"]:
        fail(errors, f"{label}: byte count mismatch for {expected}")
        ok = False
    if record.get("sha256") != actual["sha256"]:
        fail(errors, f"{label}: SHA-256 mismatch for {expected}")
        ok = False
    return ok


def metadata_target_rows(metadata: dict, errors: list[str]) -> list[dict]:
    member_ids = set(metadata.get("workspace_members", []))
    packages = [p for p in metadata.get("packages", []) if p.get("id") in member_ids]
    package_ids = {p.get("id") for p in packages}
    if member_ids != package_ids:
        fail(errors, f"metadata workspace members missing package records: {sorted(member_ids - package_ids)}")
    rows = []
    for package in packages:
        for target in package.get("targets", []):
            source = target.get("src_path")
            row = {
                "package_id": package.get("id"),
                "package_name": package.get("name"),
                "target_name": target.get("name"),
                "kind": target.get("kind") or [],
                "src_path": source,
                "test": target.get("test"),
                "doctest": target.get("doctest"),
                "doc": target.get("doc"),
                "src_sha256": None,
            }
            if not isinstance(source, str):
                fail(errors, f"metadata target has no source path: {row}")
            else:
                source_path = Path(source)
                if not source_path.is_file():
                    fail(errors, f"metadata target source is missing: {source}")
                else:
                    row["src_sha256"] = sha_file(source_path)
            kinds = row["kind"]
            if len(kinds) != 1 or kinds[0] not in KINDS:
                fail(errors, f"unsupported metadata target kind: {row}")
            elif row["test"] is False and not (kinds == ["example"]):
                fail(errors, f"test=false target outside compile-only example handling: {row}")
            elif kinds == ["test"] and row["test"] is not True:
                fail(errors, f"integration test target is not test-enabled: {row}")
            rows.append(row)
    return sorted(rows, key=lambda row: (row["package_id"], row["target_name"], row["kind"]))


def target_key(row: dict) -> tuple:
    return (row.get("package_id"), row.get("target_name"), tuple(row.get("kind") or []))


def selection_record(row: dict) -> dict:
    return {key: row.get(key) for key in (
        "package_id", "package_name", "target_name", "kind", "src_path", "test", "doctest")}


def expected_groups(targets: list[dict]) -> list[dict]:
    groups = []
    kinds = {row["kind"][0] for row in targets if len(row["kind"]) == 1}
    for kind, flag in (("lib", "--lib"), ("bin", "--bins"),
                       ("example", "--examples"), ("bench", "--benches")):
        if kind in kinds:
            selected = [row for row in targets if row["kind"] == [kind]]
            groups.append({
                "id": kind,
                "kind": kind,
                "target_names": sorted({row["target_name"] for row in selected}),
                "cargo_args": [flag],
                "targets": selected,
            })
    test_names = sorted({row["target_name"] for row in targets if row["kind"] == ["test"]})
    for offset in range(0, len(test_names), CHUNK_SIZE):
        names = test_names[offset:offset + CHUNK_SIZE]
        selected = [row for row in targets
                    if row["kind"] == ["test"] and row["target_name"] in names]
        args = [arg for name in names for arg in ("--test", name)]
        groups.append({
            "id": f"tests-{offset // CHUNK_SIZE:03d}",
            "kind": "test",
            "target_names": names,
            "cargo_args": args,
            "targets": selected,
        })
    return groups


def parse_test_results(stdout: str, errors: list[str], label: str) -> dict:
    lines = stdout.splitlines()
    found = []
    for number, line in enumerate(lines, 1):
        if "test result:" not in line:
            continue
        match = TEST_RESULT.fullmatch(line.strip())
        if not match:
            fail(errors, f"{label}: unrecognized test-result line {number}: {line}")
            continue
        status, passed, failed_count, ignored, measured, filtered = match.groups()
        row = {
            "status": status,
            "passed": int(passed),
            "failed": int(failed_count),
            "ignored": int(ignored),
            "measured": int(measured),
            "filtered_out": int(filtered),
        }
        found.append(row)
        if status != "ok" or row["failed"] != 0:
            fail(errors, f"{label}: failed test-result row: {row}")
    return {
        "result_rows": len(found),
        "passed": sum(row["passed"] for row in found),
        "failed": sum(row["failed"] for row in found),
        "ignored": sum(row["ignored"] for row in found),
        "measured": sum(row["measured"] for row in found),
        "filtered_out": sum(row["filtered_out"] for row in found),
        "rows": found,
    }


def main() -> int:
    errors: list[str] = []
    warnings: list[str] = []
    report = {
        "schema": "partitioned-workspace-root-verification-v1",
        "verification": "not-run",
        "manifest_path": str(MANIFEST_PATH),
        "seeds": list(SEEDS),
        "errors": errors,
        "warnings": warnings,
    }
    manifest = read_json(MANIFEST_PATH, errors, "manifest")
    if not isinstance(manifest, dict):
        report["verification"] = "failed"
        OUTPUT_PATH.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(json.dumps({"verification": report["verification"], "errors": errors}, ensure_ascii=False))
        return 1

    report["manifest_status"] = manifest.get("status")
    if manifest.get("schema") != "partitioned-workspace-gates-v1":
        fail(errors, f"unexpected manifest schema: {manifest.get('schema')!r}")
    if manifest.get("status") != "complete" or manifest.get("coverage_complete") is not True:
        fail(errors, "manifest does not record a complete two-seed workspace run")
    if manifest.get("unsupported_metadata"):
        fail(errors, "manifest reports unsupported metadata targets")

    # The raw metadata captured by the runner is the source of truth for this run's target closure.
    metadata_record = manifest.get("metadata", {})
    metadata_stdout_record = metadata_record.get("stdout", {})
    metadata_stderr_record = metadata_record.get("stderr", {})
    if metadata_record.get("command") != ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"]:
        fail(errors, "metadata command is not the locked fresh metadata command")
    if metadata_record.get("exit") != 0:
        fail(errors, "metadata command did not exit zero")
    for rec, label in ((metadata_stdout_record, "metadata stdout"), (metadata_stderr_record, "metadata stderr")):
        try:
            expected = resolve_manifest_file(rec.get("path", ""))
            verify_recorded_file(rec, expected, errors, label)
        except Exception as error:
            fail(errors, f"{label}: invalid file reference: {error}")
    raw_metadata_path = resolve_manifest_file(metadata_stdout_record.get("path", "cargo-metadata.stdout"))
    metadata = read_json(raw_metadata_path, errors, "captured cargo metadata")
    targets = metadata_target_rows(metadata, errors) if isinstance(metadata, dict) else []
    report["metadata_target_count"] = len(targets)
    report["workspace_package_count"] = len(set(row["package_id"] for row in targets))

    if isinstance(metadata, dict):
        if Path(metadata.get("workspace_root", "")).resolve() != ROOT.resolve():
            fail(errors, "captured metadata workspace_root does not match this checkout")
        if Path(metadata.get("target_directory", "")).resolve() != (ROOT / "target").resolve():
            fail(errors, "captured metadata target_directory does not match this checkout")
    if manifest.get("workspace_root") != str(ROOT):
        fail(errors, "manifest workspace_root does not match this checkout")
    if isinstance(metadata, dict) and manifest.get("workspace_members") != metadata.get("workspace_members"):
        fail(errors, "manifest workspace member IDs differ from captured metadata")

    base_fields = ("package_id", "package_name", "target_name", "kind", "src_path", "test", "doctest", "doc", "src_sha256")
    observed_targets = manifest.get("workspace_targets", [])
    observed_base = [{key: row.get(key) for key in base_fields} for row in observed_targets]
    expected_base = [{key: row.get(key) for key in base_fields} for row in targets]
    if observed_base != expected_base:
        fail(errors, "manifest workspace target set/metadata/source hashes differ from fresh metadata")
    if len({target_key(row) for row in targets}) != len(targets):
        fail(errors, "fresh metadata contains duplicate package/target/kind identities")

    # Pin both runner identities to the current source and each command's recorded executable.
    for path_field, sha_field, path in (
        ("runner_path", "runner_sha256", RUNNER_PATH),
        ("gate_runner_path", "gate_runner_sha256", GATE_RUNNER_PATH),
    ):
        if manifest.get(path_field) != str(path):
            fail(errors, f"manifest {path_field} points elsewhere")
        try:
            current_sha = sha_file(path)
            if manifest.get(sha_field) != current_sha:
                fail(errors, f"manifest {sha_field} does not match current runner source")
        except OSError as error:
            fail(errors, f"cannot hash {path}: {error}")

    expected = expected_groups(targets)
    observed_groups = manifest.get("partitions", [])
    report["expected_partition_count"] = len(expected)
    report["observed_partition_count"] = len(observed_groups)
    if len(expected) != len(observed_groups):
        fail(errors, "manifest partition count differs from independently derived groups")

    group_by_id = {}
    for index, group in enumerate(expected):
        if index >= len(observed_groups):
            break
        observed = observed_groups[index]
        group_id = group["id"]
        group_by_id[group_id] = group
        if observed.get("id") != group_id or observed.get("kind") != group["kind"]:
            fail(errors, f"partition order/identity mismatch at {group_id}")
        if observed.get("target_names") != group["target_names"]:
            fail(errors, f"{group_id}: cargo target names differ from fresh metadata selection")
        if observed.get("cargo_args") != group["cargo_args"]:
            fail(errors, f"{group_id}: Cargo selection args differ from independently derived group")
        expected_selection = [selection_record(row) for row in group["targets"]]
        if observed.get("targets") != expected_selection:
            fail(errors, f"{group_id}: selected package/id/name/kind/source rows differ from metadata")

    command_by_key = {}
    commands = manifest.get("commands", [])
    for command in commands:
        key = (command.get("label"), command.get("seed"))
        if key in command_by_key:
            fail(errors, f"duplicate command record: {key}")
        command_by_key[key] = command
    expected_command_count = len(expected) * len(SEEDS)
    report["command_count"] = len(commands)
    if len(commands) != expected_command_count:
        fail(errors, f"expected {expected_command_count} group/seed command records, found {len(commands)}")

    runs_by_group_seed = {}
    for group in expected:
        group_id = group["id"]
        observed = next((g for g in observed_groups if g.get("id") == group_id), {})
        seed_runs = observed.get("seed_runs", [])
        got_seeds = [str(row.get("seed")) for row in seed_runs]
        if got_seeds != list(SEEDS):
            fail(errors, f"{group_id}: seed runs are not the exact ordered pair {SEEDS}")
        for seed in SEEDS:
            label = f"partitioned-workspace-gates-v1/{group_id}/seed-{seed}"
            command = command_by_key.get((label, seed))
            seed_run = next((r for r in seed_runs if str(r.get("seed")) == seed), None)
            if command is None or seed_run is None:
                fail(errors, f"missing actual command/result evidence for {group_id}, seed {seed}")
                continue
            runs_by_group_seed[(group_id, seed)] = (command, seed_run)
            cargo_argv = ["cargo", "test", "--workspace", "--all-features", "--locked", *group["cargo_args"]]
            if command.get("cargo_argv") != cargo_argv:
                fail(errors, f"{label}: Cargo argv does not exactly match workspace/features/locked plus metadata selection")
            wrapper_argv = command.get("argv", [])
            if (len(wrapper_argv) != 3 + len(cargo_argv)
                    or wrapper_argv[1] != str(GATE_RUNNER_PATH)
                    or wrapper_argv[2] != label
                    or wrapper_argv[3:] != cargo_argv):
                fail(errors, f"{label}: wrapper argv does not carry the exact expected Cargo command")
            for field in ("exit", "runner_exit", "wrapper_process_exit"):
                if command.get(field) != 0:
                    fail(errors, f"{label}: {field} is not zero ({command.get(field)!r})")
            if command.get("disk_stop_reason") is not None:
                fail(errors, f"{label}: disk guard stopped the command")
            if seed_run.get("exit") != 0 or seed_run.get("runner_exit") != 0:
                fail(errors, f"{label}: partition seed record is not successful")
            if seed_run.get("disk_stop_reason") is not None:
                fail(errors, f"{label}: partition seed record reports a disk stop")
            if seed_run.get("result_path") != command.get("result_path"):
                fail(errors, f"{label}: partition and command result paths differ")

            try:
                result_path = (RESULTS / command["result_path"]).resolve()
                if RESULTS.resolve() not in result_path.parents:
                    raise ValueError("result path escapes results directory")
                result_bytes = result_path.read_bytes()
                if sha_bytes(result_bytes) != command.get("result_sha256"):
                    fail(errors, f"{label}: result.json SHA does not match command record")
                result = json.loads(result_bytes)
            except Exception as error:
                fail(errors, f"{label}: cannot read result.json: {type(error).__name__}: {error}")
                continue
            if result.get("schema") != "root-command-v2-disk-guard":
                fail(errors, f"{label}: unexpected root gate result schema")
            if result.get("argv") != cargo_argv or result.get("cwd") != str(ROOT):
                fail(errors, f"{label}: actual wrapper result argv/cwd differs")
            if result.get("exit") != 0 or result.get("runner_exit") != 0 or result.get("disk_stop_reason") is not None:
                fail(errors, f"{label}: actual wrapper result is not a clean zero exit")
            if result.get("runner", {}).get("path") != str(GATE_RUNNER_PATH):
                fail(errors, f"{label}: recorded gate-runner path differs")
            if result.get("runner", {}).get("sha256") != manifest.get("gate_runner_sha256"):
                fail(errors, f"{label}: actual gate-runner SHA differs from manifest/current source")
            try:
                runner_record = result["runner"]
                if runner_record.get("bytes") != GATE_RUNNER_PATH.stat().st_size or runner_record.get("sha256") != sha_file(GATE_RUNNER_PATH):
                    fail(errors, f"{label}: gate-runner identity record does not match current file")
            except OSError as error:
                fail(errors, f"{label}: cannot verify gate-runner identity: {error}")

            env = result.get("env", {})
            expected_env = {
                "CARGO_BUILD_JOBS": "1",
                "CARGO_INCREMENTAL": "0",
                "RUST_TEST_THREADS": "1",
                "CARGO_PROFILE_DEV_DEBUG": "0",
                "CARGO_PROFILE_TEST_DEBUG": "0",
                "CARGO_PROFILE_DEV_STRIP": "symbols",
                "CARGO_PROFILE_TEST_STRIP": "symbols",
                "PROPTEST_RNG_SEED": seed,
            }
            for key, value in expected_env.items():
                if env.get(key) != value:
                    fail(errors, f"{label}: recorded environment {key}={env.get(key)!r}, expected {value!r}")
            opt_keys = sorted(key for key in env if key.startswith("CARGO_PROFILE_") and key.endswith("_OPT_LEVEL"))
            if opt_keys:
                fail(errors, f"{label}: recorded environment overrides Cargo opt-level: {opt_keys}")
            profile = result.get("profile_settings", {})
            if profile.get("opt_level") != "Cargo default; not overridden":
                fail(errors, f"{label}: profile_settings does not state default opt-level")
            if profile.get("debug_assertions") != "Cargo default remains enabled; not overridden":
                fail(errors, f"{label}: debug_assertions are not recorded as Cargo default")
            flags = env.get("RUSTFLAGS", "")
            if re.search(r"(?:^|\s)-C\s*opt-level(?:=|\s)", flags):
                fail(errors, f"{label}: recorded RUSTFLAGS overrides opt-level")

            # The gate records the fixed environment it explicitly constructs, not the complete
            # inherited host environment passed to Cargo. Keep this evidence limitation visible.
            if not result.get("effective_environment_captured"):
                warning = ("gate result records configured environment but not all inherited host variables; "
                           "absence of an unrecorded host opt-level override cannot be independently proven from this manifest")
                if warning not in warnings:
                    warnings.append(warning)

            for record_name in ("stdout", "stderr"):
                record = result.get(record_name, {})
                actual_path = result_path.parent / record_name
                if Path(record.get("path", "")).resolve() != actual_path.resolve():
                    fail(errors, f"{label}: result {record_name} path does not match its result directory")
                verify_recorded_file(record, actual_path, errors, f"{label} raw {record_name}")
            try:
                stdout_text = (result_path.parent / "stdout").read_text(encoding="utf-8", errors="replace")
            except OSError as error:
                fail(errors, f"{label}: cannot read raw Cargo stdout: {error}")
                stdout_text = ""
            parsed = parse_test_results(stdout_text, errors, label)
            if parsed["result_rows"] != len(group["targets"]):
                fail(errors, f"{label}: stdout has {parsed['result_rows']} test-result rows for {len(group['targets'])} selected metadata targets")
            current_summary = report.setdefault("per_seed_test_results", {}).setdefault(seed, {
                "result_rows": 0, "passed": 0, "failed": 0, "ignored": 0,
                "measured": 0, "filtered_out": 0, "partition_count": 0,
            })
            for metric in ("result_rows", "passed", "failed", "ignored", "measured", "filtered_out"):
                current_summary[metric] += parsed[metric]
            current_summary["partition_count"] += 1

            for record_name, record_key in (("wrapper_stdout", "wrapper-stdout"), ("wrapper_stderr", "wrapper-stderr")):
                record = command.get(record_name, {})
                try:
                    wrapper_path = resolve_manifest_file(record.get("path", ""))
                    verify_recorded_file(record, wrapper_path, errors, f"{label} {record_key}")
                except Exception as error:
                    fail(errors, f"{label}: invalid {record_key} reference: {error}")
            try:
                wrapper_text = resolve_manifest_file(command["wrapper_stdout"]["path"]).read_text(
                    encoding="utf-8", errors="replace")
                wrapper_rows = [json.loads(line) for line in wrapper_text.splitlines() if line.strip()]
                if len(wrapper_rows) != 1:
                    fail(errors, f"{label}: wrapper stdout must contain exactly one result row")
                elif (wrapper_rows[0].get("label") != label
                      or wrapper_rows[0].get("exit") != 0
                      or wrapper_rows[0].get("runner_exit") != 0
                      or wrapper_rows[0].get("disk_stop_reason") is not None):
                    fail(errors, f"{label}: wrapper stdout does not report a clean zero exit")
            except Exception as error:
                fail(errors, f"{label}: cannot verify wrapper stdout result: {type(error).__name__}: {error}")

            if seed_run.get("result_sha256") != command.get("result_sha256"):
                fail(errors, f"{label}: seed-run result SHA differs from command evidence")

    # Every metadata target must be attached to the exact partition and have two concrete successful runs.
    target_results = []
    group_for_key = {}
    for group in expected:
        for row in group["targets"]:
            group_for_key[target_key(row)] = group["id"]
    observed_by_key = {target_key(row): row for row in observed_targets}
    for target in targets:
        key = target_key(target)
        observed = observed_by_key.get(key, {})
        group_id = group_for_key.get(key)
        partition_id = observed.get("partition_id")
        if group_id is None or partition_id != group_id:
            fail(errors, f"target lacks its metadata-derived partition: {key}")
        runs = observed.get("gate_runs", [])
        seed_rows = []
        for seed in SEEDS:
            label = f"partitioned-workspace-gates-v1/{group_id}/seed-{seed}" if group_id else ""
            command, seed_run = runs_by_group_seed.get((group_id, seed), (None, None))
            success = bool(command and seed_run
                           and command.get("exit") == 0
                           and command.get("runner_exit") == 0
                           and command.get("wrapper_process_exit") == 0
                           and seed_run.get("exit") == 0
                           and seed_run.get("runner_exit") == 0)
            seed_rows.append({"seed": seed, "label": label, "successful_command": success})
            if not success:
                fail(errors, f"target {key} lacks an actual successful command for seed {seed}")
        if len(runs) != 2:
            fail(errors, f"target {key} gate_runs length is {len(runs)}, expected two concrete seed records")
        else:
            for run, expected_seed in zip(runs, SEEDS):
                command, seed_run = runs_by_group_seed.get((group_id, expected_seed), (None, None))
                expected_run = ({
                    "label": command.get("label"),
                    "seed": expected_seed,
                    "exit": command.get("exit"),
                    "runner_exit": command.get("runner_exit"),
                    "result_sha256": command.get("result_sha256"),
                } if command else None)
                if run != expected_run or not seed_run or seed_run.get("result_sha256") != run.get("result_sha256"):
                    fail(errors, f"target {key} gate_runs entry differs from its actual command/result evidence")
        target_results.append({
            **{key_name: target.get(key_name) for key_name in base_fields},
            "partition_id": group_id,
            "compile_only_example": target["kind"] == ["example"] and target.get("test") is False,
            "seed_runs": seed_rows,
        })
    report["targets"] = target_results
    report["metadata_targets_with_two_successful_commands"] = sum(
        1 for row in target_results if all(seed["successful_command"] for seed in row["seed_runs"]))
    report["compile_only_examples"] = [
        {"package_id": row["package_id"], "package_name": row["package_name"],
         "target_name": row["target_name"], "src_path": row["src_path"],
         "partition_id": row["partition_id"],
         "coverage": "compiled and empty harness ran under --examples; no example test cases"}
        for row in target_results if row["compile_only_example"]
    ]
    for seed in SEEDS:
        summary = report.get("per_seed_test_results", {}).get(seed)
        if summary is not None:
            if summary["result_rows"] != len(targets):
                fail(errors, f"seed {seed}: total stdout test-result rows {summary['result_rows']} do not close against {len(targets)} metadata targets")
            summary["metadata_target_result_rows_expected"] = len(targets)
            summary["test_enabled_metadata_targets"] = sum(
                1 for row in targets if row.get("test") is True)
            summary["compile_only_example_targets"] = sum(
                1 for row in targets if row["kind"] == ["example"] and row.get("test") is False)

    # Cross-check every saved file record against a fresh recursive inventory of the evidence tree.
    declared = manifest.get("files", [])
    declared_by_path = {}
    for row in declared:
        relative = row.get("path")
        if not isinstance(relative, str) or relative in declared_by_path:
            fail(errors, f"duplicate or malformed manifest.files entry: {row}")
            continue
        declared_by_path[relative] = row
        try:
            actual_path = resolve_manifest_file(relative)
            verify_recorded_file(row, actual_path, errors, f"manifest.files {relative}")
        except Exception as error:
            fail(errors, f"manifest.files invalid entry {row}: {error}")
    actual_files = {
        str(path.relative_to(OUT))
        for path in OUT.rglob("*")
        if path.is_file() and path != MANIFEST_PATH
    }
    if actual_files != set(declared_by_path):
        fail(errors, f"manifest.files inventory mismatch: missing={sorted(actual_files - set(declared_by_path))}, stale={sorted(set(declared_by_path) - actual_files)}")
    report["manifest_file_count"] = len(declared_by_path)
    report["actual_evidence_file_count"] = len(actual_files)

    # Source fingerprints are checked again after all seed runs, not trusted from a boolean marker.
    for target in targets:
        try:
            actual_sha = sha_file(Path(target["src_path"]))
            if actual_sha != target["src_sha256"]:
                fail(errors, f"target source changed since metadata snapshot: {target['src_path']}")
        except OSError as error:
            fail(errors, f"cannot re-hash target source {target.get('src_path')}: {error}")
    before_fingerprints = {}
    for command in commands:
        label = command.get("label", "unknown command")
        try:
            result_path = (RESULTS / command["result_path"]).resolve()
            result = json.loads(result_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        for record in result.get("before", []):
            source_path = Path(record.get("path", ""))
            try:
                actual = file_identity(source_path)
                if record.get("bytes") != actual["bytes"] or record.get("sha256") != actual["sha256"]:
                    fail(errors, f"{label}: source fingerprint changed: {source_path}")
                before_fingerprints.setdefault(str(source_path), set()).add(record.get("sha256"))
            except OSError as error:
                fail(errors, f"{label}: cannot verify source fingerprint {source_path}: {error}")
    for source_path, fingerprints in before_fingerprints.items():
        if len(fingerprints) != 1:
            fail(errors, f"source fingerprint differs across gate commands: {source_path}")

    before_paths = {
        str(ROOT / "crates/jarde-java/src/init.rs"),
        str(ROOT / "crates/jarde-java/src/report.rs"),
        str(ROOT / "tests/p3_constructor_primitive_conversion_arguments.rs"),
    }
    if set(before_fingerprints) != before_paths:
        fail(errors, f"gate before-fingerprint set differs from expected frozen inputs: {sorted(set(before_fingerprints) ^ before_paths)}")

    inherited_env = read_json(INHERITED_ENV_PATH, errors, "root inherited environment observation")
    if isinstance(inherited_env, dict):
        values = inherited_env.get("values", {})
        for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER",
                     "RUSTC_WORKSPACE_WRAPPER", "CARGO_TARGET_DIR"):
            if values.get(name) is not None:
                fail(errors, f"root inherited environment observation records {name}={values.get(name)!r}")
        report["root_inherited_environment_observation"] = {
            "path": str(INHERITED_ENV_PATH),
            "observation": inherited_env.get("observation"),
            "values": values,
            "limitation": "This is a contemporaneous root-shell observation, not a retroactive full child-process environment capture.",
        }

    report["verification"] = "passed" if not errors else "failed"
    report["limitation"] = (
        "The saved gate result records the explicitly configured environment but not every inherited host environment variable. "
        "Therefore this script can verify no opt-level override appears in the recorded configuration, but cannot independently "
        "prove that an unrecorded host variable was absent."
    )
    OUTPUT_PATH.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    summary = {
        "verification": report["verification"],
        "metadata_targets": report.get("metadata_target_count", 0),
        "partitions": report.get("expected_partition_count", 0),
        "commands": report.get("command_count", 0),
        "files": report.get("manifest_file_count", 0),
        "errors": errors,
        "output": str(OUTPUT_PATH),
    }
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
