#!/usr/bin/env python3
"""Deterministic generator for the bounded P1 fuzz corpus (design 3.3).

The corpus is deliberately tiny and byte-stable. Every seed is either the smallest
artifact that reaches a scan (one class, one jar) or a damaged variant of it:

    corpus/query/minimal-class          standalone CLASS root with one `probe` body
    corpus/query/minimal-jar            ZIP with the class, a Manifest and a service file
    corpus/query/damaged-candidate.jar  same ZIP with the class magic broken
    corpus/query/truncated-class        the valid class cut inside the header
    corpus/artifact_tree/minimal-jar            one class entry
    corpus/artifact_tree/multi-release.jar      base + META-INF/versions/9 variant
    corpus/artifact_tree/nested.jar             a jar stored inside another jar
    corpus/artifact_tree/damaged-entry.jar      stored class payload with a broken magic
    corpus/artifact_tree/truncated-jar          the minimal jar cut before its EOCD

The class the seeds carry is `com/example/Seed`, whose `probe()V` body mentions the
fixed fuzz symbol `com/example/Fuzz.target:()V` (an `invokestatic`), the class
`com/example/Fuzz` (a `new`) and the string literal `fuzz-literal`. Those are the three
targets the `query` target asks about, so every valid seed answers at least one request
shape without any mutation.

Regenerate (rewrites `corpus/`, prints the digest of every file):

    python3 fuzz/corpus/generate_seeds.py

Nothing here drives the fuzzer: cargo-fuzz mutates these seeds itself. The fuzz targets
never call this script, and the smoke gate never needs it.
"""

from __future__ import annotations

import hashlib
import io
import pathlib
import zipfile

HERE = pathlib.Path(__file__).resolve().parent

CALLEE_OWNER = b"com/example/Fuzz"
CALLEE_MEMBER = b"target"
CALLEE_DESCRIPTOR = b"()V"
LITERAL = b"fuzz-literal"
THIS_CLASS = b"com/example/Seed"

MANIFEST = b"Manifest-Version: 1.0\r\nMain-Class: com.example.Seed\r\n\r\n"
MR_MANIFEST = (
    b"Manifest-Version: 1.0\r\n"
    b"Main-Class: com.example.Seed\r\n"
    b"Multi-Release: true\r\n"
    b"\r\n"
)
SERVICE_KEY = b"com.example.Seed"


def u2(value: int) -> bytes:
    return value.to_bytes(2, "big")


def u4(value: int) -> bytes:
    return value.to_bytes(4, "big")


class Pool:
    """Constant pool builder; indexes are 1-based like the class file format."""

    def __init__(self) -> None:
        self.entries: list[bytes] = []

    def add(self, entry: bytes) -> int:
        self.entries.append(entry)
        return len(self.entries)

    def utf8(self, text: bytes) -> int:
        return self.add(b"\x01" + u2(len(text)) + text)

    def class_(self, name_index: int) -> int:
        return self.add(b"\x07" + u2(name_index))

    def string(self, value_index: int) -> int:
        return self.add(b"\x08" + u2(value_index))

    def name_and_type(self, name: int, descriptor: int) -> int:
        return self.add(b"\x0c" + u2(name) + u2(descriptor))

    def methodref(self, class_index: int, name_and_type: int) -> int:
        return self.add(b"\x0a" + u2(class_index) + u2(name_and_type))

    def bytes(self) -> bytes:
        return b"".join(self.entries)


