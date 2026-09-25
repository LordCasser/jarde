#!/usr/bin/env python3
"""Freeze class-retention declaration annotation boundaries without a Jarde build."""

from __future__ import annotations

import hashlib
import json
import shutil
import struct
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
SOURCES = ROOT / "tests/fixtures/class-annotation-uses/src"
OUT = HERE / "generated"


def u2(data: bytes | bytearray, at: int) -> int:
    return struct.unpack_from(">H", data, at)[0]


def u4(data: bytes | bytearray, at: int) -> int:
    return struct.unpack_from(">I", data, at)[0]


def put_u2(data: bytearray, at: int, value: int) -> None:
    struct.pack_into(">H", data, at, value)


def put_u4(data: bytearray, at: int, value: int) -> None:
    struct.pack_into(">I", data, at, value)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def cp_info(data: bytes) -> tuple[dict[int, str], dict[int, tuple[int, int]]]:
    count = u2(data, 8)
    index = 1
    at = 10
    text: dict[int, str] = {}
    text_spans: dict[int, tuple[int, int]] = {}
    while index < count:
        tag = data[at]
        start = at
        at += 1
        if tag == 1:
            size = u2(data, at)
            at += 2
            raw = data[at : at + size]
            text[index] = raw.decode("utf-8", errors="replace")
            text_spans[index] = (at, size)
            at += size
        elif tag in (3, 4):
            at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            at += 4
        elif tag == 15:
            at += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag} at {start}")
        index += 1
    return text, text_spans


def class_attributes(data: bytes) -> list[dict[str, int | str]]:
    text, _ = cp_info(data)
    at = 10
    count = u2(data, 8)
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            at += 2 + size
        elif tag in (3, 4):
            at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            at += 4
        elif tag == 15:
            at += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1
    at += 6
    interfaces = u2(data, at)
    at += 2 + interfaces * 2
    for _ in range(2):
        members = u2(data, at)
        at += 2
        for _ in range(members):
            at += 6
            attributes = u2(data, at)
            at += 2
            for _ in range(attributes):
                length = u4(data, at + 2)
                at += 6 + length
    count = u2(data, at)
    at += 2
    result = []
    for _ in range(count):
        start = at
        name = text[u2(data, at)]
        length = u4(data, at + 2)
        content = at + 6
        result.append({
            "name": name,
            "shell_start": start,
            "content_start": content,
            "content_length": length,
            "shell_end": content + length,
        })
        at = content + length
    if at != len(data):
        raise ValueError(f"class attributes end at {at}, file ends at {len(data)}")
    return result


def annotation_entries(data: bytes, attr: dict[str, int | str]) -> tuple[list[dict[str, int | str]], int | None]:
    text, _ = cp_info(data)
    start = int(attr["content_start"])
    count = u2(data, start)
    at = start + 2
    entries = []
    first_value_tag = None
    for _ in range(count):
        type_index = u2(data, at)
        pair_count = u2(data, at + 2)
        at += 4
        pairs = []
        for _ in range(pair_count):
            name_index = u2(data, at)
            tag_at = at + 2
            tag = chr(data[tag_at])
            value_index = u2(data, tag_at + 1)
            pairs.append({"name": text[name_index], "tag": tag, "cp_index": value_index, "tag_offset": tag_at})
            first_value_tag = first_value_tag if first_value_tag is not None else tag_at
            at += 5
        entries.append({"type_descriptor": text[type_index], "pairs": pairs})
    if at != start + int(attr["content_length"]):
        raise ValueError("fixture uses unexpected annotation grammar")
    return entries, first_value_tag


def command(args: list[str], log: Path) -> str:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {log}")
    return result.stdout


