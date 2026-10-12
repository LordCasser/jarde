#!/usr/bin/env python3
"""Check real published transfer mappings in eight frozen retry reports."""
import hashlib
import json
import sys
from pathlib import Path

base = Path(sys.argv[1])
observed = []
for version, expected in (("v2", {46: "break;", 92: "continue;"}),
                          ("v3", {26: "break;", 52: "continue;"})):
    for lane in ("debug-javac8", "none-javac8", "debug-javac23", "none-javac23"):
        path = base / "candidate" / f"retry-{version}-{lane}" / "report.json"
        data = path.read_bytes()
        doc = json.loads(data)
        method = next(m for m in doc["methods"]
                      if m["item"]["name"]["escaped"] == "test"
                      and m["item"]["descriptor"]["escaped"] == "()I")
        identity = method["item"]["identity"]
        report = method["outcome"]["report"]
        text = report["text"].encode("utf-8")
        found = {}
        for segment in report["source_map"]["segments"]:
            origin = segment["origin"]
            anchors = [origin["primary"], *origin["derived"]]
            for anchor in anchors:
                assert anchor["method"] == identity
                bci = anchor["bci"]
                if bci not in expected:
                    continue
                start, end = segment["start"], segment["end"]
                assert 0 <= start < end <= len(text)
                mapped = text[start:end].decode("utf-8").strip()
                assert mapped == expected[bci], (path, bci, mapped)
                found.setdefault(bci, []).append({"start": start, "end": end,
                                                "text": mapped,
                                                "provenance": anchor["provenance"]})
        assert set(found) == set(expected), (path, found)
        observed.append({"lane": f"{version}-{lane}", "physical_method": identity,
                         "report_sha256": hashlib.sha256(data).hexdigest(),
                         "transfers": found})
print(json.dumps({"status": "accepted_published_transfer_maps_only",
                  "reports": observed, "product_runtime_accepted": False}, indent=2))
