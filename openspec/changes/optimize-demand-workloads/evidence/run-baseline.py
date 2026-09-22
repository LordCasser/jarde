#!/usr/bin/env python3
"""Run one sample per process and keep every one of them.

The harness (`tests/p5_optimize_workloads.rs`) prints one JSON line per sample and asserts no
duration. This driver fixes the executable, interleaves the configurations of one campaign, wraps
every process so its own wall time and peak RSS are recorded beside the line, appends each row to a
raw JSONL file that is never rewritten, and prints a summary that states the sample count it read.

    python3 run-baseline.py --tag fixture --repeats 10 --out baseline-fixture-raw.jsonl \
        --features test-support --run w1: --run w2: --run w6a:workers=1:sink=discard ...

A configuration is `workload:key=value:key=value`. The keys are the harness's own variables without
their `JARDE_OPTIMIZE_` prefix; `artifact=` names a corpus file and omitting it measures the in-repo
fixture. The samples of a *failed* run are recorded as a failure row rather than dropped, and the
summary counts them.

Nothing here asserts a threshold, a speed-up or a tail quantile: ten samples are ten samples, and a
median of ten says nothing about a p95 that this file does not claim.
"""

import argparse
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
TEST_TARGET = "p5_optimize_workloads"
ENV_PREFIX = "JARDE_OPTIMIZE_"


def build_binary(features):
    """The executable cargo just built for the harness target, from its own message stream."""
    command = ["cargo", "test", "--release", "--test", TEST_TARGET, "--no-run",
               "--message-format=json"]
    if features:
        command += ["--features", features]
    built = subprocess.run(command, cwd=REPO, capture_output=True, text=True, check=True)
    executable = None
    for line in built.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-artifact":
            continue
        if message.get("target", {}).get("name") != TEST_TARGET:
            continue
        if message.get("executable"):
            executable = message["executable"]
    if not executable:
        raise SystemExit("cargo built no executable for the harness target")
    return executable


def load_record():
    """The machine's own load, recorded beside the samples rather than asserted."""
    return {
        "load_average": list(os.getloadavg()),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "python": platform.python_version(),
    }


def parse_config(text):
    parts = text.split(":")
    workload = parts[0]
    settings = {}
    for part in parts[1:]:
        if not part:
            continue
        key, _, value = part.partition("=")
        settings[key] = value
    return workload, settings


def environment_for(settings):
    env = dict(os.environ)
    for key in list(env):
        if key.startswith(ENV_PREFIX):
            del env[key]
    env["JARDE_OPTIMIZE_WORKLOAD"] = "unset"
    for key, value in settings.items():
        env[ENV_PREFIX + key.upper()] = value
    return env


def one_run(binary, workload, settings):
    """One process: the sample lines it printed, its own wall time, its own peak RSS."""
    env = environment_for(settings)
    env[ENV_PREFIX + "WORKLOAD"] = workload
    command = [binary, "--exact", "measure_workload", "--ignored", "--nocapture"]
    timer = shutil.which("/usr/bin/time")
    if timer:
        command = [timer, "-l"] + command
    started = time.perf_counter()
    process = subprocess.run(command, cwd=REPO, env=env, capture_output=True, text=True)
    wall = time.perf_counter() - started
    row = {
        "wall_seconds": round(wall, 4),
        "exit_code": process.returncode,
        "max_rss_bytes": None,
        "cpu_seconds": None,
    }
    for line in process.stderr.splitlines():
        fields = line.split()
        if "maximum resident set size" in line and fields and fields[0].isdigit():
            row["max_rss_bytes"] = int(fields[0])
        # BSD `time -l` prints "x.xx real  y.yy user  z.zz sys" as one line of the summary block.
        if line.endswith(" sys") and len(fields) >= 6 and fields[-2] == "user":
            try:
                row["cpu_seconds"] = round(float(fields[-3]) + float(fields[-1]), 4)
            except ValueError:
                pass
    samples = []
    for line in process.stdout.splitlines():
        line = line.strip()
        if line.startswith("{"):
            try:
                samples.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    if process.returncode != 0 or not samples:
        row["failure"] = process.stderr.strip()[-800:]
    return row, samples


def summary_rows(rows):
    """The samples as medians and extremes per (configuration, sample), with `n` stated."""
    groups = {}
    for row in rows:
        for sample in row.get("samples", []):
            key = (row["config"], sample["sample"])
            groups.setdefault(key, []).append((row, sample))
    return groups


def print_summary(rows, names=None):
    print()
    print("=== summary (medians with extremes; `n` is what the file really holds) ===")
    for (config, name), entries in sorted(summary_rows(rows).items()):
        windows = sorted(entry[1]["window_micros"] for entry in entries)
        print(f"{config:<44} {name:<28} n={len(windows):>2} "
              f"window ms median/min/max {statistics.median(windows)/1000:9.2f} / "
              f"{windows[0]/1000:9.2f} / {windows[-1]/1000:9.2f}")
        keys = sorted(set().union(*[set(entry[1]["numbers"]) for entry in entries]))
        for key in keys:
            values = sorted(entry[1]["numbers"][key] for entry in entries)
            print(f"{'':<44} {'':<28} {key:<28} median {statistics.median(values):>12} "
                  f"min {values[0]:>12} max {values[-1]:>12}")
    failures = [row for row in rows if row.get("exit_code") != 0 or "failure" in row]
    print(f"failures: {len(failures)}")
    for row in failures[:10]:
        print(f"  {row['config']} repeat {row['repeat']}: exit {row['exit_code']} "
              f"{(row.get('failure') or '')[:200]}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--repeats", type=int, default=10)
    parser.add_argument("--binary", default=None)
    parser.add_argument("--features", default="")
    parser.add_argument("--run", action="append", default=[],
                        help="workload[:key=value...]; later flags add configurations")
    arguments = parser.parse_args()
    if not arguments.run:
        raise SystemExit("at least one --run is required: this driver measures what it is told to")

    binary = arguments.binary or build_binary(arguments.features)
    configs = [parse_config(text) for text in arguments.run]
    out = arguments.out if os.path.isabs(arguments.out) else os.path.join(HERE, arguments.out)
    print(f"binary: {binary}")
    print(f"out:    {out}")
    print(f"load:   {load_record()}")

    rows = []
    with open(out, "w") as raw:
        for repeat in range(arguments.repeats):
            for workload, settings in configs:
                row, samples = one_run(binary, workload, settings)
                row["tag"] = arguments.tag
                row["repeat"] = repeat
                row["workload"] = workload
                row["settings"] = settings
                row["config"] = f"{workload}:" + ":".join(
                    f"{key}={value}" for key, value in sorted(settings.items())) if settings \
                    else f"{workload}:"
                row["samples"] = samples
                rows.append(row)
                raw.write(json.dumps(row, sort_keys=True) + "\n")
                raw.flush()
                line = (f"rep {repeat:>2} {row['config']:<44} wall {row['wall_seconds']:8.2f}s "
                        f"rss {'' if row['max_rss_bytes'] is None else int(row['max_rss_bytes'])/2**20:8.1f}MiB "
                        f"samples {len(samples)} exit {row['exit_code']}")
                print(line, flush=True)
    print_summary(rows)


if __name__ == "__main__":
    sys.exit(main())
