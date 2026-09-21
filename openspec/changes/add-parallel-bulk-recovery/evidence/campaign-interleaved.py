#!/usr/bin/env python3
"""Protocol-shaped campaign: A–E arms over the frozen corpus.

A  frozen per-method sweep, no store          (jarde-sweep3 @8807fa5, the historical anchor)
B  frozen per-method sweep, retaining store   (same binary, --cache)
C  bulk export, workers=1                     (this change's CLI, release)
D  bulk export, workers=2/4/6                 (same, only the worker count changes)
E  degenerate shapes                          (tiny output allowance, skewed artifact, cancel)

Interleaved repeats per configuration; every sample is appended to raw2.jsonl. Nothing here
asserts a speed ratio — the samples and the machine's load record are the evidence.
"""
import json, os, statistics, subprocess, time

REPO = "/Users/lordcasser/workspace/projects/jarde"
V = "/Users/lordcasser/workspace/vulnerability/vulhub"
OUT = "/tmp/jarde-b10"
RAW = f"{OUT}/raw2.jsonl"
SWEEP = "/tmp/jarde-bench/sweep3/target/release/jarde-sweep3"
SUBJECTS = {
    "bcprov": (f"{V}/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar", "plain-jar"),
    "s2-009": (f"{V}/struts2/s2-009/S2-009.war", "explicit-classpath"),
}


def record(row):
    with open(RAW, "a") as fh:
        fh.write(json.dumps(row, sort_keys=True) + "\n")


def timed(cmd, label, artifact, arm, config, out=None, pre=None):
    if pre:
        pre()
    t0 = time.perf_counter()
    proc = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    wall = time.perf_counter() - t0
    size = os.path.getsize(out) if out and os.path.exists(out) else None
    row = {"artifact": artifact, "arm": arm, "config": config, "wall_s": round(wall, 3),
           "exit": proc.returncode, "output_bytes": size}
    record(row)
    print(f"{label:<44} {wall:8.2f}s exit={proc.returncode}"
          f"{'' if size is None else f' out={size/1e6:.1f}MB'}", flush=True)
    return row


def sweep_cmd(artifact, cache):
    return [SWEEP, "--artifact", artifact, "--label", "campaign", "--mode", "full",
            "--cache", cache, "--out", f"{OUT}/sweep-{cache.split(':')[0]}.jsonl"]


def export_cmd(label, artifact, policy, jobs, out):
    return [f"{REPO}/target/release/jarde-cli", "export", "--input", artifact, "--policy", policy,
            "--release", "8", "--jobs", str(jobs),
            "--scope", json.dumps({"kind": "artifact_tree", "root_container": "root"}),
            "--roots", f"@{OUT}/roots/{label}.json",
            "--output", out]


def main():
    os.makedirs(OUT, exist_ok=True)
    reps = int(os.environ.get("REPS", "10"))
    arms = os.environ.get("ARMS", "C,D")
    for rep in range(reps):
        for label, (artifact, policy) in SUBJECTS.items():
            out = f"{OUT}/campaign-{label}.jsonl"
            jobs_list = [1] if "C" in arms else []
            if "D" in arms:
                jobs_list += [2, 4, 6]
            for jobs in jobs_list:
                arm = "C" if jobs == 1 else "D"
                timed(export_cmd(label, artifact, policy, jobs, out),
                      f"{label} {arm} jobs={jobs} rep{rep}", label, arm, f"jobs={jobs}",
                      out=out, pre=lambda: os.path.exists(out) and os.remove(out))
            if "A" in arms:
                timed(sweep_cmd(artifact, "off"),
                      f"{label} A sweep/no-store rep{rep}", label, "A", "cache=off")
            if "B" in arms:
                timed(sweep_cmd(artifact, "retaining:2000,67108864"),
                      f"{label} B sweep/retaining rep{rep}", label, "B", "cache=retaining")
            if "E" in arms:
                # A whole-scope export whose output allowance cannot hold the stream: the run must
                # stop with a readable prefix rather than overshoot its own declaration.
                timed(export_cmd(label, artifact, policy, 4, out) + ["--budget", "output_bytes=1048576"],
                      f"{label} E tiny output allowance rep{rep}", label, "E", "output_bytes=1MiB",
                      out=out, pre=lambda: os.path.exists(out) and os.remove(out))
    summarize()


def summarize():
    rows = [json.loads(line) for line in open(RAW)]
    groups = {}
    for row in rows:
        groups.setdefault((row["artifact"], row["arm"], row["config"]), []).append(row)
    print("\n--- samples ---")
    print(f"{'artifact':<10}{'arm':<4}{'config':<22}{'n':>3}{'median s':>10}{'min':>8}{'max':>8}{'exits':>10}")
    for (artifact, arm, config), samples in sorted(groups.items()):
        walls = sorted(s["wall_s"] for s in samples)
        exits = sorted({s["exit"] for s in samples})
        print(f"{artifact:<10}{arm:<4}{config:<22}{len(samples):>3}"
              f"{statistics.median(walls):>10.2f}{walls[0]:>8.2f}{walls[-1]:>8.2f}{str(exits):>10}")


if __name__ == "__main__":
    main()
