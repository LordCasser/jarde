#!/usr/bin/env python3
"""Contrast a legal interface default call with an abstract-owner classfile substitution."""
from __future__ import annotations

import argparse
from hashlib import sha256
import json
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZipFile


HERE = Path(__file__).resolve().parent


def command(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(*args: object) -> subprocess.CompletedProcess[str]:
    result = command(*args)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, args))}\n{result.stdout}{result.stderr}")
    return result


def save(path: Path, value: str) -> None:
    path.write_text(value)


def archive(classes: Path, jar: Path) -> None:
    with ZipFile(jar, "w") as output:
        for item in sorted(classes.glob("*.class")):
            output.write(item, item.name)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, default=Path("/opt/homebrew/bin/jadx"))
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    jadx = args.jadx.resolve()
    if not cli.is_file() or not jadx.is_file():
        parser.error("Jarde CLI and JADX executables must exist")

    generated: list[Path] = []
    hashes: list[str] = []
    javac_version = checked("javac", "-version")
    jadx_version = checked(jadx, "--version")
    versions = (
        (javac_version.stderr or javac_version.stdout).strip(),
        (jadx_version.stderr or jadx_version.stdout).strip(),
    )
    version_file = HERE / "tool-versions.txt"
    save(version_file, "\n".join(versions) + "\n")
    generated.append(version_file)
    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        with TemporaryDirectory(prefix=f"jarde-abstract-super-{label}-") as temporary:
            work = Path(temporary)
            valid = work / "valid"
            abstract = work / "abstract"
            inherited = work / "inherited"
            hybrid = work / "hybrid"
            rebuilt = work / "rebuilt"
            for path in (valid, abstract, inherited, rebuilt):
                path.mkdir()
            checked(
                "javac", "--release", "8", "-Xlint:-options", debug, "-d", valid,
                HERE / "Parent.java", HERE / "default" / "Child.java", HERE / "Probe.java",
            )
            checked(
                "javac", "--release", "8", "-Xlint:-options", debug, "-d", abstract,
                HERE / "Parent.java", HERE / "abstract" / "Child.java",
            )
            checked(
                "javac", "--release", "8", "-Xlint:-options", debug, "-d", inherited,
                HERE / "Parent.java", HERE / "inherited" / "Child.java", HERE / "Probe.java",
            )
            shutil.copytree(valid, hybrid)
            shutil.copy2(abstract / "Child.class", hybrid / "Child.class")
            hashes.extend(
                f"{sha256(path.read_bytes()).hexdigest()}  {label}/{kind}/{path.name}"
                for kind, directory in (("valid", valid), ("hybrid", hybrid), ("inherited", inherited))
                for path in sorted(directory.glob("*.class"))
            )
            valid_run = checked("java", "-Xverify:all", "-cp", valid, "Probe")
            hybrid_run = command("java", "-Xverify:all", "-cp", hybrid, "Probe")
            inherited_run = checked("java", "-Xverify:all", "-cp", inherited, "Probe")
            if valid_run.stdout != "4\n" or inherited_run.stdout != "3\n" or hybrid_run.returncode == 0 or "AbstractMethodError" not in hybrid_run.stderr:
                raise RuntimeError("fixture does not preserve the intended default/abstract behavior")
            for name, value in (
                (f"valid-run-{label}.txt", valid_run.stdout),
                (f"inherited-run-{label}.txt", inherited_run.stdout),
                (f"hybrid-run-{label}.txt", f"exit={hybrid_run.returncode}\n{hybrid_run.stdout}{hybrid_run.stderr}"),
                (f"hybrid-javap-{label}.txt", checked("javap", "-c", "-p", "-v", "-classpath", hybrid, "Probe", "Child").stdout.replace(str(hybrid), "<CLASS_DIR>")),
            ):
                path = HERE / name
                save(path, value)
                generated.append(path)

            jar = work / "hybrid.jar"
            archive(hybrid, jar)
            report = work / "jarde.json"
            checked(cli, "class-source", "--input", jar, "--class", "Probe", "--format", "json", "--output", report)
            data = json.loads(report.read_text())
            jarde_source = HERE / f"jarde-Probe-{label}.java"
            save(jarde_source, data["text"])
            generated.append(jarde_source)
            compile_source = work / "Probe.java"
            compile_source.write_text(data["text"])
            jarde_compile = command("javac", "--release", "8", "-Xlint:-options", "-cp", jar, "-d", rebuilt, compile_source)
            jarde_log = HERE / f"jarde-javac-{label}.txt"
            save(jarde_log, f"exit={jarde_compile.returncode}\n{jarde_compile.stdout}{jarde_compile.stderr}".replace(str(work), "<WORK>"))
            generated.append(jarde_log)
            if jarde_compile.returncode == 0 or "Child.super.value()" not in data["text"]:
                raise RuntimeError("Jarde baseline did not expose the abstract-interface source defect")

            jadx_out = work / "jadx"
            checked(jadx, "-d", jadx_out, jar)
            jadx_source = HERE / f"jadx-Probe-{label}.java"
            save(jadx_source, (jadx_out / "sources" / "defpackage" / "Probe.java").read_text())
            generated.append(jadx_source)
            jadx_compile_source = work / "jadx-compile" / "Probe.java"
            jadx_compile_source.parent.mkdir()
            jadx_compile_source.write_text(jadx_source.read_text().replace("package defpackage;\n", ""))
            jadx_compile_dir = work / "jadx-rebuilt"
            jadx_compile_dir.mkdir()
            # The generated package line is a JADX input-jar presentation artifact. After removing
            # it, javac still rejects the emitted bare class-super dispatch.
            jadx_compile = command("javac", "--release", "8", "-Xlint:-options", "-cp", jar, "-d", jadx_compile_dir, jadx_compile_source)
            jadx_compile_log = HERE / f"jadx-javac-{label}.txt"
            save(jadx_compile_log, f"exit={jadx_compile.returncode}\n{jadx_compile.stdout}{jadx_compile.stderr}".replace(str(work), "<WORK>"))
            generated.append(jadx_compile_log)

            inherited_jar = work / "inherited.jar"
            archive(inherited, inherited_jar)
            inherited_report = work / "inherited-jarde.json"
            checked(cli, "class-source", "--input", inherited_jar, "--class", "Probe", "--format", "json", "--output", inherited_report)
            inherited_data = json.loads(inherited_report.read_text())
            inherited_source = HERE / f"inherited-jarde-Probe-{label}.java"
            save(inherited_source, inherited_data["text"])
            generated.append(inherited_source)
            inherited_compile_source = work / "inherited-compile" / "Probe.java"
            inherited_compile_source.parent.mkdir()
            inherited_compile_source.write_text(inherited_data["text"])
            inherited_rebuilt = work / "inherited-rebuilt"
            inherited_rebuilt.mkdir()
            checked("javac", "--release", "8", "-Xlint:-options", "-cp", inherited_jar, "-d", inherited_rebuilt, inherited_compile_source)
            inherited_rebuilt_run = checked("java", "-Xverify:all", "-cp", f"{inherited_rebuilt}:{inherited_jar}", "Probe")
            if inherited_rebuilt_run.stdout != inherited_run.stdout or "Child.super.value()" not in inherited_data["text"]:
                raise RuntimeError("Jarde did not preserve the inherited direct-interface default")
            inherited_jarde_log = HERE / f"inherited-jarde-run-{label}.txt"
            save(inherited_jarde_log, inherited_rebuilt_run.stdout)
            generated.append(inherited_jarde_log)

            inherited_jadx_out = work / "inherited-jadx"
            checked(jadx, "-d", inherited_jadx_out, inherited_jar)
            inherited_jadx_source = HERE / f"inherited-jadx-Probe-{label}.java"
            save(inherited_jadx_source, (inherited_jadx_out / "sources" / "defpackage" / "Probe.java").read_text())
            generated.append(inherited_jadx_source)
            inherited_jadx_compile_source = work / "inherited-jadx-compile" / "Probe.java"
            inherited_jadx_compile_source.parent.mkdir()
            inherited_jadx_compile_source.write_text(inherited_jadx_source.read_text().replace("package defpackage;\n", ""))
            inherited_jadx_rebuilt = work / "inherited-jadx-rebuilt"
            inherited_jadx_rebuilt.mkdir()
            inherited_jadx_compile = command(
                "javac", "--release", "8", "-Xlint:-options", "-cp", inherited_jar,
                "-d", inherited_jadx_rebuilt, inherited_jadx_compile_source,
            )
            inherited_jadx_log = HERE / f"inherited-jadx-javac-{label}.txt"
            save(inherited_jadx_log, f"exit={inherited_jadx_compile.returncode}\n{inherited_jadx_compile.stdout}{inherited_jadx_compile.stderr}".replace(str(work), "<WORK>"))
            generated.append(inherited_jadx_log)
            if inherited_jadx_compile.returncode == 0 or "super.value()" not in inherited_jadx_source.read_text():
                raise RuntimeError("JADX baseline unexpectedly recovered the inherited interface default")
    digest_file = HERE / "classfile-sha256.txt"
    save(digest_file, "\n".join(hashes) + "\n")
    generated.append(digest_file)
    manifest = HERE / "SHA256SUMS.txt"
    inputs = [HERE / "Parent.java", HERE / "Probe.java", HERE / "default" / "Child.java", HERE / "abstract" / "Child.java", HERE / "inherited" / "Child.java", HERE / "analysis.md", HERE / "replay.py"]
    save(manifest, "\n".join(
        f"{sha256(path.read_bytes()).hexdigest()}  {path.relative_to(HERE)}"
        for path in sorted((*inputs, *generated), key=str)
    ) + "\n")
    print("abstract interface-super evidence refreshed; all class/jar outputs removed")


if __name__ == "__main__":
    main()
