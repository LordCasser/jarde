#!/usr/bin/env python3
"""Replay a candidate CLI against frozen inputs into a separate output directory."""

from pathlib import Path
import argparse
import csv
import hashlib
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "source"
DEFAULT_JADX = Path("/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx")
JDKS = {
    "jdk8": (Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"), ["-source", "8", "-target", "8"]),
    "jdk23": (Path("/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"), ["--release", "8"]),
}
CASES = [
    ("RawParam", []), ("StaticRawLocal", []), ("InstanceRawLocal", []),
    ("TypedReceiver", []), ("ThisReceiver", []), ("ShadowMethodT", []),
    ("DirectInstanceRawParam", []), ("WideRawParam", []), ("ArrayRawParam", []),
    ("Array2DRawParam", []), ("PrimitiveArrayRawParam", []),
    ("BoundRawParam", []), ("NullRawParam", []), ("RetainedAliasRawParam", []),
    ("MultiFormalRawParam", []), ("RawOwnerChild", ["RawOwnerBase"]),
]
VARIANTS = [("debug", ["-g"]), ("no-debug", ["-g:none"])]
OUTPUT = None
ROWS = []


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_nonempty(path, data):
    if data:
        Path(path).parent.mkdir(parents=True, exist_ok=True)
        Path(path).write_bytes(data)


def record(scope, args, result, stdout_path=None):
    if stdout_path is not None:
        Path(stdout_path).parent.mkdir(parents=True, exist_ok=True)
        Path(stdout_path).write_bytes(result.stdout)
    if result.returncode:
        failure = OUTPUT / "failures" / scope
        write_nonempty(failure.with_suffix(".stdout.txt"), result.stdout)
        write_nonempty(failure.with_suffix(".stderr.txt"), result.stderr)
    ROWS.append((scope, str(result.returncode), " ".join(map(str, args))))
    return result


def run(scope, args, stdout_path=None):
    result = subprocess.run([str(x) for x in args], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return record(scope, args, result, stdout_path)


def compile_java(scope, javac, release_args, debug_args, output, sources):
    output_path = Path(output)
    empty_classpath = output_path.parent / "empty-classpath"
    empty_sourcepath = output_path.parent / "empty-sourcepath"
    empty_classpath.mkdir(parents=True, exist_ok=True)
    empty_sourcepath.mkdir(parents=True, exist_ok=True)
    if any(empty_classpath.iterdir()) or any(empty_sourcepath.iterdir()):
        raise RuntimeError(f"{scope}: javac isolation directories must remain empty")
    args = [javac, *release_args, *debug_args, "-classpath", str(empty_classpath),
            "-sourcepath", str(empty_sourcepath), "-d", output, *sources]
    return run(scope, args)


def runtime(scope, java, classes, class_name, destination):
    return run(scope, [java, "-Xverify:all", "-cp", classes, "ReceiverProbe", class_name], destination)


def extract_package(source_text):
    for line in source_text.splitlines():
        if line.startswith("package ") and line.endswith(";"):
            return line[len("package "):-1]
    return None


def behavior(lines):
    result = []
    for line in lines:
        if line.startswith("behavior.value=") and line != "behavior.value=null":
            value = line.partition("=")[2]
            result.append("behavior.value=" + value.rsplit(".", 1)[-1])
        elif line.startswith("behavior."):
            result.append(line)
    return result


def main():
    global OUTPUT
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True, help="absolute path to the candidate Jarde CLI")
    parser.add_argument("--cli-sha256", required=True, help="expected candidate CLI SHA-256")
    parser.add_argument("--jadx", default=str(DEFAULT_JADX), help="already-built JADX CLI path")
    parser.add_argument("--out", required=True, help="new output directory, kept separate from frozen evidence")
    parser.add_argument("--label", required=True, help="label for this candidate run")
    args = parser.parse_args()
    cli, jadx = Path(args.cli).resolve(), Path(args.jadx).resolve()
    OUTPUT = Path(args.out).resolve()
    if not cli.is_file() or not jadx.is_file():
        raise SystemExit("Jarde/JADX executable path is missing")
    if sha(cli) != args.cli_sha256:
        raise SystemExit("candidate Jarde CLI SHA-256 does not match --cli-sha256")
    if OUTPUT == ROOT or ROOT in OUTPUT.parents:
        raise SystemExit("--out must be outside the frozen evidence directory")
    OUTPUT.mkdir(parents=True, exist_ok=False)
    (OUTPUT / "run-label.txt").write_text(args.label + "\n")

    versions = OUTPUT / "versions"
    run("versions/jarde-cli", [cli, "--version"], versions / "jarde-cli.txt")
    run("versions/jadx-cli", [jadx, "--version"], versions / "jadx-cli.txt")
    tool_rows = [("jarde-cli", str(cli), sha(cli)), ("jadx-cli", str(jadx), sha(jadx))]
    for jar in sorted((jadx.parent.parent / "lib").glob("*.jar")):
        tool_rows.append(("jadx-lib/" + jar.name, str(jar), sha(jar)))
    (versions / "tool-hashes.tsv").write_text("tool\tpath\tsha256\n" + "".join("\t".join(row) + "\n" for row in tool_rows))

    test_fixture_root = ROOT.parents[2] / "tests" / "fixtures" / "raw-receiver-field-selection"
    for leg, (jdk, release_args) in JDKS.items():
        java, javac, jar_tool, javap = (jdk / "bin" / name for name in ("java", "javac", "jar", "javap"))
        leg_root = OUTPUT / leg
        for name, tool in [("java", java), ("javac", javac)]:
            result = run(f"{leg}/versions/{name}", [tool, "-version"])
            write_nonempty(leg_root / "versions" / f"{name}.txt", result.stdout + result.stderr)

        for case, helpers in CASES:
            for variant, debug_args in VARIANTS:
                scope = f"{leg}/{variant}/{case}"
                frozen_jar = ROOT / leg / variant / "input" / f"{case}.jar"
                if not frozen_jar.is_file():
                    raise SystemExit(f"missing frozen input: {frozen_jar}")
                contents = run(scope + "/jar-contents", [jar_tool, "tf", frozen_jar]).stdout.decode().splitlines()
                expected = [name + ".class" for name in [case] + helpers]
                if contents != expected:
                    raise SystemExit(f"{scope}: frozen jar entries differ: {contents} != {expected}")
                if case in {"RawParam", "StaticRawLocal", "WideRawParam", "NullRawParam", "ArrayRawParam", "Array2DRawParam", "PrimitiveArrayRawParam", "BoundRawParam", "RawOwnerChild"}:
                    fixture_jar = test_fixture_root / f"{case}.jar"
                    if not fixture_jar.is_file():
                        raise SystemExit(f"missing test fixture jar: {fixture_jar}")

                area = leg_root / variant
                with tempfile.TemporaryDirectory(prefix="jarde-raw-receiver-replay-") as tmp_name:
                    tmp = Path(tmp_name)
                    original_dir = tmp / "original"
                    original_dir.mkdir()
                    original_sources = [SRC / f"{case}.java", *[SRC / f"{helper}.java" for helper in helpers]]
                    compiled = compile_java(scope + "/original-javac", javac, release_args, debug_args,
                                            original_dir, original_sources)
                    if compiled.returncode == 0:
                        # Verify deterministic compiler output against the frozen class/jar bytes.
                        with zipfile.ZipFile(frozen_jar) as archive:
                            for class_name in [case] + helpers:
                                entry = class_name + ".class"
                                compiled_bytes = (original_dir / entry).read_bytes()
                                if compiled_bytes != archive.read(entry):
                                    raise SystemExit(f"{scope}: recompiled {entry} differs from frozen input")
                        run(scope + "/javap", [javap, "-p", "-c", "-s", "-v", "-classpath", frozen_jar, case],
                            area / "javap" / f"{case}.txt")
                        original_probe = tmp / "original-probe"
                        original_probe.mkdir()
                        original_probe_compile = compile_java(scope + "/original-probe-javac", javac, release_args,
                            debug_args, original_probe, [SRC / "ReceiverProbe.java"])
                        if original_probe_compile.returncode == 0:
                            runtime(scope + "/original-run", java, f"{original_dir}:{original_probe}", case,
                                    area / "runtime" / f"{case}.original.txt")

                        # Every Jarde source is obtained from the same frozen input jar. For a required
                        # helper, query Jarde itself and compile that independent presentation too.
                        jarde_dir = area / "jarde"
                        jarde_sources = []
                        for selected in [case] + helpers:
                            source_path = jarde_dir / (selected + ".java")
                            result = run(scope + f"/jarde-{selected}", [cli, "class-source", "--input", frozen_jar,
                                "--class", selected, "--policy", "plain-jar", "--release", "8", "--format", "text"])
                            write_nonempty(source_path, result.stdout)
                            write_nonempty(jarde_dir / f"{selected}.stderr.txt", result.stderr)
                            source_text = result.stdout.decode(errors="replace")
                            ok = bool(source_text.strip()) and f"presentation of `{selected}`" in source_text and f"class {selected}" in source_text
                            ROWS.append((scope + f"/jarde-{selected}-source-header", "0" if ok else "1", "nonempty self-describing source"))
                            if source_path.is_file(): jarde_sources.append(source_path)
                        if len(jarde_sources) == len([case] + helpers):
                            jarde_classes = tmp / "jarde-classes"
                            jarde_classes.mkdir()
                            result = compile_java(scope + "/jarde-javac", javac, release_args, debug_args,
                                                  jarde_classes, [*jarde_sources, SRC / "ReceiverProbe.java"])
                            if result.returncode == 0:
                                runtime(scope + "/jarde-verify-run", java, str(jarde_classes), case,
                                        area / "runtime" / f"{case}.jarde.txt")

                        # JADX decompiles the exact frozen input. A helper family is decompiled together,
                        # so no original helper source/class is used to make its candidate compilable.
                        jadx_dir = area / "jadx" / case
                        jadx_args = [jadx, "--no-res"]
                        if not helpers:
                            jadx_args += ["--single-class", case]
                        jadx_args += ["-d", jadx_dir, frozen_jar]
                        jadx_result = run(scope + "/jadx", jadx_args)
                        write_nonempty(jadx_dir / "stderr.txt", jadx_result.stderr)
                        jadx_subject = next(jadx_dir.rglob(case + ".java"), None)
                        if jadx_subject is None:
                            ROWS.append((scope + "/jadx-source-header", "1", "class source not emitted"))
                            continue
                        jadx_text = jadx_subject.read_text(errors="replace")
                        package = extract_package(jadx_text)
                        jadx_class = (package + "." if package else "") + case
                        jadx_sources = sorted(jadx_dir.rglob("*.java")) if helpers else [jadx_subject]
                        ROWS.append((scope + "/jadx-source-header", "0" if f"class {case}" in jadx_text else "1", "nonempty class source"))
                        jadx_classes = tmp / "jadx-classes"
                        jadx_classes.mkdir()
                        compiled = compile_java(scope + "/jadx-javac", javac, release_args, debug_args,
                                                jadx_classes, [*jadx_sources, SRC / "ReceiverProbe.java"])
                        if compiled.returncode == 0:
                            runtime(scope + "/jadx-verify-run", java, str(jadx_classes), jadx_class,
                                    area / "runtime" / f"{case}.jadx.txt")

    with (OUTPUT / "runtime-comparisons.tsv").open("w") as stream:
        stream.write("leg\tvariant\tcase\tjarde_behavior_vs_original\tjadx_behavior_vs_original\toriginal_field_type\tjarde_field_type\tjadx_field_type\n")
        for leg in JDKS:
            for variant, _ in VARIANTS:
                area = OUTPUT / leg / variant
                for case, _ in CASES:
                    outputs = {name: (area / "runtime" / f"{case}.{name}.txt").read_text().splitlines()
                               if (area / "runtime" / f"{case}.{name}.txt").is_file() else []
                               for name in ("original", "jarde", "jadx")}
                    baseline = behavior(outputs["original"])
                    matches = [behavior(outputs[name]) == baseline and bool(outputs[name]) for name in ("jarde", "jadx")]
                    fields = [next((line.partition("=")[2] for line in outputs[name] if line.startswith("field.type=")), "missing")
                              for name in ("original", "jarde", "jadx")]
                    stream.write(f"{leg}\t{variant}\t{case}\t{str(matches[0]).lower()}\t{str(matches[1]).lower()}\t" + "\t".join(fields) + "\n")

    repo = ROOT.parents[2]
    source_rows = [(path.relative_to(ROOT).as_posix(), sha(path)) for path in sorted(SRC.glob("*.java"))]
    generated_sources = list(OUTPUT.glob("jdk*/*/jarde/**/*.java")) + list(OUTPUT.glob("jdk*/*/jadx/**/*.java"))
    for path in sorted(generated_sources):
        source_rows.append((str(Path(args.label) / path.relative_to(OUTPUT)), sha(path)))
    for path in sorted((repo / "tests/fixtures/raw-receiver-field-selection").glob("*.java")):
        source_rows.append((str(Path("tests/fixtures/raw-receiver-field-selection") / path.name), sha(path)))
    versions.mkdir(parents=True, exist_ok=True)
    (versions / "source-hashes.tsv").write_text("source\tsha256\n" + "".join(f"{path}\t{digest}\n" for path, digest in source_rows))
    (versions / "frozen-input-jar-hashes.tsv").write_text(
        "input_jar\tsha256\n" + "".join(f"{p.relative_to(ROOT)}\t{sha(p)}\n" for p in sorted(ROOT.glob("jdk*/*/input/*.jar")))
    )
    with (OUTPUT / "commands.tsv").open("w", newline="") as stream:
        writer = csv.writer(stream, delimiter="\t", lineterminator="\n")
        writer.writerow(["scope", "exit_code", "command"])
        writer.writerows(ROWS)


if __name__ == "__main__":
    main()
