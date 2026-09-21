#!/usr/bin/env python3
"""Reconcile jadx's member set against jarde's declared set for one artifact (task 6.2).

jadx side: the join document the earlier campaign wrote (`timing/jadx_join_<artifact>.json`),
which lists the classes and the `(class, name, descriptor)` triples jadx's own output holds.

jarde side: a fresh `export` JSONL stream, read line by line so a 2 GB stream never has to be
held in memory. Every method record carries the class identity, the raw name/descriptor and the
delivery state, which is what makes "declared", "produced", "explanation only" and "not produced"
comparable per key.
"""
import json, os, subprocess, sys, tempfile

REPO = "/Users/lordcasser/workspace/projects/jarde"
V = "/Users/lordcasser/workspace/vulnerability/vulhub"
JOINS = "/tmp/jarde-bench/candidate-8586356/timing"
SUBJECTS = {
    "bcprov": (f"{V}/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar", "plain-jar",
               f"{JOINS}/jadx_join_weblogic__weak_password__decrypt__lib__bcprov-jdk15on-152_jar.json"),
    "s2-009": (f"{V}/struts2/s2-009/S2-009.war", "explicit-classpath",
               f"{JOINS}/jadx_join_struts2__s2-009__S2-009_war.json"),
}


def internal(dotted):
    """jadx prints source names; jarde's export states internal names with `$` for nested types."""
    return dotted.replace(".", "/")


def jadx_methods(path):
    doc = json.load(open(path))
    return {(internal(c), n, d) for c, n, d in doc["methods"]}, {internal(c) for c in doc["classes"]}


def jarde_methods(label, artifact, policy):
    out = tempfile.mktemp(suffix=".jsonl", dir="/tmp/jarde-b10")
    cmd = [f"{REPO}/target/release/jarde-cli", "export", "--input", artifact, "--policy", policy,
           "--release", "8", "--jobs", "4",
           "--scope", json.dumps({"kind": "artifact_tree", "root_container": "root"}),
           "--roots", f"@/tmp/jarde-b10/roots/{label}.json", "--output", out]
    proc = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    keys, outcomes, classes = set(), {}, set()
    with open(out) as fh:
        for line in fh:
            record = json.loads(line)
            if record.get("kind") == "class_prepared":
                # the class identity an entry records is its physical location, not a name; the
                # method records below carry the owner's internal name instead.
                continue
            if record.get("kind") != "method":
                continue
            owner = bytes(record["method"]["owner"]["location"]["entry"]["raw_name"]) \
                if "entry" in record["method"]["owner"]["location"] else b""
            name = bytes(record["method"]["name"]).decode()
            descriptor = bytes(record["method"]["descriptor"]).decode()
            state = record["delivery"]["state"]
            content = (record["delivery"].get("recovery") or {}).get("content")
            key = (name, descriptor, state, content)
            keys.add((owner.decode(), name, descriptor))
            outcomes[(name, descriptor, state, content)] = outcomes.get(
                (name, descriptor, state, content), 0) + 1
    os.path.exists(out) and os.remove(out)
    return keys, outcomes, proc.returncode


def report(label):
    artifact, policy, join = SUBJECTS[label]
    jadx_keys, jadx_classes = jadx_methods(join)
    jarde_keys, outcomes, exit_code = jarde_methods(label, artifact, policy)
    # jarde's keys carry the entry path; the comparable identity is owner class + name + descriptor.
    jarde_by_member = {(owner.rsplit("/", 1)[-1].removesuffix(".class"), n, d)
                       for owner, n, d in jarde_keys}
    jadx_by_member = {(c.rsplit("/", 1)[-1], n, d) for c, n, d in jadx_keys}
    print(f"=== {label} (jarde export exit={exit_code})")
    print(f"jadx methods: {len(jadx_keys)} over {len(jadx_classes)} classes")
    print(f"jarde methods: {len(jarde_keys)}")
    print(f"only in jadx: {len(jadx_by_member - jarde_by_member)}"
          f" | only in jarde: {len(jarde_by_member - jadx_by_member)}")
    for name in ("<clinit>", "<init>"):
        only_jadx = [k for k in jadx_by_member - jarde_by_member if k[1] == name]
        only_jarde = [k for k in jarde_by_member - jadx_by_member if k[1] == name]
        print(f"  {name}: only-jadx={len(only_jadx)} only-jarde={len(only_jarde)}")
    buckets = {}
    for (_, _, state, content), count in outcomes.items():
        buckets[(state, content)] = buckets.get((state, content), 0) + count
    print("  jarde outcomes:", json.dumps({f"{s}/{c}": n for (s, c), n in sorted(buckets.items())}))


if __name__ == "__main__":
    for label in (sys.argv[1:] or list(SUBJECTS)):
        report(label)
