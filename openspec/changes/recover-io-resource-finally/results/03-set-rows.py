#!/usr/bin/env python3
"""Toggle the two `java.io` rows of `platform_reference_argument_widens`, idempotently.

Usage: 03-set-rows.py <build.rs> on|off

`on` inserts the two rows and their comment after the java.util table's own tail, `off` removes
them; a file already in the requested state is left untouched. The script is part of the
per-component gating (`03-component-gating.sh`) and refuses to guess: it stops if the table's tail
is not where it expects it.
"""

import sys

BLOCK = """        // The two `java.io` edges of the three-layer stream chain: `FileInputStream` at
        // `InputStreamReader(InputStream, String)` and `InputStreamReader` at
        // `BufferedReader(Reader)`. Each is the class's own `extends` clause (javap,
        // release 8), transcribed with its header in the widening-row-sources protocol file.
        ("java.io.FileInputStream", "java.io.InputStream"),
        ("java.io.InputStreamReader", "java.io.Reader"),
"""
TAIL = """        ("java.util.NavigableMap", "java.util.SortedMap"),
    ];"""


def main() -> int:
    path, wanted = sys.argv[1], sys.argv[2] == "on"
    text = open(path).read()
    present = BLOCK in text
    if wanted == present:
        return 0
    if wanted:
        if TAIL not in text:
            print("the java.util table's tail is not where this script expects it", file=sys.stderr)
            return 2
        text = text.replace(TAIL, TAIL.replace("    ];", "") + BLOCK + "    ];", 1)
    else:
        text = text.replace(BLOCK, "", 1)
    open(path, "w").write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
