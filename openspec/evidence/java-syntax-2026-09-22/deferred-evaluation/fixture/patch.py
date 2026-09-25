"""Compile the source-only fixture and apply the verifier-valid interleaved-effect patches.

The patch deliberately edits only complete Code attributes in DeferredValueOrder.class.  It inserts
an existing DeferredEffects.mark() method reference after each producer and before the original
return, then runs the patched class with -Xverify:all.  No class-file parser or production parser is
needed: the source javap output identifies the constant-pool operands, while each complete method
body is matched byte-for-byte.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import struct
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE_ROOT = ROOT.parents[4] / "tests" / "fixtures" / "p3-deferred-value-order"
SOURCE_NAMES = ["DeferredValueOrder", "DeferredEffects", "OrderValue", "DeferredValueOrderRunner"]


def run(command: list[str], log: Path) -> None:
    result = subprocess.run(command, capture_output=True, text=True, timeout=30)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise SystemExit(f"command failed ({result.returncode}): {' '.join(command)}")


def constant(javap: str, pattern: str) -> int:
    match = re.search(pattern, javap)
    if match is None:
        raise SystemExit(f"constant-pool entry not found: {pattern}")
    return int(match.group(1))


def code_attribute(code: bytes, max_stack: int, max_locals: int) -> bytes:
    payload = struct.pack(">HHI", max_stack, max_locals, len(code)) + code + b"\0\0\0\0"
    return struct.pack(">I", len(payload)) + payload


def patch_once(
    data: bytes,
    name: str,
    code: bytes,
    max_stack: int,
    max_locals: int,
    invoke_mark: bytes,
) -> tuple[bytes, dict[str, str]]:
    after = code[:-1] + invoke_mark + code[-1:]
    before_attribute = code_attribute(code, max_stack, max_locals)
    after_attribute = code_attribute(after, max_stack, max_locals)
    if data.count(before_attribute) != 1:
        raise SystemExit(f"{name}: complete Code attribute was not unique")
    return data.replace(before_attribute, after_attribute, 1), {
        "method": name,
        "before": code.hex(),
        "after": after.hex(),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--work",
        type=Path,
        default=Path("/tmp/jarde-deferred-value-order-fixture"),
        help="empty temporary output directory",
    )
    args = parser.parse_args()
    work = args.work.resolve()
    if work.exists():
        if any(work.iterdir()):
            raise SystemExit(f"refusing to replace non-empty work directory: {work}")
    else:
        work.mkdir(parents=True)
    source = work / "source"
    original = work / "original"
    source.mkdir(parents=True)
    original.mkdir()
    for name in SOURCE_NAMES:
        shutil.copy2(SOURCE_ROOT / f"{name}.java", source / f"{name}.java")

    source_javac = work / "source-javac.log"
    run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(original),
            *[str(source / f"{name}.java") for name in SOURCE_NAMES],
        ],
        source_javac,
    )
    original_class = original / "DeferredValueOrder.class"
    source_javap = work / "source-javap.txt"
    run(["javap", "-verbose", "-classpath", str(original), "DeferredValueOrder"], source_javap)
    javap = source_javap.read_text()

    value = constant(javap, r"#(\d+) = Methodref.*// DeferredEffects\.value:\(\)I")
    field = constant(javap, r"#(\d+) = Fieldref.*// DeferredEffects\.field:I")
    order_value_field = constant(javap, r"#(\d+) = Fieldref.*// OrderValue\.value:I")
    divisor = constant(javap, r"#(\d+) = Fieldref.*// DeferredEffects\.divisor:I")
    long_divisor = constant(javap, r"#(\d+) = Fieldref.*// DeferredEffects\.longDivisor:J")
    mark = constant(javap, r"#(\d+) = Methodref.*// DeferredEffects\.mark:\(\)V")
    string = constant(javap, r"#(\d+) = Class.*// java/lang/String")
    multi_array = constant(javap, r'#(\d+) = Class.*// "\[\[I"')
    order_value = constant(javap, r"#(\d+) = Class.*// OrderValue")
    constructor = constant(javap, r'#(\d+) = Methodref.*// OrderValue\."<init>":\(\)V')
    invoke_mark = b"\xb8" + struct.pack(">H", mark)

    patches = [
        ("call", b"\xb8" + struct.pack(">H", value) + b"\xac", 1, 0),
        ("field", b"\xb2" + struct.pack(">H", field) + b"\xac", 1, 0),
        ("array", bytes.fromhex("2a032eac"), 2, 1),
        (
            "instanceField",
            b"\x2a\xb4" + struct.pack(">H", order_value_field) + b"\xac",
            1,
            1,
        ),
        ("arrayLength", bytes.fromhex("2abeac"), 1, 1),
        ("newArray", bytes.fromhex("1abc0ab0"), 1, 1),
        (
            "anewArray",
            b"\x1a\xbd" + struct.pack(">H", string) + b"\xb0",
            1,
            1,
        ),
        (
            "multiArray",
            b"\x1a\x04\xc5" + struct.pack(">H", multi_array) + b"\x02\xb0",
            2,
            1,
        ),
        (
            "divInt",
            b"\x1a\xb2" + struct.pack(">H", divisor) + b"\x6c\xac",
            2,
            1,
        ),
        (
            "remInt",
            b"\x1a\xb2" + struct.pack(">H", divisor) + b"\x70\xac",
            2,
            1,
        ),
        (
            "divLong",
            b"\x1e\xb2" + struct.pack(">H", long_divisor) + b"\x6d\xad",
            4,
            2,
        ),
        (
            "remLong",
            b"\x1e\xb2" + struct.pack(">H", long_divisor) + b"\x71\xad",
            4,
            2,
        ),
        ("cast", b"\x2a\xc0" + struct.pack(">H", string) + b"\xb0", 1, 1),
        (
            "direct",
            b"\xbb"
            + struct.pack(">H", order_value)
            + b"\x59\xb7"
            + struct.pack(">H", constructor)
            + b"\xb0",
            2,
            0,
        ),
    ]
    data = original_class.read_bytes()
    manifest: list[dict[str, str]] = []
    for patch in patches:
        data, record = patch_once(data, *patch, invoke_mark)
        manifest.append(record)

    patched_class = work / "DeferredValueOrder.class"
    patched_class.write_bytes(data)
    shutil.copy2(patched_class, original_class)
    run(["javap", "-verbose", "-classpath", str(original), "DeferredValueOrder"], work / "patched-javap.txt")
    run(["java", "-Xverify:all", "-cp", str(original), "DeferredValueOrderRunner"], work / "original.txt")
    (work / "patch-manifest.json").write_text(
        json.dumps(
            {
                "class_sha256": hashlib.sha256(data).hexdigest(),
                "class_bytes": len(data),
                "patches": manifest,
            },
            indent=2,
        )
        + "\n"
    )
    print(json.dumps({"class_sha256": hashlib.sha256(data).hexdigest(), "class_bytes": len(data), "patches": len(manifest)}))


if __name__ == "__main__":
    main()
