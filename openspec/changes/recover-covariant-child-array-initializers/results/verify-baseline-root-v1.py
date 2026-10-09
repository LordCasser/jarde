#!/usr/bin/env python3
"""Read-only independent hash/closure/input/raw-stream verification; no target execution."""
import hashlib
import io
import json
from pathlib import Path
import re
import tarfile
import zipfile

RESULTS = Path(__file__).resolve().parent
ROOT = RESULTS.parents[3]
AUDIT = RESULTS / "baseline-reference-audit-v1.json"
OUTPUT = RESULTS / "baseline-root-verification-v1.json"
errors = []
checks = 0


def check(value, label):
    global checks
    checks += 1
    if not value:
        errors.append(label)


def digest(data):
    return hashlib.sha256(data).hexdigest()


audit = json.loads(AUDIT.read_text())
archive_path = Path(audit["old_jadx"]["archive_path"])
with tarfile.open(archive_path) as archive:
    members = [m for m in archive.getmembers() if m.isfile()]
    archived = {m.name: archive.extractfile(m).read() for m in members}
check(len(archived) == len(members) == 213, "archive unique regular-file closure")


def data_at(path):
    if "!/" in path:
        base, member = path.split("!/", 1)
        check(Path(base) == archive_path, "archive reference identity")
        return archived[member]
    return Path(path).read_bytes()


def verify_record(record, prefix=""):
    data = data_at(record["path"])
    check(digest(data) == record["sha256"], prefix + record["path"] + " hash")
    if "bytes" in record:
        check(len(data) == record["bytes"], prefix + record["path"] + " bytes")


def walk_records(value):
    if isinstance(value, dict):
        if isinstance(value.get("path"), str) and isinstance(value.get("sha256"), str):
            verify_record(value)
        for nested in value.values():
            walk_records(nested)
    elif isinstance(value, list):
        for nested in value:
            walk_records(nested)


walk_records(audit)
inventories = []
for path in [audit["numeric_legacy_regressions"]["manifest_path"],
             audit["old_heterogeneous_jarde"]["candidate_manifest_path"],
             audit["old_heterogeneous_jarde"]["replay_v2_manifest_path"]]:
    manifest_path = Path(path)
    manifest = json.loads(manifest_path.read_text())
    rows = manifest["files"]
    expected = {row["path"] for row in rows}
    actual = {str(p.relative_to(manifest_path.parent)) for p in manifest_path.parent.rglob("*")
              if p.is_file() and p != manifest_path}
    check(len(expected) == len(rows) and actual == expected, path + " closed inventory")
    for row in rows:
        verify_record({**row, "path": str(manifest_path.parent / row["path"])})
    inventories.append({"manifest": path, "files": len(rows)})

archive_files = json.loads(archived["jadx-v3-reference/files-manifest-v2.json"])
expected = {"jadx-v3-reference/" + row["path"] for row in archive_files["files"]}
excluded = {"jadx-v3-reference/" + name for name in archive_files["excluded_files"]}
check(len(expected) == 211 and set(archived) == expected | excluded, "archive payload closed set")
for row in archive_files["files"]:
    verify_record({**row, "path": str(archive_path) + "!/jadx-v3-reference/" + row["path"]})

current = audit["current_frozen_direct"]
names = ["Base", "DerivedA", "DerivedB", "LocalInterface", "Main", "Mid"]
check(current["class_names"] == names, "six original class denominator")
class_hashes = {leg: {row["class"]: row["sha256"] for row in rows}
                for leg, rows in current["class_hashes_by_jdk"].items()}
check(set(class_hashes) == {"javac8", "javac23"}, "two actual compiler input legs")


def verify_jar(data, leg, label):
    with zipfile.ZipFile(io.BytesIO(data)) as jar:
        check(sorted(n[:-6] for n in jar.namelist() if n.endswith(".class")) == names,
              label + " complete six classes")
        for name in names:
            check(digest(jar.read(name + ".class")) == class_hashes[leg][name], label + "/" + name)


for case in audit["numeric_legacy_regressions"]["direct_cases"]:
    verify_jar(data_at(case["copied_input_jar"]["path"]), case["leg"], "numeric/" + case["leg"])
    check([row["class"] for row in case["jarde_sources"]] == names, "numeric complete sources")
    check(case["candidate_compile"]["exit"] == 1 and not case["candidate_success"],
          "numeric baseline failure preserved")

originals = {leg["leg"]: leg["original_run"] for leg in audit["direct_javap_evidence"]["legs"]}
fixture = Path(current["root"]).parent
jadx_semantics = []
for case in audit["old_jadx"]["direct_legs"]:
    leg = case["leg"]
    verify_jar(archived[f"jadx-v3-reference/inputs/direct-{leg}.jar"], leg, "jadx/" + case["key"])
    check([row["class"] for row in case["generated_sources"]] == names, "JADX complete source set")
    runtime = case["commands"]["runtime"]
    original = originals[leg]
    same = runtime["exit"] == original["exit"] and all(
        data_at(runtime["streams"][stream]["path"]) == (fixture / original[stream + "_path"]).read_bytes()
        for stream in ("stdout", "stderr"))
    check(same == (case["profile"] == "rename-flags-none"), "JADX profile raw semantics")
    jadx_semantics.append({"key": case["key"], "matches_original": same})

expected_stores = {"numberGridDirect": [19, 20, 37, 38],
                   "collectionGridDirect": [19, 20, 36, 37], "ownGridDirect": [23, 24, 44, 45]}
for leg in audit["direct_javap_evidence"]["legs"]:
    command = next(c for c in leg["commands"] if c["label"].endswith("javap-main"))
    listing = data_at(command["streams"]["stdout"]["path"]).decode()
    for name, stores in expected_stores.items():
        block = listing.split(" " + name + "(", 1)[1].split("\n  public ", 1)[0]
        actual = [int(n) for n in re.findall(r"^\s*(\d+):\s+aastore", block, re.M)]
        check(actual == stores, leg["leg"] + "/" + name + " physical stores")

report = {"verification": "passed" if not errors else "failed", "checks": checks, "errors": errors,
          "audit_sha256": digest(AUDIT.read_bytes()), "inventories": inventories,
          "archive_files": len(archived), "archive_payload_files": len(expected),
          "jadx_direct_semantics": jadx_semantics, "fresh_execution": False,
          "scope": "Independent existing-byte verification; no original/JADX/Jarde rerun; exact prior code CI remains separate"}
assert not OUTPUT.exists(), "never overwrite root evidence"
OUTPUT.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report))
raise SystemExit(bool(errors))
