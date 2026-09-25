#!/usr/bin/env python3
"""Compare a Java 8 assertion class with complete JADX and Jarde class source."""

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent
CLASS_SHA256 = "7933b18c609d21d549b1c2c5b9356ad4ebd40a811b48cbbe06af3af66437dfde"
EXPECTED = {"enabled": "1|0;bad:-1|2|1\n", "disabled": "0|0;0|0\n"}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, output, label):
    result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=60)
    (output / f"{label}.stdout.txt").write_text(result.stdout, encoding="utf-8")
    (output / f"{label}.stderr.txt").write_text(result.stderr, encoding="utf-8")
    return result


def execute(classes, main, output, label):
    results = {}
    for mode, flag in (("enabled", "-ea"), ("disabled", "-da")):
        result = run(
            ["java", "-Xverify:all", flag, "-cp", str(classes), main],
            output,
            f"{label}-{mode}",
        )
        results[mode] = {"exit": result.returncode, "stdout": result.stdout}
    return results


def execute_selective(classes, main, output, label):
    result = run(
        ["java", "-Xverify:all", f"-ea:{main}", "-da:java.lang.StringBuilder",
         "-cp", str(classes), main],
        output,
        f"{label}-selective",
    )
    return {"exit": result.returncode, "stdout": result.stdout}


def compile_source(source, output, label):
    classes = output / f"{label}-classes"
    classes.mkdir(parents=True, exist_ok=True)
    result = run(
        ["javac", "--release", "8", "-g", "-Xlint:-options", "-d", str(classes), str(source)],
        output,
        f"{label}-javac",
    )
    return classes, result.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    output = args.out.resolve()
    output.mkdir(parents=True, exist_ok=True)

    original_classes, original_javac = compile_source(HERE / "AssertProbe.java", output, "original")
    assert original_javac == 0
    original_class = original_classes / "AssertProbe.class"
    assert digest(original_class) == CLASS_SHA256
    assert digest(HERE / "AssertProbe.class") == CLASS_SHA256
    original_execution = execute(original_classes, "AssertProbe", output, "original")
    assert all(
        original_execution[mode] == {"exit": 0, "stdout": expected}
        for mode, expected in EXPECTED.items()
    )
    wrong_owner_classes = output / "wrong-owner-classes"
    wrong_owner_classes.mkdir(exist_ok=True)
    wrong_owner_class = wrong_owner_classes / "AssertProbe.class"
    patch = run(
        [sys.executable, str(HERE / "patch_wrong_owner.py"), "--input", str(original_class),
         "--output", str(wrong_owner_class)],
        output,
        "wrong-owner-patch",
    )
    assert patch.returncode == 0
    assert digest(wrong_owner_class) == digest(HERE / "AssertProbe-wrong-owner.class")
    original_selective = execute_selective(original_classes, "AssertProbe", output, "original")
    wrong_owner_selective = execute_selective(wrong_owner_classes, "AssertProbe", output, "wrong-owner")
    assert original_selective == {"exit": 0, "stdout": EXPECTED["enabled"]}
    assert wrong_owner_selective == {"exit": 0, "stdout": EXPECTED["disabled"]}

    jarde = run(
        [
            str(args.cli.resolve()), "class-source", "--input", str(original_class),
            "--class", "AssertProbe", "--policy", "single-class", "--release", "8",
            "--format", "text",
        ],
        output,
        "jarde",
    )
    assert jarde.returncode == 0
    jarde_dir = output / "jarde"
    jarde_dir.mkdir(exist_ok=True)
    jarde_source = jarde_dir / "AssertProbe.java"
    jarde_source.write_text(jarde.stdout, encoding="utf-8")

    jadx_dir = output / "jadx-decompiled"
    jadx = run(["jadx", "--no-res", "-d", str(jadx_dir), str(original_class)], output, "jadx")
    assert jadx.returncode == 0
    jadx_text = (jadx_dir / "sources" / "defpackage" / "AssertProbe.java").read_text(encoding="utf-8")
    assert jadx_text == (HERE / "JadxAssertProbe.java").read_text(encoding="utf-8")
    jadx_source = output / "jadx" / "AssertProbe.java"
    jadx_source.parent.mkdir(exist_ok=True)
    jadx_source.write_text(jadx_text, encoding="utf-8")

    summary = {
        "class_sha256": CLASS_SHA256,
        "original": original_execution,
        "wrong_owner_sha256": digest(wrong_owner_class),
        "selective_assertions": {
            "original": original_selective,
            "wrong_owner": wrong_owner_selective,
        },
    }
    for label, source, main_class in (
        ("jarde", jarde_source, "AssertProbe"),
        ("jadx", jadx_source, "defpackage.AssertProbe"),
    ):
        classes, javac_exit = compile_source(source, output, label)
        summary[label] = {
            "source_sha256": digest(source),
            "javac_exit": javac_exit,
            "assert_syntax": "assert guard(value) : detail(value);" in source.read_text(encoding="utf-8"),
        }
        if javac_exit == 0:
            summary[label]["execution"] = execute(classes, main_class, output, label)

    wrong_jarde = run(
        [
            str(args.cli.resolve()), "class-source", "--input", str(wrong_owner_class),
            "--class", "AssertProbe", "--policy", "single-class", "--release", "8",
            "--format", "text",
        ],
        output,
        "wrong-owner-jarde",
    )
    assert wrong_jarde.returncode == 0
    wrong_jarde_source = output / "wrong-owner-jarde" / "AssertProbe.java"
    wrong_jarde_source.parent.mkdir(exist_ok=True)
    wrong_jarde_source.write_text(wrong_jarde.stdout, encoding="utf-8")
    wrong_jadx_dir = output / "wrong-owner-jadx-decompiled"
    wrong_jadx = run(
        ["jadx", "--no-res", "-d", str(wrong_jadx_dir), str(wrong_owner_class)],
        output,
        "wrong-owner-jadx",
    )
    assert wrong_jadx.returncode == 0
    wrong_jadx_text = (wrong_jadx_dir / "sources" / "defpackage" / "AssertProbe.java").read_text(encoding="utf-8")
    assert wrong_jadx_text == (HERE / "JadxAssertProbe-wrong-owner.java").read_text(encoding="utf-8")
    wrong_jadx_source = output / "wrong-owner-jadx" / "AssertProbe.java"
    wrong_jadx_source.parent.mkdir(exist_ok=True)
    wrong_jadx_source.write_text(wrong_jadx_text, encoding="utf-8")
    for label, source, main_class in (
        ("wrong-owner-jarde", wrong_jarde_source, "AssertProbe"),
        ("wrong-owner-jadx", wrong_jadx_source, "defpackage.AssertProbe"),
    ):
        classes, javac_exit = compile_source(source, output, label)
        summary[label] = {
            "source_sha256": digest(source),
            "javac_exit": javac_exit,
            "assert_syntax": "assert guard(value) : detail(value);" in source.read_text(encoding="utf-8"),
        }
        if javac_exit == 0:
            summary[label]["selective"] = execute_selective(classes, main_class, output, label)
    (output / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