def seed_class(literal: bytes = LITERAL) -> bytes:
    """The smallest class whose `probe()V` body carries all three fixed targets."""
    pool = Pool()
    this_class = pool.class_(pool.utf8(THIS_CLASS))
    super_class = pool.class_(pool.utf8(b"java/lang/Object"))
    probe = pool.utf8(b"probe")
    void_descriptor = pool.utf8(b"()V")
    code_attribute = pool.utf8(b"Code")
    callee_owner = pool.class_(pool.utf8(CALLEE_OWNER))
    callee = pool.methodref(
        callee_owner,
        pool.name_and_type(pool.utf8(CALLEE_MEMBER), void_descriptor),
    )
    literal_index = pool.string(pool.utf8(literal))

    code = (
        b"\xbb" + u2(callee_owner)   # new com/example/Fuzz
        + b"\x57"                    # pop
        + b"\xb8" + u2(callee)       # invokestatic com/example/Fuzz.target:()V
        + b"\x12" + bytes([literal_index])  # ldc "fuzz-literal"
        + b"\x57"                    # pop
        + b"\xb1"                    # return
    )
    body = u2(2) + u2(0) + u4(len(code)) + code + u2(0) + u2(0)
    method = (
        u2(0x0009) + u2(probe) + u2(void_descriptor)
        + u2(1) + u2(code_attribute) + u4(len(body)) + body
    )

    return (
        u4(0xCAFEBABE)
        + u2(0)                     # minor
        + u2(52)                    # major: Java 8
        + u2(len(pool.entries) + 1)
        + pool.bytes()
        + u2(0x0021) + u2(this_class) + u2(super_class)
        + u2(0)                     # interfaces
        + u2(0)                     # fields
        + u2(1) + method            # methods
        + u2(0)                     # class attributes
    )


def zip_bytes(entries: list[tuple[bytes, bytes]]) -> bytes:
    """A STORED zip with a fixed timestamp, so the bytes are reproducible."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
        for name, data in entries:
            info = zipfile.ZipInfo(name.decode("utf-8"), date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 0
            info.external_attr = 0
            archive.writestr(info, data)
    return buffer.getvalue()


def minimal_jar() -> bytes:
    return zip_bytes(
        [
            (b"META-INF/MANIFEST.MF", MANIFEST),
            (b"com/example/Seed.class", seed_class()),
            (b"META-INF/services/" + SERVICE_KEY, b"com.example.Seed\n"),
        ]
    )


def multi_release_jar() -> bytes:
    return zip_bytes(
        [
            (b"META-INF/MANIFEST.MF", MR_MANIFEST),
            (b"com/example/Seed.class", seed_class()),
            (b"META-INF/versions/9/com/example/Seed.class", seed_class(b"fuzz-literal-9")),
        ]
    )


def nested_jar() -> bytes:
    return zip_bytes([(b"lib/inner.jar", minimal_jar())])


def damaged_candidate() -> bytes:
    """A ZIP whose stored class payload starts with the wrong magic.

    The archive is built from the damaged payload, so its CRC is consistent: the entry
    really materializes and the *class candidate* rule is what rejects it, instead of the
    ZIP integrity check that a rewritten byte would trip first.
    """
    broken = bytes([0x00]) + seed_class()[1:]
    return zip_bytes(
        [
            (b"META-INF/MANIFEST.MF", MANIFEST),
            (b"com/example/Seed.class", broken),
        ]
    )


def corrupt_payload() -> bytes:
    """A structurally intact ZIP whose stored class payload contradicts its CRC."""
    bytes_ = minimal_jar()
    position = bytes_.index(seed_class()[:4])
    return bytes_[:position] + bytes([0x00]) + bytes_[position + 1 :]


def truncated(bytes_: bytes, keep: int) -> bytes:
    return bytes_[:keep]


SEEDS: dict[str, dict[str, bytes]] = {
    "query": {
        "minimal-class": seed_class(),
        "minimal-jar": minimal_jar(),
        "damaged-candidate.jar": damaged_candidate(),
        "corrupt-payload.jar": corrupt_payload(),
        "truncated-class": truncated(seed_class(), 20),
    },
    "artifact_tree": {
        "minimal-jar": minimal_jar(),
        "multi-release.jar": multi_release_jar(),
        "nested.jar": nested_jar(),
        "damaged-entry.jar": damaged_candidate(),
        "truncated-jar": truncated(minimal_jar(), 80),
    },
}


def main() -> None:
    for target, seeds in SEEDS.items():
        directory = HERE / target
        directory.mkdir(parents=True, exist_ok=True)
        for name, data in seeds.items():
            (directory / name).write_bytes(data)
            print(f"{hashlib.sha256(data).hexdigest()}  {len(data):>6}  {target}/{name}")


if __name__ == "__main__":
    main()
