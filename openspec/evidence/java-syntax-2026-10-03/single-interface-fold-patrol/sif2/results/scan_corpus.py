#!/usr/bin/env python3
"""One leg of the sif2 jar corpus scan (change `recover-single-static-interface-fold`).

Per jar under `tests/fixtures`, `fuzz/corpus` and `openspec/evidence`, per class candidate the jar
lists, the SHA256 of `class-source --format text` on stdout with its trailing newlines stripped —
the convention the committed sif/fcp legs use (a shell `$(...)` capture, which also strips the
trailing blank line). A non-zero exit records that exit instead of a digest.

Usage: scan_corpus.py <jarde-cli> <manifest-out>
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

cli = Path(sys.argv[1]).resolve()
manifest = Path(sys.argv[2])
jars = sorted(
    path
    for tree in ("tests/fixtures", "fuzz/corpus", "openspec/evidence")
    for path in Path(tree).rglob("*.jar")
)
rows = []
failures = 0
for jar in jars:
    listed = subprocess.run(
        [
            str(cli),
            "list-classes",
            "--input",
            str(jar),
            "--evidence",
            "candidates",
            "--format",
            "json",
        ],
        capture_output=True,
    )
    try:
        document = json.loads(listed.stdout)
    except json.JSONDecodeError:
        rows.append((str(jar), "<listing-failed>", f"EXIT{listed.returncode}"))
        continue
    names = []
    for item in document.get("items", []):
        if item.get("kind") != "class_candidate":
            continue
        raw = bytes(item["location"]["entry"]["raw_name"]).decode("utf-8", "replace")
        if raw.endswith(".class"):
            names.append(raw[: -len(".class")].replace("/", "."))
    for internal in sorted(set(names)):
        answer = subprocess.run(
            [
                str(cli),
                "class-source",
                "--input",
                str(jar),
                "--class",
                internal,
                "--format",
                "text",
            ],
            capture_output=True,
        )
        if answer.returncode != 0:
            failures += 1
            rows.append((str(jar), internal, f"EXIT{answer.returncode}"))
            continue
        rows.append(
            (str(jar), internal, hashlib.sha256(answer.stdout.rstrip()).hexdigest())
        )
with manifest.open("w") as out:
    for jar, internal, value in rows:
        separator = "-" if value.startswith("EXIT") else ""
        out.write(f"{value} {separator} {jar}:{internal}".replace("  ", " ") + "\n")
print(f"jars={len(jars)} classes={len(rows)} failures={failures}")