#!/usr/bin/env python3
"""Replay a verifier-valid interface call whose Java qualifier is redundant via superclass."""
from __future__ import annotations

from hashlib import sha256
import json
from pathlib import Path
import runpy
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZipFile


HERE = Path(__file__).resolve().parent
PATCH = runpy.run_path(str(HERE.parent / "redundant-interface-super" / "replay.py"))[
    "replace_interface_owner"
]


def command(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(*args: object) -> subprocess.CompletedProcess[str]:
    result = command(*args)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, args))}\n{result.stdout}{result.stderr}")
    return result


def digest(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        parser.error(f"missing Jarde CLI: {cli}")

    hashes: list[str] = []
    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        with TemporaryDirectory(prefix=f"jarde-superclass-interface-{label}-") as temporary:
            work = Path(temporary)
            classes = work / "classes"
            classes.mkdir()
            checked("javac", "--release", "8", "-Xlint:-options", debug,
                    "-d", classes, HERE / "Probe.java")
            original = checked("java", "-Xverify:all", "-cp", classes, "p.Runner").stdout
            if original != "2\n":
                raise RuntimeError(f"{label}: unexpected original output {original!r}")
            child = classes / "p" / "Child.class"
            child.write_bytes(PATCH(child.read_bytes(), "p/B", "p/A", "m"))
            hashes.append(f"{digest(child)}  {label}/Child-patched.class")
            patched = checked("java", "-Xverify:all", "-cp", classes, "p.Runner").stdout
            if patched != "1\n":
                raise RuntimeError(f"{label}: unexpected patched output {patched!r}")
            (HERE / f"original-run-{label}.txt").write_text(original)
            (HERE / f"patched-run-{label}.txt").write_text(patched)
            javap = checked("javap", "-v", "-p", "-c", "-classpath", classes, "p.Child").stdout
            (HERE / f"patched-javap-{label}.txt").write_text(
                "\n".join(line for line in javap.splitlines()
                          if not line.lstrip().startswith("Last modified"))
                .replace(str(classes), "<CLASS_DIR>") + "\n"
            )

            illegal = work / "Probe.java"
            illegal.write_text((HERE / "Probe.java").read_text().replace("B.super.m()", "A.super.m()"))
            illegal_result = command("javac", "--release", "8", "-Xlint:-options", debug,
                                     "-d", work / "illegal", illegal)
            if illegal_result.returncode == 0 or "冗余接口" not in illegal_result.stderr and "redundant interface" not in illegal_result.stderr:
                raise RuntimeError(f"{label}: expected redundancy diagnosis: {illegal_result.stderr}")
            (HERE / f"illegal-javac-{label}.txt").write_text(
                f"exit={illegal_result.returncode}\n" + illegal_result.stderr.replace(str(work), "<WORK>")
            )

            jar = work / "patched.jar"
            with ZipFile(jar, "w") as archive:
                for path in sorted(classes.rglob("*.class")):
                    archive.write(path, path.relative_to(classes))
            report_path = work / "jarde.json"
            checked(cli, "class-source", "--input", jar, "--class", "p/Child",
                    "--format", "json", "--output", report_path)
            report = json.loads(report_path.read_text())
            if report["execution"]["status"] != "complete":
                raise RuntimeError(f"{label}: Jarde baseline request stopped")
            source = report["text"]
            (HERE / f"baseline-jarde-Child-{label}.java").write_text(source)
            rebuilt_source = work / "jarde" / "p" / "Child.java"
            rebuilt_source.parent.mkdir(parents=True)
            rebuilt_source.write_text(source)
            compile_result = command("javac", "--release", "8", "-Xlint:-options", debug,
                                     "-cp", classes, "-d", work / "jarde-classes", rebuilt_source)
            if "A.super.m()" not in source or compile_result.returncode == 0:
                raise RuntimeError(f"{label}: expected pre-fix illegal Jarde qualifier")
            (HERE / f"baseline-jarde-javac-{label}.txt").write_text(
                f"exit={compile_result.returncode}\n"
                + compile_result.stderr.replace(str(work), "<WORK>")
            )
            print(f"{label}: original=2, patched JVM=1, Java/Jarde qualifier rejected")

            # The parent itself has no interface row; the redundant A is inherited from Root.
            transitive = work / "transitive"
            transitive.mkdir()
            checked("javac", "--release", "8", "-Xlint:-options", debug,
                    "-d", transitive, HERE / "TransitiveProbe.java")
            deeper_child = transitive / "p" / "Child.class"
            deeper_child.write_bytes(PATCH(deeper_child.read_bytes(), "p/B", "p/A", "m"))
            hashes.append(f"{digest(deeper_child)}  {label}/Child-patched-transitive.class")
            deeper_result = checked("java", "-Xverify:all", "-cp", transitive, "p.Runner").stdout
            if deeper_result != "1\n":
                raise RuntimeError(f"{label}: unexpected transitive output {deeper_result!r}")
            (HERE / f"transitive-patched-run-{label}.txt").write_text(deeper_result)
            deeper_illegal = work / "TransitiveProbe.java"
            deeper_illegal.write_text(
                (HERE / "TransitiveProbe.java").read_text().replace("B.super.m()", "A.super.m()")
            )
            deeper_javac = command("javac", "--release", "8", "-Xlint:-options", debug,
                                   "-d", work / "transitive-illegal", deeper_illegal)
            if deeper_javac.returncode == 0 or "冗余接口" not in deeper_javac.stderr and "redundant interface" not in deeper_javac.stderr:
                raise RuntimeError(f"{label}: expected transitive redundancy diagnosis: {deeper_javac.stderr}")
            (HERE / f"transitive-illegal-javac-{label}.txt").write_text(
                f"exit={deeper_javac.returncode}\n"
                + deeper_javac.stderr.replace(str(work), "<WORK>")
            )

    (HERE / "classfile-sha256.txt").write_text("\n".join(hashes) + "\n")
    manifest = HERE / "SHA256SUMS.txt"
    manifest.write_text("\n".join(
        f"{digest(path)}  {path.relative_to(HERE)}"
        for path in sorted(HERE.rglob("*")) if path.is_file() and path != manifest
    ) + "\n")


if __name__ == "__main__":
    main()
