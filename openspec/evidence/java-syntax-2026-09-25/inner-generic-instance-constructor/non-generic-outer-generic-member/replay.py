#!/usr/bin/env python3
"""Reproduce evidence for a generic member-class qualified construction."""
from __future__ import annotations

import hashlib
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory


HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"
EXPECTED_OBJECT_OUTPUT = "minimal.Outer$Inner:1\nnull:1\n"


def run(*args: object, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run([str(arg) for arg in args], cwd=HERE,
                            text=True, capture_output=True)
    if check and result.returncode != 0:
        raise RuntimeError(f"command failed ({result.returncode}): {args}\n"
                           f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    return result


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def compile_record(label: str, output: Path, *sources: Path) -> subprocess.CompletedProcess[str]:
    output.mkdir()
    result = run("javac", "--release", "8", "-Xlint:-options", "-d", output,
                 *sources, check=False)
    write(HERE / f"{label}.txt",
          f"exit={result.returncode}\nstdout:\n{result.stdout}stderr:\n{result.stderr}")
    return result


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


version = run("javac", "-version")
write(HERE / "tool-versions.txt",
      f"{(version.stderr or version.stdout).strip()}\n"
      f"jadx {run('jadx', '--version').stdout.strip()}\n")
hashes: list[str] = []

with TemporaryDirectory(prefix="jarde-inner-member-generic-") as temporary:
    work = Path(temporary)
    original_no_debug: Path | None = None

    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        classes = work / f"original-{label}"
        classes.mkdir()
        run("javac", "--release", "8", "-Xlint:-options", debug,
            "-d", classes, FIXTURE / "Outer.java", FIXTURE / "Use.java",
            FIXTURE / "UseObject.java")
        original_use = run("java", "-Xverify:all", "-cp", classes,
                           "minimal.Use").stdout
        assert original_use == "7\n", original_use
        write(HERE / f"original-{label}-run.txt", original_use)
        original_object = run("java", "-Xverify:all", "-cp", classes,
                              "minimal.UseObject").stdout
        assert original_object == EXPECTED_OBJECT_OUTPUT, original_object
        write(HERE / f"original-object-{label}-run.txt", original_object)

        outer = classes / "minimal/Outer.class"
        inner = classes / "minimal/Outer$Inner.class"
        javap = run("javap", "-classpath", classes, "-p", "-s", "-v",
                    "minimal.Outer$Inner").stdout.replace(str(classes), "<CLASS_DIR>")
        write(HERE / f"javap-inner-{label}.txt", javap)
        assert "(Lminimal/Outer;Ljava/lang/Object;)V" in javap
        assert "// (TV;)V" in javap
        hashes.extend((f"{label} Outer.class {digest(outer)}",
                       f"{label} Outer$Inner.class {digest(inner)}"))

        jadx_dir = work / f"jadx-{label}"
        jadx_result = run("jadx", "-d", jadx_dir, outer, inner,
                          classes / "minimal/Use.class",
                          classes / "minimal/UseObject.class")
        write(HERE / f"jadx-{label}.log", jadx_result.stdout + jadx_result.stderr)
        source_dir = jadx_dir / "sources/minimal"
        stored_sources = HERE / f"jadx-{label}"
        stored_sources.mkdir(exist_ok=True)
        for name in ("Outer.java", "Use.java", "UseObject.java"):
            shutil.copyfile(source_dir / name, stored_sources / name)
        for name in ("Use.java", "UseObject.java"):
            hashes.append(f"{label} JADX {name} {digest(stored_sources / name)}")
        hashes.append(f"{label} JADX Outer.java {digest(stored_sources / 'Outer.java')}")

        # Compile the complete decompiler output and each caller independently.
        full_out = work / f"jadx-full-compiled-{label}"
        full = compile_record(f"jadx-full-{label}-javac", full_out,
                              *(stored_sources / name for name in
                                ("Outer.java", "Use.java", "UseObject.java")))
        if full.returncode == 0:
            for owner in ("Use", "UseObject"):
                output = run("java", "-Xverify:all", "-cp", full_out,
                             f"minimal.{owner}").stdout
                write(HERE / f"jadx-full-{label}-{owner}-run.txt", output)

        use_out = work / f"jadx-use-compiled-{label}"
        use = compile_record(f"jadx-Use-{label}-javac", use_out,
                             stored_sources / "Outer.java", stored_sources / "Use.java")
        if use.returncode == 0:
            use_run = run("java", "-Xverify:all", "-cp", use_out, "minimal.Use").stdout
            assert use_run == "7\n", use_run
            write(HERE / f"jadx-Use-{label}-run.txt", use_run)

        object_out = work / f"jadx-object-compiled-{label}"
        object_compile = compile_record(f"jadx-UseObject-{label}-javac", object_out,
                                        stored_sources / "Outer.java",
                                        stored_sources / "UseObject.java")
        if object_compile.returncode == 0:
            object_run = run("java", "-Xverify:all", "-cp", object_out,
                             "minimal.UseObject").stdout
            write(HERE / f"jadx-UseObject-{label}-run.txt", object_run)

        if label == "g-none":
            original_no_debug = classes

    assert original_no_debug is not None
    frozen_jar = HERE / "original-outer-g-none.jar"
    jar_result = run("jar", "--create", "--date=2000-01-01T00:00:00Z",
                     "--file", frozen_jar, "-C", original_no_debug,
                     "minimal/Outer.class", "-C", original_no_debug,
                     "minimal/Outer$Inner.class")
    write(HERE / "freeze-jar.txt", jar_result.stdout + jar_result.stderr)
    hashes.append(f"frozen original Outer jar {digest(frozen_jar)}")

    # Only the Object-returning caller is compiled, against the frozen originals.
    caller = work / "caller-only-frozen"
    caller.mkdir()
    caller_result = run("javac", "--release", "8", "-Xlint:-options", "-g:none",
                        "-classpath", frozen_jar, "-d", caller,
                        FIXTURE / "UseObject.java", check=False)
    write(HERE / "caller-only-javac.txt",
          f"exit={caller_result.returncode}\nstdout:\n{caller_result.stdout}"
          f"stderr:\n{caller_result.stderr}")
    assert caller_result.returncode == 0, caller_result.stderr
    caller_output = run("java", "-Xverify:all", "-cp",
                        f"{caller}:{frozen_jar}", "minimal.UseObject").stdout
    assert caller_output == EXPECTED_OBJECT_OUTPUT, caller_output
    write(HERE / "caller-only-run.txt", caller_output)
    hashes.extend((f"fixture Outer.java {digest(FIXTURE / 'Outer.java')}",
                   f"fixture Use.java {digest(FIXTURE / 'Use.java')}",
                   f"fixture UseObject.java {digest(FIXTURE / 'UseObject.java')}"))

write(HERE / "sha256.txt", "\n".join(hashes) + "\n")
print(f"Evidence refreshed in {HERE}")
