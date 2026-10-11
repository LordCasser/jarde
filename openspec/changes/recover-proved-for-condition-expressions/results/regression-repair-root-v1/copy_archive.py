#!/usr/bin/env python3
"""Copy only the reviewed inputs listed in copy-plan.json.

Dry run is the default. To copy, provide --destination DIR and --apply.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
PLAN = HERE / "copy-plan.json"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, required=True, help="archive root directory")
    parser.add_argument("--apply", action="store_true", help="perform copies; without this flag only validate and print")
    args = parser.parse_args()

    plan = json.loads(PLAN.read_text(encoding="utf-8"))
    root = args.destination.expanduser().resolve()
    if args.apply:
        root.mkdir(parents=True, exist_ok=True)

    # Validate every source and every destination before writing any file.
    prepared = []
    for item in plan["entries"]:
        source = Path(item["source"])
        if not source.is_file():
            raise SystemExit(f"missing source: {source}")
        actual_bytes = source.stat().st_size
        actual_sha = sha256(source)
        if actual_bytes != item["bytes"] or actual_sha != item["sha256"]:
            raise SystemExit(f"source changed since review: {source}")

        relative = Path(item["destination_relative"])
        if relative.is_absolute() or ".." in relative.parts:
            raise SystemExit(f"unsafe destination path: {relative}")
        destination = (root / relative).resolve()
        if destination != root and root not in destination.parents:
            raise SystemExit(f"destination escapes archive root: {relative}")
        if destination.exists():
            if not destination.is_file():
                raise SystemExit(f"destination exists and is not a file: {destination}")
            if destination.stat().st_size != item["bytes"] or sha256(destination) != item["sha256"]:
                raise SystemExit(f"refusing to overwrite differing destination: {destination}")
        prepared.append((source, destination, item))

    mode = "COPY" if args.apply else "DRY-RUN"
    print(f"{mode}: {len(prepared)} verified files -> {root}")
    if not args.apply:
        return 0

    for source, destination, item in prepared:
        if destination.exists():
            print(f"SKIP identical {item['destination_relative']}")
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        temporary = destination.with_name(destination.name + ".archive-copy-tmp")
        try:
            shutil.copyfile(source, temporary)
            if temporary.stat().st_size != item["bytes"] or sha256(temporary) != item["sha256"]:
                raise SystemExit(f"copied bytes failed verification: {item['destination_relative']}")
            os.replace(temporary, destination)
        finally:
            if temporary.exists():
                temporary.unlink()
        print(f"COPIED {item['destination_relative']}")
    print("All archive copies verified.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
