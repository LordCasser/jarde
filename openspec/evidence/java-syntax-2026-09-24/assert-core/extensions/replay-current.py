#!/usr/bin/env python3
"""Replay assertion extension fixtures against a supplied current Jarde CLI."""

import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, out, stem, expect=0):
    result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=120)
    (out / f"{stem}.stdout.txt").write_text(result.stdout, encoding="utf-8")
    (out / f"{stem}.stderr.txt").write_text(result.stderr, encoding="utf-8")
    if expect is not None and result.returncode != expect:
        raise RuntimeError(f"{stem}: expected exit {expect}, got {result.returncode}")
    return result


def compile_java(out, stem, sources, classpath=None):
    classes = out / f"{stem}-classes"
    classes.mkdir(parents=True, exist_ok=True)
    args = ["javac", "--release", "8", "-g", "-Xlint:-options", "-d", str(classes)]
    if classpath:
        args += ["-classpath", str(classpath)]
    run(args + [str(path) for path in sources], out, f"{stem}-javac")
    return classes


def execute(out, stem, classes, main):
    modes = {}
    for mode, flag in (("enabled", "-ea"), ("disabled", "-da")):
        result = run(["java", "-Xverify:all", flag, "-cp", str(classes), main],
                     out, f"{stem}-{mode}")
        modes[mode] = result.stdout
    return modes


def add_synthetic_flag(path, field_name):
    """Set ACC_SYNTHETIC on a Java-compiled status field; code and attributes stay intact."""
    data = bytearray(path.read_bytes())
    cp_count = int.from_bytes(data[8:10], "big")
    pool, offset, index = {}, 10, 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = int.from_bytes(data[offset:offset + 2], "big")
            offset += 2
            pool[index] = data[offset:offset + size].decode("utf-8")
            offset += size
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1
    offset += 6
    count = int.from_bytes(data[offset:offset + 2], "big")
    offset += 2 + count * 2
    count = int.from_bytes(data[offset:offset + 2], "big")
    offset += 2
    found = []
    for _ in range(count):
        flags_at = offset
        flags = int.from_bytes(data[offset:offset + 2], "big")
        name_index = int.from_bytes(data[offset + 2:offset + 4], "big")
        desc_index = int.from_bytes(data[offset + 4:offset + 6], "big")
        offset += 6
        attr_count = int.from_bytes(data[offset:offset + 2], "big")
        offset += 2
        for _ in range(attr_count):
            size = int.from_bytes(data[offset + 2:offset + 6], "big")
            offset += 6 + size
        if pool.get(name_index) == field_name:
            found.append((flags_at, flags, pool.get(desc_index)))
    if len(found) != 1 or found[0][2] != "Z":
        raise ValueError(f"expected one {field_name}:Z field, found {found}")
    flags_at, flags, _ = found[0]
    if flags & 0x0008 == 0 or flags & 0x0007:
        raise ValueError(f"unexpected source field flags 0x{flags:04x}")
    flags_after = flags | 0x1010
    data[flags_at:flags_at + 2] = flags_after.to_bytes(2, "big")
    path.write_bytes(data)
    return {"descriptor": "Z", "flags_before": f"0x{flags:04x}",
            "flags_after": f"0x{flags_after:04x}", "changed_bytes": 2}


def compile_observed(out, stem, sources, classpath=None):
    classes = out / f"{stem}-classes"
    classes.mkdir(parents=True, exist_ok=True)
    args = ["javac", "--release", "8", "-g", "-Xlint:-options", "-d", str(classes)]
    if classpath:
        args += ["-classpath", str(classpath)]
    result = run(args + [str(path) for path in sources], out, f"{stem}-javac", expect=None)
    (out / f"{stem}-javac.exit").write_text(f"{result.returncode}\n", encoding="utf-8")
    return classes, result.returncode


def jarde_source(out, cli, label, class_file, class_name):
    result = run([str(cli), "class-source", "--input", str(class_file), "--class", class_name,
                  "--policy", "single-class", "--release", "8", "--format", "text"],
                 out, f"{label}-jarde")
    source_dir = out / f"{label}-jarde-source"
    source_dir.mkdir(exist_ok=True)
    source = source_dir / f"{class_name}.java"
    source.write_text(result.stdout, encoding="utf-8")
    return source


