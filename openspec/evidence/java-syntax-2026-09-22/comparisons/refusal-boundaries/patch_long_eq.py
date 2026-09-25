#!/usr/bin/env python3
"""Patch only the frozen long_eq Code attribute for verifier-valid refusal probes.

The complete method Code byte sequence is matched once. No constant-pool or class-wide parser is
used: the two u32 lengths immediately before that code and the one StackMapTable frame owned by the
method are the only class-file fields changed. The original fixture is never written.
"""

from __future__ import annotations

import argparse
from pathlib import Path


LONG_EQ = bytes.fromhex("1e 20 94 9a 00 06 10 07 ac 10 09 ac")


def patch(data: bytes, mode: str) -> bytes:
    sites = [i for i in range(len(data) - len(LONG_EQ) + 1) if data.startswith(LONG_EQ, i)]
    assert len(sites) == 1, f"long_eq Code must occur once, got {sites}"
    start = sites[0]
    code_length_at = start - 4
    attribute_length_at = start - 12
    max_locals_at = start - 6
    code_length = int.from_bytes(data[code_length_at:start], "big")
    attribute_length = int.from_bytes(data[attribute_length_at:start - 8], "big")
    max_locals = int.from_bytes(data[max_locals_at:start - 4], "big")
    assert code_length == len(LONG_EQ), (code_length, len(LONG_EQ))
    assert max_locals == 4, max_locals

    if mode == "dup-pop":
        insertion = bytes.fromhex("59 57")
        frame = bytes.fromhex("0b")
        stack_map_delta = 0
        locals_delta = 0
    elif mode == "local":
        insertion = bytes.fromhex("36 04 15 04")
        # The branch target gains four bytes and now sees local slot 4, so the old same_frame
        # becomes append_frame(offset_delta=13, Integer).
        frame = bytes.fromhex("fc 00 0d 01")
        stack_map_delta = len(frame) - 1
        locals_delta = 1
    else:
        raise ValueError(mode)

    # lcmp is the first three bytes of the matched method; put the extra uses
    # between it and the original zero branch so the branch is no longer its
    # immediate consumer.
    insertion_at = start + 3
    patched = bytearray(data[:insertion_at] + insertion + data[insertion_at:])
    patched[code_length_at:start] = (code_length + len(insertion)).to_bytes(4, "big")
    patched[max_locals_at:start - 4] = (max_locals + locals_delta).to_bytes(2, "big")
    patched[attribute_length_at:start - 8] = (
        attribute_length + len(insertion) + stack_map_delta
    ).to_bytes(4, "big")

    # Code is followed by exception_table_length (2), attributes_count (2), the
    # StackMapTable name index (2), its length (4), and number_of_entries (2).
    new_frame_at = start + len(LONG_EQ) + len(insertion) + 12
    if len(frame) == 1:
        assert patched[new_frame_at] == 9, hex(patched[new_frame_at])
        patched[new_frame_at:new_frame_at + 1] = frame
    else:
        assert patched[new_frame_at] == 9, hex(patched[new_frame_at])
        stack_map_length_at = new_frame_at - 6
        old_stack_map_length = int.from_bytes(
            patched[stack_map_length_at:stack_map_length_at + 4], "big"
        )
        assert old_stack_map_length == 3, old_stack_map_length
        patched[stack_map_length_at:stack_map_length_at + 4] = (6).to_bytes(4, "big")
        patched[new_frame_at:new_frame_at + 1] = frame
    return bytes(patched)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("mode", choices=("dup-pop", "local"))
    args = parser.parse_args()
    args.output.write_bytes(patch(args.input.read_bytes(), args.mode))


if __name__ == "__main__":
    main()
