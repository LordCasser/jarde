#!/usr/bin/env python3
"""The O2 minimal experiment's campaign: two arms of one build, interleaved, one process per sample.

The experiment is `openspec/changes/optimize-demand-workloads` task 3.3's O2 arm: *reuse the read an
operation already performed for the definition it selected, inside the same store and snapshot*.
The prototype that implements it is kept out of the repository (see `g1g2-o2-prototype.patch`) and is
switched by `JARDE_OPTIMIZE_READ_REUSE=on`; with the variable unset **nothing is looked up and
nothing is retained**, so the `off` arm is the engine as it stands and both arms are the same
binary, the same build, the same corpus, the same store capacity and the same workload — the one
variable is whether a definition read may be answered from retention.

    python3 g1g2-o2-campaign.py --binary <harness> --repeats 10

What it writes: `g1g2-o2-raw-<corpus>.jsonl` (one row per process, never rewritten) and a table of
medians with the sample count the file really holds. Nothing here asserts a threshold or a speed-up:
it states, per cell, what the two arms did.

The corpora are the in-repo fixture and `bcprov-jdk15on-152.jar`
(sha256[:16] `5329ddefb3c92927`, 2,903,072 B), passed as `artifact=<path>`; no network and no `/tmp`
is used, and every sample's own scratch files go to cargo's test temporary directory.
"""

import argparse
import json
import os
import platform
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
ENV_PREFIX = "JARDE_OPTIMIZE_"
BCPROV = ("/Users/lordcasser/workspace/vulnerability/vulhub/weblogic/weak_password/decrypt/"
          "lib/bcprov-jdk15on-152.jar")

# The cells: workload, corpus and the factors each arm declares. `read_reuse` is the experiment's
# one variable; `capacity` is a second factor of the *capacity-degradation* half of the experiment,
# and it is varied in both arms (never across them) so a refusal can be told from a hit.
CELLS = [
    ("w1", None, {}),
    ("w1", BCPROV, {}),
    ("w2", None, {}),
    ("w2", BCPROV, {}),
    ("w3", None, {}),
    ("w3", BCPROV, {}),
    ("w5", None, {"capacity": "roomy"}),
    ("w5", None, {"capacity": "tiny"}),
    ("w5", None, {"capacity": "none"}),
    ("w5", BCPROV, {"capacity": "roomy"}),
    ("w5", BCPROV, {"capacity": "tiny"}),
]


def environment_for(workload, artifact, settings, arm):
    env = dict(os.environ)
    for key in list(env):
        if key.startswith(ENV_PREFIX):
            del env[key]
    env[ENV_PREFIX + "WORKLOAD"] = workload
    if artifact:
        env[ENV_PREFIX + "ARTIFACT"] = artifact
    for key, value in settings.items():
        env[ENV_PREFIX + key.upper()] = value
    if arm == "on":
        env[ENV_PREFIX + "READ_REUSE"] = "on"
    return env


def one_run(binary, workload, artifact, settings, arm):
    env = environment_for(workload, artifact, settings, arm)
    command = ["/usr/bin/time", "-l", binary, "--exact", "measure_workload", "--ignored",
               "--nocapture"]
    started = time.perf_counter()
    process = subprocess.run(command, cwd=REPO, env=env, capture_output=True, text=True)
    wall = time.perf_counter() - started
    row = {
        "wall_seconds": round(wall, 4),
        "exit_code": process.returncode,
        "max_rss_bytes": None,
        "corpus": os.path.basename(artifact) if artifact else "tests/fixtures (in-process fixture)",
        "workload": workload,
        "settings": settings,
        "arm": arm,
        "load_average": list(os.getloadavg()),
        "samples": [],
    }
    for line in process.stderr.splitlines():
        fields = line.split()
        if "maximum resident set size" in line and fields and fields[0].isdigit():
            row["max_rss_bytes"] = int(fields[0])
    for line in process.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            row["samples"].append(json.loads(line))
        except json.JSONDecodeError:
            pass
    if process.returncode != 0 or not row["samples"]:
        row["failure"] = process.stderr.strip()[-600:]
    return row


