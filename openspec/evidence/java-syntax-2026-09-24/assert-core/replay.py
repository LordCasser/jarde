#!/usr/bin/env python3
"""Rebuild and compare original/JADX/Jarde Java 8 assertion classes."""

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent
CORE_SHA256 = "71753fbec0b66a6511bbb2b66855c50589060b729bc963cb6cd78377b75f4aaf"
RUNNER_SHA256 = "b821b8f12c8b4bdddeb36271dc283f37a6f0cc016499cc6ed1b67050afc8bb66"
WRONG_SHA256 = "af3c1190da44c938138da327a6232220acd3b8e8161808e2d9d517f11a4e71dc"
ENABLED = "1|0;bad|2|1\n"
DISABLED = "0|0;0|0\n"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, out, label):
    result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=60)
    (out / f"{label}.stdout.txt").write_text(result.stdout, encoding="utf-8")
    (out / f"{label}.stderr.txt").write_text(result.stderr, encoding="utf-8")
    return result


def compile_java(sources, classes, out, label):
    classes.mkdir(parents=True, exist_ok=True)
    return run(
        ["javac", "--release", "8", "-g", "-Xlint:-options", "-d", str(classes)]
        + [str(source) for source in sources],
        out,
        f"{label}-javac",
    ).returncode


def execute(classes, main, out, label, mode):
    assertion_flags = {
        "enabled": ["-ea"],
        "disabled": ["-da"],
        "selective": ["-ea:" + main.replace("AssertRunner", "AssertCore"), "-da:java.lang.StringBuilder"],
    }
    result = run(
        ["java", "-Xverify:all", *assertion_flags[mode], "-cp", str(classes), main],
        out,
        f"{label}-{mode}",
    )
    return {"exit": result.returncode, "stdout": result.stdout}


def generated(label, case, class_file, cli, out):
    if label == "jarde":
        result = run(
            [str(cli), "class-source", "--input", str(class_file), "--class", "AssertCore",
             "--policy", "single-class", "--release", "8", "--format", "text"],
            out,
            f"{case}-jarde",
        )
        assert result.returncode == 0
        subject_text = result.stdout
        main = "AssertRunner"
    else:
        decompiled = out / f"{case}-jadx-decompiled"
        result = run(["jadx", "--no-res", "-d", str(decompiled), str(class_file)], out, f"{case}-jadx")
        assert result.returncode == 0
        subject_text = (decompiled / "sources" / "defpackage" / "AssertCore.java").read_text(encoding="utf-8")
        frozen = HERE / ("JadxAssertCore.java" if case == "original" else "JadxAssertCore-wrong-owner.java")
        assert subject_text == frozen.read_text(encoding="utf-8")
        main = "defpackage.AssertRunner"

    source_dir = out / f"{case}-{label}-source"
    source_dir.mkdir(exist_ok=True)
    subject = source_dir / "AssertCore.java"
    subject.write_text(subject_text, encoding="utf-8")
    runner = source_dir / "AssertRunner.java"
    runner_text = (HERE / "AssertRunner.java").read_text(encoding="utf-8")
    runner.write_text(("package defpackage;\n" if label == "jadx" else "") + runner_text, encoding="utf-8")
    classes = out / f"{case}-{label}-classes"
    javac_exit = compile_java([subject, runner], classes, out, f"{case}-{label}")
    answer = {
        "source_sha256": digest(subject),
        "javac_exit": javac_exit,
        "assert_syntax": "assert guard(ok) : detail();" in subject_text,
    }
    if javac_exit == 0:
        modes = ("enabled", "disabled") if case == "original" else ("selective",)
        answer["execution"] = {
            mode: execute(classes, main, out, f"{case}-{label}", mode)
            for mode in modes
        }
    return answer


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    cli = args.cli.resolve()

    original_classes = out / "original-classes"
    assert compile_java([HERE / "AssertCore.java", HERE / "AssertRunner.java"], original_classes, out, "original") == 0
    core_class = original_classes / "AssertCore.class"
    runner_class = original_classes / "AssertRunner.class"
    assert digest(core_class) == digest(HERE / "AssertCore.class") == CORE_SHA256
    assert digest(runner_class) == digest(HERE / "AssertRunner.class") == RUNNER_SHA256
    original = {
        mode: execute(original_classes, "AssertRunner", out, "original", mode)
        for mode in ("enabled", "disabled", "selective")
    }
    assert original == {
        "enabled": {"exit": 0, "stdout": ENABLED},
        "disabled": {"exit": 0, "stdout": DISABLED},
        "selective": {"exit": 0, "stdout": ENABLED},
    }

    wrong_classes = out / "wrong-owner-classes"
    wrong_classes.mkdir(exist_ok=True)
    wrong_class = wrong_classes / "AssertCore.class"
    patch = run(
        [sys.executable, str(HERE / "patch_wrong_owner.py"), "--input", str(core_class),
         "--output", str(wrong_class)],
        out,
        "wrong-owner-patch",
    )
    assert patch.returncode == 0
    assert digest(wrong_class) == digest(HERE / "AssertCore-wrong-owner.class") == WRONG_SHA256
    shutil.copyfile(runner_class, wrong_classes / "AssertRunner.class")
    wrong = execute(wrong_classes, "AssertRunner", out, "wrong-owner", "selective")
    assert wrong == {"exit": 0, "stdout": DISABLED}

    summary = {
        "class_sha256": CORE_SHA256,
        "wrong_owner_sha256": WRONG_SHA256,
        "original": original,
        "wrong_owner": wrong,
    }
    for case, class_file in (("original", core_class), ("wrong-owner", wrong_class)):
        for label in ("jadx", "jarde"):
            summary[f"{case}_{label}"] = generated(label, case, class_file, cli, out)
    (out / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
