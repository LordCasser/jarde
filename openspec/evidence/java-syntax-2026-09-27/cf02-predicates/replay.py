#!/usr/bin/env python3
"""Replay fixed JADX CF-02 predicates and compare complete Java 8 class sources."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf02"
EFFECTS = HERE / "input/effects/cf02"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "test/java/jadx/tests/integration/conditions/TestCmpOp.java": "5b8d2d6b4f1a32fe06f609ce6458fd6344290214ebbf787fb4f15b8ca1aaf7d0",
    "test/java/jadx/tests/integration/conditions/TestCmpOp2.java": "b9f50b29f3b2f323137c4026b2c509b609086b854e7f48d286dcb078b9e2ce9a",
    "test/java/jadx/tests/integration/conditions/TestConditions7.java": "45cb838ad4ddd173a5260c433d50b920184e36e16211f723b03a35dbaeff1d5c",
    "test/java/jadx/tests/integration/conditions/TestTernary3.java": "73f3017522b72f24d96ff65d5919d859fec787b17668e3213e71236bda99cbea",
    "main/java/jadx/core/dex/instructions/IfNode.java": "fa1ce4ebf7e99b05434e61f07858293f10a0ee048602adc195bfed90377e1191",
    "main/java/jadx/core/dex/instructions/IfOp.java": "a4e19f99e4024160328e65dcb9eb6e76365d021e2ef551db709bfe2a1d4eaba8",
    "main/java/jadx/core/dex/regions/conditions/Compare.java": "e6e1fbb428f3903b7d564470b46ca4c384ed6869ba9b0898f9a71580b3c9d774",
    "main/java/jadx/core/dex/regions/conditions/IfCondition.java": "4d80dcce617f8bd799a5ef136dbaca981fd6a08c4d242410809184fdefc3af5e",
    "main/java/jadx/core/codegen/ConditionGen.java": "dee46ba02afb449f7323d4deffd18053e5585b8425ff19f69caf6699f1895e55",
}
EXPECTED = "false\ntrue\nfalse\nfalse\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue"
EXPECTED_EFFECTS = "false:0\nfalse:0\nfalse:1\ntrue:1"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True,
                            timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    log.write_text("\n".join(line.rstrip() for line in lines.splitlines()) + "\n")
    if result.returncode:
        raise RuntimeError(f"command failed: {log}")
    return result


def compile_run(label, source, output, temporary, runner=None, main="cf02.Runner"):
    classes = temporary / f"{label}-classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-Xlint:-options", "-g:none", "-d", classes,
         source, runner or INPUT / "Runner.java"], output / label / "javac.log")
    result = run(["java", "-Xverify:all", "-cp", classes, main],
                 output / label / "runtime.log")
    return classes, result.stdout.strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("--out must be empty")
    output.mkdir(parents=True, exist_ok=True)
    rev = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
              output / "jadx-revision.log").stdout.strip()
    if rev != JADX_REV:
        raise RuntimeError("JADX revision changed")
    for name, expected in PINS.items():
        if sha(args.jadx_checkout / "jadx-core/src" / name) != expected:
            raise RuntimeError(f"JADX source changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-cf02-") as temp_name:
        temporary = Path(temp_name)
        classes, original = compile_run("original", INPUT / "Predicates.java", output, temporary)
        jar = temporary / "fixture.jar"
        run(["jar", "cf", jar, "-C", classes, "cf02/Predicates.class"], output / "jar.log")
        jadx_dir = temporary / "jadx"
        run([args.jadx, "-d", jadx_dir, jar], output / "jadx.log")
        jadx_source = output / "source/jadx/cf02/Predicates.java"
        jadx_source.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(jadx_dir / "sources/cf02/Predicates.java", jadx_source)
        jarde_source = output / "source/jarde/cf02/Predicates.java"
        jarde_source.parent.mkdir(parents=True, exist_ok=True)
        generated = run([args.jarde, "class-source", "--input", jar, "--class", "cf02.Predicates",
                         "--policy", "plain-jar", "--release", "8", "--format", "text"],
                        output / "jarde.log")
        jarde_source.write_text(generated.stdout)
        probe_env = dict(os.environ, JRE_JOIN_PROBE="1", JRE_PREFIX_PROBE="1")
        probe = subprocess.run(
            [str(args.jarde), "class-source", "--input", str(jar), "--class", "cf02.Predicates",
             "--policy", "plain-jar", "--release", "8", "--format", "text"],
            capture_output=True, text=True, timeout=180, env=probe_env)
        if probe.returncode:
            raise RuntimeError("Jarde region probe failed")
        (output / "region-probe.log").write_text(
            "\n".join(line for line in probe.stderr.splitlines()
                      if line.startswith(("P3JOIN", "P3VISITED", "P3LOST"))) + "\n")
        _, jadx = compile_run("jadx", jadx_source, output, temporary)
        _, jarde = compile_run("jarde", jarde_source, output, temporary)
        if original != EXPECTED or jadx != EXPECTED or jarde != EXPECTED:
            raise RuntimeError(f"unexpected predicate behavior: {original!r} {jadx!r} {jarde!r}")
        effects_classes, effects_original = compile_run(
            "effects-original", EFFECTS / "PredicateEffects.java", output, temporary,
            EFFECTS / "EffectsRunner.java", "cf02.EffectsRunner")
        effects_jar = temporary / "effects.jar"
        run(["jar", "cf", effects_jar, "-C", effects_classes,
             "cf02/PredicateEffects.class"], output / "effects-jar.log")
        effects_source = output / "source/jarde/cf02/PredicateEffects.java"
        effects_source.write_text(run(
            [args.jarde, "class-source", "--input", effects_jar,
             "--class", "cf02.PredicateEffects", "--policy", "plain-jar",
             "--release", "8", "--format", "text"],
            output / "effects-jarde.log").stdout)
        _, effects_jarde = compile_run(
            "effects-jarde", effects_source, output, temporary,
            EFFECTS / "EffectsRunner.java", "cf02.EffectsRunner")
        if effects_original != EXPECTED_EFFECTS or effects_jarde != EXPECTED_EFFECTS:
            raise RuntimeError(f"unexpected call counts: {effects_original!r} {effects_jarde!r}")
        summary = {"jadx_revision": rev, "jadx_pins": PINS,
                   "jarde_cli_sha256": sha(args.jarde),
                   "input_sha256": {"predicates/" + p.name: sha(p) for p in INPUT.glob("*.java")}
                       | {"effects/" + p.name: sha(p) for p in EFFECTS.glob("*.java")},
                   "original_class_sha256": sha(classes / "cf02/Predicates.class"),
                   "effects_class_sha256": sha(effects_classes / "cf02/PredicateEffects.class"),
                   "source_sha256": {"jadx": sha(jadx_source), "jarde": sha(jarde_source),
                                     "effects_jarde": sha(effects_source)},
                   "original_stdout": original, "jadx_stdout": jadx, "jarde_stdout": jarde,
                   "effects_original_stdout": effects_original,
                   "effects_jarde_stdout": effects_jarde}
        (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps({"original": original, "jadx": jadx, "jarde": jarde}, indent=2))


if __name__ == "__main__":
    main()
