#!/usr/bin/env python3
"""Make a verifier-valid missing-null-check control from the frozen stage-one jar."""

from pathlib import Path
from zipfile import ZIP_STORED, ZipFile, ZipInfo


HERE = Path(__file__).resolve().parent
SOURCE = HERE.parent / "named-member-family-stage1" / "fixture.jar"
OUTPUT = HERE / "null-check-removed.jar"
PATTERN = bytes.fromhex("bb0019592b59b8001b57b70021")


with ZipFile(SOURCE) as source, ZipFile(OUTPUT, "w") as output:
    for name in source.namelist():
        contents = source.read(name)
        if name == "NamedMemberFamilyStage1.class":
            assert contents.count(PATTERN) == 1
            start = contents.index(PATTERN) + 6
            contents = contents[:start] + b"\0\0\0" + contents[start + 3 :]
        entry = ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        entry.compress_type = ZIP_STORED
        output.writestr(entry, contents)
