"""Replay complete recovered Main from diagnostic selected-header renders.

The added platform classes are input facts only; never on javac/runtime paths.
This accepts the requested Main's behavior, not recovery of those platform classes.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess

RESULTS = Path(__file__).resolve().parent
INPUT = RESULTS / "bigdecimal-selected-headers-v1"
OUT = RESULTS / "bigdecimal-selected-headers-execution-v1"
HOMES = {
    "javac8": Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"),
    "javac23": Path("/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    OUT.mkdir(exist_ok=False)
    manifest = json.loads((INPUT / "manifest.json").read_text())
    commands, cases = [], []
    environment = os.environ.copy()
    removed = {key: environment.pop(key, None) is not None for key in ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")}

    def run(label, argv, cwd):
        result = subprocess.run(argv, cwd=cwd, env=environment, capture_output=True, timeout=45)
        stdout, stderr = OUT / f"{label}.stdout", OUT / f"{label}.stderr"
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        commands.append({"label": label, "argv": [str(a) for a in argv], "cwd": str(cwd), "exit": result.returncode,
                         "stdout": stdout.name, "stdout_sha256": sha(result.stdout), "stderr": stderr.name, "stderr_sha256": sha(result.stderr)})
        return result

    for record in manifest["records"]:
        label = f"{record['leg']}-{record['variant']}"
        case = OUT / label
        case.mkdir()
        classes, empty = case / "classes", case / "empty-classpath-sourcepath"
        classes.mkdir()
        empty.mkdir()
        raw = INPUT / record["stdout"]
        assert sha(raw.read_bytes()) == record["stdout_sha256"]
        report = json.loads(raw.read_text())
        source = case / "Main.java"
        source.write_text(report["text"])
        jdk = HOMES[record["leg"]]
        compile_argv = [str(jdk / "bin/javac"), "-source", "8", "-target", "8", "-g:none", "-classpath", str(empty), "-sourcepath", str(empty), "-d", str(classes), str(source)]
        compiled = run(label + "-compile", compile_argv, case)
        assert compiled.returncode == 0
        executed = run(label + "-run", [str(jdk / "bin/java"), "-Xverify:all", "-cp", str(classes), "Main"], case)
        success = executed.returncode == 0 and executed.stdout == b"1:1.25\n" and executed.stderr == b""
        assert success == (record["variant"] != "original-only")
        cases.append({"leg": record["leg"], "variant": record["variant"], "source_sha256": sha(source.read_bytes()),
                      "generated_main_sha256": sha((classes / "Main.class").read_bytes()), "semantic_match": success,
                      "expected_stdout_sha256": sha(b"1:1.25\n"), "java_sha256": sha((jdk / "bin/java").read_bytes()),
                      "javac_sha256": sha((jdk / "bin/javac").read_bytes())})
    result = {"purpose": "complete requested Main replay only, platform headers are input facts and not compilation/runtime classes",
              "input_manifest_sha256": sha((INPUT / "manifest.json").read_bytes()), "environment_removed": removed,
              "cases": cases, "commands": commands, "semantic_passes": sum(c["semantic_match"] for c in cases), "total_cases": len(cases)}
    (OUT / "manifest.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"passes": result["semantic_passes"], "cases": result["total_cases"], "expected_original_only_failures": 2}))


if __name__ == "__main__":
    main()