def jadx_source(out, class_file, label, class_name):
    decompiled = out / f"{label}-jadx"
    run(["jadx", "--no-res", "-d", str(decompiled), str(class_file)], out,
        f"{label}-jadx-run")
    path = decompiled / "sources" / "defpackage" / f"{class_name}.java"
    if not path.is_file():
        raise RuntimeError(f"JADX output missing {path}")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    out, cli = args.out.resolve(), args.cli.resolve()
    if out.exists() and any(out.iterdir()):
        raise RuntimeError(f"output directory must be empty: {out}")
    out.mkdir(parents=True, exist_ok=True)
    legacy = out / "existing-extension-replay"
    run(["python3", str(HERE / "replay.py"), "--out", str(legacy)], out, "existing-replay")
    core_replay = out / "assert-core-current-replay"
    run(["python3", str(HERE.parent / "replay.py"), "--cli", str(cli), "--out",
         str(core_replay)], out, "assert-core-replay")
    (core_replay / "README.txt").write_text(
        "Core original/JADX/Jarde and wrong-owner control were replayed with the supplied CLI. "
        "See summary.json, source files, javap output, javac transcripts, and -Xverify:all outputs.\n",
        encoding="utf-8")
    summary = {"git_revision": run(["git", "rev-parse", "HEAD"], out, "git-revision").stdout.strip(),
               "tools": {"cli_sha256": sha(cli),
                         "javac": run(["javac", "-version"], out, "javac-version").stdout.strip(),
                         "java": run(["java", "-version"], out, "java-version").stderr.splitlines()[0],
                         "jadx": run(["jadx", "--version"], out, "jadx-version").stdout.strip()},
               "assert_core_replay": "see assert-core-current-replay/summary.json",
               "existing_extension_replay": "see existing-extension-replay/summary.json"}

    non_boolean_dir = out / "non-boolean-original"
    non_boolean_dir.mkdir()
    non_boolean_class = non_boolean_dir / "AssertCore.class"
    shutil.copy2(ROOT / "tests/fixtures/p3-assert-core/AssertCore-non01-arms.class",
                 non_boolean_class)
    run(["javap", "-p", "-c", "-v", str(non_boolean_class)], out, "non-boolean-javap")
    original_runner_classes = compile_java(out, "non-boolean-original",
                                           [ROOT / "tests/fixtures/p3-assert-core/AssertRunner.java"],
                                           classpath=non_boolean_dir)
    shutil.copy2(non_boolean_class, original_runner_classes / "AssertCore.class")
    original_modes = execute(out, "non-boolean-original", original_runner_classes, "AssertRunner")
    non_boolean_jadx = jadx_source(out, non_boolean_class, "non-boolean", "AssertCore")
    non_boolean_jarde = jarde_source(out, cli, "non-boolean", non_boolean_class, "AssertCore")
    jadx_overlay = out / "non-boolean-jadx-overlay"
    jadx_overlay.mkdir()
    jadx_runner = jadx_overlay / "AssertRunner.java"
    jadx_runner.write_text("package defpackage;\n" +
                           (ROOT / "tests/fixtures/p3-assert-core/AssertRunner.java")
                           .read_text(encoding="utf-8"), encoding="utf-8")
    jadx_classes, jadx_compile_exit = compile_observed(out, "non-boolean-jadx",
                                                       [non_boolean_jadx, jadx_runner])
    jarde_runner = out / "non-boolean-JardeRunner" / "AssertRunner.java"
    jarde_runner.parent.mkdir()
    jarde_runner.write_text((ROOT / "tests/fixtures/p3-assert-core/AssertRunner.java")
                            .read_text(encoding="utf-8"), encoding="utf-8")
    jarde_classes, jarde_compile_exit = compile_observed(out, "non-boolean-jarde",
                                                         [non_boolean_jarde, jarde_runner])
    summary["non_boolean"] = {
        "class_sha256": sha(non_boolean_class),
        "source_sha256": sha(ROOT / "tests/fixtures/p3-assert-core/AssertCore.java"),
        "patch_replay_script_sha256": sha(ROOT / "tests/fixtures/p3-assert-core/patch_non01_arms.py"),
        "original_modes": original_modes,
        "javap": "see non-boolean-javap.stdout.txt",
        "jadx_source_sha256": sha(non_boolean_jadx), "jadx_javac_exit": jadx_compile_exit,
        "jarde_source_sha256": sha(non_boolean_jarde), "jarde_javac_exit": jarde_compile_exit,
    }
    if jadx_compile_exit == 0:
        summary["non_boolean"]["jadx_modes"] = execute(
            out, "non-boolean-jadx", jadx_classes, "defpackage.AssertRunner")
    if jarde_compile_exit == 0:
        summary["non_boolean"]["jarde_modes"] = execute(
            out, "non-boolean-jarde", jarde_classes, "AssertRunner")

    positive = legacy / "positive-classes" / "AssertVariants.class"

    duplicate_classes = compile_java(out, "duplicate-status-original", [
        HERE / "AssertDuplicateStatusWrite.java",
        HERE / "AssertDuplicateStatusWriteRunner.java"])
    duplicate_class = duplicate_classes / "AssertDuplicateStatusWrite.class"
    duplicate_flag_patch = add_synthetic_flag(duplicate_class, "$assertionsDisabled")
    run(["javap", "-p", "-c", "-v", str(duplicate_class)], out,
        "duplicate-status-javap")
    duplicate_modes = execute(out, "duplicate-status-original", duplicate_classes,
                              "AssertDuplicateStatusWriteRunner")
    duplicate_jadx = jadx_source(out, duplicate_class, "duplicate-status",
                                 "AssertDuplicateStatusWrite")
    duplicate_jarde = jarde_source(out, cli, "duplicate-status", duplicate_class,
                                   "AssertDuplicateStatusWrite")
    duplicate_jadx_overlay = out / "duplicate-status-jadx-overlay"
    duplicate_jadx_overlay.mkdir()
    duplicate_jadx_runner = duplicate_jadx_overlay / "AssertDuplicateStatusWriteRunner.java"
    duplicate_jadx_runner.write_text(
        "package defpackage;\n" +
        (HERE / "AssertDuplicateStatusWriteRunner.java").read_text(encoding="utf-8"),
        encoding="utf-8")
    duplicate_jadx_classes, duplicate_jadx_exit = compile_observed(
        out, "duplicate-status-jadx", [duplicate_jadx, duplicate_jadx_runner])
    duplicate_jarde_overlay = out / "duplicate-status-jarde-overlay"
    duplicate_jarde_overlay.mkdir()
    duplicate_jarde_runner = duplicate_jarde_overlay / "AssertDuplicateStatusWriteRunner.java"
    duplicate_jarde_runner.write_text(
        (HERE / "AssertDuplicateStatusWriteRunner.java").read_text(encoding="utf-8"),
        encoding="utf-8")
    duplicate_jarde_classes, duplicate_jarde_exit = compile_observed(
        out, "duplicate-status-jarde", [duplicate_jarde, duplicate_jarde_runner])
    summary["duplicate_status_write"] = {
        "class_sha256": sha(duplicate_class), "flag_patch": duplicate_flag_patch,
        "sources": {p.name: sha(p) for p in (
            HERE / "AssertDuplicateStatusWrite.java",
            HERE / "AssertDuplicateStatusWriteRunner.java")},
        "javap": "see duplicate-status-javap.stdout.txt", "original_modes": duplicate_modes,
        "jadx_source_sha256": sha(duplicate_jadx), "jadx_javac_exit": duplicate_jadx_exit,
        "jarde_source_sha256": sha(duplicate_jarde), "jarde_javac_exit": duplicate_jarde_exit,
    }
    if duplicate_jadx_exit == 0:
        summary["duplicate_status_write"]["jadx_modes"] = execute(
            out, "duplicate-status-jadx", duplicate_jadx_classes,
            "defpackage.AssertDuplicateStatusWriteRunner")
    if duplicate_jarde_exit == 0:
        summary["duplicate_status_write"]["jarde_modes"] = execute(
            out, "duplicate-status-jarde", duplicate_jarde_classes,
            "AssertDuplicateStatusWriteRunner")

    handler_classes = compile_java(out, "handler-original", [
        HERE / "AssertHandlerBoundary.java", HERE / "AssertHandlerBoundaryRunner.java"])
    handler_class = handler_classes / "AssertHandlerBoundary.class"
    synthetic = add_synthetic_flag(handler_class, "$assertionsDisabled")
    summary["handler"] = {"class_sha256": sha(handler_class), "flag_patch": synthetic,
                          "sources": {p.name: sha(p) for p in (
                              HERE / "AssertHandlerBoundary.java",
                              HERE / "AssertHandlerBoundaryRunner.java")},
                          "modes": execute(out, "handler-original", handler_classes,
                                           "AssertHandlerBoundaryRunner")}
    run(["javap", "-p", "-c", "-v", str(handler_class)], out, "handler-javap")
    handler_jadx = jadx_source(out, handler_class, "handler", "AssertHandlerBoundary")
    handler_jarde = jarde_source(out, cli, "handler", handler_class, "AssertHandlerBoundary")
    summary["handler"]["jadx_source_sha256"] = sha(handler_jadx)
    summary["handler"]["jarde_source_sha256"] = sha(handler_jarde)
    for label, source in (("jadx", handler_jadx), ("jarde", handler_jarde)):
        src = out / f"handler-{label}-overlay"
        src.mkdir()
        runner = src / "AssertHandlerBoundaryRunner.java"
        package_prefix = "package defpackage;\n" if label == "jadx" else ""
        runner.write_text(package_prefix +
                          (HERE / "AssertHandlerBoundaryRunner.java").read_text(encoding="utf-8"),
                          encoding="utf-8")
        if label == "jarde":
            classes, compile_exit = compile_observed(out, f"handler-{label}", [source, runner])
            summary["handler"]["jarde_javac_exit"] = compile_exit
            if compile_exit:
                continue
        else:
            classes = compile_java(out, f"handler-{label}", [source, runner])
        summary["handler"][f"{label}_source_sha256"] = sha(source)
        if label != "jarde" or compile_exit == 0:
            summary["handler"][f"{label}_modes"] = execute(
                out, f"handler-{label}", classes,
                "defpackage.AssertHandlerBoundaryRunner" if label == "jadx"
                else "AssertHandlerBoundaryRunner")

    # Capture Jarde presentation and Java 8 compilation for the existing positive and refusal shapes.
    generated_positive = jarde_source(out, cli, "positive", positive, "AssertVariants")
    positive_overlay = out / "positive-jarde-overlay"
    positive_overlay.mkdir()
    positive_runner = positive_overlay / "AssertVariantsRunner.java"
    positive_runner.write_text((HERE / "AssertVariantsRunner.java").read_text(encoding="utf-8"),
                               encoding="utf-8")
    positive_jarde_classes, positive_jarde_exit = compile_observed(
        out, "positive-jarde", [generated_positive, positive_runner])
    summary["positive"] = {"class_sha256": sha(positive),
                           "jarde_source_sha256": sha(generated_positive),
                           "javac_exit": positive_jarde_exit}
    if positive_jarde_exit == 0:
        summary["positive"]["modes"] = execute(out, "positive-jarde", positive_jarde_classes,
                                                "AssertVariantsRunner")

    negative_class_dir = legacy / "negative-classes"
    generated_negative = [
        jarde_source(out, cli, "extra-field", negative_class_dir / "AssertExtraFieldAccess.class",
                     "AssertExtraFieldAccess"),
        jarde_source(out, cli, "different-constructor",
                     negative_class_dir / "AssertDifferentConstructor.class",
                     "AssertDifferentConstructor"),
    ]
    negative_overlay = out / "negative-jarde-overlay"
    negative_overlay.mkdir()
    negative_runner = negative_overlay / "RejectRunner.java"
    negative_runner.write_text((HERE / "RejectRunner.java").read_text(encoding="utf-8"),
                               encoding="utf-8")
    negative_jarde_classes, negative_jarde_exit = compile_observed(
        out, "negative-jarde", generated_negative + [negative_runner])
    summary["negative_jarde"] = {
        name: {"class_sha256": sha(negative_class_dir / f"{class_name}.class"),
               "jarde_source_sha256": sha(generated), "javac_exit": 0}
        for name, class_name, generated in (
            ("extra-field", "AssertExtraFieldAccess", generated_negative[0]),
            ("different-constructor", "AssertDifferentConstructor", generated_negative[1]),
        )}
    summary["negative_jarde"]["javac_exit"] = negative_jarde_exit
    if negative_jarde_exit == 0:
        summary["negative_jarde"]["modes"] = execute(
            out, "negative-jarde", negative_jarde_classes, "RejectRunner")
    (out / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
