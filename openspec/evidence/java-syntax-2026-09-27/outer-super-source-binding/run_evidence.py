#!/usr/bin/env python3
"""Compile and inspect Java 8 Outer.super source-binding counterexamples."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import os


ROOT = Path(__file__).resolve().parent
SRC = ROOT / "src"


def run(argv, *, cwd=None, expect=0):
    env = os.environ.copy()
    env["LC_ALL"] = "C"
    result = subprocess.run(
        [str(part) for part in argv], cwd=cwd, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env,
    )
    if result.returncode != expect:
        raise RuntimeError(
            f"expected exit {expect}, got {result.returncode}: {' '.join(map(str, argv))}\n"
            f"{result.stdout}"
        )
    return result.stdout


def write(name, text):
    (ROOT / name).write_text(text, encoding="utf-8")


def main():
    with tempfile.TemporaryDirectory(prefix="outer-super-source-binding-") as temp:
        temp = Path(temp)
        classes = temp / "classes"
        classes.mkdir()
        sources = sorted(path for path in SRC.rglob("*.java") if path.name != "CheckedNarrowed.java")
        compile_log = run([
            "javac", "--release", "8", "-Xlint:-options", "-g:none",
            "-d", classes, *sources,
        ])
        write("javac-success.log", compile_log or "(no diagnostics)\n")

        outputs = []
        for classname in (
            "overload.OverloadCases",
            "generic.GenericCases",
            "generic.GenericRawCases",
            "checked.CheckedOverloadCases",
        ):
            output = run(["java", "-Xverify:all", "-cp", classes, classname])
            outputs.append(f"$ java -Xverify:all {classname}\n{output}")
        write("run-results.txt", "\n".join(outputs))

        narrowed = run([
            "javac", "-J-Duser.language=en", "-J-Duser.country=US",
            "--release", "8", "-Xlint:-options", "-g:none",
            "-classpath", classes, "-d", temp / "narrowed", SRC / "checked/CheckedNarrowed.java",
        ], expect=1)
        write("checked-narrowed-compile-failure.txt", narrowed.replace(str(SRC), "<source>"))

        inspected = (
            "overload.OverloadCases",
            "overload.OverloadCases$Member",
            "generic.GenericParent",
            "generic.GenericCases",
            "generic.GenericCases$Member",
            "generic.GenericRawCases",
            "generic.GenericRawCases$Member",
            "checked.CheckedParent",
            "checked.CheckedOverloadCases",
            "checked.CheckedOverloadCases$Member",
        )
        for classname in inspected:
            output = run(["javap", "-v", "-c", "-p", "-classpath", classes, classname])
            output = output.replace(str(classes), "<classes>")
            filename = classname.replace("$", "-").replace(".", "-") + ".javap.txt"
            write(filename, output)

        jadx = shutil.which("jadx")
        if jadx:
            out = temp / "jadx"
            run([jadx, "-d", out, classes])
            for path in sorted(out.rglob("*.java")):
                text = path.read_text(encoding="utf-8")
                (ROOT / "jadx" / path.relative_to(out)).parent.mkdir(parents=True, exist_ok=True)
                (ROOT / "jadx" / path.relative_to(out)).write_text(text, encoding="utf-8")
            decompiled_sources = sorted((ROOT / "jadx").rglob("*.java"))
            jadx_classes = temp / "jadx-classes"
            jadx_classes.mkdir()
            jadx_compile = run([
                "javac", "--release", "8", "-Xlint:-options", "-g:none",
                "-d", jadx_classes, *decompiled_sources,
            ])
            write("jadx-javac.log", jadx_compile or "(no diagnostics)\n")
            jadx_runs = []
            for classname in (
                "overload.OverloadCases",
                "generic.GenericCases",
                "generic.GenericRawCases",
                "checked.CheckedOverloadCases",
            ):
                output = run(["java", "-Xverify:all", "-cp", jadx_classes, classname])
                jadx_runs.append(f"$ java -Xverify:all {classname}\n{output}")
            write("jadx-run-results.txt", "\n".join(jadx_runs))
        else:
            write("jadx-javac.log", "jadx unavailable; decompilation comparison skipped\n")

    print("Java 8 compilation, four -Xverify:all runs, overload rejection compile, javap, and optional JADX completed.")


if __name__ == "__main__":
    main()
