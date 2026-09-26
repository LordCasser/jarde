from pathlib import Path
import hashlib
import os
import shlex
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
    if name.startswith("jarde-fixed-"):
        target_name = name
    elif name.startswith("jarde-"):
        target_name = "jarde-fixed-" + name.removeprefix("jarde-")
    else:
        target_name = name.removesuffix(".log") + "-fixed.log"
    (EVD / target_name).write_text(normalized(result, work))


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
    (EVD / "original-javap-fixed.txt").write_text(normalized(javap, work))
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
    (EVD / "jadx-fixed-files.txt").write_text(
        "".join(f"{path.relative_to(jadx_dir)}\n" for path in jadx_files)
    )
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
    jarde_snapshot = EVD / "jarde-fixed-source"
    jarde_snapshot.mkdir(exist_ok=True)
    root_args = [str(cli), "class-source", "--input", str(jar), "--class", "AnonymousSuperDirect", "--policy", "plain-jar", "--release", "8", "--format", "text"]
    root_all = run([*root_args, "--evidence", "all"])
    save("jarde-fixed-all-cli.log", root_all, work)
    root_essential = run([*root_args, "--evidence", "essential"])
    save("jarde-fixed-essential-cli.log", root_essential, work)
    if root_all.returncode or root_essential.returncode or root_all.stdout != root_essential.stdout:
        raise SystemExit("anonymous superclass projection depended on optional evidence selection")
    if "new Base(next(), next()) {" not in root_all.stdout:
        raise SystemExit("Jarde did not project the proved ordered superclass arguments")
    root_source = jarde_snapshot / "AnonymousSuperDirect.java"
    root_source.write_text(root_all.stdout)

    # The fixed source unit compiles the projected root with the real Base only. The physical child
    # is queried separately because javac emits an anonymous `$1` for the fixed root itself.
    base = run([str(cli), "class-source", "--input", str(jar), "--class", "Base", "--policy", "plain-jar", "--release", "8", "--format", "text", "--evidence", "all"])
    save("jarde-Base-cli.log", base, work)
    if base.returncode:
        raise SystemExit("Jarde could not recover the physical Base declaration")
    base_source = jarde_snapshot / "Base.java"
    base_source.write_text(base.stdout)

    child_name = "AnonymousSuperDirect$1"
    child = run([str(cli), "class-source", "--input", str(jar), "--class", child_name, "--policy", "plain-jar", "--release", "8", "--format", "text", "--evidence", "all"])
    save(f"jarde-{child_name}-cli.log", child, work)
    if child.returncode or "class AnonymousSuperDirect$1" not in child.stdout:
        raise SystemExit("the independent physical anonymous child query disappeared")
    child_snapshot = EVD / "jarde-fixed-physical-child"
    child_snapshot.mkdir(exist_ok=True)
    (child_snapshot / f"{child_name}.java").write_text(child.stdout)
    (EVD / "jarde-fixed-cli-status.txt").write_text(
        f"AnonymousSuperDirect(all/essential): exit={root_all.returncode}/{root_essential.returncode}\n"
        f"Base: exit={base.returncode}\n{child_name}: exit={child.returncode}\n"
    )

    jarde_classes = work / "jarde-classes"
    jarde_classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", str(jarde_classes), str(root_source), str(base_source)])
    save("jarde-fixed-javac.log", compiled, work)
    if compiled.returncode:
        raise SystemExit(compiled.returncode)
    launched = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "AnonymousSuperDirect"])
    save("jarde-fixed-run.log", launched, work)
    if launched.returncode or launched.stdout != EXPECTED:
        raise SystemExit("Jarde source output differed from original")

    versions = []
    for command in (("java", "-version"), ("javac", "-version"), ("rustc", "--version"), ("cargo", "--version")):
        result = run(command)
        versions.append(f"$ {' '.join(command)}\n{result.stdout}{result.stderr}exit={result.returncode}\n")
    jadx_head = run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT)
    versions.append(f"JADX source HEAD: {jadx_head.stdout.strip()}\n")
    (EVD / "toolchain-fixed.log").write_text("".join(versions))