def main() -> None:
    shutil.rmtree(OUT, ignore_errors=True)
    original = OUT / "original"
    original.mkdir(parents=True)
    generated = sorted(SOURCES.glob("*.java"))
    command(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(original),
             *(str(path) for path in generated)], OUT / "javac.log")
    shutil.copy2(SOURCES / "BoundaryRunner.java", OUT / "BoundaryRunner.java")
    command(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-cp", str(original),
             "-d", str(original), str(SOURCES / "BoundaryRunner.java")], OUT / "runner-javac.log")
    reflection = command(["java", "-Xverify:all", "-cp", str(original), "BoundaryRunner"], OUT / "reflection.log")

    originals = {}
    for name in ("HiddenTarget", "HiddenTag", "DuplicateTarget", "Tag", "Tags", "EmptyTarget"):
        path = original / f"{name}.class"
        originals[name] = path.read_bytes()
    hidden = originals["HiddenTarget"]
    attr = next(attr for attr in class_attributes(hidden) if attr["name"] == "RuntimeInvisibleAnnotations")
    annotations, value_tag = annotation_entries(hidden, attr)

    # The class file declares two byte-identical entries of a non-repeatable annotation type.
    duplicate = bytearray(hidden)
    body_start = int(attr["content_start"])
    body_end = body_start + int(attr["content_length"])
    one_entry = bytes(duplicate[body_start + 2 : body_end])
    put_u2(duplicate, body_start, 2)
    duplicate[body_end:body_end] = one_entry
    put_u4(duplicate, int(attr["shell_start"]) + 2, int(attr["content_length"]) + len(one_entry))

    # The value tag is outside element_value's grammar; the shell and span remain valid.
    damaged = bytearray(hidden)
    assert value_tag is not None
    damaged[value_tag] = ord("Q")

    # Keep the UTF8 entry length and all spans stable while making the annotation descriptor
    # impossible to write as a Java source name.
    unspellable = bytearray(hidden)
    _, spans = cp_info(hidden)
    descriptor_offset, descriptor_size = next(
        span for index, span in spans.items() if cp_info(hidden)[0][index] == "LHiddenTag;"
    )
    replacement = b"LHidden-xx;"
    assert len(replacement) == descriptor_size
    unspellable[descriptor_offset : descriptor_offset + descriptor_size] = replacement

    boundary_files = {
        "DuplicateEntries.class": bytes(duplicate),
        "DamagedValueTag.class": bytes(damaged),
        "UnspellableType.class": bytes(unspellable),
    }
    for filename, data in boundary_files.items():
        (OUT / filename).write_bytes(data)
    javap_hidden = command(["javap", "-v", "-c", "-p", str(original / "HiddenTarget.class")], OUT / "javap-hidden.log")
    javap_duplicate = command(["javap", "-v", "-c", "-p", str(original / "DuplicateTarget.class")], OUT / "javap-repeatable-source.log")

    summary = {
        "compiler": command(["javac", "-version"], OUT / "javac-version.log").strip(),
        "release": 8,
        "reflection_output": reflection.splitlines(),
        "original_class_sha256": {f"{name}.class": sha(data) for name, data in originals.items()},
        "original_class_bytes": {f"{name}.class": len(data) for name, data in originals.items()},
        "class_retention_attribute": {
            **attr,
            "annotation_count": u2(hidden, int(attr["content_start"])),
            "annotations": annotations,
        },
        "controlled_boundaries": {
            name: {
                "sha256": sha(data),
                "bytes": len(data),
                "annotation_attribute": next(
                    item for item in class_attributes(data) if item["name"] == "RuntimeInvisibleAnnotations"
                ),
                "provenance": {
                    "DuplicateEntries.class": "复制同一个合法 CLASS-retention annotation_info 两次并将 count 改为 2",
                    "DamagedValueTag.class": f"将原 annotation element_value tag 在 class byte offset {value_tag} 改为 Q",
                    "UnspellableType.class": f"将原 CONSTANT_Utf8 描述符字节在 class byte offset {descriptor_offset} 替换为 {replacement.decode()}",
                }[name],
            }
            for name, data in boundary_files.items()
        },
        "javap_confirms_runtime_invisible": "RuntimeInvisibleAnnotations:" in javap_hidden,
        "javap_confirms_javac_repeatable_container": "RuntimeInvisibleAnnotations:" in javap_duplicate,
        "reader_expected_outcomes": {
            "HiddenTarget.class": "one complete annotation fact; not runtime visible",
            "DuplicateEntries.class": "two complete facts; class-source presentation refuses both as duplicate type",
            "DamagedValueTag.class": "classfile_invalid_attribute_content; no partial facts",
            "UnspellableType.class": "one complete raw fact; class-source presentation refuses the name",
            "attribute_budget": "covered by class_annotation_facts integration test",
        },
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
