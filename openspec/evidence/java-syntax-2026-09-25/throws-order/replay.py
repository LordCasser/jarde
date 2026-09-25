#!/usr/bin/env python3
"""Compare Java 8 Exceptions order across javac, JADX, and Jarde class-source."""

from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
FIXTURE = Path(__file__).resolve().parent / "fixture" / "throwsorder"
ORDERS = {
    "A": "java.io.IOException,java.sql.SQLException,java.lang.ReflectiveOperationException",
    "B": "java.lang.ReflectiveOperationException,java.sql.SQLException,java.io.IOException",
    "C": "java.sql.SQLException,java.io.IOException,java.lang.ReflectiveOperationException",
}
EXPECTED_JADX = "SQLException,IOException,ReflectiveOperationException"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(arg) for arg in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{command} failed ({result.returncode}):\n{result.stdout}\n{result.stderr}")
    return result


def exception_order(classfile: Path) -> str:
    listing = run("javap", "-v", classfile).stdout
    match = re.search(r"^    Exceptions:\n      throws (.+)$", listing, re.MULTILINE)
    if not match:
        raise AssertionError(f"no Exceptions attribute in {classfile}")
    return match.group(1).replace(" ", "")


def source_order(source: Path) -> str:
    declaration = next(line.strip() for line in source.read_text().splitlines() if "run()" in line)
    match = re.search(r"throws (.+) \{", declaration)
    if not match:
        raise AssertionError(f"no throws clause in {source}: {declaration}")
    return match.group(1).replace(" ", "")


def simple_names(order: str) -> str:
    return ",".join(name.rsplit(".", 1)[-1] for name in order.split(","))


with TemporaryDirectory(prefix="jarde-throws-order-") as temporary:
    work = Path(temporary)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")

    for debug_label, debug_flag in (("g", "-g"), ("none", "-g:none")):
        classes = work / f"original-{debug_label}"
        classes.mkdir()
        run("javac", "--release", "8", "-Xlint:-options", debug_flag, "-d", classes,
            *(FIXTURE / f"{name}.java" for name in ORDERS), FIXTURE / "Reflect.java")

        for name, original in ORDERS.items():
            boundary = classes / "throwsorder" / f"{name}.class"
            raw = exception_order(boundary)
            reflected = run("java", "-Xverify:all", "-cp", classes,
                            "throwsorder.Reflect", name).stdout.strip().split("=", 1)[1]
            assert raw == original, (debug_label, name, raw, original)
            assert reflected == original, (debug_label, name, reflected, original)

            jadx_dir = work / f"jadx-{debug_label}-{name}"
            run("jadx", "-d", jadx_dir, boundary)
            jadx_source = jadx_dir / "sources" / "throwsorder" / f"{name}.java"
            jadx_order = source_order(jadx_source)
            assert simple_names(jadx_order) == EXPECTED_JADX, (
                debug_label, name, jadx_order, EXPECTED_JADX
            )
            jadx_classes = work / f"jadx-classes-{debug_label}-{name}"
            jadx_classes.mkdir()
            run("javac", "--release", "8", "-Xlint:-options", "-d", jadx_classes,
                jadx_source, FIXTURE / "Reflect.java")
            jadx_reflected = run("java", "-Xverify:all", "-cp", jadx_classes,
                                 "throwsorder.Reflect", name).stdout.strip().split("=", 1)[1]
            assert simple_names(jadx_reflected) == simple_names(jadx_order), (
                debug_label, name, jadx_order, jadx_reflected
            )

            jarde_result = run(
                "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
                "--input", boundary, "--class", f"throwsorder.{name}",
                "--policy", "single-class", env=env,
            )
            jarde_source = work / f"jarde-source-{debug_label}-{name}" / "throwsorder" / f"{name}.java"
            jarde_source.parent.mkdir(parents=True)
            jarde_source.write_text(jarde_result.stdout)
            jarde_order = source_order(jarde_source)
            jarde_classes = work / f"jarde-classes-{debug_label}-{name}"
            jarde_classes.mkdir()
            run("javac", "--release", "8", "-Xlint:-options", "-d", jarde_classes,
                jarde_source, FIXTURE / "Reflect.java")
            jarde_reflected = run("java", "-Xverify:all", "-cp", jarde_classes,
                                  "throwsorder.Reflect", name).stdout.strip().split("=", 1)[1]
            assert simple_names(jarde_order) == simple_names(original), (
                debug_label, name, "Jarde source", jarde_order, original
            )
            assert simple_names(jarde_reflected) == simple_names(original), (
                debug_label, name, "Jarde rebuild", jarde_reflected, original
            )

            print(f"{debug_label}/{name}: javap={raw}; original-reflection={reflected}; "
                  f"JADX={jadx_order}; "
                  f"JADX-reflection={jadx_reflected}; Jarde={jarde_order}; "
                  f"Jarde-reflection={jarde_reflected}")

    print(run("java", "-version").stderr.strip().splitlines()[0])
    print("jadx " + run("jadx", "--version").stdout.strip())
