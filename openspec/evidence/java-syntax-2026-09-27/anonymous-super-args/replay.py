from pathlib import Path
import hashlib
import os
import shlex
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / "tests/fixtures/proved-java-structure/anonymous-super-args"
EVD = Path(__file__).resolve().parent
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
EXPECTED = "arg:capture|arg:super-label|arg:super-value|base:explicit:17\nexplicit:17:captured\n"


def run(args, *, cwd=None, env=None):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)


def record(name, result):
    body = result.stdout + result.stderr + f"exit={result.returncode}\n"
    (EVD / name).write_text("\n".join(line.rstrip() for line in body.splitlines()) + "\n")


with tempfile.TemporaryDirectory(prefix="jarde-dt06-anonymous-super-args-") as temp_name:
    work = Path(temp_name)
    target = work / "cargo-target"
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    env["CARGO_BUILD_JOBS"] = "1"
    env["CARGO_INCREMENTAL"] = "0"

    build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
    record("jarde-build.log", build)
    if build.returncode:
        raise SystemExit(build.returncode)
    cli = target / "debug/jarde-cli"

    frozen_classes = sorted(FIX.glob("*.class"))
    if not frozen_classes:
        raise SystemExit("no frozen class files")
    hashes = "".join(
        f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n"
        for path in frozen_classes
    )
    (EVD / "class-sha256.txt").write_text(hashes)
    javap = run(["javap", "-classpath", str(FIX), "-p", "-c", "-s", "AnonymousSuperArgs", "Base", "AnonymousSuperArgs$1"])
    record("javap.log", javap)
    if javap.returncode:
        raise SystemExit(javap.returncode)

    original = run(["java", "-Xverify:all", "-cp", str(FIX), "AnonymousSuperArgs"])
    record("original-run.log", original)
    if original.returncode or original.stdout != EXPECTED:
        raise SystemExit("frozen class output differs from the source oracle")

    class_dir = work / "classes"
    class_dir.mkdir()
    for path in frozen_classes:
        shutil.copyfile(path, class_dir / path.name)
    jar = work / "fixture.jar"
    result = run(["jar", "--create", "--file", str(jar), "-C", str(class_dir), "."])
    record("jar.log", result)
    if result.returncode:
        raise SystemExit(result.returncode)

    jadx_dir = work / "jadx-source"
    jadx_args = shlex.join(["-d", str(jadx_dir), str(jar)])
    jadx = run([str(JADX_ROOT / "gradlew"), "--no-daemon", ":jadx-cli:run", f"--args={jadx_args}"], cwd=JADX_ROOT)
    record("jadx.log", jadx)
    if jadx.returncode:
        raise SystemExit(jadx.returncode)
    jadx_files = sorted(jadx_dir.rglob("*.java"))
    jadx_snapshot = EVD / "jadx-source"
    if jadx_snapshot.exists():
        shutil.rmtree(jadx_snapshot)
    jadx_snapshot.mkdir()
    for source in jadx_files:
        shutil.copyfile(source, jadx_snapshot / source.name)
    (EVD / "jadx-files.txt").write_text("".join(f"{p.name}\n" for p in jadx_files))
    jadx_classes = work / "jadx-classes"
    jadx_classes.mkdir()
    compiled = run(["javac", "--release", "8", "-d", str(jadx_classes), *map(str, jadx_files)])
    record("jadx-javac.log", compiled)
    jadx_compile_code = compiled.returncode
    jadx_run_code = "not run"
    if compiled.returncode == 0:
        launched = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.AnonymousSuperArgs"])
        record("jadx-run.log", launched)
        jadx_run_code = launched.returncode

    jarde_snapshot = EVD / "jarde-source"
    if jarde_snapshot.exists():
        shutil.rmtree(jarde_snapshot)
    jarde_snapshot.mkdir()
    jarde_sources = []
    for class_name in ("AnonymousSuperArgs", "Base", "AnonymousSuperArgs$1"):
        source = run([
            str(cli), "class-source", "--input", str(jar), "--class", class_name,
            "--policy", "plain-jar", "--release", "8", "--format", "text",
        ])
        (EVD / f"jarde-{class_name}.log").write_text(
            source.stdout + source.stderr + f"exit={source.returncode}\n"
        )
        if source.returncode:
            continue
        saved = jarde_snapshot / f"{class_name}.java"
        saved.write_text(source.stdout)
        jarde_sources.append(saved)

    jarde_classes = work / "jarde-classes"
    jarde_classes.mkdir()
    if len(jarde_sources) == 3:
        compiled = run(["javac", "--release", "8", "-d", str(jarde_classes), *map(str, jarde_sources)])
    else:
        compiled = subprocess.CompletedProcess([], 99, "", "one or more class-source requests failed\n")
    record("jarde-javac.log", compiled)
    jarde_compile_code = compiled.returncode
    jarde_run_code = "not run"
    if compiled.returncode == 0:
        launched = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "AnonymousSuperArgs"])
        record("jarde-run.log", launched)
        jarde_run_code = launched.returncode

    tool_versions = []
    for command in (["java", "-version"], ["javac", "-version"], ["rustc", "--version"], ["cargo", "--version"]):
        version = run(command)
        tool_versions.append(f"$ {' '.join(command)}\n{version.stdout}{version.stderr}exit={version.returncode}\n")
    jadx_commit = run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT)
    tool_versions.append(f"JADX source HEAD: {jadx_commit.stdout.strip()}\n")
    (EVD / "toolchain.log").write_text("\n".join(tool_versions))

    # report.md is the reviewed baseline analysis; replay refreshes the raw evidence only.
