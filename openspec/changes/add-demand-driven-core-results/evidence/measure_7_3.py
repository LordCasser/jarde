#!/usr/bin/env python3
"""Run the 7.3 protocol: interleaved samples per workload and arm, raw + medians.

The protocol is fixed in `../verification.md` §7.3: every workload is run with both
arms alternating (never one arm's samples first), each run is wrapped so its peak RSS
is recorded, and the report states medians and extremes. The script prints the raw
JSON lines and a summary table; it never drops a sample.

    python3 measure_7_3.py <label> <artifact> <roots.json> [repeats] [workloads...]
"""

import json
import statistics
import subprocess
import sys
import time

EXAMPLE = "./target/release/examples/demand_workloads"
# The scope every workload must run under, as the CLI's own operations state it: a whole package is
# walked as a tree (nested containers included), a standalone class as the snapshot it is.
TREE_SCOPE = '{"kind":"artifact_tree","root_container":"root"}'
CLASS_SCOPE = '{"kind":"snapshot_all"}'
WORKLOADS = [
    "nav-class",
    "nav-decl",
    "recover-one",
    "sweep",
    "expand",
    "page-small",
    "page-abandon",
]
ARMS = ["essential", "all"]


def one_run(artifact, roots, scope, workload, arm):
    """One measured run: the example's own JSON line plus this process's peak RSS."""
    started = time.perf_counter()
    process = subprocess.run(
        ["/usr/bin/time", "-l", EXAMPLE, artifact, roots, scope, workload, arm, "1"],
        capture_output=True,
        text=True,
    )
    wall = time.perf_counter() - started
    if process.returncode != 0:
        return {"workload": workload, "arm": arm, "failed": process.returncode,
                "stderr": process.stderr.strip()[-400:]}
    line = [entry for entry in process.stdout.splitlines() if entry.startswith("{")][-1]
    sample = json.loads(line)
    sample["process_wall_seconds"] = round(wall, 4)
    for entry in process.stderr.splitlines():
        # `/usr/bin/time -l` on macOS prints the peak in bytes, number first and column name after;
        # a machine whose `time` prints it the other way round would still be read, because the first
        # numeric field of the named line is what is taken.
        if "maximum resident set size" in entry:
            numbers = [field for field in entry.split() if field.isdigit()]
            if numbers:
                sample["max_rss_bytes"] = int(numbers[0])
    return sample


def main():
    label, artifact, roots = sys.argv[1], sys.argv[2], sys.argv[3]
    repeats = int(sys.argv[4]) if len(sys.argv) > 4 else 10
    workloads = sys.argv[5:] or WORKLOADS

    scope = CLASS_SCOPE if artifact.endswith(".class") else TREE_SCOPE
    samples = []
    for repeat in range(repeats):
        for workload in workloads:
            for arm in ARMS:
                sample = one_run(artifact, roots, scope, workload, arm)
                sample["repeat"] = repeat
                sample["label"] = label
                samples.append(sample)
                print(json.dumps(sample, sort_keys=True), flush=True)

    print()
    print(f"=== {label}: {repeats} interleaved repeats, scope {scope} ===")
    header = f"{'workload':<13} {'arm':<10} {'first ms (median/min/max)':<30} " \
             f"{'total ms (median/min/max)':<30} {'returned B':>12} {'RSS MiB':>9}"
    print(header)
    for workload in workloads:
        for arm in ARMS:
            cells = [s for s in samples if s.get("workload") == workload
                     and s.get("arm") == arm and "failed" not in s]
            if not cells:
                print(f"{workload:<13} {arm:<10} (no successful sample)")
                continue
            firsts = sorted(c["first_micros"] for c in cells)
            totals = sorted(c["total_micros"] for c in cells)
            rss = [c["max_rss_bytes"] for c in cells if "max_rss_bytes" in c]
            print(
                f"{workload:<13} {arm:<10} "
                f"{statistics.median(firsts)/1000:9.2f} / {firsts[0]/1000:8.2f} / {firsts[-1]/1000:8.2f}   "
                f"{statistics.median(totals)/1000:9.2f} / {totals[0]/1000:8.2f} / {totals[-1]/1000:8.2f}   "
                f"{int(statistics.median([c['returned_bytes'] for c in cells])):>12} "
                f"{max(rss)/2**20 if rss else 0:>9.1f}"
            )


if __name__ == "__main__":
    main()
