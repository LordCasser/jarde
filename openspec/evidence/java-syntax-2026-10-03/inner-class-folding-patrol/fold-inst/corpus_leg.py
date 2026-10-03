#!/usr/bin/env python3
"""Capture one leg of the instance-fold corpus scan: per jar, per class, the SHA256 of
`class-source --format text`. Usage: corpus_leg.py <jarde-cli> <manifest-out>"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

cli = Path(sys.argv[1]).resolve()
manifest = Path(sys.argv[2])
root = Path(__file__).resolve().parents[3]  # .../openspec/evidence
jars = sorted(p for p in root.rglob("*.jar") if "target" not in p.parts)
rows = []
failures = 0
for jar in jars:
    listed = subprocess.run(
        [str(cli), "list-classes", "--input", str(jar), "--format", "json"],
        capture_output=True,
    )
    try:
        document = json.loads(listed.stdout)
    except json.JSONDecodeError:
        rows.append((str(jar), "<listing-failed>", hashlib.sha256(listed.stdout).hexdigest()))
        continue

    def walk(node):
        if isinstance(node, dict):
            for key, value in node.items():
                if key == "raw_name" and isinstance(value, list):
                    yield bytes(value)
                else:
                    yield from walk(value)
        elif isinstance(node, list):
            for item in node:
                yield from walk(item)

    names = sorted({name.decode() for name in walk(document) if name.endswith(b".class")})
    for entry in names:
        internal = entry[: -len(".class")]
        answer = subprocess.run(
            [str(cli), "class-source", "--input", str(jar), "--class", internal, "--format", "text"],
            capture_output=True,
        )
        if answer.returncode != 0:
            failures += 1
            rows.append((str(jar), internal, f"exit{answer.returncode}"))
            continue
        rows.append((str(jar), internal, hashlib.sha256(answer.stdout).hexdigest()))
with manifest.open("w") as out:
    for jar, internal, digest in rows:
        out.write(f"{digest}  {jar}:{internal}\n")
print(f"jars={len(jars)} classes={len(rows)} failures={failures}")
