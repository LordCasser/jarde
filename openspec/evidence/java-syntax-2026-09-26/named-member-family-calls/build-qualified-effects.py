#!/usr/bin/env python3
"""Compile the Java 8 effect-order family into a deterministic test jar."""

from pathlib import Path
from subprocess import run
from tempfile import TemporaryDirectory
from zipfile import ZIP_STORED, ZipFile, ZipInfo


HERE = Path(__file__).resolve().parent
with TemporaryDirectory(prefix="jarde-family-calls-") as temporary:
    build = Path(temporary)
    run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-d",
            str(build),
            str(HERE / "NamedMemberFamilyCalls.java"),
        ],
        check=True,
    )
    with ZipFile(HERE / "qualified-effects.jar", "w") as output:
        for name in (
            "NamedMemberFamilyCalls.class",
            "NamedMemberFamilyCalls$Member.class",
        ):
            entry = ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_STORED
            output.writestr(entry, (build / name).read_bytes())
