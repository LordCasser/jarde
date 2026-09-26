#!/usr/bin/env python3
"""Rebuild the EM-04 package-info source comparison."""
from pathlib import Path
import hashlib
import os
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / "tests/fixtures/proved-java-structure/package-info-basic"
EVD = Path(__file__).resolve().parent
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
EXPECTED = "true\n"
JAVAC = ["javac", "-J-Duser.language=en", "-J-Duser.country=US"]


def run(args, *, cwd=None, env=None):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)


def save(name, value, work):
    if isinstance(value, subprocess.CompletedProcess):
        value = value.stdout + value.stderr + f"exit={value.returncode}\n"
    for path, label in ((str(work), "<WORK>"), (str(ROOT), "<REPO>"), (str(JADX_ROOT), "<JADX>")):
        value = value.replace(path, label)
    lines = (line.rstrip() for line in value.splitlines())
    (EVD / name).write_text("\n".join(line for line in lines if not line.lstrip().startswith("Last modified ")) + "\n")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


with tempfile.TemporaryDirectory(prefix="jarde-em04-package-info-") as temp:
    work = Path(temp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    env["CARGO_INCREMENTAL"] = "0"
    env["CARGO_BUILD_JOBS"] = "1"

    original_classes = work / "original-classes"
    original_classes.mkdir()
    compiled = run([
        *JAVAC, "--release", "8", "-g:none", "-d", str(original_classes),
        str(FIX / "p/package-info.java"), str(FIX / "p/Check.java"),
    ])
    save("original-javac.log", compiled, work)
    if compiled.returncode:
        raise SystemExit("original Java 8 source failed to compile")
    frozen = sorted((FIX / "v8/p").glob("*.class"))
    sums = "".join(f"{sha(path)}  v8/p/{path.name}\n" for path in frozen)
    if sums != (FIX / "SHA256SUMS").read_text():
        raise SystemExit("frozen class hashes differ")
    for path in frozen:
        rebuilt = original_classes / "p" / path.name
        if not rebuilt.exists() or sha(rebuilt) != sha(path):
            raise SystemExit(f"recompiled class differs: {path.name}")
    original = run(["java", "-Xverify:all", "-cp", str(original_classes), "p.Check"])
    save("original-run.log", original, work)
    if original.returncode or original.stdout != EXPECTED:
        raise SystemExit("original runtime differs")
    javap = run(["javap", "-v", "-p", "-classpath", str(FIX / "v8"), "p.package-info"])
    save("original-javap.txt", javap, work)
    if javap.returncode:
        raise SystemExit("javap failed")

    jar = work / "input.jar"
    packed = run(["jar", "--create", "--file", str(jar), "-C", str(original_classes), "."])
    save("jar.log", packed, work)
    if packed.returncode:
        raise SystemExit("jar packing failed")

    jadx_dir = work / "jadx-source"
    jadx = run([str(JADX), "-d", str(jadx_dir), str(jar)])
    save("jadx.log", jadx, work)
    if jadx.returncode:
        raise SystemExit("JADX failed")
    jadx_files = sorted(jadx_dir.rglob("*.java"))
    if len(jadx_files) != 2:
        raise SystemExit(f"JADX produced {len(jadx_files)} Java files")
    jadx_snapshot = EVD / "jadx-source"
    if jadx_snapshot.exists():
        shutil.rmtree(jadx_snapshot)
    for source in jadx_files:
        target = jadx_snapshot / source.relative_to(jadx_dir)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(source.read_text().rstrip() + "\n")
    package_source = next((path for path in jadx_files if path.name == "package-info.java"), None)
    if package_source is None or "@Deprecated\npackage p;" not in package_source.read_text():
        raise SystemExit("JADX package annotation/source shape changed")
    jadx_classes = work / "jadx-classes"
    jadx_classes.mkdir()
    compiled = run([*JAVAC, "--release", "8", "-g:none", "-d", str(jadx_classes), *map(str, jadx_files)])
    save("jadx-javac.log", compiled, work)
    if compiled.returncode:
        raise SystemExit("JADX full source failed Java 8 compilation")
    launched = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "p.Check"])
    save("jadx-run.log", launched, work)
    if launched.returncode or launched.stdout != EXPECTED:
        raise SystemExit("JADX runtime differs")

    built = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
    save("jarde-build.log", f"exit={built.returncode}\n", work)
    if built.returncode:
        raise SystemExit("Jarde build failed")
    cli = work / "cargo-target/debug/jarde-cli"
    jarde_snapshot = EVD / "jarde-source"
    if jarde_snapshot.exists():
        shutil.rmtree(jarde_snapshot)
    sources = []
    for class_name in ("p/package-info", "p/Check"):
        output = run([
            str(cli), "class-source", "--input", str(jar), "--class", class_name,
            "--policy", "plain-jar", "--release", "8", "--format", "text",
        ])
        save(f"jarde-{class_name.replace('/', '-')}-status.log", f"exit={output.returncode}\n", work)
        if output.returncode:
            raise SystemExit(f"Jarde could not present {class_name}")
        target = jarde_snapshot / (class_name + ".java")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(output.stdout)
        sources.append(target)
    physical_package = (jarde_snapshot / "p/package-info.java").read_text()
    if "interface package-info {" not in physical_package or "@java.lang.Deprecated" not in physical_package:
        raise SystemExit("Jarde baseline package-info shape changed")
    jarde_classes = work / "jarde-classes"
    jarde_classes.mkdir()
    compiled = run([*JAVAC, "--release", "8", "-g:none", "-d", str(jarde_classes), *map(str, sources)])
    save("jarde-javac.log", compiled, work)
    if compiled.returncode == 0:
        raise SystemExit("Jarde baseline unexpectedly compiled")
    save("jarde-run.log", "not run: complete Jarde source set failed javac --release 8\n", work)

    versions = []
    for command in (("java", "-version"), ("javac", "-version"), ("rustc", "--version"), ("cargo", "--version")):
        result = run(command)
        versions.append(f"$ {' '.join(command)}\n{result.stdout}{result.stderr}exit={result.returncode}\n")
    jadx_head = run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT)
    versions.append(f"JADX source HEAD: {jadx_head.stdout.strip()}\n")
    save("toolchain.log", "".join(versions), work)
