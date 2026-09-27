#!/usr/bin/env python3
"""Recompile the fixed Outer.super family through original, JADX, and Jarde Java 8 source."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCE = ROOT / "openspec/evidence/java-syntax-2026-09-26/named-member-outer-receiver/variants/OuterReceiverCases.java"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "test/java/jadx/tests/integration/invoke/TestSuperInvoke.java": "4eae09fd53c675d6be5d239341ee7bd8d90f4c50070957f80b2c2936183373e4",
    "test/java/jadx/tests/integration/invoke/TestSuperInvokeWithGenerics.java": "bee4ceabeb2cf33a692a9ba01adf0abfeff4a527b27a79b402b53b241818d9b8",
    "test/java/jadx/tests/integration/inline/TestSyntheticInline2.java": "14a8e2a426e59c76fa481db41623f1959d22b74b9a21831b4e5acfb03c4e8c05",
    "test/java/jadx/tests/integration/others/TestShadowingSuperMember.java": "3ccaaa8d19c8f502992b14b49cb414a1984949d64aab841a91ac1f2009d1b59f",
    "main/java/jadx/core/dex/visitors/ConstructorVisitor.java": "477a662a43413d0e3dd162ee5ddcf6bb739347f678bb4a8c36687e780c74cc3b",
    "main/java/jadx/core/dex/visitors/InlineMethods.java": "9f6668390a733b496ce3069244120c66eba520929c4daa8f73ff3d5e38f2d92f",
    "main/java/jadx/core/codegen/InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
DEPENDENCIES = ("OuterReceiverCases", "ReceiverBase", "ReceiverMemberBase")
EXPECTED = "20:10:1:3"
BINDING = HERE / "input/em12"
BINDING_CLASSES = ("Case", "Parent", "Arg", "NarrowArg")


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        projection = "\n".join(line for line in result.stderr.splitlines()
                               if line.startswith("member_family.projection."))
        log.write_text(f"exit={result.returncode}\n"
                       f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
                       f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n"
                       + (f"{projection}\n" if projection else ""))
    else:
        lines = f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        log.write_text("\n".join(line.rstrip() for line in lines.splitlines()) + "\n")
    if result.returncode:
        raise RuntimeError(f"command failed: {log}")
    return result


def compile_run(label, sources, output, temporary, main_class, expected):
    classes = temporary / f"{label}-classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-Xlint:-options", "-d", classes, *sources],
        output / label / "javac.log")
    result = run(["java", "-Xverify:all", "-cp", classes, main_class],
                 output / label / "runtime.log")
    if result.stdout.strip() != expected:
        raise RuntimeError(f"{label}: unexpected runtime {result.stdout!r}")
    return classes


def binding_regression(args, out, temporary):
    output = out / "binding"
    original = compile_run("binding-original", [BINDING / f"{name}.java" for name in BINDING_CLASSES],
                           output, temporary, "em12.Case", "number")
    javap_log = output / "javap.log"
    run(["javap", "-classpath", original, "-s", "-c", "-p", "em12.Case",
         "em12.Case$Member", "em12.Parent", "em12.Arg", "em12.NarrowArg"],
        javap_log)
    normalized = javap_log.read_text().replace(str(temporary), "<temporary>")
    normalized = "\n".join(
        "  Last modified <omitted>; size" + line.split("; size", 1)[1]
        if "Last modified " in line and "; size" in line else line
        for line in normalized.splitlines()) + "\n"
    javap_log.write_text(normalized)
    jar = temporary / "binding.jar"
    run(["jar", "cf", jar, "-C", original, "."], output / "jar.log")
    jadx_dir = temporary / "binding-jadx"
    run([args.jadx, "-d", jadx_dir, jar], output / "jadx.log")
    sources = {"jadx": [], "jarde": []}
    projection = None
    for name in BINDING_CLASSES:
        target = output / "source/jadx/em12" / f"{name}.java"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(jadx_dir / "sources/em12" / f"{name}.java", target)
        sources["jadx"].append(target)
        target = output / "source/jarde/em12" / f"{name}.java"
        target.parent.mkdir(parents=True, exist_ok=True)
        result = run([args.jarde, "class-source", "--input", jar, "--class", f"em12.{name}",
                      "--policy", "plain-jar", "--release", "8", "--format", "text"],
                     output / "jarde-logs" / f"{name}.log", compact=True)
        target.write_text(result.stdout)
        sources["jarde"].append(target)
        if name == "Case":
            projection = (output / "jarde-logs/Case.log").read_text()
    compile_run("binding-jadx", sources["jadx"], output, temporary, "em12.Case", "number")
    if 'member_family.projection.reason = "Outer.super source binding refused: source hierarchy contains a competing same-name method"' not in projection:
        raise RuntimeError("Jarde did not expose the expected source-binding refusal")
    classes = temporary / "binding-jarde-classes"
    compile_command = ["javac", "--release", "8", "-Xlint:-options", "-d", classes,
                       *sources["jarde"]]
    result = subprocess.run([str(part) for part in compile_command], capture_output=True,
                            text=True, timeout=180)
    (output / "binding-jarde").mkdir()
    (output / "binding-jarde/javac.log").write_text(
        f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    if result.returncode == 0:
        raise RuntimeError("source-binding regression unexpectedly recompiled")
    return {"input_sha256": {name: digest(BINDING / f"{name}.java") for name in BINDING_CLASSES},
            "original_classes_sha256": {p.name: digest(p) for p in original.rglob("*.class")},
            "original_runtime": "number", "jadx_runtime": "number",
            "jarde_javac_exit": result.returncode,
            "jarde_refusal": "source hierarchy contains a competing same-name method"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be empty")
    out.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   out / "jadx-revision.log").stdout.strip()
    if revision != JADX_REV:
        raise RuntimeError("JADX revision changed")
    for name, expected in PINS.items():
        if digest(args.jadx_checkout / "jadx-core/src" / name) != expected:
            raise RuntimeError(f"JADX source changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-em12-") as temporary_name:
        temporary = Path(temporary_name)
        original_classes = compile_run("original", [SOURCE], out, temporary,
                                       "OuterReceiverCases", EXPECTED)
        jar = temporary / "fixture.jar"
        run(["jar", "cf", jar, "-C", original_classes, "."], out / "jar.log")
        jadx_dir = temporary / "jadx"
        run([args.jadx, "-d", jadx_dir, jar], out / "jadx.log")
        jadx_sources = []
        jarde_sources = []
        for name in DEPENDENCIES:
            generated = jadx_dir / "sources/defpackage" / f"{name}.java"
            if not generated.exists():
                generated = jadx_dir / "sources" / f"{name}.java"
            target = out / "source/jadx" / f"{name}.java"
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(generated, target)
            jadx_sources.append(target)
            target = out / "source/jarde" / f"{name}.java"
            target.parent.mkdir(parents=True, exist_ok=True)
            result = run([args.jarde, "class-source", "--input", jar, "--class", name,
                          "--policy", "plain-jar", "--release", "8", "--format", "text"],
                         out / "jarde-logs" / f"{name}.log", compact=True)
            target.write_text(result.stdout)
            jarde_sources.append(target)
        compile_run("jadx", jadx_sources, out, temporary,
                    "defpackage.OuterReceiverCases", EXPECTED)
        compile_run("jarde", jarde_sources, out, temporary,
                    "OuterReceiverCases", EXPECTED)
        report = (out / "jarde-logs/OuterReceiverCases.log").read_text()
        source = (out / "source/jarde/OuterReceiverCases.java").read_text()
        if 'member_family.projection.state = "projected"' not in report:
            raise RuntimeError("Jarde did not project the family")
        if "OuterReceiverCases.super.value()" not in source:
            raise RuntimeError("Jarde lost lexical outer-super spelling")
        binding = binding_regression(args, out, temporary)
        summary = {
            "jadx_revision": revision,
            "jadx_pins": PINS,
            "jarde_cli_sha256": digest(args.jarde),
            "input_sha256": digest(SOURCE),
            "original_classes_sha256": {p.name: digest(p) for p in original_classes.rglob("*.class")},
            "runtime": EXPECTED,
            "jarde_member_projection": "projected",
            "binding_regression": binding,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
