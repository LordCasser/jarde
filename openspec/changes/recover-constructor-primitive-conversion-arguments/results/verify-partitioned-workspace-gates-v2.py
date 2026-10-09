#!/usr/bin/env python3
"""Independently verify v2 partitioned Cargo evidence and its reused v1 dependencies.

This verifier reads saved manifests and raw evidence only. It does not import a gate runner,
launch Cargo, or infer a successful run from a manifest boolean alone.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
OUT = RESULTS / "partitioned-workspace-gates-v2"
PRIOR_OUT = RESULTS / "partitioned-workspace-gates-v1"
MANIFEST_PATH = OUT / "manifest.json"
PRIOR_MANIFEST_PATH = PRIOR_OUT / "manifest.json"
OUTPUT_PATH = RESULTS / "partitioned-workspace-root-verification-v2.json"
RUNNER_PATH = RESULTS / "run-partitioned-workspace-gates-v2.py"
V1_HELPER_PATH = RESULTS / "run-partitioned-workspace-gates-v1.py"
GATE_RUNNER_PATH = RESULTS / "run-root-gate-v2.py"
NUMERIC_ARCHIVE_PATH = RESULTS / "numeric-test-source-before-clippy-v1.rs.txt"
CANDIDATE_IDENTITY_PATH = RESULTS / "candidate-cli-v1.json"
SEEDS = ("5350648285461741569", "5350648285461741570")
KINDS = {"lib", "bin", "example", "test", "bench"}
CHUNK_SIZE = 10
TEST_RESULT = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in .+$"
)
FREEZE_PATHS = (
    ROOT / "crates/jarde-java/src/init.rs",
    ROOT / "crates/jarde-java/src/report.rs",
    ROOT / "crates/jarde-java/src/build.rs",
    ROOT / "Cargo.lock",
)
PRODUCT_PATHS = {str(FREEZE_PATHS[0]), str(FREEZE_PATHS[1])}
SAMPLE_PATH = str(ROOT / "tests/p3_constructor_primitive_conversion_arguments.rs")


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def identity(path: Path) -> dict:
    data = path.read_bytes()
    return {"bytes": len(data), "sha256": sha_bytes(data)}


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def read_json(path: Path, errors: list[str], label: str):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as error:
        fail(errors, f"{label}: cannot read JSON {path}: {type(error).__name__}: {error}")
        return None


def contained_file(base: Path, relative: str) -> Path:
    path = (base / relative).resolve()
    if base.resolve() not in path.parents:
        raise ValueError(f"path escapes evidence directory: {relative}")
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"not a regular file: {path}")
    return path


def verify_file_record(record: dict, path: Path, errors: list[str], label: str) -> bool:
    if not isinstance(record, dict):
        fail(errors, f"{label}: missing file identity")
        return False
    try:
        actual = identity(path)
    except OSError as error:
        fail(errors, f"{label}: cannot read {path}: {error}")
        return False
    if record.get("bytes") != actual["bytes"] or record.get("sha256") != actual["sha256"]:
        fail(errors, f"{label}: recorded identity differs from {path}")
        return False
    return True


def target_key(row: dict) -> tuple:
    return row.get("package_id"), row.get("target_name"), tuple(row.get("kind") or [])


def metadata_target_rows(metadata: dict, errors: list[str]) -> list[dict]:
    member_ids = set(metadata.get("workspace_members", []))
    packages = [package for package in metadata.get("packages", []) if package.get("id") in member_ids]
    package_ids = {package.get("id") for package in packages}
    if member_ids != package_ids:
        fail(errors, f"metadata members lack package records: {sorted(member_ids - package_ids)}")
    rows = []
    for package in packages:
        for target in package.get("targets", []):
            source = target.get("src_path")
            row = {
                "package_id": package.get("id"), "package_name": package.get("name"),
                "target_name": target.get("name"), "kind": target.get("kind") or [],
                "src_path": source, "test": target.get("test"),
                "doctest": target.get("doctest"), "doc": target.get("doc"),
                "src_sha256": None,
            }
            if not isinstance(source, str) or not Path(source).is_file():
                fail(errors, f"metadata target source missing: {source!r}")
            else:
                row["src_sha256"] = sha_file(Path(source))
            if len(row["kind"]) != 1 or row["kind"][0] not in KINDS:
                fail(errors, f"unsupported metadata target kind: {row}")
            elif row["test"] is False and row["kind"] != ["example"]:
                fail(errors, f"test=false target is not an example: {row}")
            elif row["kind"] == ["test"] and row["test"] is not True:
                fail(errors, f"integration target is not test-enabled: {row}")
            rows.append(row)
    return sorted(rows, key=lambda row: (row["package_id"], row["target_name"], row["kind"]))


def selection_record(row: dict) -> dict:
    return {key: row.get(key) for key in (
        "package_id", "package_name", "target_name", "kind", "src_path", "test", "doctest")}


def expected_groups(targets: list[dict]) -> list[dict]:
    groups = []
    present = {row["kind"][0] for row in targets if len(row["kind"]) == 1}
    for kind, flag in (("lib", "--lib"), ("bin", "--bins"),
                       ("example", "--examples"), ("bench", "--benches")):
        if kind in present:
            selected = [row for row in targets if row["kind"] == [kind]]
            groups.append({"id": kind, "kind": kind,
                           "target_names": sorted({row["target_name"] for row in selected}),
                           "cargo_args": [flag], "targets": selected})
    test_names = sorted({row["target_name"] for row in targets if row["kind"] == ["test"]})
    for offset in range(0, len(test_names), CHUNK_SIZE):
        names = test_names[offset:offset + CHUNK_SIZE]
        selected = [row for row in targets
                    if row["kind"] == ["test"] and row["target_name"] in names]
        groups.append({"id": f"tests-{offset // CHUNK_SIZE:03d}", "kind": "test",
                       "target_names": names,
                       "cargo_args": [arg for name in names for arg in ("--test", name)],
                       "targets": selected})
    return groups


def parse_test_results(text: str, errors: list[str], label: str) -> dict:
    rows = []
    for number, line in enumerate(text.splitlines(), 1):
        if "test result:" not in line:
            continue
        match = TEST_RESULT.fullmatch(line.strip())
        if not match:
            fail(errors, f"{label}: unrecognized test-result row {number}: {line}")
            continue
        status, passed, failed_count, ignored, measured, filtered = match.groups()
        row = {"status": status, "passed": int(passed), "failed": int(failed_count),
               "ignored": int(ignored), "measured": int(measured), "filtered_out": int(filtered)}
        rows.append(row)
        if status != "ok" or row["failed"]:
            fail(errors, f"{label}: failed test-result row: {row}")
    return {"result_rows": len(rows),
            **{name: sum(row[name] for row in rows) for name in
               ("passed", "failed", "ignored", "measured", "filtered_out")},
            "rows": rows}


def verify_closed_inventory(out: Path, manifest_path: Path, manifest: dict,
                            errors: list[str], label: str) -> dict[str, dict]:
    listed = {}
    for record in manifest.get("files", []):
        relative = record.get("path")
        if not isinstance(relative, str) or relative in listed:
            fail(errors, f"{label}: malformed or duplicate file entry {record}")
            continue
        try:
            path = contained_file(out, relative)
            verify_file_record(record, path, errors, f"{label} file {relative}")
            listed[relative] = record
        except Exception as error:
            fail(errors, f"{label}: invalid listed file {relative!r}: {error}")
    actual = set()
    for path in out.rglob("*"):
        if path.is_symlink():
            fail(errors, f"{label}: symlink in evidence tree: {path}")
        elif path.is_file() and path != manifest_path:
            actual.add(path.relative_to(out).as_posix())
    if actual != set(listed):
        fail(errors, f"{label}: inventory is not closed: missing={sorted(actual - set(listed))}, "
                      f"stale={sorted(set(listed) - actual)}")
    return listed


def verify_prior_closed_inventory(prior: dict, errors: list[str]) -> dict[str, dict]:
    listed = {}
    for record in prior.get("files", []):
        relative = record.get("path")
        if not isinstance(relative, str) or relative in listed:
            fail(errors, f"v1: malformed or duplicate file entry {record}")
            continue
        try:
            path = contained_file(PRIOR_OUT, relative)
            verify_file_record(record, path, errors, f"v1 evidence {relative}")
            listed[relative] = record
        except Exception as error:
            fail(errors, f"v1: invalid listed file {relative!r}: {error}")
    actual = set()
    for path in PRIOR_OUT.rglob("*"):
        if path.is_symlink():
            fail(errors, f"v1: symlink in old evidence tree: {path}")
        elif path.is_file() and path != PRIOR_MANIFEST_PATH:
            actual.add(path.relative_to(PRIOR_OUT).as_posix())
    if actual != set(listed):
        fail(errors, f"v1: old inventory is not closed: missing={sorted(actual - set(listed))}, "
                      f"stale={sorted(set(listed) - actual)}")
    return listed


def command_result(command: dict, errors: list[str], label: str) -> tuple[dict | None, Path | None]:
    try:
        path = contained_file(RESULTS, command["result_path"])
        raw = path.read_bytes()
        if sha_bytes(raw) != command.get("result_sha256"):
            fail(errors, f"{label}: result sha differs from command record")
        return json.loads(raw), path
    except Exception as error:
        fail(errors, f"{label}: cannot verify result JSON: {type(error).__name__}: {error}")
        return None, None


def verify_gate_result(command: dict, result: dict, result_path: Path, group: dict,
                       seed: str, expected_label: str, gate_sha: str, errors: list[str],
                       label: str) -> dict:
    expected_argv = ["cargo", "test", "--workspace", "--all-features", "--locked",
                     *group["cargo_args"]]
    if command.get("cargo_argv") != expected_argv:
        fail(errors, f"{label}: cargo argv does not match exact metadata selection")
    if command.get("exit") != 0 or command.get("runner_exit") != 0 or command.get("wrapper_process_exit") != 0:
        fail(errors, f"{label}: command/wrapper exits are not all zero")
    if command.get("disk_stop_reason") is not None:
        fail(errors, f"{label}: disk guard stopped the command")
    if result.get("schema") != "root-command-v2-disk-guard":
        fail(errors, f"{label}: unexpected gate result schema")
    if result.get("argv") != expected_argv or result.get("cwd") != str(ROOT):
        fail(errors, f"{label}: raw result argv/cwd mismatch")
    if result.get("exit") != 0 or result.get("runner_exit") != 0 or result.get("disk_stop_reason") is not None:
        fail(errors, f"{label}: raw gate result not clean")
    if result.get("env", {}).get("PROPTEST_RNG_SEED") != seed:
        fail(errors, f"{label}: raw result seed differs")
    if result.get("runner", {}).get("path") != str(GATE_RUNNER_PATH):
        fail(errors, f"{label}: gate runner path mismatch")
    if result.get("runner", {}).get("sha256") != gate_sha:
        fail(errors, f"{label}: gate runner hash mismatch")
    expected_env = {"CARGO_BUILD_JOBS": "1", "CARGO_INCREMENTAL": "0",
                    "RUST_TEST_THREADS": "1", "CARGO_PROFILE_DEV_DEBUG": "0",
                    "CARGO_PROFILE_TEST_DEBUG": "0", "CARGO_PROFILE_DEV_STRIP": "symbols",
                    "CARGO_PROFILE_TEST_STRIP": "symbols", "PROPTEST_RNG_SEED": seed}
    env = result.get("env", {})
    for key, value in expected_env.items():
        if env.get(key) != value:
            fail(errors, f"{label}: configured env {key} differs from {value!r}")
    opt_keys = sorted(key for key in env if key.startswith("CARGO_PROFILE_") and key.endswith("_OPT_LEVEL"))
    if opt_keys:
        fail(errors, f"{label}: opt-level overrides recorded: {opt_keys}")
    profile = result.get("profile_settings", {})
    if profile.get("opt_level") != "Cargo default; not overridden":
        fail(errors, f"{label}: opt-level default statement differs")
    if profile.get("debug_assertions") != "Cargo default remains enabled; not overridden":
        fail(errors, f"{label}: debug-assertion statement differs")
    if re.search(r"(?:^|\s)-C\s*opt-level(?:=|\s)", env.get("RUSTFLAGS", "")):
        fail(errors, f"{label}: recorded RUSTFLAGS overrides opt-level")
    for stream_name in ("stdout", "stderr"):
        record = result.get(stream_name, {})
        actual_path = result_path.parent / stream_name
        if Path(record.get("path", "")).resolve() != actual_path.resolve():
            fail(errors, f"{label}: {stream_name} path is not beside result JSON")
        verify_file_record(record, actual_path, errors, f"{label} raw {stream_name}")
    stdout_path = result_path.parent / "stdout"
    parsed = parse_test_results(stdout_path.read_text(encoding="utf-8", errors="replace"), errors, label)
    if parsed["result_rows"] != len(group["targets"]):
        fail(errors, f"{label}: {parsed['result_rows']} test-result rows for {len(group['targets'])} selected targets")
    # The gate records configured variables, not every inherited host variable.
    if not result.get("effective_environment_captured"):
        pass
    wrapper_record = command.get("wrapper_stdout", {})
    wrapper_path = (OUT / wrapper_record.get("path", "")).resolve()
    if OUT.resolve() not in wrapper_path.parents:
        fail(errors, f"{label}: wrapper stdout path escapes v2 evidence")
    verify_file_record(wrapper_record, wrapper_path, errors, f"{label} wrapper stdout")
    wrapper_stderr_record = command.get("wrapper_stderr", {})
    wrapper_stderr_path = (OUT / wrapper_stderr_record.get("path", "")).resolve()
    if OUT.resolve() not in wrapper_stderr_path.parents:
        fail(errors, f"{label}: wrapper stderr path escapes v2 evidence")
    verify_file_record(wrapper_stderr_record, wrapper_stderr_path, errors, f"{label} wrapper stderr")
    try:
        wrapper_rows = [json.loads(line) for line in wrapper_path.read_text(
            encoding="utf-8", errors="replace").splitlines() if line.strip()]
        if len(wrapper_rows) != 1:
            fail(errors, f"{label}: wrapper stdout must contain exactly one result row")
        else:
            wrapper = wrapper_rows[0]
            if (wrapper.get("label") != expected_label or wrapper.get("exit") != 0
                    or wrapper.get("runner_exit") != 0 or wrapper.get("disk_stop_reason") is not None):
                fail(errors, f"{label}: wrapper stdout identity/result mismatch")
    except Exception as error:
        fail(errors, f"{label}: cannot parse wrapper stdout: {error}")
    return parsed


def verify_old_before_snapshots(prior: dict, errors: list[str]) -> dict:
    prior_target_by_path = {row.get("src_path"): row for row in prior.get("workspace_targets", [])}
    archived = identity(NUMERIC_ARCHIVE_PATH) if NUMERIC_ARCHIVE_PATH.is_file() else None
    if archived is None or NUMERIC_ARCHIVE_PATH.is_symlink():
        fail(errors, "historical numeric source archive is missing or unsafe")
        return {}
    sampled = {}
    product_records = {}
    seen_before_sets = []
    for command in prior.get("commands", []):
        result, _ = command_result(command, errors, f"v1 {command.get('label')}")
        if not isinstance(result, dict):
            continue
        entries = result.get("before", [])
        seen_before_sets.append({entry.get("path") for entry in entries})
        for entry in entries:
            path_string = entry.get("path")
            if path_string in PRODUCT_PATHS:
                path = Path(path_string)
                if not path.is_file() or path.is_symlink():
                    fail(errors, f"v1 before product missing/unsafe: {path}")
                    continue
                actual = {"path": path_string, **identity(path)}
                if actual != {key: entry.get(key) for key in ("path", "bytes", "sha256")}:
                    fail(errors, f"v1 before product fingerprint mismatch: {path}")
                if path_string in product_records and product_records[path_string] != actual:
                    fail(errors, f"v1 product fingerprint varies between commands: {path}")
                product_records[path_string] = actual
            elif path_string == SAMPLE_PATH:
                old_target = prior_target_by_path.get(path_string)
                expected = {key: entry.get(key) for key in ("path", "bytes", "sha256")}
                if (not old_target or old_target.get("src_sha256") != entry.get("sha256")
                        or archived["sha256"] != entry.get("sha256")
                        or archived["bytes"] != entry.get("bytes")):
                    fail(errors, "v1 numeric before fingerprint is not bound to archived source and old target")
                sampled[path_string] = expected
            else:
                fail(errors, f"v1 command recorded unexpected before-source path: {path_string}")
    expected_before = PRODUCT_PATHS | {SAMPLE_PATH}
    if not seen_before_sets or any(paths != expected_before for paths in seen_before_sets):
        fail(errors, "v1 command before-fingerprint sets do not consistently contain init/report/numeric test")
    if set(product_records) != PRODUCT_PATHS:
        fail(errors, "v1 before evidence did not consistently bind both product source files")
    current_sample = Path(SAMPLE_PATH)
    current_sample_record = {"path": SAMPLE_PATH, **identity(current_sample)} if current_sample.is_file() else None
    if current_sample_record is None:
        fail(errors, "current numeric test source is missing")
    return {"product_sources": [product_records[path] for path in sorted(product_records)],
            "sampled_target_original": sampled.get(SAMPLE_PATH),
            "sampled_target_current": current_sample_record,
            "sampled_target_archive": {"path": NUMERIC_ARCHIVE_PATH.name, **archived}}


def verify_prior_commands(prior: dict, old_groups: dict, old_targets: dict,
                          errors: list[str]) -> dict:
    commands_by_label = {}
    for command in prior.get("commands", []):
        label = command.get("label")
        if label in commands_by_label:
            fail(errors, f"v1 has duplicate command label: {label}")
        commands_by_label[label] = command
    prior_summary = {}
    for group_id, group in old_groups.items():
        targets = group.get("targets", [])
        seeds = group.get("seed_runs", [])
        clean = len(seeds) == len(SEEDS) and [str(row.get("seed")) for row in seeds] == list(SEEDS)
        seed_details = []
        expected_args = group.get("cargo_args")
        if not isinstance(expected_args, list):
            expected_args = []
        for seed_row in seeds:
            seed = str(seed_row.get("seed"))
            label = seed_row.get("label")
            expected_label = f"partitioned-workspace-gates-v1/{group_id}/seed-{seed}"
            command = commands_by_label.get(label)
            if label != expected_label or not command or seed not in SEEDS:
                clean = False
                fail(errors, f"v1 {group_id}: missing exact command/seed identity {label!r}")
                continue
            if command.get("fresh_execution") is not None:
                fail(errors, f"v1 {label}: unexpected v2 execution marker in historical command")
            if command.get("argv", [None, None, None])[-len(command.get("cargo_argv", [])):] != command.get("cargo_argv"):
                fail(errors, f"v1 {label}: wrapper argv tail differs from Cargo argv")
            expected_argv = ["cargo", "test", "--workspace", "--all-features", "--locked", *expected_args]
            if command.get("cargo_argv") != expected_argv:
                clean = False
                fail(errors, f"v1 {label}: old concrete target selection differs")
            result, result_path = command_result(command, errors, f"v1 {label}")
            successful = bool(command.get("exit") == command.get("runner_exit") ==
                              command.get("wrapper_process_exit") == 0 and
                              command.get("disk_stop_reason") is None and
                              seed_row.get("exit") == seed_row.get("runner_exit") == 0 and
                              seed_row.get("disk_stop_reason") is None and
                              result and result.get("exit") == result.get("runner_exit") == 0 and
                              result.get("disk_stop_reason") is None)
            if not successful:
                clean = False
            if seed_row.get("result_path") != command.get("result_path") or seed_row.get("result_sha256") != command.get("result_sha256"):
                clean = False
                fail(errors, f"v1 {label}: partition and command result identity differs")
            if result is not None and result_path is not None:
                if result.get("env", {}).get("PROPTEST_RNG_SEED") != seed:
                    clean = False
                    fail(errors, f"v1 {label}: old raw seed differs")
                for stream_name in ("stdout", "stderr"):
                    stream = result.get(stream_name, {})
                    try:
                        stream_path = Path(stream.get("path", ""))
                        if not stream_path.is_relative_to(PRIOR_OUT) and not stream_path.resolve().is_relative_to(PRIOR_OUT.resolve()):
                            raise ValueError("raw stream is outside old evidence tree")
                        verify_file_record(stream, stream_path, errors, f"v1 {label} raw {stream_name}")
                    except Exception as error:
                        clean = False
                        fail(errors, f"v1 {label}: invalid raw {stream_name}: {error}")
                historical_parse_errors = []
                parsed = parse_test_results((result_path.parent / "stdout").read_text(
                    encoding="utf-8", errors="replace"), historical_parse_errors, f"v1 {label}")
                if parsed["result_rows"] != len(targets):
                    clean = False
                if historical_parse_errors:
                    clean = False
            for stream_name in ("wrapper_stdout", "wrapper_stderr"):
                rec = command.get(stream_name, {})
                try:
                    path = contained_file(PRIOR_OUT, rec.get("path", ""))
                    verify_file_record(rec, path, errors, f"v1 {label} {stream_name}")
                    if stream_name == "wrapper_stdout":
                        rows = [json.loads(line) for line in path.read_text(
                            encoding="utf-8", errors="replace").splitlines() if line.strip()]
                        if len(rows) != 1 or rows[0].get("label") != label:
                            raise ValueError("old wrapper output label/row count mismatch")
                        if successful and (rows[0].get("exit") != 0 or rows[0].get("runner_exit") != 0):
                            raise ValueError("old wrapper output does not report success")
                except Exception as error:
                    clean = False
                    fail(errors, f"v1 {label}: invalid wrapper evidence: {error}")
            seed_details.append({"seed": seed, "label": label, "command": command,
                                 "seed_record": seed_row, "result": result,
                                 "successful": successful})
        selected_keys = {target_key(row) for row in targets}
        if any(key not in old_targets for key in selected_keys):
            clean = False
            fail(errors, f"v1 {group_id}: selected source identity absent from old metadata targets")
        prior_summary[group_id] = {"clean_both_seeds": clean, "seed_details": seed_details,
                                   "target_keys": selected_keys}
    return prior_summary


def main() -> int:
    errors: list[str] = []
    warnings = ["Gate records explicitly configured variables, not every inherited host variable; "
                "absence of an unrecorded host override cannot be proven retroactively."]
    report = {"schema": "partitioned-workspace-root-verification-v2", "verification": "not-run",
              "manifest_path": str(MANIFEST_PATH), "seeds": list(SEEDS), "errors": errors,
              "warnings": warnings}
    manifest = read_json(MANIFEST_PATH, errors, "v2 manifest")
    prior = read_json(PRIOR_MANIFEST_PATH, errors, "v1 prior manifest")
    if not isinstance(manifest, dict) or not isinstance(prior, dict):
        report["verification"] = "failed"
        OUTPUT_PATH.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(json.dumps({"verification": "failed", "errors": errors}, ensure_ascii=False))
        return 1

    # Establish the old evidence base before accepting any reused v1 result.
    prior_file_map = verify_prior_closed_inventory(prior, errors)
    report["prior_file_count"] = len(prior_file_map)
    if prior.get("schema") != "partitioned-workspace-gates-v1":
        fail(errors, f"unexpected old manifest schema: {prior.get('schema')!r}")
    if len(prior_file_map) != 296:
        fail(errors, f"old v1 evidence inventory has {len(prior_file_map)} files, expected pinned 296")
    if manifest.get("prior_file_closure") != {"files_checked": len(prior_file_map), "closed_set": True}:
        fail(errors, "v2 prior_file_closure does not exactly report verified old closed inventory")
    try:
        prior_manifest_sha = sha_file(PRIOR_MANIFEST_PATH)
        if manifest.get("prior_manifest_sha256") != prior_manifest_sha:
            fail(errors, "v2 prior manifest hash is not bound to actual v1 manifest")
        if manifest.get("v1_helper_path") != str(V1_HELPER_PATH):
            fail(errors, "v2 v1_helper_path differs from expected frozen helper")
        v1_helper_sha = sha_file(V1_HELPER_PATH)
        if (manifest.get("v1_helper_sha256") != v1_helper_sha
                or prior.get("runner_sha256") != v1_helper_sha):
            fail(errors, "v1 helper identity is not consistent across current helper and both manifests")
        if manifest.get("prior_manifest_path") != str(PRIOR_MANIFEST_PATH):
            fail(errors, "v2 prior manifest path differs")
        if manifest.get("prior_numeric_source_archive_path") != str(NUMERIC_ARCHIVE_PATH):
            fail(errors, "v2 numeric archive path differs")
        archive_sha = sha_file(NUMERIC_ARCHIVE_PATH)
        if manifest.get("prior_numeric_source_archive_sha256") != archive_sha:
            fail(errors, "v2 numeric archive SHA differs from actual archived source")
    except OSError as error:
        fail(errors, f"cannot verify prior helper/archive identity: {error}")
        archive_sha = None
    prior_freeze = verify_old_before_snapshots(prior, errors)
    if manifest.get("prior_current_freeze") != {
        "product_sources": prior_freeze.get("product_sources"),
        "sampled_target_source": {
            "path": SAMPLE_PATH,
            "prior_src_sha256": (prior_freeze.get("sampled_target_original") or {}).get("sha256"),
            "current_src_sha256": (prior_freeze.get("sampled_target_current") or {}).get("sha256"),
            "current_bytes": (prior_freeze.get("sampled_target_current") or {}).get("bytes"),
            "matches_prior": ((prior_freeze.get("sampled_target_current") or {}).get("sha256") ==
                              (prior_freeze.get("sampled_target_original") or {}).get("sha256")),
            "role": "gate-wide sampled integration source; its target group is independently hash-checked",
        },
        "sampled_target_original_bytes": {
            "path": NUMERIC_ARCHIVE_PATH.name,
            "bytes": (prior_freeze.get("sampled_target_archive") or {}).get("bytes"),
            "sha256": (prior_freeze.get("sampled_target_archive") or {}).get("sha256"),
        },
    }:
        fail(errors, "v2 prior_current_freeze does not match independently verified prior source/archive facts")
    if manifest.get("prior_current_freeze_verified") is not True:
        fail(errors, "v2 did not record successful prior-current-freeze verification")

    # Validate the frozen baseline candidate metadata using its actual field names.
    candidate = read_json(CANDIDATE_IDENTITY_PATH, errors, "frozen candidate CLI metadata")
    if isinstance(candidate, dict):
        report["candidate_cli_identity"] = {key: candidate.get(key) for key in
            ("cli_path", "cli_sha256", "source_commit_base", "build_result", "build_result_sha256")}
        candidate_path = Path(candidate.get("cli_path", ""))
        if not candidate_path.is_file() or candidate_path.is_symlink():
            fail(errors, "frozen baseline CLI binary is missing or unsafe")
        elif sha_file(candidate_path) != candidate.get("cli_sha256"):
            fail(errors, "frozen baseline CLI binary SHA differs from candidate metadata")
        at_start = {row.get("path"): row for row in manifest.get("workspace_freeze", {}).get("at_start", [])}
        for relative, expected_sha in candidate.get("candidate_sources", {}).items():
            absolute = str(ROOT / relative)
            row = at_start.get(absolute)
            if not row or row.get("sha256") != expected_sha:
                fail(errors, f"baseline candidate source identity differs from v2 start freeze: {relative}")

    if manifest.get("schema") != "partitioned-workspace-gates-v2":
        fail(errors, f"unexpected v2 manifest schema: {manifest.get('schema')!r}")
    if manifest.get("status") != "complete" or manifest.get("coverage_complete") is not True:
        fail(errors, "v2 manifest does not record complete metadata-target coverage")
    if manifest.get("unsupported_metadata"):
        fail(errors, "v2 reports unsupported Cargo metadata targets")

    # Pin the current runner and common gate runner identities.
    for path_field, sha_field, path in (("runner_path", "runner_sha256", RUNNER_PATH),
                                        ("gate_runner_path", "gate_runner_sha256", GATE_RUNNER_PATH)):
        if manifest.get(path_field) != str(path):
            fail(errors, f"manifest {path_field} differs from expected path")
        try:
            if manifest.get(sha_field) != sha_file(path):
                fail(errors, f"manifest {sha_field} does not match current file")
        except OSError as error:
            fail(errors, f"cannot hash {path}: {error}")
    if prior.get("gate_runner_sha256") != manifest.get("gate_runner_sha256"):
        fail(errors, "old and current gate runner identities differ")

    # V2 must freeze all four product/build inputs from start through end and current disk.
    freeze = manifest.get("workspace_freeze", {})
    start_rows = freeze.get("at_start") or []
    end_rows = freeze.get("at_end") or []
    expected_freeze_paths = [str(path) for path in FREEZE_PATHS]
    if [row.get("path") for row in start_rows] != expected_freeze_paths:
        fail(errors, "workspace at_start does not list the exact ordered four-file freeze")
    if [row.get("path") for row in end_rows] != expected_freeze_paths:
        fail(errors, "workspace at_end does not list the exact ordered four-file freeze")
    if start_rows != end_rows:
        fail(errors, "workspace freeze changed between start and end")
    for row, path in zip(start_rows, FREEZE_PATHS):
        if not path.is_file() or path.is_symlink() or row != {"path": str(path), **identity(path)}:
            fail(errors, f"current freeze differs from v2 recorded start/end: {path}")

    # Independently re-derive the fresh metadata target set and expected partitioning.
    metadata_record = manifest.get("metadata", {})
    metadata_command = ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"]
    if metadata_record.get("command") != metadata_command or metadata_record.get("argv") != metadata_command:
        fail(errors, "metadata command is not the locked --no-deps snapshot")
    if metadata_record.get("exit") != 0 or metadata_record.get("cwd") != str(ROOT):
        fail(errors, "metadata command did not complete successfully in this checkout")
    meta_stdout_record = metadata_record.get("stdout", {})
    meta_stderr_record = metadata_record.get("stderr", {})
    try:
        metadata_stdout_path = contained_file(OUT, meta_stdout_record.get("path", ""))
        metadata_stderr_path = contained_file(OUT, meta_stderr_record.get("path", ""))
        verify_file_record(meta_stdout_record, metadata_stdout_path, errors, "v2 metadata stdout")
        verify_file_record(meta_stderr_record, metadata_stderr_path, errors, "v2 metadata stderr")
        metadata = json.loads(metadata_stdout_path.read_text(encoding="utf-8"))
    except Exception as error:
        fail(errors, f"cannot load captured Cargo metadata: {error}")
        metadata = {}
    if (Path(metadata.get("workspace_root", "")).resolve() != ROOT.resolve()
            or Path(metadata.get("target_directory", "")).resolve() != (ROOT / "target").resolve()):
        fail(errors, "captured metadata workspace/target directory differs from checkout")
    if manifest.get("workspace_root") != str(ROOT) or manifest.get("workspace_members") != metadata.get("workspace_members"):
        fail(errors, "manifest workspace root/members differ from captured metadata")
    targets = metadata_target_rows(metadata, errors)
    target_count = len(targets)
    report["metadata_target_count"] = target_count
    expected_groups_list = expected_groups(targets)
    report["expected_partition_count"] = len(expected_groups_list)
    if len({target_key(row) for row in targets}) != target_count:
        fail(errors, "fresh metadata has duplicate package/target/kind identities")
    target_fields = ("package_id", "package_name", "target_name", "kind", "src_path", "test",
                     "doctest", "doc", "src_sha256")
    expected_target_rows = [{field: row.get(field) for field in target_fields} for row in targets]
    actual_target_rows = [{field: row.get(field) for field in target_fields}
                          for row in manifest.get("workspace_targets", [])]
    if actual_target_rows != expected_target_rows:
        fail(errors, "v2 workspace target identities/source hashes differ from captured fresh metadata")

    old_targets = {target_key(row): row for row in prior.get("workspace_targets", [])}
    old_groups = {group.get("id"): group for group in prior.get("partitions", [])}
    prior_checks = verify_prior_commands(prior, old_groups, old_targets, errors)
    current_targets = {target_key(row): row for row in targets}
    changed = []
    for key, old in old_targets.items():
        new = current_targets.get(key)
        if not new or old.get("src_path") != new.get("src_path") or old.get("src_sha256") != new.get("src_sha256"):
            changed.append({"target_key": [key[0], key[1], list(key[2])],
                            "prior_src_path": old.get("src_path"), "prior_src_sha256": old.get("src_sha256"),
                            "current_src_path": new.get("src_path") if new else None,
                            "current_src_sha256": new.get("src_sha256") if new else None})
    if manifest.get("source_changes_since_v1") != changed:
        fail(errors, "source_changes_since_v1 differs from independently compared metadata targets")
    changed_keys = {tuple((row["target_key"][0], row["target_key"][1], tuple(row["target_key"][2])))
                    for row in changed}
    changed_group_ids = {group["id"] for group in expected_groups_list
                         if any(target_key(row) in changed_keys for row in group["targets"])}
    report["changed_source_target_count"] = len(changed)
    report["changed_source_groups"] = sorted(changed_group_ids)

    # Historical before snapshots are gate-wide samples. Independently require each current target
    # source hash, and only bind the historical sample to its archive and own old metadata target.
    command_by_key = {}
    commands = manifest.get("commands", [])
    for command in commands:
        key = (command.get("label"), str(command.get("seed")))
        if key in command_by_key:
            fail(errors, f"duplicate v2 command key: {key}")
        command_by_key[key] = command
    partitions = manifest.get("partitions", [])
    if len(partitions) != len(expected_groups_list):
        fail(errors, "v2 partition count differs from fresh metadata selection")
    if len(commands) != len(expected_groups_list) * len(SEEDS):
        fail(errors, "v2 command count does not close over partitions × seeds")

    actual_group_by_id = {group.get("id"): group for group in partitions}
    if len(actual_group_by_id) != len(partitions):
        fail(errors, "v2 has duplicate partition identities")
    old_command_labels = {command.get("label") for command in prior.get("commands", [])}
    expected_run_by_group_seed = {}
    seed_totals = {seed: {"result_rows": 0, "passed": 0, "failed": 0, "ignored": 0,
                          "measured": 0, "filtered_out": 0, "partition_count": 0}
                   for seed in SEEDS}
    used_command_keys = set()

    for index, group in enumerate(expected_groups_list):
        observed = actual_group_by_id.get(group["id"], {})
        if index >= len(partitions) or partitions[index].get("id") != group["id"]:
            fail(errors, f"partition order/identity mismatch at {group['id']}")
        for field in ("kind", "target_names", "cargo_args"):
            if observed.get(field) != group.get(field):
                fail(errors, f"{group['id']}: {field} differs from independently derived selection")
        expected_selected = [{**selection_record(row), "src_sha256": row["src_sha256"]}
                             for row in group["targets"]]
        if observed.get("targets") != expected_selected:
            fail(errors, f"{group['id']}: selected target/source hashes differ from metadata")
        seed_runs = observed.get("seed_runs", [])
        if [str(row.get("seed")) for row in seed_runs] != list(SEEDS):
            fail(errors, f"{group['id']}: exact ordered seed pair missing")
        is_reused = isinstance(observed.get("reused_from"), dict)
        if is_reused:
            reuse_review = observed.get("reuse_review", {})
            if reuse_review.get("eligible") is not True:
                fail(errors, f"{group['id']}: reused group lacks positive eligibility review")
            if group["id"] in changed_group_ids:
                fail(errors, f"{group['id']}: source-changed group was incorrectly reused")
            prior_info = prior_checks.get(group["id"], {})
            if prior_info.get("clean_both_seeds") is not True:
                fail(errors, f"{group['id']}: reused group was not a verified clean two-seed v1 group")
            old_group = old_groups.get(group["id"], {})
            if observed["reused_from"].get("manifest") != str(PRIOR_MANIFEST_PATH.relative_to(RESULTS)):
                fail(errors, f"{group['id']}: reuse references an unexpected old manifest")
            if (observed["reused_from"].get("prior_group") != group["id"]
                    or observed["reused_from"].get("target_names") != group["target_names"]
                    or observed["reused_from"].get("cargo_args") != group["cargo_args"]):
                fail(errors, f"{group['id']}: reuse identity/selection does not match old group")
            expected_old_labels = [row.get("label") for row in old_group.get("seed_runs", [])]
            if observed["reused_from"].get("labels") != expected_old_labels:
                fail(errors, f"{group['id']}: reused labels differ from old concrete command labels")
            for row in group["targets"]:
                old = old_targets.get(target_key(row))
                if not old or old.get("src_path") != row.get("src_path") or old.get("src_sha256") != row.get("src_sha256"):
                    fail(errors, f"{group['id']}: reused target source differs from old/current hash: {target_key(row)}")
        else:
            prior_info = prior_checks.get(group["id"], {})
            if group["id"] in changed_group_ids and any(not row.get("fresh_execution") for row in seed_runs):
                fail(errors, f"{group['id']}: changed-source group is not entirely fresh")
            if prior_info.get("clean_both_seeds") and group["id"] not in changed_group_ids:
                old_group = old_groups.get(group["id"], {})
                old_keys = {target_key(row) for row in old_group.get("targets", [])}
                selected_keys = {target_key(row) for row in group["targets"]}
                same_sources = old_keys == selected_keys and all(
                    old_targets.get(key, {}).get("src_path") == current_targets.get(key, {}).get("src_path")
                    and old_targets.get(key, {}).get("src_sha256") == current_targets.get(key, {}).get("src_sha256")
                    for key in selected_keys)
                if same_sources:
                    fail(errors, f"{group['id']}: eligible unchanged clean v1 group was rerun instead of reused")

        for seed, seed_row in zip(SEEDS, seed_runs):
            command_key = (seed_row.get("label"), seed)
            command = command_by_key.get(command_key)
            if command is None:
                fail(errors, f"{group['id']}: missing command for seed {seed}: {command_key}")
                continue
            used_command_keys.add(command_key)
            label = command_key[0]
            expected_label = (f"partitioned-workspace-gates-v1/{group['id']}/seed-{seed}"
                              if is_reused else f"partitioned-workspace-gates-v2/{group['id']}/seed-{seed}")
            if label != expected_label or seed_row.get("label") != label:
                fail(errors, f"{group['id']}: label is not bound to the correct old-v1/new-v2 source")
            marker = command.get("fresh_execution")
            if marker is not (not is_reused) or seed_row.get("fresh_execution") is not (not is_reused):
                fail(errors, f"{label}: fresh_execution markers conflict with execution origin")
            if is_reused:
                old_command = next((row for row in prior.get("commands", []) if row.get("label") == label), None)
                old_seed = next((row for row in old_groups[group["id"]].get("seed_runs", [])
                                 if str(row.get("seed")) == seed), None)
                if not old_command or not old_seed:
                    fail(errors, f"{label}: old command/seed row missing")
                    continue
                for field in ("argv", "cargo_argv", "exit", "runner_exit", "disk_stop_reason",
                              "min_free", "result_path", "result_sha256", "wrapper_process_exit"):
                    if command.get(field) != old_command.get(field):
                        fail(errors, f"{label}: reused {field} differs from original v1 command")
                if (seed_row.get("result_path") != old_seed.get("result_path")
                        or seed_row.get("result_sha256") != old_seed.get("result_sha256")
                        or seed_row.get("min_free") != old_seed.get("min_free")
                        or seed_row.get("disk_stop_reason") != old_seed.get("disk_stop_reason")):
                    fail(errors, f"{label}: reused seed row differs from original v1 identity/resources")
                old_result, _ = command_result(old_command, errors, f"old {label}")
                if old_result is not None:
                    expected_reused_from = {"partition": "partitioned-workspace-gates-v1",
                                            "result_path": old_command.get("result_path"),
                                            "result_sha256": old_command.get("result_sha256"),
                                            "raw_streams": {key: old_result.get(key) for key in ("stdout", "stderr")}}
                    if command.get("reused_from") != expected_reused_from:
                        fail(errors, f"{label}: reused_from raw/result identity differs from original v1")
                for stream_name in ("wrapper_stdout", "wrapper_stderr"):
                    old_rec = old_command.get(stream_name, {})
                    old_path = contained_file(PRIOR_OUT, old_rec.get("path", ""))
                    new_rec = command.get(stream_name, {})
                    new_path = contained_file(OUT, new_rec.get("path", ""))
                    if old_path.read_bytes() != new_path.read_bytes():
                        fail(errors, f"{label}: copied {stream_name} does not exactly match old raw wrapper output")
                    verify_file_record(new_rec, new_path, errors, f"{label} copied {stream_name}")
                if command.get("cargo_argv") != old_command.get("cargo_argv"):
                    fail(errors, f"{label}: reused target selection command is not the old concrete argv")
            else:
                if command.get("reused_from") is not None:
                    fail(errors, f"{label}: fresh command carries reused_from identity")
                if label in old_command_labels:
                    fail(errors, f"{label}: fresh v2 execution uses an old v1 label")
                argv = command.get("argv", [])
                expected_cargo = ["cargo", "test", "--workspace", "--all-features", "--locked",
                                  *group["cargo_args"]]
                if (len(argv) != 3 + len(expected_cargo) or argv[1] != str(GATE_RUNNER_PATH)
                        or argv[2] != label or argv[3:] != expected_cargo):
                    fail(errors, f"{label}: fresh wrapper argv differs from exact selected cargo command")
            result, result_path = command_result(command, errors, label)
            if result is None or result_path is None:
                continue
            expected_result_root = PRIOR_OUT if is_reused else OUT
            if not result_path.resolve().is_relative_to(expected_result_root.resolve()):
                fail(errors, f"{label}: result path is outside its declared execution-origin tree")
            if seed_row.get("result_path") != command.get("result_path") or seed_row.get("result_sha256") != command.get("result_sha256"):
                fail(errors, f"{label}: partition seed result identity differs from command")
            parsed = verify_gate_result(command, result, result_path, group, seed, label,
                                        manifest.get("gate_runner_sha256"), errors, label)
            expected_run_by_group_seed[(group["id"], seed)] = (command, seed_row, parsed)
            total = seed_totals[seed]
            for metric in ("result_rows", "passed", "failed", "ignored", "measured", "filtered_out"):
                total[metric] += parsed[metric]
            total["partition_count"] += 1

    expected_command_keys = {(command.get("label"), str(command.get("seed"))) for command in commands}
    if used_command_keys != expected_command_keys:
        fail(errors, f"v2 commands not exactly consumed by target groups: extra={sorted(expected_command_keys-used_command_keys)}, "
                      f"missing={sorted(used_command_keys-expected_command_keys)}")
    for seed, total in seed_totals.items():
        total["metadata_target_result_rows_expected"] = target_count
        total["compile_only_example_targets"] = sum(
            1 for row in targets if row["kind"] == ["example"] and row.get("test") is False)
        if total["result_rows"] != target_count:
            fail(errors, f"seed {seed}: {total['result_rows']} test-result rows do not close against {target_count} metadata targets")
        if total["failed"] != 0:
            fail(errors, f"seed {seed}: aggregate failed tests are nonzero")
    report["per_seed_test_results"] = seed_totals

    # Bind every target gate_runs record to the actual command. Source hashes are checked now,
    # regardless of whether its historical command was reused or freshly rerun.
    group_for_target = {target_key(row): group["id"] for group in expected_groups_list for row in group["targets"]}
    observed_by_target = {target_key(row): row for row in manifest.get("workspace_targets", [])}
    target_reports = []
    for target in targets:
        key = target_key(target)
        group_id = group_for_target.get(key)
        observed_target = observed_by_target.get(key, {})
        if observed_target.get("partition_id") != group_id:
            fail(errors, f"target {key}: partition_id differs from independently derived group")
        try:
            current_sha = sha_file(Path(target["src_path"]))
            if current_sha != target["src_sha256"]:
                fail(errors, f"target source changed after metadata snapshot: {target['src_path']}")
        except OSError as error:
            fail(errors, f"cannot rehash target source {target.get('src_path')}: {error}")
        runs = observed_target.get("gate_runs", [])
        if len(runs) != 2:
            fail(errors, f"target {key}: gate_runs length {len(runs)} != two")
        else:
            for run, seed in zip(runs, SEEDS):
                actual = expected_run_by_group_seed.get((group_id, seed))
                if actual is None:
                    fail(errors, f"target {key}: no concrete command for seed {seed}")
                    continue
                command, seed_row, _ = actual
                expected = {"label": command.get("label"), "seed": seed,
                            "exit": command.get("exit"), "runner_exit": command.get("runner_exit"),
                            "result_sha256": command.get("result_sha256"),
                            "fresh_execution": command.get("fresh_execution")}
                if run != expected or seed_row.get("result_sha256") != run.get("result_sha256"):
                    fail(errors, f"target {key}: gate_runs differs from concrete command/result record for {seed}")
        target_reports.append({"target_key": [key[0], key[1], list(key[2])],
                               "src_path": target["src_path"], "src_sha256": target["src_sha256"],
                               "partition_id": group_id,
                               "compile_only_example": target["kind"] == ["example"] and target.get("test") is False,
                               "seed_labels": [row.get("label") for row in runs]})
    report["targets"] = target_reports
    report["metadata_targets_with_two_concrete_runs"] = len(target_reports) if all(
        len(row.get("gate_runs", [])) == 2 for row in manifest.get("workspace_targets", [])) else None
    report["compile_only_examples"] = [
        {"target_key": row["target_key"], "partition_id": row["partition_id"],
         "coverage": "compiled and empty harness ran under --examples; no example test cases"}
        for row in target_reports if row["compile_only_example"]]

    # Cleanup is an explicitly recorded startup action plus one action after each fresh group.
    # Check only the declared target-name scope and safe non-recursive locations; deleted files
    # are historical inputs and are not expected to remain on disk for re-hashing.
    fresh_group_ids = [group["id"] for group in expected_groups_list
                       if not isinstance(actual_group_by_id.get(group["id"], {}).get("reused_from"), dict)]
    cleanup_rows = manifest.get("cleanup", [])
    expected_cleanup_phases = ["startup-before-v2-gates"] + [f"after-two-seeds:{group_id}"
                                                             for group_id in fresh_group_ids]
    if [row.get("phase") for row in cleanup_rows] != expected_cleanup_phases:
        fail(errors, "cleanup phases do not match startup plus each freshly executed group")
    expected_cleanup_dirs = [str(ROOT / "target/debug/deps"), str(ROOT / "target/debug/examples")]
    for cleanup in cleanup_rows:
        phase = cleanup.get("phase", "unknown")
        expected_names = sorted({row["target_name"].replace("-", "_")
                                 for row in targets if phase == "startup-before-v2-gates"
                                 or any(row["target_name"] == name for name in
                                        next((group["target_names"] for group in expected_groups_list
                                              if phase == f"after-two-seeds:{group['id']}"), []))})
        if cleanup.get("matched_target_names") != expected_names:
            fail(errors, f"cleanup {phase}: matched target names differ from selected metadata names")
        if (cleanup.get("allowed_directories") != expected_cleanup_dirs
                or cleanup.get("recursive") is not False or cleanup.get("cargo_clean_used") is not False):
            fail(errors, f"cleanup {phase}: directory scope/recursion/cargo-clean declaration differs")
        removed = cleanup.get("removed_files", [])
        seen_removed = set()
        total_removed = 0
        for entry in removed:
            path_text = entry.get("path")
            path = Path(path_text or "")
            if path_text in seen_removed:
                fail(errors, f"cleanup {phase}: duplicate removed path {path_text}")
            seen_removed.add(path_text)
            if (str(path.parent) not in expected_cleanup_dirs or entry.get("exists_after") is not False
                    or not isinstance(entry.get("bytes"), int) or entry["bytes"] < 0
                    or not isinstance(entry.get("sha256"), str)
                    or not re.fullmatch(r"[0-9a-f]{64}", entry.get("sha256", ""))):
                fail(errors, f"cleanup {phase}: malformed or out-of-scope removed-file record {entry}")
            total_removed += entry.get("bytes", 0) if isinstance(entry.get("bytes"), int) else 0
        if cleanup.get("removed_bytes") != total_removed:
            fail(errors, f"cleanup {phase}: removed byte total differs from file records")
        if not isinstance(cleanup.get("free_before"), int) or not isinstance(cleanup.get("free_after"), int):
            fail(errors, f"cleanup {phase}: disk-space observations missing")
    report["cleanup_phase_count"] = len(cleanup_rows)

    # Verify all newly recorded files against a fresh closed inventory; old reused files were
    # separately validated in full above and remain rooted in their v1 manifest identities.
    current_file_map = verify_closed_inventory(OUT, MANIFEST_PATH, manifest, errors, "v2")
    report["v2_file_count"] = len(current_file_map)
    report["prior_numeric_archive"] = {"path": NUMERIC_ARCHIVE_PATH.name, "bytes": identity(NUMERIC_ARCHIVE_PATH)["bytes"],
                                       "sha256": archive_sha}

    # Check baseline candidate/build evidence without treating its old sample source as every
    # target's input. Current target closure above is the authoritative fresh metadata snapshot.
    if isinstance(candidate, dict):
        build_path = contained_file(RESULTS, candidate.get("build_result", ""))
        if sha_file(build_path) != candidate.get("build_result_sha256"):
            fail(errors, "baseline candidate CLI build result SHA differs from recorded identity")
        build_result = read_json(build_path, errors, "baseline candidate CLI build result")
        if isinstance(build_result, dict) and build_result.get("exit") != 0:
            fail(errors, "baseline candidate CLI build result was not successful")

    report["verification"] = "passed" if not errors else "failed"
    report["limitations"] = [
        "This continuation reuses only explicitly identified v1 groups; it is not two fresh whole-workspace invocations.",
        "The gate captures configured environment values, not every inherited host variable.",
        "The historical numeric source snapshot is gate-wide evidence and is bound only to its own old metadata target; all current targets are independently source-hash checked.",
    ]
    OUTPUT_PATH.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    summary = {"verification": report["verification"], "metadata_targets": target_count,
               "partitions": len(expected_groups_list), "commands": len(commands),
               "prior_files": len(prior_file_map), "v2_files": len(current_file_map),
               "errors": errors, "output": str(OUTPUT_PATH)}
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
