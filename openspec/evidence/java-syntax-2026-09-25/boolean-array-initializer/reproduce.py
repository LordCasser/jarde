#!/usr/bin/env python3
"""Replay the Java 8 int[] initializer and descriptor/atype/store patch."""
import hashlib, json, os, shutil, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "reproduced"

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def run(name, command):
    result = subprocess.run(command, text=True, capture_output=True, check=False)
    (OUT / "logs").mkdir(parents=True, exist_ok=True)
    (OUT / "logs" / f"{name}.json").write_text(json.dumps({
        "command": command, "cwd": str(Path.cwd()), "exit_status": result.returncode,
        "stdout": result.stdout, "stderr": result.stderr
    }, indent=2) + "\n")
    return result
def need_ok(result, name):
    if result.returncode: raise SystemExit(f"{name} failed ({result.returncode}); see reproduced/logs")

def main():
    if len(sys.argv) != 3: raise SystemExit("usage: reproduce.py JARDE_CLI JADX")
    jarde, jadx = sys.argv[1:]
    classes = OUT / "classes"
    original, patched, runner = classes / "original", classes / "patched", classes / "runner"
    for path in (original, patched, runner): path.mkdir(parents=True, exist_ok=True)
    versions = {}
    for name, cmd in (("java-version", ["java", "-version"]), ("javac-version", ["javac", "-version"]),
                      ("jadx-version", [jadx, "--version"]), ("jarde-version", [jarde, "--version"])):
        result = run(name, cmd)
        versions[name] = {"command": cmd, "status": result.returncode, "stdout": result.stdout,
                          "stderr": result.stderr, "executable": shutil.which(cmd[0])}
    compile_input = run("javac-original", ["javac", "--release", "8", "-g:none", "-d", str(original), str(HERE / "fixtures/BoolInit.java")])
    need_ok(compile_input, "javac original")
    (patched / "BoolInit.class").write_bytes((original / "BoolInit.class").read_bytes())
    patch = run("patch-class", [sys.executable, str(HERE / "patch_class.py"), str(original / "BoolInit.class"),
                                str(patched / "BoolInit.class"), str(OUT / "patch.json")])
    need_ok(patch, "patch")
    for variant, root in (("original", original), ("patched", patched)):
        result = run(f"javap-{variant}", ["javap", "-classpath", str(root), "-c", "-s", "-v", "BoolInit"])
        need_ok(result, f"javap {variant}")
    compile_runner = run("javac-runner", ["javac", "--release", "8", "-g:none", "-d", str(runner), str(HERE / "fixtures/BoolInitRunner.java")])
    need_ok(compile_runner, "javac runner")
    jvm = {}
    for variant, root in (("original", original), ("patched", patched)):
        result = run(f"jvm-{variant}", ["java", "-Xverify:all", "-cp", os.pathsep.join((str(root), str(runner))), "BoolInitRunner"])
        need_ok(result, f"JVM {variant}")
        jvm[variant] = {"status": result.returncode, "stdout": result.stdout.strip(), "stderr": result.stderr.strip()}
    jadx_dir = OUT / "jadx"
    result = run("jadx", [jadx, "-d", str(jadx_dir), str(patched / "BoolInit.class")])
    source = jadx_dir / "sources" / "BoolInit.java"
    if not source.exists():
        candidates = list(jadx_dir.rglob("BoolInit.java"))
        if not candidates: raise SystemExit("JADX did not emit BoolInit.java")
        source = candidates[0]
    compile_jadx = run("javac-jadx", ["javac", "--release", "8", "-g:none", "-d", str(jadx_dir / "classes"), str(source)])
    jarde_result = run("jarde", [jarde, "class-source", "--input", str(patched / "BoolInit.class"), "--class", "BoolInit", "--policy", "single-class", "--evidence", "all"])
    summary = {
        "versions": versions,
        "class_hashes": {"original": digest(original / "BoolInit.class"), "patched": digest(patched / "BoolInit.class")},
        "cli_sha256": digest(Path(jarde)), "jvm": jvm,
        "jadx": {"exit_status": result.returncode, "source": str(source), "source_sha256": digest(source), "javac_status": compile_jadx.returncode},
        "jarde": {"exit_status": jarde_result.returncode, "stdout_sha256": hashlib.sha256(jarde_result.stdout.encode()).hexdigest(),
                  "stderr_sha256": hashlib.sha256(jarde_result.stderr.encode()).hexdigest(), "bytecode_markers": jarde_result.stdout.count("@bytecode")},
        "all_generated_outputs_under": str(OUT)
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
if __name__ == "__main__": main()
