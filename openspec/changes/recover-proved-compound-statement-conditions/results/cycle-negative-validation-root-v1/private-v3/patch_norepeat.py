#!/usr/bin/env python3
"""Add the one required same_frame and retarget one branch in the pinned fixture."""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

import patch_cycle as parser

EXPECTED_SHA256 = "2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa"
METHOD = "test"
DESCRIPTOR = "(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;"
BRANCH_BCI = 26
OLD_OPCODE = 0x99  # ifeq
OLD_DELTA = 14
NEW_TARGET = 51


def stack_map_frames(data: bytes) -> list[tuple[int, int, int, int]]:
    count = parser.u2(data, 0)
    pos = 2
    previous = -1
    frames: list[tuple[int, int, int, int]] = []

    def verification_type(p: int) -> int:
        tag = data[p]
        if tag <= 6:
            return p + 1
        if tag in (7, 8):
            return p + 3
        raise ValueError(f"invalid verification_type_info tag {tag}")

    for _ in range(count):
        start = pos
        frame = data[pos]
        pos += 1
        if frame <= 63:
            delta = frame
        elif frame <= 127:
            delta = frame - 64
            pos = verification_type(pos)
        elif frame == 247:
            delta = parser.u2(data, pos)
            pos += 2
            pos = verification_type(pos)
        elif 248 <= frame <= 251:
            delta = parser.u2(data, pos)
            pos += 2
        elif 252 <= frame <= 254:
            delta = parser.u2(data, pos)
            pos += 2
            for _ in range(frame - 251):
                pos = verification_type(pos)
        elif frame == 255:
            delta = parser.u2(data, pos)
            locals_count = parser.u2(data, pos + 2)
            pos += 4
            for _ in range(locals_count):
                pos = verification_type(pos)
            stack_count = parser.u2(data, pos)
            pos += 2
            for _ in range(stack_count):
                pos = verification_type(pos)
        else:
            raise ValueError(f"reserved StackMapTable frame_type {frame}")
        current = previous + delta + 1
        frames.append((current, start, pos, frame))
        previous = current
    if pos != len(data):
        raise ValueError("StackMapTable did not close exactly")
    return frames


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_norepeat.py frozen.class output.class")
    source = Path(sys.argv[1]).read_bytes()
    source_hash = hashlib.sha256(source).hexdigest()
    if source_hash != EXPECTED_SHA256:
        raise SystemExit(f"source SHA-256 mismatch: {source_hash}")
    code_start, code_length, code_body, nested = parser.code_attribute(source)
    code = source[code_start:code_start + code_length]
    starts = parser.instruction_starts(code)
    if BRANCH_BCI not in starts or code[BRANCH_BCI] != OLD_OPCODE:
        raise SystemExit("pinned BCI 26 is not the expected ifeq opcode")
    old_delta = struct.unpack_from(">h", code, BRANCH_BCI + 1)[0]
    if old_delta != OLD_DELTA or BRANCH_BCI + old_delta != 40:
        raise SystemExit(f"pinned old target mismatch: delta={old_delta}")
    if NEW_TARGET not in starts:
        raise SystemExit("target BCI 51 is not an instruction boundary")
    maps = [(offset, length) for name, offset, length in nested if name == "StackMapTable"]
    if len(maps) != 1:
        raise SystemExit("expected one method StackMapTable")
    map_start, map_length = maps[0]
    map_bytes = source[map_start:map_start + map_length]
    frames = stack_map_frames(map_bytes)
    if [frame[0] for frame in frames] != [40, 62, 74, 83]:
        raise SystemExit(f"frozen frame offsets differ: {[frame[0] for frame in frames]}")
    if any(frame[0] == NEW_TARGET for frame in frames):
        raise SystemExit("BCI 51 unexpectedly already has a frame")
    frame_40 = next(frame for frame in frames if frame[0] == 40)
    frame_62 = next(frame for frame in frames if frame[0] == 62)
    if frame_62[3] != 21:
        raise SystemExit(f"frame at 62 is not the expected same_frame delta: {frame_62[3]}")
    if NEW_TARGET - frame_40[0] - 1 != 10 or frame_62[0] - NEW_TARGET - 1 != 10:
        raise SystemExit("same_frame deltas around BCI 51 do not match expected offsets")

    patched = bytearray(source)
    branch_operand = code_start + BRANCH_BCI + 1
    patched[branch_operand:branch_operand + 2] = struct.pack(">h", NEW_TARGET - BRANCH_BCI)
    # Insert same_frame(delta=10) immediately after the existing frame at BCI 40.
    insertion = map_start + frame_40[2]
    patched[insertion:insertion] = b"\x0a"
    # The inserted byte shifts the existing frame at 62 right by one. Its offset delta becomes 10.
    patched[map_start + frame_62[1] + 1] = 10
    patched[map_start:map_start + 2] = struct.pack(">H", len(frames) + 1)
    # The StackMapTable and enclosing Code attribute both grew by one byte.
    patched[map_start - 4:map_start] = struct.pack(">I", map_length + 1)
    old_code_attr_length = parser.u4(source, code_body - 4)
    patched[code_body - 4:code_body] = struct.pack(">I", old_code_attr_length + 1)

    output = Path(sys.argv[2])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(patched)
    if len(patched) != len(source) + 1:
        output.unlink(missing_ok=True)
        raise SystemExit("class length did not grow by exactly one byte")
    new_map_start = map_start
    new_map_length = map_length + 1
    new_frames = stack_map_frames(patched[new_map_start:new_map_start + new_map_length])
    if [frame[0] for frame in new_frames] != [40, 51, 62, 74, 83]:
        output.unlink(missing_ok=True)
        raise SystemExit(f"patched StackMapTable offsets differ: {[frame[0] for frame in new_frames]}")
    changes = [
        "test Code BCI 26 ifeq displacement: +14 -> +25 (target 40 -> 51)",
        "StackMapTable frame count: 4 -> 5; same_frame(delta=10) added at 51",
        "existing frame delta at 62: 21 -> 10",
        "StackMapTable attribute_length and enclosing Code attribute_length each +1",
    ]
    manifest = {
        "source": str(Path(sys.argv[1]).resolve()),
        "source_sha256": source_hash,
        "source_bytes": len(source),
        "patched_sha256": hashlib.sha256(patched).hexdigest(),
        "patched_bytes": len(patched),
        "method": METHOD,
        "descriptor": DESCRIPTOR,
        "branch_bci": BRANCH_BCI,
        "opcode": "ifeq",
        "old_target_bci": BRANCH_BCI + OLD_DELTA,
        "new_target_bci": NEW_TARGET,
        "classfile_changes": changes,
        "stackmap_offsets_before": [40, 62, 74, 83],
        "stackmap_offsets_after": [40, 51, 62, 74, 83],
        "expected_graph": ["26 -> 29,51", "29 -> 40,62", "40 -> 51,74", "51 -> 62,74"],
        "purpose": "shared decision test requires read-once expression repetition; never invoke this method",
    }
    output.with_suffix(output.suffix + ".json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
