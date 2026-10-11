#!/usr/bin/env python3
"""Create a second-entry SCC in the pinned Java 8 compound-condition fixture."""
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
BRANCH_BCI = 71
OLD_OPCODE = 0xA7  # goto
OLD_DELTA = 12
NEW_TARGET = 40


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_irreducible.py frozen.class output.class")
    source = Path(sys.argv[1]).read_bytes()
    source_hash = hashlib.sha256(source).hexdigest()
    if source_hash != EXPECTED_SHA256:
        raise SystemExit(f"source SHA-256 mismatch: {source_hash}")
    code_start, code_length, _body, nested = parser.code_attribute(source)
    code = source[code_start:code_start + code_length]
    starts = parser.instruction_starts(code)
    if BRANCH_BCI not in starts or code[BRANCH_BCI] != OLD_OPCODE:
        raise SystemExit("pinned BCI 71 is not the expected goto opcode")
    old_delta = struct.unpack_from(">h", code, BRANCH_BCI + 1)[0]
    if old_delta != OLD_DELTA or BRANCH_BCI + old_delta != 83:
        raise SystemExit(f"pinned old target mismatch: delta={old_delta}")
    if NEW_TARGET not in starts:
        raise SystemExit("target BCI 40 is not an instruction boundary")
    stack_maps = [raw for name, offset, length in nested
                  if name == "StackMapTable"
                  for raw in [source[offset:offset + length]]]
    if len(stack_maps) != 1 or NEW_TARGET not in parser.stack_map_offsets(stack_maps[0]):
        raise SystemExit("target BCI 40 lacks its existing StackMapTable frame")
    new_delta = NEW_TARGET - BRANCH_BCI
    patched = bytearray(source)
    operand_offset = code_start + BRANCH_BCI + 1
    patched[operand_offset:operand_offset + 2] = struct.pack(">h", new_delta)
    output = Path(sys.argv[2])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(patched)
    changes = [index for index, (before, after) in enumerate(zip(source, patched))
               if before != after]
    if changes != [operand_offset, operand_offset + 1]:
        output.unlink(missing_ok=True)
        raise SystemExit(f"unexpected byte diff offsets: {changes}")
    manifest = {
        "source": str(Path(sys.argv[1]).resolve()),
        "source_sha256": source_hash,
        "source_bytes": len(source),
        "patched_sha256": hashlib.sha256(patched).hexdigest(),
        "patched_bytes": len(patched),
        "method": METHOD,
        "descriptor": DESCRIPTOR,
        "branch_bci": BRANCH_BCI,
        "opcode": "goto",
        "old_signed_delta": OLD_DELTA,
        "old_target_bci": BRANCH_BCI + OLD_DELTA,
        "new_signed_delta": new_delta,
        "new_target_bci": NEW_TARGET,
        "changed_class_offsets": changes,
        "target_has_existing_stackmap_frame": True,
        "constructed_edges": ["40 -> 51", "51 -> 62", "62 -> 40"],
        "second_entry_edges": ["26 -> 40", "29 -> 62"],
        "purpose": "reader-produced irreducible three-node SCC refusal; never invoke this method",
    }
    output.with_suffix(output.suffix + ".json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
