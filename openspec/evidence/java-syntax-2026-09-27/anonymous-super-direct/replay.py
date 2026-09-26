from pathlib import Path
import hashlib
import os
import shlex
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / "tests/fixtures/proved-java-structure/anonymous-super-direct"
EVD = Path(__file__).resolve().parent
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
EXPECTED = "13:2\n"
CLASSES = ("AnonymousSuperDirect", "AnonymousSuperDirect$1", "Base")


def run(args, *, cwd=None, env=None):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)


def normalized(result, work):
    value = result.stdout + result.stderr
    for path, label in ((str(work), "<WORK>"), (str(ROOT), "<REPO>"), (str(JADX_ROOT), "<JADX>")):
        value = value.replace(path, label)
    return "\n".join(line.rstrip() for line in value.splitlines()) + f"\nexit={result.returncode}\n"


def save(name, result, work):
    (EVD / name).write_text(normalized(result, work))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


with tempfile.TemporaryDirectory(prefix="jarde-dt06a-anonymous-super-direct-") as temp_name:
    work = Path(temp_name)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    env["CARGO_BUILD_JOBS"] = "1"
    env["CARGO_INCREMENTAL"] = "0"

    # Rebuild the frozen Java 8 sources and require byte-for-byte class identity.
    original_classes = work / "original-classes"
    original_classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", str(original_classes), str(FIX / "AnonymousSuperDirect.java")])
    save("original-javac.log", compiled, work)
    if compiled.returncode:
        raise SystemExit(compiled.returncode)
    frozen = sorted(FIX.glob("*.class"))
    for frozen_class in frozen:
        built = original_classes / frozen_class.name
        if not built.exists() or sha(built) != sha(frozen_class):
            raise SystemExit(f"frozen class mismatch: {frozen_class.name}")

    actual_sums = "".join(f"{sha(path)}  {path.name}\n" for path in frozen)
    expected_sums = (FIX / "SHA256SUMS").read_text()
    if actual_sums != expected_sums:
        raise SystemExit("SHA256SUMS does not match frozen class files")

    original = run(["java", "-Xverify:all", "-cp", str(original_classes), "AnonymousSuperDirect"])
    save("original-run.log", original, work)
    if original.returncode or original.stdout != EXPECTED:
        raise SystemExit("original class did not match expected output")

    javap = run(["javap", "-classpath", str(FIX), "-p", "-c", "-s", *CLASSES])
    (EVD / "original-javap.txt").write_text(normalized(javap, work))
    if javap.returncode:
        raise SystemExit(javap.returncode)

    jar = work / "fixture.jar"
    packed = run(["jar", "--create", "--file", str(jar), "-C", str(original_classes), "."])
    save("jar.log", packed, work)
    if packed.returncode:
        raise SystemExit(packed.returncode)

    # JADX full-project source decode and Java 8 recompile.
    jadx_dir = work / "jadx-source"
    args = shlex.join(["-d", str(jadx_dir), str(jar)])
    jadx = run([str(JADX_ROOT / "gradlew"), "--no-daemon", ":jadx-cli:run", f"--args={args}"], cwd=JADX_ROOT)
    save("jadx.log", jadx, work)
    if jadx.returncode:
        raise SystemExit(jadx.returncode)
    jadx_files = sorted(jadx_dir.rglob("*.java"))
    snapshot = EVD / "jadx-source"
    if snapshot.exists():
        shutil.rmtree(snapshot)
    snapshot.mkdir()
    for source in jadx_files:
        rel = source.relative_to(jadx_dir)
        target = snapshot / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    (EVD / "jadx-files.txt").write_text("".join(f"{path.relative_to(jadx_dir)}\n" for path in jadx_files))
    jadx_classes = work / "jadx-classes"
    jadx_classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", str(jadx_classes), *map(str, jadx_files)])
    save("jadx-javac.log", compiled, work)
    if compiled.returncode:
        raise SystemExit(compiled.returncode)
    launched = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.AnonymousSuperDirect"])
    save("jadx-run.log", launched, work)
    if launched.returncode or launched.stdout != EXPECTED:
        raise SystemExit("JADX source output differed from original")

    # Fresh CLI build and complete physical-class source set.
    build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
    save("jarde-build.log", build, work)
    if build.returncode:
        raise SystemExit(build.returncode)
    cli = work / "cargo-target/debug/jarde-cli"
    jarde_snapshot = EVD / "jarde-source"
    if jarde_snapshot.exists():
        shutil.rmtree(jarde_snapshot)
    jarde_snapshot.mkdir()
    jarde_sources = []
    cli_results = []
    for class_name in CLASSES:
        result = run([str(cli), "class-source", "--input", str(jar), "--class", class_name, "--policy", "plain-jar", "--release", "8", "--format", "text"])
        save(f"jarde-{class_name}-cli.log", result, work)
        cli_results.append(f"{class_name}: exit={result.returncode}\n")
        if result.returncode:
            continue
        source_path = jarde_snapshot / f"{class_name}.java"
        source_path.write_text(result.stdout)
        jarde_sources.append(source_path)
    (EVD / "jarde-cli-status.txt").write_text("".join(cli_results))

    jarde_classes = work / "jarde-classes"
    jarde_classes.mkdir()
    if len(jarde_sources) == len(CLASSES):
        compiled = run(["javac", "--release", "8", "-g:none", "-d", str(jarde_classes), *map(str, jarde_sources)])
    else:
        compiled = subprocess.CompletedProcess([], 99, "", "one or more class-source requests failed\n")
    save("jarde-javac.log", compiled, work)
    if compiled.returncode:
        raise SystemExit(compiled.returncode)
    launched = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "AnonymousSuperDirect"])
    save("jarde-run.log", launched, work)
    if launched.returncode or launched.stdout != EXPECTED:
        raise SystemExit("Jarde source output differed from original")

    versions = []
    for command in (("java", "-version"), ("javac", "-version"), ("rustc", "--version"), ("cargo", "--version")):
        result = run(command)
        versions.append(f"$ {' '.join(command)}\n{result.stdout}{result.stderr}exit={result.returncode}\n")
    jadx_head = run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT)
    versions.append(f"JADX source HEAD: {jadx_head.stdout.strip()}\n")
    (EVD / "toolchain.log").write_text("".join(versions))