COUNTERS = ("container_hits", "container_consultations", "container_misses", "directory_parses",
            "nested_materializations", "refused_capacity", "refused_capacity_bytes",
            "consultations", "hits", "misses", "stored",
            "definition_read_hits", "definition_read_consultations", "definition_read_misses",
            "definition_read_stored")


def deltas(sample, previous):
    """One sample's own counter readings: the store's report is cumulative, so it is differenced."""
    cache = sample.get("cache") or {}
    out = {}
    for field in COUNTERS:
        current = cache.get(field)
        earlier = (previous or {}).get(field)
        out[field] = None if current is None else current - (earlier or 0)
    return out


def metrics(sample, previous=None):
    """The numbers this experiment reads out of one printed sample."""
    usage = sample.get("usage") or {}
    cache = sample.get("cache") or {}
    own = deltas(sample, previous)
    counts = sample.get("counts") or {}
    numbers = sample.get("numbers") or {}
    phases = {phase["name"]: phase["micros"] for phase in sample.get("phases") or []}
    return {
        "window_us": sample.get("window_micros"),
        "request_us": phases.get("request"),
        "prepare_us": phases.get("prepare"),
        "output_us": phases.get("output"),
        "sequence_us": numbers.get("sequence_micros"),
        "control_us": numbers.get("control_repeat_micros"),
        "first_result_us": numbers.get("first_result_micros"),
        "class_materializations": counts.get("class_materializations"),
        "class_preparations": counts.get("class_preparations"),
        "body_decodes": counts.get("body_decodes"),
        "class_bytes": usage.get("class_bytes"),
        "entry_bytes": usage.get("entry_bytes"),
        "read_bytes": usage.get("read_bytes"),
        "archive_entries": usage.get("archive_entries"),
        "class_headers": usage.get("class_headers"),
        "container_hits": own["container_hits"],
        "directory_parses": own["directory_parses"],
        "nested_materializations": own["nested_materializations"],
        "definition_read_hits": own["definition_read_hits"],
        "definition_read_consultations": own["definition_read_consultations"],
        "definition_read_misses": own["definition_read_misses"],
        "definition_read_stored": own["definition_read_stored"],
        "definition_reads": cache.get("definition_reads"),
        "retained_bytes": cache.get("retained_bytes"),
        "refused_capacity": own["refused_capacity"],
        "refused_capacity_bytes": own["refused_capacity_bytes"],
        "domain": result_domain(sample),
    }


