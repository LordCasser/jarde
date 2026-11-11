#!/usr/bin/env python3
"""Compile and verify the explicitly new, pinned Corretto 8 S2b baseline."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
PLATFORM = Path("crates/jarde-java/tests/fixtures/s2b-declared-local-consumer-r5/platform")
EXPECTED_STDOUT = "s2b-inherited-generics: platform and custom checks passed\n"
FLAGS = ["-g", "-encoding", "UTF-8", "-source", "8", "-target", "8", "-proc:none", "-implicit:none"]
CLASS_NAMES = ["CustomChild", "Driver", "Mid", "Parent", "PlatformChild"]


def digest(path):
    data = path.read_bytes()
    return {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def inventory(root, paths):
    return [{"path": path.relative_to(root).as_posix(), **digest(path)} for path in sorted(paths)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jdk", type=Path, required=True, help="Pinned Corretto 8 JDK directory")
    parser.add_argument("--output", type=Path, required=True, help="New, nonexistent output directory")
    parser.add_argument("--repo", type=Path, default=HERE.parents[2], help="Repository containing the frozen platform headers")
    parser.add_argument("--recovered-sources", type=Path, help="Optional directory containing Engine-emitted CustomChild.java and PlatformChild.java")
    args = parser.parse_args()
    jdk = args.jdk.resolve()
    output = args.output.resolve()
    repo = args.repo.resolve()
    pins = json.loads((HERE / "toolchain.json").read_text())
    for relative, expected in pins["files"].items():
        actual = digest(jdk / relative)
        if actual != expected:
            raise SystemExit("Pinned toolchain mismatch for %s: %s" % (relative, actual))
    if output.exists():
        raise SystemExit("Refusing to overwrite existing output: %s" % output)
    sources = sorted((HERE / "src").glob("*.java"))
    if {path.stem for path in sources} != set(CLASS_NAMES):
        raise SystemExit("Expected exactly the five documented Java source files")
    environment = os.environ.copy()
    for name in ("CLASSPATH", "JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS"):
        environment.pop(name, None)
    environment.update({"LC_ALL": "C", "LANG": "C", "TZ": "UTC", "JAVA_HOME": str(jdk)})
    output.mkdir(parents=True)
    commands = []

    def run(arguments):
        result = subprocess.run(arguments, cwd=output, env=environment, text=True, capture_output=True, timeout=90)
        commands.append({"argv": [str(value) for value in arguments], "exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode:
            raise RuntimeError("Command failed: %s\n%s" % (arguments, result.stderr))
        return result

    def compile_classes(selected_sources, destination, classpath=None):
        destination.mkdir()
        run([str(jdk / "bin/javac"), *FLAGS, "-sourcepath", "", "-classpath", str(classpath or destination), "-d", str(destination), *map(str, selected_sources)])

    def execute(destination):
        result = run([str(jdk / "bin/java"), "-Xverify:all", "-cp", str(destination), "Driver"])
        if result.stdout != EXPECTED_STDOUT or result.stderr:
            raise RuntimeError("Driver output mismatch: %r, %r" % (result.stdout, result.stderr))
        return {"exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr}

    version = run([str(jdk / "bin/javac"), "-version"])
    version_text = (version.stdout + version.stderr).strip()
    if version_text != pins["javac_version"]:
        raise SystemExit("Pinned javac version mismatch: %s" % version_text)
    classes = output / "classes"
    repeat = output / "repeat-classes"
    compile_classes(sources, classes)
    compile_classes(sources, repeat)
    expected_files = {name + ".class" for name in CLASS_NAMES}
    for directory in (classes, repeat):
        if {path.name for path in directory.iterdir()} != expected_files:
            raise RuntimeError("Unexpected compiler output in %s" % directory)
    for name in sorted(expected_files):
        original = (classes / name).read_bytes()
        if original != (repeat / name).read_bytes():
            raise RuntimeError("Nondeterministic class bytes: %s" % name)
        if original[:8] != bytes.fromhex("cafebabe00000034"):
            raise RuntimeError("Expected Java 8 classfile version 52.0: %s" % name)
    original_runtime = execute(classes)
    platform = repo / PLATFORM
    platform_files = sorted(platform.rglob("*.class"))
    if len(platform_files) != 13:
        raise RuntimeError("Expected the existing 13-class frozen platform input")
    manifest = {
        "schema": "s2b-inherited-generics-new-baseline-v1",
        "provenance": "Explicit new source-built baseline. Not recovered historical dev-v3 jar bytes, not dev-v2 javac23 bytes, and not an independent holdout.",
        "compiler": pins,
        "compiler_flags": FLAGS + ["-sourcepath", "", "-classpath", "<isolated-output>", "-d", "<isolated-output>"],
        "sources": inventory(HERE, sources),
        "classes": inventory(output, classes.glob("*.class")),
        "platform": {
            "source": PLATFORM.as_posix(),
            "provenance": "Existing frozen repository headers reused without modification; not regenerated by this script.",
            "classes": inventory(platform, platform_files),
        },
        "validation": {
            "classfile_version": "52.0",
            "two_independent_compilations_byte_identical": True,
            "original_isolated_driver": original_runtime,
            "recovered_class_validation": "Not performed by the baseline build. Use --recovered-sources to separately compile and execute complete Engine-emitted target classes.",
        },
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    if args.recovered_sources:
        recovered_sources = args.recovered_sources.resolve()
        selected = [recovered_sources / (name + ".java") for name in ("CustomChild", "PlatformChild")]
        for path in selected:
            if not path.is_file():
                raise SystemExit("Missing complete recovered class source: %s" % path)
        recovered = output / "recovered-classes"
        recovered.mkdir()
        for name in ("Parent", "Mid"):
            shutil.copyfile(classes / (name + ".class"), recovered / (name + ".class"))
        # Neither the original target classes nor original Driver are on this compile/runtime classpath.
        run([str(jdk / "bin/javac"), *FLAGS, "-sourcepath", "", "-classpath", str(recovered), "-d", str(recovered), *map(str, selected)])
        run([str(jdk / "bin/javac"), *FLAGS, "-sourcepath", "", "-classpath", str(recovered), "-d", str(recovered), str(HERE / "src/Driver.java")])
        recovered_runtime = execute(recovered)
        recovered_manifest = {
            "provenance": "Full Engine-emitted targets compiled without source edits, with only original Parent/Mid support classes; Driver separately compiled against the recovered class set.",
            "sources": inventory(recovered_sources, selected),
            "classes": inventory(output, recovered.glob("*.class")),
            "runtime": recovered_runtime,
            "stdout_matches_new_original_baseline": recovered_runtime["stdout"] == original_runtime["stdout"],
        }
        (output / "recovered-validation.json").write_text(json.dumps(recovered_manifest, indent=2) + "\n")
    (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
    print("Verified new baseline: five Java 8 classes, two byte-identical builds, isolated Driver passed.")
    if args.recovered_sources:
        print("Verified complete recovered target classes: isolated Driver matched the new baseline.")


if __name__ == "__main__":
    main()
