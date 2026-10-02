#!/usr/bin/env python3
"""Corpus before/after scan for `recover-nested-type-source-spelling`.

Walks every `.class` artifact under the repository's `tests/` trees, asks the given jarde-cli leg
for each class's `class-source` text, and writes one output file per artifact into the destination
directory. Run once per leg (baseline and changed); `diff -r` between the two destinations is the
artifact-level comparison the slice's acceptance records: the only expected differences are the
nested `$` names the change re-spells.

The class's own internal name is read from the file's `this_class` (a minimal constant-pool walk),
so packaged fixtures resolve without guessing a root. Failures (unspellable names, budget stops)
are recorded in the output file rather than aborting the scan, so both legs see the same shape.
"""

import struct
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[5] / "tests"

TAG_SIZES = {
    1: 1,  # Utf8: u2 length + bytes (handled specially)
    3: 4,
    4: 4,
    5: 8,
    6: 8,
    7: 2,
    8: 2,
    9: 4,
    10: 4,
    11: 4,
    12: 4,
    15: 3,
    16: 2,
    17: 4,
    18: 4,
    19: 2,
    20: 2,
}


def this_class_name(data):
    """The internal name a class file's own `this_class` states, or None if unreadable."""
    try:
        if data[:4] != b"\xca\xfe\xba\xbe":
            return None
        cursor = 8
        (count,) = struct.unpack_from(">H", data, cursor)
        cursor += 2
        utf8 = {}
        class_names = {}
        index = 1
        while index < count:
            tag = data[cursor]
            cursor += 1
            if tag == 1:
                (length,) = struct.unpack_from(">H", data, cursor)
                cursor += 2
                utf8[index] = data[cursor : cursor + length].decode("utf-8", "replace")
                cursor += length
                index += 1
                continue
            if tag == 7:
                (name_index,) = struct.unpack_from(">H", data, cursor)
                class_names[index] = name_index
            size = TAG_SIZES.get(tag)
            if size is None:
                return None
            cursor += size
            # JVMS 4.4.5: `long` and `double` take two pool slots.
            index += 2 if tag in (5, 6) else 1
        # After the pool: access_flags (u2), then this_class (u2) — a Class entry whose own
        # name_index states the internal name.
        (this_class,) = struct.unpack_from(">H", data, cursor + 2)
        return utf8.get(class_names.get(this_class, 0))
    except (struct.error, IndexError):
        return None


def main() -> int:
    cli, destination = Path(sys.argv[1]), Path(sys.argv[2])
    destination.mkdir(parents=True, exist_ok=True)
    scanned = failed = 0
    for artifact in sorted(REPO.rglob("*.class")):
        data = artifact.read_bytes()
        internal = this_class_name(data) or artifact.stem
        out = destination / artifact.relative_to(REPO).with_suffix(".txt")
        out.parent.mkdir(parents=True, exist_ok=True)
        run = subprocess.run(
            [
                str(cli),
                "class-source",
                "--input",
                str(artifact),
                "--class",
                internal,
                "--policy",
                "single-class",
            ],
            capture_output=True,
            text=True,
        )
        out.write_text(run.stdout if run.returncode == 0 else f"EXIT {run.returncode}\n")
        scanned += 1
        failed += run.returncode != 0
    print(f"scanned {scanned} class files into {destination} ({failed} non-zero exits)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
