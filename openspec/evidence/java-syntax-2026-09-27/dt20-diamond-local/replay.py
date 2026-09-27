#!/usr/bin/env python3
"""Replay the JADX DT-20 diamond example with and without local generic debug data."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile


HERE = Path(__file__).resolve().parent
JADX_REPO = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_REPO / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode:
        raise SystemExit(f"{label}: exit {result.returncode}: {result.stderr[-3000:]}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_sources(classes: Path, debug: str, *sources: Path) -> None:
    classes.mkdir(parents=True)
    checked(run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                debug, "-Xlint:-options", "-d", classes, *sources), "javac")


def verified_run(classes: Path, label: str) -> str:
    result = run("java", "-Xverify:all", "-cp", classes, "dt20.Runner")
    checked(result, label)
    if result.stdout != "true\n":
        raise SystemExit(f"{label}: unexpected runtime output {result.stdout!r}")
    return result.stdout


if len(sys.argv) != 2 or not Path(sys.argv[1]).is_file() or not JADX.is_file():
    raise SystemExit("usage: replay.py /absolute/path/to/jarde-cli")
cli = Path(sys.argv[1])
head = run("git", "-C", JADX_REPO, "rev-parse", "HEAD")
status = run("git", "-C", JADX_REPO, "status", "--porcelain")
if head.returncode or head.stdout.strip() != JADX_HEAD or status.returncode or status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen commit")

output = HERE / "outputs"
output.mkdir(exist_ok=True)
results = {"jadx_head": JADX_HEAD, "javac_version": run("javac", "-version").stdout.strip(),
           "source_sha256": {name: sha(HERE / name) for name in ("Diamond.java", "Runner.java")},
           "variants": {}}

with tempfile.TemporaryDirectory(prefix="jarde-dt20-audit-") as temporary:
    work = Path(temporary)
    for variant, debug in (("debug", "-g"), ("no-debug", "-g:none")):
        folder = output / variant
        folder.mkdir(exist_ok=True)
        original = work / variant / "original"
        compile_sources(original, debug, HERE / "Diamond.java", HERE / "Runner.java")
        original_run = verified_run(original, f"{variant} original")
        javap = run("javap", "-v", "-c", "-p", "-classpath", original, "dt20.Diamond")
        checked(javap, f"{variant} javap")
        (folder / "original-javap.txt").write_text(javap.stdout.replace(str(original), "<CLASSES>"))
        if ("LocalVariableTypeTable" in javap.stdout) != (variant == "debug"):
            raise SystemExit(f"{variant}: unexpected local generic debug attributes")

        jar = work / variant / "input.jar"
        with zipfile.ZipFile(jar, "w") as archive:
            info = zipfile.ZipInfo("dt20/Diamond.class", date_time=(2000, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, (original / "dt20/Diamond.class").read_bytes())
        jadx_root = work / variant / "jadx"
        checked(run(JADX, "-d", jadx_root, jar), f"{variant} JADX")
        jadx_source = jadx_root / "sources/dt20/Diamond.java"
        if not jadx_source.is_file():
            raise SystemExit(f"{variant}: JADX did not produce complete source")
        jadx_text = jadx_source.read_text()
        (folder / "jadx-Diamond.java.txt").write_text(jadx_text)
        jadx_classes = work / variant / "jadx-classes"
        compile_sources(jadx_classes, debug, jadx_source, HERE / "Runner.java")
        jadx_run = verified_run(jadx_classes, f"{variant} JADX")

        jarde = run(cli, "class-source", "--input", jar, "--class", "dt20/Diamond",
                    "--policy", "plain-jar", "--release", "8", "--format", "text")
        checked(jarde, f"{variant} Jarde")
        jarde_text = jarde.stdout
        (folder / "jarde-Diamond.java.txt").write_text(jarde_text)
        jarde_source = work / variant / "jarde/dt20/Diamond.java"
        jarde_source.parent.mkdir(parents=True)
        jarde_source.write_text(jarde_text)
        jarde_classes = work / variant / "jarde-classes"
        compile_sources(jarde_classes, debug, jarde_source, HERE / "Runner.java")
        jarde_run = verified_run(jarde_classes, f"{variant} Jarde")
        if variant == "debug":
            if "Map<String, String> map = new HashMap<>();" not in jadx_text:
                raise SystemExit("JADX debug source lost its asserted diamond and generic local")
            if "new java.util.HashMap()" not in jarde_text or "HashMap<" in jarde_text:
                raise SystemExit("Jarde debug baseline constructor spelling changed")
        elif "new HashMap().get(\"test\")" not in jadx_text:
            raise SystemExit("JADX no-debug raw/cast control changed")
        results["variants"][variant] = {
            "original_class_sha256": sha(original / "dt20/Diamond.class"),
            "original_verified_run": original_run,
            "jadx_verified_run": jadx_run,
            "jarde_verified_run": jarde_run,
        }

(output / "results.json").write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n")
print("DT-20 baseline replay complete")
