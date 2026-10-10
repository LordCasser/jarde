#!/usr/bin/env python3
"""Capture the exact GitHub CLI CI summary and two job logs for root review."""

from __future__ import annotations

import datetime
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
FINAL = Path("/private/tmp/jarde-proved-if-join-ci-v1/ci-product-v1")
FAILURES = Path("/private/tmp/jarde-proved-if-join-ci-v1/failures")
RUN_ID = None
EXPECTED_HEAD = None
EXPECTED_LIB_COUNT = None
EXPECTED_GATEWAY_COUNT = None
EXPECTED_TOTAL_PASSED = None
GH_TIMEOUT_SECONDS = 35
APPENDED_NO_PROXY = ("results-receiver.actions.githubusercontent.com",
                     "productionresultssa1.blob.core.windows.net")
LOG_JOBS = ()

API_ARGV = None


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def utc_now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def exclusive_write(path: Path, data: bytes) -> None:
    with path.open("xb") as stream:
        stream.write(data)


def record_bytes(path: Path, data: bytes) -> dict:
    exclusive_write(path, data)
    return {"path": path.name, "bytes": len(data), "sha256": sha(data)}


def environment_for_call() -> tuple[dict[str, str], dict]:
    env = os.environ.copy()
    record = {}
    for key in ("NO_PROXY", "no_proxy"):
        original = env.get(key, "")
        pieces = [part for part in original.split(",") if part]
        for host in APPENDED_NO_PROXY:
            if host not in pieces:
                pieces.append(host)
        value = ",".join(pieces)
        env[key] = value
        record[key] = {"appended": list(APPENDED_NO_PROXY), "final_value_sha256": sha(value.encode())}
    return env, record


def run(argv: list[str], cwd: Path) -> dict:
    env, env_override = environment_for_call()
    started = utc_now()
    began = time.monotonic()
    timed_out = False
    spawn_error = None
    try:
        result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True,
                                timeout=GH_TIMEOUT_SECONDS, check=False)
        stdout = result.stdout or b""
        stderr = result.stderr or b""
        exit_code = result.returncode
    except subprocess.TimeoutExpired as error:
        timed_out = True
        stdout = error.stdout or b""
        stderr = error.stderr or b""
        exit_code = None
    except OSError as error:
        stdout = b""
        stderr = str(error).encode("utf-8", errors="replace")
        exit_code = None
        spawn_error = f"{type(error).__name__}: {error}"
    return {"argv": argv, "cwd": str(cwd), "started_at": started,
            "duration_seconds": time.monotonic() - began, "timeout_seconds": GH_TIMEOUT_SECONDS,
            "timed_out": timed_out, "exit_code": exit_code, "spawn_error": spawn_error,
            "environment_override": env_override, "_stdout": stdout, "_stderr": stderr}


def save_api(directory: Path, row: dict, *, prefix: str = "") -> None:
    stem = f"{prefix}ci-run-v1" if prefix else "ci-run-v1"
    row["stdout"] = record_bytes(directory / f"{stem}.json", row.pop("_stdout"))
    row["stderr"] = record_bytes(directory / f"{stem}.stderr.raw", row.pop("_stderr"))


def api_is_complete(stdout: bytes, row: dict) -> tuple[bool, str | None, dict | None]:
    if row["exit_code"] != 0 or row["timed_out"]:
        return False, "GitHub CLI run summary command did not complete successfully", None
    try:
        document = json.loads(stdout)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        return False, f"run summary was not valid JSON: {error}", None
    if document.get("headSha") != EXPECTED_HEAD:
        return False, "run headSha does not match the pinned product commit", document
    if document.get("status") != "completed" or document.get("conclusion") != "success":
        return False, "run is not completed successfully", document
    jobs = document.get("jobs")
    if not isinstance(jobs, list) or len(jobs) != 4:
        return False, "run summary did not contain exactly four jobs", document
    for job in jobs:
        if job.get("status") != "completed" or job.get("conclusion") != "success":
            return False, f"job is not complete and successful: {job.get('name')}", document
        steps = job.get("steps")
        if not isinstance(steps, list) or any(
                step.get("status") != "completed" or step.get("conclusion") != "success"
                for step in steps):
            return False, f"job has an incomplete or unsuccessful step: {job.get('name')}", document
    return True, None, document


