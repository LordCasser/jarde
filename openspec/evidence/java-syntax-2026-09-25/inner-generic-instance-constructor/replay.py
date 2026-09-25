#!/usr/bin/env python3
"""Compare an inner-class instance constructor against Java 8, JADX, and Jarde."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
SOURCE = Path(__file__).resolve().parent / "fixture/OuterGeneric.java"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], cwd=ROOT, env=env,
                          text=True, capture_output=True)


def require(result: subprocess.CompletedProcess[str], label: str) -> str:
    assert result.returncode == 0, (label, result.stdout, result.stderr)
    return result.stdout


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-inner-generic-") as temporary:
    work = Path(temporary)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    require(run("cargo", "build", "-q", "-p", "jarde-cli", env=env), "build Jarde")
    binary = work / "cargo-target/debug/jarde-cli"
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        classes = work / f"classes-{variant}"
        classes.mkdir()
        require(run("javac", "--release", "8", "-Xlint:-options", debug,
                    "-d", classes, SOURCE), f"compile original {variant}")
        actual = require(run("java", "-Xverify:all", "-cp", classes,
                             "nested.OuterGeneric"), f"run original {variant}").strip()
        assert actual == "3", actual
        class_files = [classes / f"nested/OuterGeneric{suffix}.class"
                       for suffix in ("", "$A", "$A$B")]
        jadx = work / f"jadx-{variant}"
        require(run("jadx", "-d", jadx, *class_files), f"run JADX {variant}")
        jadx_source = jadx / "sources/nested/OuterGeneric.java"
        jadx_text = jadx_source.read_text()
        # Without a LocalVariableTable JADX also loses the local generic types,
        # but both outputs construct B without its required enclosing instance.
        assert "new A.B" in jadx_text, jadx_text
        jadx_classes = work / f"jadx-classes-{variant}"
        jadx_classes.mkdir()
        jadx_compile = run("javac", "--release", "8", "-Xlint:-options",
                           "-d", jadx_classes, jadx_source)
        assert jadx_compile.returncode != 0, jadx_compile.stdout
        assert "A.B" in jadx_compile.stderr, jadx_compile.stderr
        print(f"{variant} JADX complete class javac={jadx_compile.returncode}; "
              f"source sha256={digest(jadx_source)}")
        for suffix, owner in (("", "nested.OuterGeneric"),
                              ("$A", "nested.OuterGeneric$A"),
                              ("$A$B", "nested.OuterGeneric$A$B")):
            output = require(run(binary, "class-source", "--input",
                                 classes / f"nested/OuterGeneric{suffix}.class",
                                 "--class", owner, "--policy", "single-class"),
                             f"run Jarde {owner} {variant}")
            if suffix:
                assert "class_generic_source_unproved" in output, (owner, output)
            else:
                assert "no shape this run verified" in output, output
            if suffix == "$A$B":
                assert "jvm_signature_scope_unproved" in output, output
            print(f"{variant} Jarde {owner}: "
                  f"class sha256={digest(classes / f'nested/OuterGeneric{suffix}.class')}; "
                  f"explicit refusal={bool(suffix)}")
    print(f"fixture sha256={digest(SOURCE)}")
