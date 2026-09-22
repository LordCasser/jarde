#!/usr/bin/env python3
"""Read the investigation campaigns' raw JSONL and print the tables the evidence files cite.

    python3 investigation-tables.py investigation-fixture-raw.jsonl [investigation-bcprov-raw.jsonl ...]

The input files are what `run-baseline.py` wrote: one line per process, the process's own wall time
and peak RSS beside the samples it printed. Nothing here is a threshold or a verdict; every table is
a median (with the sample count stated) of what those files really hold, and the sections are the
planes the investigations keep apart:

* `phases` — the in-process stage ledger of one sample (`open`/`prepare`/`request`/`output`), in µs;
* `numbers` — the counted readings a sample publishes (the demand-path counters mirrored as
  `count_*`, the query's items/pages, a bulk run's own classification);
* `cache` — the store's `FactsReport` (residency, reuse, refusals, directory parses);
* `series` — a sample's own per-request list (`per_request_micros`, `page_micros`);
* `process` — the process-level wall time and peak RSS of every row of one configuration.

Rows that failed are counted and never dropped silently; a configuration with a failure is printed
with its count so a reader can see the file is not all-success.
"""

import argparse
import json
import statistics


def load(path):
    rows = []
    with open(path) as raw:
        for line in raw:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def samples(rows):
    """Every (row, sample) the file holds."""
    for row in rows:
        for sample in row.get("samples", []):
            yield row, sample


def median(values):
    return statistics.median(values) if values else 0


def config_of(row):
    settings = {key: value for key, value in row["settings"].items() if key != "artifact"}
    corpus = "fixture" if "artifact" not in row["settings"] else "corpus"
    tuned = ":".join(f"{key}={value}" for key, value in sorted(settings.items()))
    return f"{corpus}/{row['workload']}" + (f":{tuned}" if tuned else "")


def groups_of(rows, select):
    groups = {}
    for row, sample in samples(rows):
        key = select(row, sample)
        if key is not None:
            groups.setdefault(key, []).append((row, sample))
    return groups


def phases(rows):
    print("\n=== phases (median micros; n is the samples of that configuration) ===")
    groups = groups_of(rows, lambda row, sample: (config_of(row), sample["sample"]))
    for (config, name), entries in sorted(groups.items()):
        stage = {}
        for _, sample in entries:
            for phase in sample["phases"]:
                stage.setdefault(phase["name"], []).append(phase["micros"])
        cells = " ".join(f"{key}={int(median(value))}" for key, value in sorted(stage.items()))
        window = median([sample["window_micros"] for _, sample in entries])
        print(f"{config:<34} {name:<36} n={len(entries):>2} window={int(window):>9} {cells}")


COUNTED = [
    "count_class_materializations",
    "count_class_preparations",
    "count_body_decodes",
    "count_recovery_runs",
    "count_owned_records",
    "count_read_detail_records",
]


def numbers(rows):
    print("\n=== numbers (medians; counted readings and a sample's own figures) ===")
    groups = groups_of(rows, lambda row, sample: (config_of(row), sample["sample"]))
    for (config, name), entries in sorted(groups.items()):
        documents = [sample for _, sample in entries]
        keys = sorted(set().union(*[set(sample["numbers"]) for sample in documents]))
        keys = [key for key in keys if key in COUNTED or not key.startswith("count_")]
        cells = []
        for key in keys:
            values = [sample["numbers"][key] for sample in documents]
            cells.append(f"{key}={int(median(values))}")
        print(f"{config:<34} {name:<36} n={len(entries):>2} " + " ".join(cells))


CACHE = [
    "entries",
    "containers",
    "retained_bytes",
    "consultations",
    "hits",
    "misses",
    "stored",
    "refused_capacity",
    "refused_capacity_bytes",
    "container_consultations",
    "container_hits",
    "container_misses",
    "container_stored",
    "directory_parses",
    "nested_materializations",
    "nested_materialized_bytes",
]


def cache(rows):
    print("\n=== cache (medians of the store's own report) ===")
    groups = groups_of(
        rows,
        lambda row, sample: (config_of(row), sample["sample"]) if sample.get("cache") else None,
    )
    for (config, name), entries in sorted(groups.items()):
        reports = [sample["cache"] for _, sample in entries]
        cells = " ".join(
            f"{key}={int(median([report[key] for report in reports]))}"
            for key in CACHE
            if key in reports[0]
        )
        print(f"{config:<34} {name:<36} n={len(entries):>2} {cells}")


def series(rows):
    print("\n=== series (per-request and per-page readings) ===")
    groups = {}
    for row, sample in samples(rows):
        for key, values in sample.get("series", {}).items():
            groups.setdefault((config_of(row), sample["sample"], key), []).append(values)
    for (config, name, key), entries in sorted(groups.items()):
        flat = [value for values in entries for value in values]
        print(
            f"{config:<34} {name:<36} {key:<20} calls={len(flat):>6} "
            f"median={int(median(flat)):>8} min={min(flat):>8} max={max(flat):>8}"
        )


def process(rows):
    print("\n=== process (one row is one process) ===")
    groups = {}
    for row in rows:
        groups.setdefault(config_of(row), []).append(row)
    for config, entries in sorted(groups.items()):
        wall = median([entry["wall_seconds"] for entry in entries])
        rss = median(
            [entry["max_rss_bytes"] for entry in entries if entry["max_rss_bytes"] is not None]
        )
        failures = [entry for entry in entries if entry.get("exit_code") != 0]
        print(
            f"{config:<34} n={len(entries):>2} wall_median={wall:8.2f}s "
            f"rss_median={rss / 2**20:7.1f}MiB failures={len(failures)}"
        )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="+")
    arguments = parser.parse_args()
    for path in arguments.files:
        rows = load(path)
        print(f"# {path}: {len(rows)} rows, {len(list(samples(rows)))} samples")
        phases(rows)
        numbers(rows)
        cache(rows)
        series(rows)
        process(rows)


if __name__ == "__main__":
    main()
