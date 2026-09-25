from __future__ import annotations

import hashlib
import shutil
import subprocess
from pathlib import Path


OUT = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-boolean-root")
EXPECTED = "8cd1f767ba8cde9928bacedc29263d44be8e5e61b46d3b6f00456fac1c7c77cd"
CLASS = "ObjectArrayForeach"
RUNNER = "ObjectArrayForeachRunner"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture(prefix: str, args: list[str]) -> int:
    process = subprocess.run(args, capture_output=True, timeout=120)
    (OUT / f"{prefix}.stdout").write_bytes(process.stdout)
    (OUT / f"{prefix}.stderr").write_bytes(process.stderr)
    (OUT / f"{prefix}.status").write_text(f"{process.returncode}\n", encoding="utf-8")
    return process.returncode


def main() -> None:
    compiled = OUT / "compiled"
    if compiled.exists():
        shutil.rmtree(compiled)
    compiled.mkdir()
    before = digest(CLI)
    (OUT / "cli-sha256-before.txt").write_text(f"{before}  {CLI.name}\n", encoding="utf-8")
    if before != EXPECTED:
        raise SystemExit(f"frozen CLI hash mismatch: {before}")

    source = OUT / f"{CLASS}.java"
    runner = OUT / f"{RUNNER}.java"
    (compiled / "original").mkdir()
    (compiled / "jarde").mkdir()
    class_compile = capture("original-javac", ["javac", "--release", "8", "-g:none", "-d",
                                                str(compiled / "original"), str(source)])
    if class_compile != 0:
        raise SystemExit("original class did not compile")
    original_class = compiled / "original" / f"{CLASS}.class"
    shutil.copyfile(original_class, OUT / f"{CLASS}.class")
    (OUT / "original-class-sha256.txt").write_text(
        f"{digest(original_class)}  {CLASS}.class\n", encoding="utf-8")
    capture("original-javap", ["javap", "-v", "-c", "-p", str(original_class)])
    shutil.copyfile(OUT / "original-javap.stdout", OUT / "original-javap.txt")
    runner_status = capture("original-runner-javac", ["javac", "--release", "8", "-g:none",
                                                       "-cp", str(compiled / "original"), "-d",
                                                       str(compiled / "original"), str(runner)])
    if runner_status != 0:
        raise SystemExit("original runner did not compile")
    runtime_status = capture("original-runtime", ["java", "-Xverify:all", "-cp",
                                                    str(compiled / "original"), RUNNER])
    shutil.copyfile(OUT / "original-runtime.stdout", OUT / "original-runtime.txt")
    if runtime_status != 0:
        raise SystemExit("original runner did not execute")

    generated = subprocess.run([str(CLI), "class-source", "--input", str(original_class),
                                 "--class", CLASS, "--policy", "single-class", "--release", "8",
                                 "--format", "text"], capture_output=True, timeout=120)
    (OUT / "jarde.java.txt").write_bytes(generated.stdout)
    (OUT / "jarde-report.txt").write_bytes(generated.stderr)
    (OUT / "jarde-cli.status").write_text(f"{generated.returncode}\n", encoding="utf-8")
    (OUT / "jarde-source.java").write_bytes(generated.stdout)
    jarde_compile = capture("jarde-javac", ["javac", "--release", "8", "-g:none", "-d",
                                             str(compiled / "jarde"), str(OUT / "jarde-source.java")])
    if jarde_compile == 0:
        jarde_runner = capture("jarde-runner-javac", ["javac", "--release", "8", "-g:none",
                                                        "-cp", str(compiled / "jarde"), "-d",
                                                        str(compiled / "jarde"), str(runner)])
        if jarde_runner == 0:
            capture("jarde-runtime", ["java", "-Xverify:all", "-cp", str(compiled / "jarde"), RUNNER])
        else:
            (OUT / "jarde-runtime.stderr").write_text("not run: jarde runner javac failed\n", encoding="utf-8")
            (OUT / "jarde-runtime.status").write_text("skipped\n", encoding="utf-8")
    else:
        for prefix in ("jarde-runner-javac", "jarde-runtime"):
            (OUT / f"{prefix}.stdout").write_text("", encoding="utf-8")
            (OUT / f"{prefix}.stderr").write_text("not run: jarde class javac failed\n", encoding="utf-8")
            (OUT / f"{prefix}.status").write_text("skipped\n", encoding="utf-8")

    after = digest(CLI)
    (OUT / "cli-sha256-after.txt").write_text(f"{after}  {CLI.name}\n", encoding="utf-8")
    if before != after:
        raise SystemExit("frozen CLI changed during audit")


if __name__ == "__main__":
    main()
