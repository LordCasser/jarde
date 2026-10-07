#!/usr/bin/env python3
"""Freeze the two legs of the double-brace capture fixture.

The same `DB.java` is compiled twice: once by the ambient javac with `--release 8` (the leg CI
runs, `v23/`) and once by a real javac 8 (`v8/`). Both legs are checked for the fact this fixture
exists for — the capture store is emitted **before** the superclass constructor call — and both
are run under `java -Xverify:all` with the expected `2/z`.

The javac 8 leg's binary comes from `JARDE_JAVAC8` (a directory or the `javac` binary itself) and
falls back to the Corretto 1.8 install this repository's evidence records; a machine without it
fails loudly rather than freezing a half-checked fixture.

    python3 tests/fixtures/proved-java-structure/double-brace-capture/freeze.py
"""

from pathlib import Path
import hashlib
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "DB.java"
CLASSES = ("DB.class", "DB$1.class", "DB$2.class")
EXPECTED = "2/z\n"
FALLBACK_JAVAC8 = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac"


def javac8() -> Path:
    given = os.environ.get("JARDE_JAVAC8", FALLBACK_JAVAC8)
    path = Path(given)
    if path.is_dir():
        path = path / "javac"
    if not path.exists():
        raise SystemExit(f"no javac 8 at {path}: set JARDE_JAVAC8 to the JDK 8 install")
    return path


def java_for(javac: Path) -> Path:
    """The `java` beside one `javac`: a javac 8 leg must run on its own runtime."""
    return javac.parent / "java"


def compile_leg(javac: Path, arguments: list[str], destination: Path) -> None:
    subprocess.run(
        [str(javac), *arguments, "-g", "-d", str(destination), str(SOURCE)],
        check=True,
        capture_output=True,
    )


def ctor_order(classes: Path) -> list[str]:
    """The `DB$2` constructor's instruction lines, as `javap -c` prints them."""
    listing = subprocess.run(
        ["javap", "-p", "-c", str(classes / "DB$2.class")],
        check=True,
        text=True,
        capture_output=True,
    ).stdout
    body = listing.split("DB$2(java.lang.String);", 1)[1]
    return [line.strip() for line in body.splitlines() if line.strip()]


def assert_capture_before_super(classes: Path, label: str) -> None:
    lines = ctor_order(classes)
    store = next(index for index, line in enumerate(lines) if "putfield" in line and "val$s" in line)
    call = next(index for index, line in enumerate(lines) if "invokespecial" in line)
    assert store < call, f"{label}: the capture store is not before the constructor call:\n" + "\n".join(lines)
    print(f"{label}: `putfield val$s` at line {store} precedes `invokespecial` at line {call}")


def run_leg(classes: Path, java: Path, label: str) -> None:
    output = subprocess.run(
        [str(java), "-Xverify:all", "-cp", str(classes), "DB"],
        check=True,
        text=True,
        capture_output=True,
    )
    assert output.stdout == EXPECTED, f"{label}: {output.stdout!r} != {EXPECTED!r}"
    print(f"{label}: java -Xverify:all prints {output.stdout!r}")


def freeze(leg: str, javac: Path, arguments: list[str]) -> None:
    with tempfile.TemporaryDirectory(prefix=f"jarde-double-brace-{leg}-") as scratch:
        classes = Path(scratch)
        compile_leg(javac, arguments, classes)
        assert_capture_before_super(classes, leg)
        run_leg(classes, java_for(javac), leg)
        destination = ROOT / leg
        destination.mkdir(exist_ok=True)
        for name in CLASSES:
            shutil.copyfile(classes / name, destination / name)
            digest = hashlib.sha256((destination / name).read_bytes()).hexdigest()
            print(f"{digest}  {leg}/{name}")


freeze("v23", Path(shutil.which("javac") or "javac"), ["--release", "8"])
freeze("v8", javac8(), [])
with open(ROOT / "SHA256SUMS", "w") as sums:
    for leg in ("v23", "v8"):
        for name in CLASSES:
            digest = hashlib.sha256((ROOT / leg / name).read_bytes()).hexdigest()
            sums.write(f"{digest}  {leg}/{name}\n")
print("frozen")
