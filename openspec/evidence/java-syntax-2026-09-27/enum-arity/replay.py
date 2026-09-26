from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile


ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / "tests/fixtures/proved-java-structure/enum-arity"
EVD = Path(__file__).resolve().parent
CLASS_NAMES = ("Empty", "One", "Four", "EnumArityRunner")
EXPECTED = "empty=0\none=ONLY:0/1\nfour=[NORTH, SOUTH, EAST, WEST]\n"


def run(args: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None):
    return subprocess.run(args, cwd=cwd, text=True, capture_output=True, env=env)


def normalized(text: str, work: Path) -> str:
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def save_log(path: Path, result: subprocess.CompletedProcess[str], work: Path) -> None:
    body = normalized(result.stdout + result.stderr, work)
    path.write_text(
        "\n".join(line.rstrip() for line in body.splitlines())
        + ("\n" if body else "")
        + f"exit={result.returncode}\n",
        encoding="utf-8",
    )


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def remove_saved_sources(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--expect-jarde",
        choices=("baseline", "fixed"),
        default="baseline",
        help="check the frozen failing baseline or require a complete Java 8 repair",
    )
    args = parser.parse_args()
    javac = os.environ.get("JAVAC", "javac")
    java = os.environ.get("JAVA", "java")
    jadx = os.environ.get("JADX") or shutil.which("jadx")
    if not jadx:
        raise SystemExit("set JADX to the executable from the frozen local JADX checkout")

    evidence_jadx = EVD / "jadx-source"
    evidence_jarde_root = EVD if args.expect_jarde == "baseline" else EVD / "fixed"
    evidence_jarde = evidence_jarde_root / "jarde-source"
    evidence_jarde_root.mkdir(parents=True, exist_ok=True)
    remove_saved_sources(evidence_jadx)
    remove_saved_sources(evidence_jarde)

    with tempfile.TemporaryDirectory(prefix="enum-arity-") as name:
        work = Path(name)
        original_classes = work / "original-classes"
        original_classes.mkdir()
        sources = [FIX / f"{class_name}.java" for class_name in CLASS_NAMES]
        source_compile = run(
            [
                javac,
                "-J-Duser.language=en",
                "-J-Duser.country=US",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(original_classes),
                *map(str, sources),
            ]
        )
        save_log(EVD / "original-javac.log", source_compile, work)
        if source_compile.returncode != 0:
            raise SystemExit("original Java 8 fixture did not compile")

        frozen = FIX / "v8"
        frozen_files = sorted(frozen.rglob("*.class"))
        generated_files = sorted(original_classes.rglob("*.class"))
        expected_names = sorted(f"probe/{name}.class" for name in CLASS_NAMES)
        if [path.relative_to(original_classes).as_posix() for path in generated_files] != expected_names:
            raise SystemExit("original compile produced an unexpected class set")
        if [path.relative_to(frozen).as_posix() for path in frozen_files] != expected_names:
            raise SystemExit("frozen fixture has an unexpected class set")
        for path in generated_files:
            checked = frozen / path.relative_to(original_classes)
            if sha256(path) != sha256(checked):
                raise SystemExit(f"frozen class SHA-256 mismatch: {checked.relative_to(FIX)}")
        lines = [f"{sha256(path)}  {path.relative_to(FIX).as_posix()}" for path in frozen_files]
        (FIX / "SHA256SUMS").write_text("\n".join(lines) + "\n", encoding="utf-8")

        original_run = run([java, "-Xverify:all", "-cp", str(original_classes), "probe.EnumArityRunner"])
        save_log(EVD / "original-run.log", original_run, work)
        if original_run.returncode != 0 or original_run.stdout != EXPECTED:
            raise SystemExit("original fixture did not produce the expected Java 8 runtime output")

        jar = work / "enum-arity.jar"
        with zipfile.ZipFile(jar, "w", compression=zipfile.ZIP_STORED) as archive:
            for path in generated_files:
                archive.write(path, path.relative_to(original_classes).as_posix())

        tool_versions = []
        for label, command in (("java", java), ("javac", javac), ("jadx", jadx)):
            result = run([command, "-version"] if label != "jadx" else [command, "--version"])
            tool_versions.append(f"{label}: {result.stdout.strip() or result.stderr.strip()}")
        (EVD / "tool-versions.txt").write_text("\n".join(tool_versions) + "\n", encoding="utf-8")

        jadx_dir = work / "jadx"
        jadx_result = run([jadx, "-d", str(jadx_dir), str(jar)])
        save_log(EVD / "jadx.log", jadx_result, work)
        if jadx_result.returncode != 0:
            raise SystemExit("JADX failed to decompile the frozen fixture")
        jadx_sources = sorted(jadx_dir.rglob("*.java"))
        if len(jadx_sources) != len(CLASS_NAMES):
            raise SystemExit(f"JADX produced {len(jadx_sources)} source files, expected {len(CLASS_NAMES)}")
        for path in jadx_sources:
            destination = evidence_jadx / path.relative_to(jadx_dir)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
        jadx_classes = work / "jadx-classes"
        jadx_classes.mkdir()
        jadx_compile = run(
            [
                javac,
                "-J-Duser.language=en",
                "-J-Duser.country=US",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(jadx_classes),
                *map(str, jadx_sources),
            ]
        )
        save_log(EVD / "jadx-javac.log", jadx_compile, work)
        if jadx_compile.returncode != 0:
            raise SystemExit("JADX complete source set did not compile for Java 8")
        jadx_run = run([java, "-Xverify:all", "-cp", str(jadx_classes), "probe.EnumArityRunner"])
        save_log(EVD / "jadx-run.log", jadx_run, work)
        if jadx_run.returncode != 0 or jadx_run.stdout != EXPECTED:
            raise SystemExit("JADX output did not preserve the runtime result")

        cargo_target = work / "cargo-target"
        cargo_env = os.environ.copy()
        cargo_env["CARGO_TARGET_DIR"] = str(cargo_target)
        build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=cargo_env)
        save_log(EVD / "jarde-build.log", build, work)
        if build.returncode != 0:
            raise SystemExit("Jarde CLI build failed; see jarde-build.log")
        cli = cargo_target / "debug/jarde-cli"
        jarde_sources = []
        cli_status = []
        for class_name in CLASS_NAMES:
            result = run(
                [
                    str(cli),
                    "class-source",
                    "--input",
                    str(jar),
                    "--class",
                    f"probe/{class_name}",
                    "--policy",
                    "plain-jar",
                    "--release",
                    "8",
                    "--format",
                    "text",
                ]
            )
            cli_status.append(f"probe/{class_name}: exit={result.returncode}")
            if result.returncode != 0:
                raise SystemExit(f"Jarde class-source failed for probe/{class_name}")
            destination = evidence_jarde / f"{class_name}.java"
            destination.write_text(result.stdout, encoding="utf-8")
            jarde_sources.append(destination)
        (evidence_jarde_root / "jarde-cli-status.txt").write_text(
            "\n".join(cli_status) + "\n", encoding="utf-8"
        )

        jarde_classes = work / "jarde-classes"
        jarde_classes.mkdir()
        jarde_compile = run(
            [
                javac,
                "-J-Duser.language=en",
                "-J-Duser.country=US",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(jarde_classes),
                *map(str, jarde_sources),
            ]
        )
        save_log(evidence_jarde_root / "jarde-javac.log", jarde_compile, work)
        if args.expect_jarde == "fixed":
            if jarde_compile.returncode != 0:
                raise SystemExit("fixed Jarde source set did not compile for Java 8")
            jarde_run = run([java, "-Xverify:all", "-cp", str(jarde_classes), "probe.EnumArityRunner"])
            save_log(evidence_jarde_root / "jarde-run.log", jarde_run, work)
            if jarde_run.returncode != 0 or jarde_run.stdout != EXPECTED:
                raise SystemExit("fixed Jarde output did not preserve the Java 8 runtime result")
            return
        if jarde_compile.returncode == 0:
            raise SystemExit("baseline Jarde source unexpectedly compiled; revise the measured gap report")
        (evidence_jarde_root / "jarde-run.log").write_text(
            "not run: the complete Jarde source set failed javac --release 8\n", encoding="utf-8"
        )
        diagnostic = (evidence_jarde_root / "jarde-javac.log").read_text(encoding="utf-8")
        if "enum constant expected here" not in diagnostic or "$VALUES" not in diagnostic:
            raise SystemExit("Jarde failed for a reason other than the expected missing enum constants")

        javap = run(["javap", "-p", "-c", "-classpath", str(original_classes), "probe.Empty"])
        save_log(EVD / "empty-javap.log", javap, work)
        if javap.returncode != 0:
            raise SystemExit("javap failed for the frozen empty enum")


if __name__ == "__main__":
    main()