def write_execution(directory: Path, payload: dict) -> None:
    path = directory / "capture-execution-root-v1.json"
    data = (json.dumps(payload, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    exclusive_write(path, data)


def run_logs(directory: Path, rows: list[dict]) -> None:
    for job_id, filename in LOG_JOBS:
        row = run(["gh", "run", "view", RUN_ID, "--job", job_id, "--log"], ROOT)
        stdout = row.pop("_stdout")
        stderr = row.pop("_stderr")
        compressed = gzip.compress(stdout, mtime=0)
        row["stdout"] = {
            "path": filename,
            "bytes": len(compressed),
            "uncompressed_bytes": len(stdout),
            "sha256": sha(compressed),
            "raw_sha256": sha(stdout),
        }
        exclusive_write(directory / filename, compressed)
        raw_name = filename.removesuffix(".gz") + ".stdout.raw"
        row["stdout_raw"] = record_bytes(directory / raw_name, stdout)
        row["stderr"] = record_bytes(directory / f"{filename}.stderr.raw", stderr)
        rows.append(row)


def main() -> int:
    global RUN_ID, EXPECTED_HEAD, EXPECTED_LIB_COUNT, EXPECTED_GATEWAY_COUNT
    global EXPECTED_TOTAL_PASSED, API_ARGV
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-commit", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--expected-lib-count", type=int, required=True)
    parser.add_argument("--expected-gateway-count", type=int, required=True)
    parser.add_argument("--expected-total-passed", type=int, required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.product_commit):
        raise SystemExit("--product-commit must be 40 lowercase hexadecimal characters")
    if not args.run_id.isdecimal():
        raise SystemExit("--run-id must be a decimal identifier")
    if min(args.expected_lib_count, args.expected_gateway_count, args.expected_total_passed) < 1:
        raise SystemExit("expected test counts must be positive")
    RUN_ID = args.run_id
    EXPECTED_HEAD = args.product_commit
    EXPECTED_LIB_COUNT = args.expected_lib_count
    EXPECTED_GATEWAY_COUNT = args.expected_gateway_count
    EXPECTED_TOTAL_PASSED = args.expected_total_passed
    API_ARGV = ["gh", "run", "view", RUN_ID, "--json", "headSha,status,conclusion,jobs,url"]
    if FINAL.exists():
        raise FileExistsError(f"refusing to overwrite CI evidence: {FINAL}")
    api = run(API_ARGV, ROOT)
    stdout = api["_stdout"]
    complete, reason, document = api_is_complete(stdout, api)
    if not complete:
        FAILURES.mkdir(parents=True, exist_ok=True)
        attempt = FAILURES / datetime.datetime.now(datetime.timezone.utc).strftime("attempt-%Y%m%dT%H%M%S.%fZ")
        attempt.mkdir(exist_ok=False)
        save_api(attempt, api, prefix="failed-")
        payload = {"schema": "preserve-proved-if-arm-join-origins-ci-capture-root-v1",
                   "status": "failed", "reason": reason, "run_id": RUN_ID,
                   "expected_head_sha": EXPECTED_HEAD,
                   "expected_test_counts": {"lib": EXPECTED_LIB_COUNT,
                                            "gateway": EXPECTED_GATEWAY_COUNT,
                                            "workspace_passed": EXPECTED_TOTAL_PASSED},
                   "commands": [{key: value for key, value in api.items() if not key.startswith("_")}],
                   "validated_run_summary": document}
        write_execution(attempt, payload)
        print(f"CI capture stopped: {reason}; attempt evidence at {attempt}")
        return 1

    global LOG_JOBS
    named_jobs = {job["name"]: job["databaseId"] for job in document["jobs"]}
    LOG_JOBS = ((str(named_jobs["stable / test and specification"]), "ci-stable-job-v1.log.gz"),
                (str(named_jobs["supply chain"]), "ci-supply-job-v1.log.gz"))
    FINAL.mkdir(parents=True, exist_ok=False)
    save_api(FINAL, api)
    rows = [{key: value for key, value in api.items() if not key.startswith("_")}]
    run_logs(FINAL, rows)
    status = "captured" if all(row["exit_code"] == 0 and not row["timed_out"]
                               for row in rows) else "failed"
    payload = {"schema": "preserve-proved-if-arm-join-origins-ci-capture-root-v1", "status": status,
               "run_id": RUN_ID, "expected_head_sha": EXPECTED_HEAD,
               "expected_test_counts": {"lib": EXPECTED_LIB_COUNT,
                                        "gateway": EXPECTED_GATEWAY_COUNT,
                                        "workspace_passed": EXPECTED_TOTAL_PASSED},
               "validated_run_summary": document, "commands": rows}
    write_execution(FINAL, payload)
    print(f"CI capture {status}: {FINAL}")
    return 0 if status == "captured" else 1


if __name__ == "__main__":
    raise SystemExit(main())
