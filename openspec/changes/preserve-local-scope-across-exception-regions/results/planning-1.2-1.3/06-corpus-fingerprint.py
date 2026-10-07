#!/usr/bin/env python3
"""The slice's corpus census: every class file committed under `tests/fixtures/` is rendered through
the shipped entry point, and the manifest pairs each input's digest with its render's digest.

This slice changes no recovery code, so the manifest is the standing zero-regression control: a later
region/guard re-slice diffs its own manifest against this one and classifies every line.

The class name a standalone file is rendered under is read out of the file's own constant pool (the
`this_class` entry), so the hand-patched fixtures whose internal name differs from their file name are
rendered rather than reported as missing.

Usage: 06-corpus-fingerprint.py [output]
"""

import hashlib
import pathlib
import struct
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
CLI = pathlib.Path(__file__).resolve().parents[5] / "target/debug/jarde-cli"
DEFAULT_OUT = pathlib.Path(__file__).resolve().parent / "fingerprint.txt"


def internal_name(data: bytes) -> str:
    """The class file's own `this_class` name, read from the constant pool."""
    count = struct.unpack_from(">H", data, 8)[0]
    pool: list[bytes | int | None] = [None] * count
    at = 10
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            length = struct.unpack_from(">H", data, at)[0]
            pool[index] = data[at + 2 : at + 2 + length]
            at += 2 + length
        elif tag == 7:
            pool[index] = struct.unpack_from(">H", data, at)[0]
            at += 2
        elif tag in (8, 16, 19, 20):
            at += 2
        elif tag == 15:
            at += 3
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        else:
            raise SystemExit(f"unknown constant pool tag {tag} at {at}")
        index += 1
    at += 2  # access_flags
    this_class = struct.unpack_from(">H", data, at)[0]
    name_index = pool[this_class]
    if not isinstance(name_index, int):
        raise SystemExit(f"`this_class` {this_class} is not a Class entry")
    name = pool[name_index]
    if not isinstance(name, bytes):
        raise SystemExit(f"the class name index {name_index} is not a UTF-8 entry")
    return name.decode()


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def render(path: pathlib.Path) -> str:
    data = path.read_bytes()
    name = internal_name(data)
    done = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--policy",
            "single-class",
            "--input",
            str(path),
            "--class",
            name,
            "--format",
            "text",
        ],
        capture_output=True,
        check=False,
    )
    # A non-zero exit with a render on standard output is the presentation's own withheld-projection
    # answer (the text is still the artifact); only an empty standard output is an error document.
    if done.stdout:
        suffix = "" if done.returncode == 0 else f":exit{done.returncode}"
        return f"{digest(done.stdout)}{suffix}"
    code = ""
    text = done.stderr.decode(errors="replace")
    marker = '"code":"'
    if marker in text:
        code = text.split(marker, 1)[1].split('"', 1)[0]
    return f"ERR:{done.returncode}:{code}"


def main() -> int:
    out = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_OUT
    fixtures = ROOT / "tests/fixtures"

    # Self-test before counting: one known render and one known refusal, so a broken CLI or a wrong
    # invocation cannot produce a manifest of error documents.
    positive = fixtures / "preserve-local-scope-plan/v8/ScopePlan.class"
    done = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--policy",
            "single-class",
            "--input",
            str(positive),
            "--class",
            "ScopePlan",
            "--format",
            "text",
        ],
        capture_output=True,
        check=False,
    )
    if not done.stdout.startswith(b"// jarde: presentation of `ScopePlan`"):
        raise SystemExit("SELF-TEST FAILED: the positive control is not a render")
    negative = fixtures / "preserve-local-scope-plan/v8/ScopePlanCrossing.class"
    done = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--policy",
            "single-class",
            "--input",
            str(negative),
            "--class",
            "ScopePlanCrossing",
            "--format",
            "text",
        ],
        capture_output=True,
        check=False,
    )
    if b"crosses a quoted fallback region" not in done.stdout:
        raise SystemExit("SELF-TEST FAILED: the negative control keeps its refusal")

    lines = [
        "# render fingerprint over tests/fixtures/**/*.class",
        "# columns: class-bytes-sha256  render-sha256  path",
    ]
    for path in sorted(fixtures.rglob("*.class")):
        lines.append(f"{digest(path.read_bytes())}  {render(path)}  {path.relative_to(ROOT)}")
    out.write_text("\n".join(lines) + "\n")
    print(f"{len(lines) - 2} class file(s) fingerprinted into {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
