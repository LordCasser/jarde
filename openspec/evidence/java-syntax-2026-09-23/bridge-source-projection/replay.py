#!/usr/bin/env python3
"""Rebuild two Java 8 bridge cases and compare complete original/JADX/Jarde classes."""

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent
EXPECTED = {
    "positive": {
        "class_sha256": "03499dad67f54ce855af6f102b61c19916f72718a6d8756a35b019e3cda2af2f",
        "support_sha256": "6452b573128f0943f407b4408704d8238fe7a9f1b50450c15d900290f9f0012a",
        "output": "value|value|value\n",
    },
    "negative": {
        "unpatched_sha256": "ff6cf4df8abd8dcde52c34d3c0d29b3caf4d35efe47b6e800288defa9755dfbb",
        "class_sha256": "1a51179c7d05a89d71b46e86dd010bc9a4f74aeae6834a56a7e76fc17bf1a420",
        "output": "value|value|1\n",
    },
    "orphan": {
        "unpatched_sha256": "41f1eba02605f559b9de2bcc982146c68377daa9b88eea7ad719975b3d75467f",
        "class_sha256": "68262cf87e3242aa0a8d4927caab3f30f9ce9911e2ae1d9290f06beb66a2de0f",
        "output": "value|value\n",
    },
}


def run(args, directory, label, out):
    result = subprocess.run(args, capture_output=True, text=True, timeout=60, check=False)
    (directory / f"{label}.stdout.txt").write_text(result.stdout, encoding="utf-8")
    (directory / f"{label}.stderr.txt").write_text(
        result.stderr.replace(str(out), "<OUT>"), encoding="utf-8"
    )
    return result


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_java(source_dir, output_dir, names, case, label, out):
    output_dir.mkdir(parents=True, exist_ok=True)
    return run(
        ["javac", "--release", "8", "-g", "-Xlint:-options", "-d", str(output_dir)]
        + [str(source_dir / name) for name in names],
        case,
        label,
        out,
    )


def build_original(label, case, out):
    source = HERE / label
    classes = case / "original-classes"
    subject = {"positive": "BridgeProbe", "negative": "FakeBridge", "orphan": "OrphanBridge"}[label]
    runner = {"positive": "BridgeRunner", "negative": "FakeRunner", "orphan": "OrphanRunner"}[label]
    assert compile_java(
        source, classes, [f"{subject}.java", f"{runner}.java"], case, "original-javac", out
    ).returncode == 0
    class_file = classes / f"{subject}.class"
    if label == "positive":
        assert digest(class_file) == EXPECTED[label]["class_sha256"]
        assert digest(classes / "BridgeApi.class") == EXPECTED[label]["support_sha256"]
        input_file = case / "input.jar"
        assert run(
            ["jar", "cf", str(input_file), "-C", str(classes), "BridgeProbe.class", "-C", str(classes), "BridgeApi.class"],
            case, "jar", out,
        ).returncode == 0
        policy = "plain-jar"
    else:
        assert digest(class_file) == EXPECTED[label]["unpatched_sha256"]
        patched = case / "patched-classes"
        patched.mkdir(exist_ok=True)
        input_file = patched / f"{subject}.class"
        assert run(
            [sys.executable, str(HERE / "negative" / "patch.py"), "--input", str(class_file), "--output", str(input_file)],
            case, "patch", out,
        ).returncode == 0
        assert digest(input_file) == EXPECTED[label]["class_sha256"]
        shutil.copyfile(classes / f"{runner}.class", patched / f"{runner}.class")
        classes = patched
        policy = "single-class"
    original = run(["java", "-Xverify:all", "-cp", str(classes), runner], case, "original-java", out)
    assert original.returncode == 0 and original.stdout == EXPECTED[label]["output"]
    javap = run(["javap", "-c", "-p", str(input_file if label == "negative" else class_file)], case, "javap", out)
    assert javap.returncode == 0
    return subject, runner, input_file, policy, javap.stdout.count("Code:")


def generated(label, tool, subject, runner, case, source, out):
    directory = case / tool
    directory.mkdir(exist_ok=True)
    if tool == "jarde":
        (directory / f"{subject}.java").write_text(source, encoding="utf-8")
        if label == "positive":
            shutil.copyfile(HERE / label / "BridgeApi.java", directory / "BridgeApi.java")
    else:
        jadx_file = case / "jadx-decompiled" / "sources" / "defpackage" / f"{subject}.java"
        shutil.copyfile(jadx_file, directory / f"{subject}.java")
        if label == "positive":
            shutil.copyfile(jadx_file.with_name("BridgeApi.java"), directory / "BridgeApi.java")
    runner_text = (HERE / label / f"{runner}.java").read_text(encoding="utf-8")
    if tool == "jadx":
        runner_text = "package defpackage;\n" + runner_text
    (directory / f"{runner}.java").write_text(runner_text, encoding="utf-8")
    classes = directory / "classes"
    names = (["BridgeApi.java"] if label == "positive" else []) + [f"{subject}.java", f"{runner}.java"]
    compiled = compile_java(directory, classes, names, case, f"{tool}-javac", out)
    result = {f"{tool}_javac_exit": compiled.returncode,
              f"{tool}_source_sha256": hashlib.sha256(source.encode()).hexdigest()}
    if compiled.returncode == 0:
        main = f"defpackage.{runner}" if tool == "jadx" else runner
        executed = run(["java", "-Xverify:all", "-cp", str(classes), main], case, f"{tool}-java", out)
        result[f"{tool}_java_exit"] = executed.returncode
        result[f"{tool}_output"] = executed.stdout
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--out", type=Path, default=HERE / "results")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    cli = args.cli.resolve()
    summary = {"cli_sha256": digest(cli), "cases": {}}
    for label in ("positive", "negative", "orphan"):
        case = out / label
        case.mkdir(exist_ok=True)
        subject, runner, input_file, policy, code_count = build_original(label, case, out)
        recovered = run(
            [str(cli), "class-source", "--input", str(input_file), "--class", subject,
             "--policy", policy, "--release", "8", "--format", "text"],
            case, "jarde", out,
        )
        assert recovered.returncode == 0
        jadx = run(["jadx", "--no-res", "-d", str(case / "jadx-decompiled"), str(input_file)], case, "jadx", out)
        assert jadx.returncode == 0
        jadx_file = case / "jadx-decompiled" / "sources" / "defpackage" / f"{subject}.java"
        jadx_source = jadx_file.read_text(encoding="utf-8")
        row = {
            "class_sha256": EXPECTED[label]["class_sha256"],
            "class_bytes": (case / "original-classes" / f"{subject}.class").stat().st_size if label == "positive" else input_file.stat().st_size,
            "code_count": code_count,
            "original_output": EXPECTED[label]["output"],
            "jarde_references": recovered.stdout.count("@bytecode"),
        }
        row.update(generated(label, "jarde", subject, runner, case, recovered.stdout, out))
        row.update(generated(label, "jadx", subject, runner, case, jadx_source, out))
        summary["cases"][label] = row
    (out / "summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
