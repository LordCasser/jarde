#!/usr/bin/env python3
"""Replay the isolated EM-25 boxing/unboxing slice against the fixed JADX checkout."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em25"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
JADX_TESTS = {
    "others/TestDeboxing.java": "dd5e91add226300c88cbfe8d5439df452d4d08de12e9e300e87fe1c86028486b",
    "others/TestDeboxing2.java": "869e00fe5919c7bf472503a2464b2bc09b9a8c87f9fcedd4e160e0e1c8f28ec0",
    "others/TestCastOfNull.java": "66411af895706dbbb5b93405ecba6d85ab7b9a2eef2d79b7a04d6f9da2bfc301",
    "others/TestDuplicateCast.java": "165b608b0e4d77b352bce2056a98272829429398eb41e2dba7ec8336f5c93220",
    "others/TestPrimitiveCasts.java": "924b8c295f10cbff4a6cc6dfba6bc0fe3c996881b5f4579ad4cf397706132cad",
    "invoke/TestCastInOverloadedInvoke.java": "ce275a64584f006b8e6ee9e516cee134fe1d85a650880f7638cbade6bb4ebc0c",
}
JADX_PRODUCTION = {
    "DeboxingVisitor.java": "e4073f2d0a34ef9fe5cf4deb82d834235d92c10410c2ecdfec3d6c724b39fb9a",
    "MethodInvokeVisitor.java": "60ee5bd34ad003d48c85bc622f7ac160f4a197acb894999140fcedf1914129ff",
    "TypeInferenceVisitor.java": "946a0f9d556fc35fa1a6850b5aa2cb66b923d6ac83bca326f548383e2a0357ea",
    "InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
EXPECTED = """java.lang.Integer:1:true
integer-bounds:-128:true:127:true
integer-outside:-129:false:128:false
integer-context:java.lang.Integer:1
java.lang.Boolean:true:true
boolean-false:java.lang.Boolean:false:true
java.lang.Byte:2:true
java.lang.Short:3:true
java.lang.Character:c:true
character-bounds:127:true:128:false
java.lang.Long:4:true
0:0:7
true:false
null-unbox:NPE"""


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, cwd=None, compact_output=False):
    result = subprocess.run(
        [str(part) for part in command], capture_output=True, text=True, timeout=180, cwd=cwd
    )
    log = Path(log)
    log.parent.mkdir(parents=True, exist_ok=True)
    combined = result.stdout + result.stderr
    if compact_output:
        combined = (
            f"exit={result.returncode}\n"
            f"stdout_bytes={len(result.stdout.encode())}\n"
            f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
            f"stderr_bytes={len(result.stderr.encode())}\n"
            f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n"
        )
    else:
        lines = combined.splitlines()
        combined = "\n".join(line.rstrip() for line in lines) + ("\n" if lines else "")
    log.write_text(combined)
    return result


def compile_and_run(label, source, output, temporary):
    classes = temporary / f"{label}-classes"
    classes.mkdir()
    compile_result = run(
        ["javac", "--release", "8", "-g:none", "-d", classes, source, INPUT / "Runner.java"],
        output / label / "javac.log",
    )
    if compile_result.returncode:
        return {
            "javac_exit": compile_result.returncode,
            "runtime_exit": None,
            "stdout": None,
            "source_sha256": digest(source),
        }
    runtime = run(
        ["java", "-Xverify:all", "-cp", classes, "em25.Runner"],
        output / label / "runtime.log",
    )
    return {
        "javac_exit": compile_result.returncode,
        "runtime_exit": runtime.returncode,
        "stdout": runtime.stdout.strip(),
        "source_sha256": digest(source),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("--out must be absent or empty")
    output.mkdir(parents=True, exist_ok=True)

    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"], output / "jadx-revision.txt")
    if revision.returncode or revision.stdout.strip() != JADX_REVISION:
        raise RuntimeError("JADX checkout revision changed")
    for name, expected_hash in JADX_TESTS.items():
        path = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration" / name
        if digest(path) != expected_hash:
            raise RuntimeError(f"fixed JADX {name} changed")
    production_dir = args.jadx_checkout / "jadx-core/src/main/java/jadx/core"
    for name, expected_hash in JADX_PRODUCTION.items():
        path = production_dir / "codegen" / name if name == "InsnGen.java" else production_dir / "dex/visitors" / name
        if name == "TypeInferenceVisitor.java":
            path = production_dir / "dex/visitors/typeinference" / name
        if digest(path) != expected_hash:
            raise RuntimeError(f"fixed JADX {name} changed")
    for tool in ("javac", "java"):
        if run([tool, "-version"], output / f"{tool}-version.txt").returncode:
            raise RuntimeError(f"{tool} unavailable")

    with tempfile.TemporaryDirectory(prefix="jarde-em25-") as temporary_name:
        temporary = Path(temporary_name)
        original_source = INPUT / "BoxingAudit.java"
        original = compile_and_run("original", original_source, output, temporary)
        if original["javac_exit"] or original["runtime_exit"] or original["stdout"] != EXPECTED:
            raise RuntimeError("original Java 8 source did not match the frozen runner")
        extra_class = run(
            ["javac", "--release", "8", "-g:none", "-d",
             temporary / "original-classes", INPUT / "SecondConsumerCases.java"],
            output / "second-consumer-javac.log",
        )
        if extra_class.returncode:
            raise RuntimeError("the separate second-consumer fixture did not compile")
        bytecode = run(
            ["javap", "-c", "-p", "-classpath", temporary / "original-classes", "em25.BoxingAudit"],
            output / "javap-BoxingAudit.txt",
        )
        if bytecode.returncode or "Integer.valueOf" not in bytecode.stdout or "Long.longValue" not in bytecode.stdout:
            raise RuntimeError("javac did not produce the boxed/unboxed paths")
        original_class = temporary / "original-classes" / "em25" / "BoxingAudit.class"
        original_class_sha256 = digest(original_class)
        jar = temporary / "input.jar"
        if run(["jar", "cf", jar, "-C", temporary / "original-classes", "."], output / "jar.log").returncode:
            raise RuntimeError("input jar failed")

        jadx_root = temporary / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], output / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        source_dir = output / "source"
        source_dir.mkdir(exist_ok=True)
        shutil.copy2(original_source, source_dir / "BoxingAudit.java")
        shutil.copy2(INPUT / "Runner.java", source_dir / "Runner.java")

        jadx_source = source_dir / "jadx-BoxingAudit.java"
        shutil.copy2(jadx_root / "sources/em25/BoxingAudit.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em25.BoxingAudit",
             "--policy", "plain-jar", "--release", "8", "--format", "text"],
            output / "jarde-report.log", compact_output=True,
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-BoxingAudit.java"
        jarde_source.write_text(jarde_result.stdout)
        second_consumer = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em25.SecondConsumerCases",
             "--policy", "plain-jar", "--release", "8", "--format", "text"],
            output / "jarde-second-consumer.log",
            compact_output=True,
        )
        if second_consumer.returncode or "Integer.valueOf(1)" not in second_consumer.stdout or "@bytecode" not in second_consumer.stdout:
            raise RuntimeError("the second-consumer shape did not retain its call and physical fallback")
        (source_dir / "jarde-SecondConsumerCases.java").write_text(second_consumer.stdout)

        mapped = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em25.BoxingAudit",
             "--policy", "plain-jar", "--release", "8", "--format", "json",
             "--evidence", "source_map"],
            output / "jarde-source-map.json.log",
        )
        if mapped.returncode:
            raise RuntimeError("Jarde source-map evidence failed")
        source_map_report = json.loads(mapped.stdout)
        for method in ("boxInteger()Ljava/lang/Object;", "boxBoolean()Ljava/lang/Object;",
                       "boxCharacter()Ljava/lang/Character;"):
            report = next(
                item["outcome"]["report"] for item in source_map_report["methods"]
                if item["outcome"].get("report", {}).get("method") == method
            )
            segments = report["source_map"]["segments"]
            direct_bcis = {segment["origin"]["primary"]["bci"] for segment in segments}
            derived_bcis = {
                origin["bci"]
                for segment in segments
                for origin in segment["origin"]["derived"]
            }
            if len(direct_bcis) < 2 or not derived_bcis:
                raise RuntimeError(f"{method} lost its literal, call or return source anchors")

        jadx_text = jadx_source.read_text()
        jarde_text = jarde_source.read_text()
        for text, required in (
            (jadx_text, ("return 1;", "return true;", "return (byte) 2;", "return (short) 3;", "return 'c';", "return 4L;")),
            (jarde_text, ("return 1;", "return true;", "return 'c';", "return -128;", "return 127;",
                          "return '\\u007f';", "Integer.valueOf(-129)", "Integer.valueOf(128)",
                          "Integer.valueOf(1);", "Byte.valueOf((byte) 2)", "Short.valueOf((short) 3)",
                          "Character.valueOf('\\u0080')", "Long.valueOf(4L)")),
        ):
            if not all(fragment in text for fragment in required):
                raise RuntimeError("fixed boxing output changed")
        for method, expected in (
            ("integerAsNumber", "return java.lang.Integer.valueOf(1);"),
            ("integerConsumedByCall", "return retain((java.lang.Integer) java.lang.Integer.valueOf(1));"),
            ("integerBelowRange", "return java.lang.Integer.valueOf(-129);"),
            ("integerAboveRange", "return java.lang.Integer.valueOf(128);"),
            ("characterAboveAscii", "return java.lang.Character.valueOf('\\u0080');"),
        ):
            method_start = jarde_text.index(f" {method}(")
            method_end = jarde_text.index("\n    }", method_start)
            if expected not in jarde_text[method_start:method_end]:
                raise RuntimeError(f"Jarde did not preserve the conservative {method} form")
        if ".longValue()" not in jadx_text or ".longValue()" not in jarde_text:
            raise RuntimeError("fixed unboxing output changed")
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            named_source = temporary / f"{label}-source" / "em25" / "BoxingAudit.java"
            named_source.parent.mkdir(parents=True)
            shutil.copy2(source, named_source)
            result = compile_and_run(label, named_source, output, temporary)
            if result["javac_exit"] or result["runtime_exit"] or result["stdout"] != EXPECTED:
                raise RuntimeError(f"{label} Java 8 source did not match the frozen runner")
            if label == "jadx":
                jadx = result
            else:
                jarde = result

    summary = {
        "jadx_revision": JADX_REVISION,
        "jadx_test_sha256": JADX_TESTS,
        "jadx_production_sha256": JADX_PRODUCTION,
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {path.name: digest(path) for path in sorted(INPUT.glob("*.java"))},
        "original_class_sha256": original_class_sha256,
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
    }
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
