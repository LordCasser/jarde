#!/usr/bin/env python3
"""Make a verifier-valid child InnerClasses disagreement from the frozen family."""

from pathlib import Path
from zipfile import ZIP_STORED, ZipFile, ZipInfo


HERE = Path(__file__).resolve().parent
SOURCE = HERE.parent / "named-member-family-stage1" / "fixture.jar"
OUTPUT = HERE / "wrong-relation.jar"
ROW = bytes.fromhex("00010002000e00230000")


with ZipFile(SOURCE) as source, ZipFile(OUTPUT, "w") as output:
    for name in source.namelist():
        contents = source.read(name)
        if name == "NamedMemberFamilyStage1$Member.class":
            assert contents.count(ROW) == 1
            start = contents.index(ROW) + 4
            contents = contents[:start] + b"\0\0" + contents[start + 2 :]
        entry = ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        entry.compress_type = ZIP_STORED
        output.writestr(entry, contents)