def result_domain(sample):
    """The charge-stripped result fingerprint of one sample, where the harness prints one.

    The harness prints four kinds: W1's per-request `result_domain`, W2's `result_sequence_domain`,
    W3's per-arm `arm_domain` (a projection of the decoded bodies alone) and W5's
    `round_trip_domain`. Each one drops the request's own `usage`, which is exactly the plane a
    reused read is allowed to change.
    """
    texts = sample.get("texts") or {}
    for key in ("result_domain", "result_sequence_domain", "arm_domain", "round_trip_domain",
                "sequence_domain"):
        if texts.get(key):
            return texts[key]
    return sample.get("domain")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--repeats", type=int, default=10)
    parser.add_argument("--cells", type=int, default=0,
                        help="run only the first N cells (a smoke run, stated in the output)")
    arguments = parser.parse_args()

    cells = CELLS if arguments.cells <= 0 else CELLS[: arguments.cells]
    rows = []
    print(f"binary: {arguments.binary}")
    print(f"platform: {platform.platform()} {platform.machine()}; python {platform.python_version()}")
    print(f"cells: {len(cells)}; repeats: {arguments.repeats}; load at start: {list(os.getloadavg())}")
    by_corpus = {}
    for workload, artifact, settings in cells:
        corpus = "bcprov" if artifact else "fixture"
        by_corpus.setdefault(corpus, []).append((workload, artifact, settings))
    for corpus, cells_for_file in by_corpus.items():
        out = os.path.join(HERE, f"g1g2-o2-raw-{corpus}.jsonl")
        with open(out, "w") as raw:
            for repeat in range(arguments.repeats):
                # The arms alternate inside one repeat: every pair of samples is one `off` run and
                # one `on` run of the same cell, so a drift in machine load is shared by the pair.
                for workload, artifact, settings in cells_for_file:
                    for arm in ("off", "on"):
                        row = one_run(arguments.binary, workload, artifact, settings, arm)
                        row["repeat"] = repeat
                        rows.append(row)
                        raw.write(json.dumps(row, sort_keys=True) + "\n")
                        raw.flush()
                        rss = row["max_rss_bytes"]
                        label = (f"{workload}:{row['corpus']}:" +
                                 ":".join(f"{k}={v}" for k, v in sorted(settings.items())) +
                                 f":{arm}")
                        print(f"rep {repeat:>2} {label:<64} wall {row['wall_seconds']:7.2f}s "
                              f"rss {'' if rss is None else int(rss) / 2**20:7.1f}MiB "
                              f"samples {len(row['samples'])} exit {row['exit_code']}", flush=True)

    groups = {}
    domains = {}
    for row in rows:
        previous = None
        for sample in row["samples"]:
            key = (row["workload"], row["corpus"],
                   ":".join(f"{k}={v}" for k, v in sorted(row["settings"].items())),
                   sample["sample"], row["arm"])
            reading = metrics(sample, previous)
            groups.setdefault(key, []).append(reading)
            previous = sample.get("cache") or {}
            identity = (row["workload"], row["corpus"], row["sample"] if "sample" in row
                        else row["workload"], sample["sample"])
            domains.setdefault(identity, {}).setdefault(row["arm"], set()).add(
                json.dumps(reading["domain"]))

    print()
    print("=== medians per (cell, sample, arm); n is what the file really holds ===")
    print(f"{'workload':<8} {'corpus':<24} {'factors':<16} {'sample':<32} {'arm':<3} {'n':>2} "
          f"{'request_us':>10} {'sequence_us':>11} {'window_us':>9} {'mat':>4} {'prep':>4} "
          f"{'entry_bytes':>11} {'class_bytes':>11} {'def h/c/s':>10} {'retained':>9} {'refused':>7}")
    for (workload, corpus, settings, sample, arm), readings in sorted(groups.items()):
        def median(field):
            values = [r[field] for r in readings if r[field] is not None]
            return "–" if not values else f"{statistics.median(values):.0f}"
        print(f"{workload:<8} {corpus[:23]:<24} {settings[:15]:<16} {sample[:31]:<32} {arm:<3} "
              f"{len(readings):>2} {median('request_us'):>10} {median('sequence_us'):>11} "
              f"{median('window_us'):>9} {median('class_materializations'):>4} "
              f"{median('class_preparations'):>4} {median('entry_bytes'):>11} "
              f"{median('class_bytes'):>11} "
              f"{(median('definition_read_hits') + '/' + median('definition_read_consultations') + '/' + median('definition_read_stored')):>10} "
              f"{median('retained_bytes'):>9} {median('refused_capacity'):>7}")

    print()
    print("=== result identity across arms (a difference is a defect, never a reading) ===")
    for key, arms in sorted(domains.items()):
        same = arms.get("off") == arms.get("on") if len(arms) == 2 else None
        print(f"{key[0]:<4} {key[1]:<24} {key[3]:<32} identical={same} "
              f"off={sorted(arms.get('off', []))} on={sorted(arms.get('on', []))}")

    failures = [row for row in rows if row.get("exit_code") != 0 or "failure" in row]
    print(f"\nfailures: {len(failures)}")
    for row in failures[:8]:
        print(f"  {row['workload']} {row['arm']} repeat {row['repeat']}: exit {row['exit_code']} "
              f"{(row.get('failure') or '')[:200]}")
    print(f"load at end: {list(os.getloadavg())}")


if __name__ == "__main__":
    sys.exit(main())
