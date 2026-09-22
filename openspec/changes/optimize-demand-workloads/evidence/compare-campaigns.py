#!/usr/bin/env python3
"""Compare a pre-admission baseline campaign against a post-admission re-run.

Both files come from `run-baseline.py` at ten interleaved repeats per configuration, so the
comparison is like for like: the *counts* decide whether anything but the reused reads moved, and
the medians are reported with their spreads because ten samples do not make a throughput claim.

    python3 compare-campaigns.py <baseline.jsonl> <after.jsonl>
"""

import json
import statistics
import sys

# The counted dimensions whose whole purpose is to fall when a read is answered from retention.
READ_KEYS = ("archive_entries", "read_bytes", "entry_bytes", "class_bytes")
# The counted dimensions that must not move: they are the work the run really did.
WORK_KEYS = ("class_headers", "method_bodies", "ir_items", "analysis_steps", "result_items",
             "output_bytes")


def rows(path):
    out = []
    for line in open(path):
        if line.strip().startswith("{"):
            out.append(json.loads(line))
    return out


def by_config(data):
    """Group by the pair that identifies one measurement: its configuration and its sample name.

    The harness gained samples between the two campaigns (a class-source read, the damaged-suffix
    series, the batch arms), so comparing whole configurations would compare different sample sets
    and attribute the harness's own growth to the engine. Comparing per sample keeps it like for
    like, and samples present on only one side are reported rather than matched to something else.
    """
    grouped = {}
    for row in data:
        if "config" not in row:
            continue
        for sample in row.get("samples", []):
            key = (row["config"], sample.get("sample"))
            grouped.setdefault(key, []).append(sample)
    return grouped


def phase_median(samples, name):
    values = []
    for sample in samples:
        for phase in sample.get("phases", []):
            if phase.get("name") == name:
                values.append(phase["micros"])
    return statistics.median(values) if values else None


def summed(samples, key):
    """A counted dimension summed over the samples of one (configuration, sample) group."""
    total = 0
    seen = False
    for sample in samples:
        usage = sample.get("usage") or {}
        if key in usage:
            total += usage[key]
            seen = True
    return total if seen else None


def main():
    baseline, after = by_config(rows(sys.argv[1])), by_config(rows(sys.argv[2]))
    print(f"measurements (configuration, sample): baseline {len(baseline)}, after {len(after)}")
    moved_reads, moved_work, one_sided = [], [], []
    for key in sorted(set(baseline) | set(after), key=lambda item: (item[0], str(item[1]))):
        config, sample = key
        before, now = baseline.get(key, []), after.get(key, [])
        if not before or not now:
            one_sided.append(key)
            continue
        # A movement is where both sides state the dimension and the figures differ: a dimension only
        # one side states (the query samples carry coverage rather than `usage`) is not a change.
        def delta(keys):
            out = {}
            for key in keys:
                before_value, now_value = summed(before, key), summed(now, key)
                if before_value is None or now_value is None:
                    continue
                if before_value != now_value:
                    out[key] = (before_value, now_value)
            return out

        read_delta = delta(READ_KEYS)
        work_delta = delta(WORK_KEYS)
        wall_before = phase_median(before, "request")
        wall_now = phase_median(now, "request")
        line = (f"  {str(sample)[:34]:<34} {config[:44]:<44}"
                f"  reads {list(read_delta) or 'same'}  work {list(work_delta) or 'same'}")
        print(line)
        if read_delta:
            moved_reads.append(config)
        if work_delta:
            moved_work.append((config, work_delta))
    print()
    print(f"measurements whose reads moved: {len(moved_reads)}")
    print(f"measurements whose WORK moved: {len(moved_work)}")
    for key, delta in moved_work:
        print(f"  !! {key}: {delta}")
    print(f"measurements present on only one side (harness grew): {len(one_sided)}")
    for config, sample in one_sided:
        print(f"  -- {sample} / {config[:60]}")


if __name__ == "__main__":
    main()
