#!/usr/bin/env python3
"""Rebuild and execute the two fixed enum-switch inputs against a selected Jarde CLI."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
from shutil import copy2
import subprocess
from tempfile import TemporaryDirectory


HERE = Path(__file__).resolve().parent
INPUT_HASHES = {
    "normal": {
        "EnumSwitchSubject.class": "05efc57231504c39db97ee3629d2f83b22add32cb4a0bc0eff1faf54fb2bf295",
        "EnumSwitchSubject$1.class": "5cbf19d12db909aa88ad1a504f6d8f8a096f6d9622cae972f0569b0b591aabd6",
        "Hue.class": "3a5d7db6f4420d09cbdd0f128dcaaa4759fd33de11196c6bebdb10917db67ab2",
        "EnumSwitchRunner.class": "6d7ed4dd46e787a3119703bd8d0d0ae9ba24cd341ea253c5bf4b58f17aa1baea",
    },
    "swapped": {
        "EnumSwitchSubject$1.class": "65e6ad3fcfc997a554dac632132f866ac1d09a6ea2821c2bf464180b912852fd",
    },
}
EXPECTED = {
    "normal": "1|1\n2|2\n3|3\nnull|0",
    "swapped": "2|2\n1|1\n3|3\nnull|0",
}


def digest(path):
    return sha256(path.read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run(
        [str(value) for value in command], capture_output=True, text=True, timeout=90
    )
    log.write_text(result.stdout + result.stderr)
    return result


def compile_java(sources, output, log, support=None):
    command = ["javac", "--release", "8", "-g:none"]
    if support:
        command.extend(["-cp", support])
    command.extend(["-d", output, *sources])
    return run(command, log)


def replay_case(label, cli, output):
    case = output / label
    case.mkdir()
    class_root = HERE / ("original" if label == "normal" else "patched-map")
    for name, expected_hash in {**INPUT_HASHES["normal"], **INPUT_HASHES[label]}.items():
        actual_hash = digest(class_root / name)
        if actual_hash != expected_hash:
            raise ValueError(f"{label}/{name}: {actual_hash} != {expected_hash}")
    actual = run(
        ["java", "-Xverify:all", "-cp", class_root, "EnumSwitchRunner"],
        case / "original-runtime.log",
    )
    if actual.returncode or actual.stdout.strip() != EXPECTED[label]:
        raise ValueError(f"{label}: original class no longer gives the frozen result")

    jadx_source = HERE / ("jadx-decompiled" if label == "normal" else "jadx-swapped")
    jadx_files = sorted((jadx_source / "sources/defpackage").glob("*.java"))
    if len(jadx_files) != 3:
        raise ValueError(f"{label}: JADX source set is incomplete")
    jadx_classes = case / "jadx-classes"
    compiled_jadx = compile_java(jadx_files, jadx_classes, case / "jadx-javac.log")
    jadx_output = None
    if compiled_jadx.returncode == 0:
        execution = run(
            ["java", "-Xverify:all", "-cp", jadx_classes, "defpackage.EnumSwitchRunner"],
            case / "jadx-runtime.log",
        )
        jadx_output = execution.stdout.strip() if execution.returncode == 0 else None

    source = case / "EnumSwitchSubject.java"
    jar = HERE / ("enum-switch.jar" if label == "normal" else "enum-switch-swapped.jar")
    jarde = run(
        [cli, "class-source", "--input", jar, "--class", "EnumSwitchSubject", "--policy",
         "plain-jar", "--release", "8", "--evidence", "all", "--format", "text"],
        case / "jarde-cli.log",
    )
    source.write_text(jarde.stdout)
    helper = case / "helper"
    helper.mkdir()
    copy2(class_root / "EnumSwitchSubject$1.class", helper / "EnumSwitchSubject$1.class")
    jarde_classes = case / "jarde-classes"
    compiled_jarde = compile_java(
        [source, HERE / "Hue.java", HERE / "EnumSwitchRunner.java"],
        jarde_classes,
        case / "jarde-javac.log",
        helper,
    ) if jarde.stdout.strip() else None
    jarde_output = None
    if compiled_jarde is not None and compiled_jarde.returncode == 0:
        execution = run(
            ["java", "-Xverify:all", "-cp", f"{jarde_classes}:{helper}", "EnumSwitchRunner"],
            case / "jarde-runtime.log",
        )
        jarde_output = execution.stdout.strip() if execution.returncode == 0 else None
    return {
        "input_sha256": {name: digest(class_root / name) for name in INPUT_HASHES["normal"]},
        "original_runtime": actual.stdout.strip(),
        "jadx_javac_exit": compiled_jadx.returncode,
        "jadx_runtime": jadx_output,
        "jarde_cli_exit": jarde.returncode,
        "jarde_source_sha256": digest(source),
        "jarde_javac_exit": compiled_jarde.returncode if compiled_jarde else None,
        "jarde_runtime": jarde_output,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    arguments = parser.parse_args()
    cli = arguments.cli.resolve()
    output = arguments.out.resolve()
    if not cli.is_file() or (output.exists() and any(output.iterdir())):
        raise ValueError("CLI must exist and output must be new or empty")
    output.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory(prefix="jarde-enum-switch-source-") as scratch:
        original = Path(scratch)
        sources = [HERE / f"{name}.java" for name in ("Hue", "EnumSwitchSubject", "EnumSwitchRunner")]
        if compile_java(sources, original, output / "source-javac.log").returncode:
            raise ValueError("the original source no longer compiles")
        for name, expected_hash in INPUT_HASHES["normal"].items():
            if digest(original / name) != expected_hash:
                raise ValueError(f"recompiled {name} no longer matches the frozen original")
    summary = {
        "cli_sha256": digest(cli),
        "cases": {label: replay_case(label, cli, output) for label in ("normal", "swapped")},
    }
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
