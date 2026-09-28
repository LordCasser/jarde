#!/usr/bin/env python3
"""Make fixed-size, verifier-valid near neighbors of the pinned Test4 class.

The two call mutations change a constant-pool UTF8 name shared by both copies. The
range mutation changes only the first exception-table start. No instruction length,
stack-map offset, or class layout is changed.
"""

from pathlib import Path
import hashlib
import struct
import sys

HERE = Path(__file__).resolve().parent
SOURCE = HERE / "classes/jadx/tests/integration/trycatch/TestTryCatchFinally4$TestCls.class"
EXPECTED = "2bf1b8932e521aade29fa1d562269903f9b8e9942f86981eb458d4f552d67060"
CLASS = "TestTryCatchFinally4$TestCls.class"
PACKAGE = Path("jadx/tests/integration/trycatch")


def one(data: bytes, old: bytes, new: bytes) -> bytes:
    assert len(old) == len(new) and data.count(old) == 1, old
    return data.replace(old, new, 1)


def main(output: Path) -> None:
    fixed = SOURCE.read_bytes()
    assert hashlib.sha256(fixed).hexdigest() == EXPECTED
    # UTF8 entries carry a two-byte length immediately before the text.
    variants = {
        "wrong-close-call": one(fixed, b"\x00\x05close", b"\x00\x05flush"),
        "wrong-delete-effect": one(fixed, b"\x00\x06delete", b"\x00\x06exists"),
        "wrong-catch-type": one(
            fixed, b"\x00\x13java/io/IOException", b"\x00\x13java/lang/Throwable"
        ),
        # The first named catch no longer catches IOException from close at BCI 23.
        "wrong-cleanup-coverage": one(
            fixed,
            struct.pack(">HHHH", 22, 31, 34, 35),
            struct.pack(">HHHH", 26, 31, 34, 35),
        ),
    }
    for name, data in variants.items():
        destination = output / name / PACKAGE / CLASS
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        print(f"{name} {hashlib.sha256(data).hexdigest()} {destination}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: mutate-neighbors.py OUTPUT_DIR")
    main(Path(sys.argv[1]))
