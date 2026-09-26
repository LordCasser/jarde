from pathlib import Path
import argparse
import hashlib
import os
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / "tests/fixtures/proved-java-structure/anonymous-inner-this"
EVD = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument("--expect-jarde", choices=("baseline", "fixed"), default="baseline")
MODE = parser.parse_args().expect_jarde
BASE_EVD = EVD
EVD = BASE_EVD / "fixed" if MODE == "fixed" else BASE_EVD
if MODE == "fixed" and EVD.exists():
    shutil.rmtree(EVD)
EVD.mkdir(parents=True, exist_ok=True)
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
EXPECTED = "true\n38\n"
JAVAC = ["javac", "-J-Duser.language=en", "-J-Duser.country=US"]


def run(args, *, cwd=None, env=None):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)


def normalized(text, work):
    return text.replace(str(work), "<WORK>").replace(str(ROOT), "<REPO>").replace(str(JADX_ROOT), "<JADX>")


def save(name, text):
    (EVD / name).write_text("\n".join(line.rstrip() for line in text.splitlines()) + "\n")


def result_text(result, work):
    return normalized(result.stdout + result.stderr, work) + f"exit={result.returncode}\n"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


with tempfile.TemporaryDirectory(prefix="jarde-dt04-anonymous-inner-this-") as temp_name:
    work = Path(temp_name)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    env["CARGO_BUILD_JOBS"] = "1"
    env["CARGO_INCREMENTAL"] = "0"

    # Rebuild and verify the frozen source oracle against every frozen physical class.
    original_classes = work / "original-classes"
    original_classes.mkdir()
    compiled = run([*JAVAC, "--release", "8", "-g:none", "-d", str(original_classes), str(FIX / "Inner.java")])
    save("original-javac.log", result_text(compiled, work))
    if compiled.returncode:
        raise SystemExit(compiled.returncode)
    frozen = sorted(FIX.glob("*.class"))
    sums = "".join(f"{sha(path)}  {path.name}\n" for path in frozen)
    if sums != (FIX / "SHA256SUMS").read_text():
        raise SystemExit("frozen class SHA256SUMS mismatch")
    for frozen_class in frozen:
        rebuilt = original_classes / frozen_class.name
        if not rebuilt.exists() or sha(rebuilt) != sha(frozen_class):
            raise SystemExit(f"source rebuild differs from frozen class: {frozen_class.name}")

    original_run = run(["java", "-Xverify:all", "-cp", str(original_classes), "Inner"])
    save("original-run.log", result_text(original_run, work))
    if original_run.returncode or original_run.stdout != EXPECTED:
        raise SystemExit("original runtime output differs from source oracle")
    javap = run(["javap", "-classpath", str(FIX), "-p", "-c", "-s", "Inner", "Inner$1"])
    save("original-javap.txt", result_text(javap, work))
    if javap.returncode:
        raise SystemExit(javap.returncode)

    jar = work / "input.jar"
    packed = run(["jar", "--create", "--file", str(jar), "-C", str(original_classes), "."])
    save("jar.log", result_text(packed, work))
    if packed.returncode:
        raise SystemExit(packed.returncode)

    # JADX emits a full source set; compile and run the entire set.
    jadx_dir = work / "jadx-source"
    jadx = run([str(JADX), "-d", str(jadx_dir), str(jar)])
    save("jadx.log", result_text(jadx, work))
    if jadx.returncode:
        raise SystemExit(jadx.returncode)
    jadx_files = sorted(jadx_dir.rglob("*.java"))
    jadx_snapshot = EVD / "jadx-source"
    if jadx_snapshot.exists():
        shutil.rmtree(jadx_snapshot)
    for source in jadx_files:
        destination = jadx_snapshot / source.relative_to(jadx_dir)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    save("jadx-files.txt", "".join(f"{path.relative_to(jadx_dir)}\n" for path in jadx_files))
    jadx_classes = work / "jadx-classes"
    jadx_classes.mkdir()
    compiled = run([*JAVAC, "--release", "8", "-g:none", "-d", str(jadx_classes), *map(str, jadx_files)])
    save("jadx-javac.log", result_text(compiled, work))
    if compiled.returncode:
        raise SystemExit(compiled.returncode)
    launched = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.Inner"])
    save("jadx-run.log", result_text(launched, work))
    if launched.returncode or launched.stdout != EXPECTED:
        raise SystemExit("JADX runtime output differs from original")

    # Build Jarde with an automatically removed, isolated Cargo target.
    build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
    if build.returncode:
        save("jarde-build.log", result_text(build, work))
    else:
        save("jarde-build.log", "$ cargo build -p jarde-cli --locked\nexit=0\n")
    if build.returncode:
        raise SystemExit(build.returncode)
    cli = work / "cargo-target/debug/jarde-cli"
    jarde_names = [path.stem for path in frozen]
    jarde_snapshot = EVD / "jarde-source"
    if jarde_snapshot.exists():
        shutil.rmtree(jarde_snapshot)
    jarde_snapshot.mkdir()
    jarde_sources = []
    status = []
    facts = []
    for class_name in jarde_names:
        output = run([str(cli), "class-source", "--input", str(jar), "--class", class_name, "--policy", "plain-jar", "--release", "8", "--format", "text"])
        status.append(f"{class_name}: exit={output.returncode}\n")
        selected = [line for line in output.stderr.splitlines() if line.startswith(("member_family.state =", "member_family.reason =", "member_family.child ="))]
        facts.extend(f"{class_name}: {line}\n" for line in selected)
        if output.returncode:
            continue
        source = jarde_snapshot / f"{class_name}.java"
        source.write_text(output.stdout)
        jarde_sources.append(source)
    save("jarde-cli-status.txt", "".join(status))
    save("jarde-family-facts.txt", "".join(facts) or "no member-family facts reported")
    if len(jarde_sources) != len(jarde_names):
        compiled = subprocess.CompletedProcess([], 99, "", "one or more class-source requests failed\n")
    else:
        anonymous_source = (jarde_snapshot / "Inner$1.java").read_text()
        jarde_classes = work / "jarde-classes"
        jarde_classes.mkdir()
        if MODE == "baseline":
            if "this.this$0 = arg1;" not in anonymous_source or "Inner.this" in anonymous_source:
                raise SystemExit("baseline changed: anonymous capture source shape differs")
            family_facts = (EVD / "jarde-family-facts.txt").read_text()
            if 'Inner$1: member_family.reason = "selected root has EnclosingMethod identity"' not in family_facts:
                raise SystemExit("baseline changed: anonymous EnclosingMethod refusal is absent")
            compiled = run([*JAVAC, "--release", "8", "-g:none", "-d", str(jarde_classes), *map(str, jarde_sources)])
        else:
            root_source = (jarde_snapshot / "Inner.java").read_text()
            if "Inner.this" not in root_source or "this$0" in root_source or "Inner$1" in root_source:
                raise SystemExit("fixed root source does not contain the complete lexical-this projection")
            physical_child = anonymous_source
            if "this$0" not in physical_child:
                raise SystemExit("fixed mode unexpectedly changed the independent physical child report")
            # The root is the complete source unit: javac itself recreates its anonymous class.
            # The separate physical $1 report is inspected above but is not a second source unit.
            compiled = run([*JAVAC, "--release", "8", "-g:none", "-d", str(jarde_classes), str(jarde_snapshot / "Inner.java")])
    save("jarde-javac.log", result_text(compiled, work))
    if compiled.returncode == 0:
        launched = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "Inner"])
        save("jarde-run.log", result_text(launched, work))
        if launched.returncode or launched.stdout != EXPECTED:
            raise SystemExit("Jarde source ran but differed from expected output")
        if MODE == "baseline":
            raise SystemExit("baseline changed: Jarde unexpectedly compiled")
    elif MODE == "fixed":
        raise SystemExit("fixed Jarde root source did not compile under Java 8")
    if MODE == "fixed":
        if compiled.returncode != 0:
            raise SystemExit("fixed Jarde root source failed Java 8 compilation")
    elif compiled.returncode == 0:
        raise SystemExit("baseline changed: Jarde unexpectedly compiled")
    if MODE == "baseline":
        if compiled.returncode != 1 or "this.this$0 = arg1;" not in compiled.stderr or "flexible constructors is a preview feature" not in compiled.stderr:
            raise SystemExit("Jarde source failed for an unexpected Java 8 diagnostic")
        save("jarde-run.log", "not run: complete Jarde source set failed javac --release 8 -g:none\n")

    versions = []
    for command in (("java", "-version"), ("javac", "-version"), ("rustc", "--version"), ("cargo", "--version")):
        result = run(command)
        versions.append(f"$ {' '.join(command)}\n{result.stdout}{result.stderr}exit={result.returncode}\n")
    head = run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT)
    versions.append(f"JADX source HEAD: {head.stdout.strip()}\n")
    save("toolchain.log", "".join(versions))
