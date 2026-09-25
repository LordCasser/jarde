#!/usr/bin/env python3
"""Freeze Java 8 method/class type-variable shadowing against JADX and Jarde."""
from __future__ import annotations

import argparse
from hashlib import sha256
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZIP_DEFLATED, ZipFile


HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"
CLASSES = ("ShadowPlain", "ShadowBounded")


def command(*args: object, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run([str(item) for item in args], capture_output=True, text=True)
    if check and result.returncode:
        raise RuntimeError(
            f"exit={result.returncode}: {' '.join(map(str, args))}\n"
            f"stdout:\n{result.stdout}stderr:\n{result.stderr}"
        )
    return result


def save(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def archive(classes: Path, output: Path) -> None:
    with ZipFile(output, "w", ZIP_DEFLATED) as jar:
        for path in sorted(classes.rglob("*.class")):
            jar.write(path, path.relative_to(classes))


def compile_sources(output: Path, classpath: Path | None, *sources: Path) -> subprocess.CompletedProcess[str]:
    output.mkdir(parents=True, exist_ok=True)
    args: list[object] = ["javac", "--release", "8", "-Xlint:-options"]
    if classpath is not None:
        args.extend(("-cp", classpath))
    args.extend(("-d", output, *sources))
    return command(*args, check=False)


def digest(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        parser.error(f"missing Jarde CLI: {cli}")
    versions = [command("javac", "-version"), command("jadx", "--version")]
    save(HERE / "tool-versions.txt", "\n".join(
        (result.stderr or result.stdout).strip() for result in versions
    ) + "\n")

    generated: list[Path] = []
    class_digests: list[str] = []
    with TemporaryDirectory(prefix="jarde-shadowed-method-vars-") as temporary:
        work = Path(temporary)
        for label, debug in (("g", "-g"), ("g-none", "-g:none")):
            original = work / f"original-{label}"
            original.mkdir()
            command(
                "javac", "--release", "8", "-Xlint:-options", debug,
                "-d", original,
                *(FIXTURE / f"{name}.java" for name in (*CLASSES, "StrongCaller")),
            )
            for path in sorted(original.rglob("*.class")):
                class_digests.append(
                    f"{digest(path)}  compiled/{label}/{path.relative_to(original)}"
                )
            trace = command("java", "-Xverify:all", "-cp", original, "shadow.StrongCaller").stdout
            output = HERE / f"original-{label}-run.txt"
            save(output, trace)
            generated.append(output)
            javap = command(
                "javap", "-v", "-p", "-classpath", original,
                "shadow.ShadowPlain", "shadow.ShadowBounded",
            ).stdout.replace(str(original), "<CLASS_DIR>")
            output = HERE / f"javap-{label}.txt"
            save(output, javap)
            generated.append(output)

            jar = work / f"original-{label}.jar"
            archive(original, jar)
            for name in CLASSES:
                output = HERE / "jarde-source" / label / f"{name}.java"
                output.parent.mkdir(parents=True, exist_ok=True)
                result = command(
                    cli, "class-source", "--input", jar,
                    "--class", f"shadow/{name}", "--format", "text",
                    "--output", output,
                )
                save(HERE / "logs" / f"jarde-{label}-{name}.txt", result.stderr)
                generated.append(output)

            # The original caller remains a source-level strong-type check against the
            # two generated declarations, not against the unchanged original classes.
            result = compile_sources(
                work / f"jarde-rebuilt-{label}", None,
                *(HERE / "jarde-source" / label / f"{name}.java" for name in CLASSES),
                FIXTURE / "StrongCaller.java",
            )
            output = HERE / "logs" / f"jarde-caller-{label}-javac.txt"
            save(output, f"exit={result.returncode}\n{result.stdout}{result.stderr}")
            generated.append(output)

            if label == "g-none":
                jadx_dir = work / "jadx"
                jadx = command("jadx", "-d", jadx_dir, jar)
                output = HERE / "logs" / "jadx.txt"
                save(output, jadx.stdout + jadx.stderr)
                generated.append(output)
                for name in CLASSES:
                    output = HERE / "jadx-source" / f"{name}.java"
                    output.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(jadx_dir / "sources" / "shadow" / f"{name}.java", output)
                    generated.append(output)
                for name in CLASSES:
                    selected = HERE / "jadx-source" / f"{name}.java"
                    other = FIXTURE / f"{next(item for item in CLASSES if item != name)}.java"
                    result = compile_sources(
                        work / f"jadx-{name}-caller", None,
                        selected, other, FIXTURE / "StrongCaller.java",
                    )
                    output = HERE / "logs" / f"jadx-{name}-caller-javac.txt"
                    save(output, f"exit={result.returncode}\n{result.stdout}{result.stderr}")
                    generated.append(output)

    manifest = [
        *(f"{digest(path)}  {path.relative_to(HERE)}" for path in sorted(
            (*FIXTURE.glob("*.java"), HERE / "replay.py", HERE / "tool-versions.txt", *generated),
            key=str,
        )),
        *class_digests,
    ]
    save(HERE / "sha256.txt", "\n".join(manifest) + "\n")
    print("shadowing evidence refreshed")


if __name__ == "__main__":
    main()
