#!/usr/bin/env python3
"""Read the accepted constructor inputs; independently replay complete call-adaptation cases."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[3]
FROZEN = REPO / "openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen"
JDKS = {
    "corretto8": (Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"), ["-source", "8", "-target", "8"]),
    "openjdk23": (Path("/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"), ["--release", "8"]),
}
DRIVER = '''public class CallProbe {
 public static void main(String[] args) throws Exception {
  Class<?> cls=Class.forName(args[0]); Object marker=new Object();
  Object receiver=cls.getConstructor(Object.class).newInstance(marker);
  System.out.println("behavior.field-is-marker="+(cls.getField("v").get(receiver)==marker));
 }
}'''

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cli", required=True)
    p.add_argument("--out", required=True)
    a = p.parse_args()
    cli = Path(a.cli).resolve()
    manifest = json.loads((REPO / "openspec/changes/recover-class-scope-constructor-parameters/results/acceptance-manifest.json").read_text())
    if sha(cli) != manifest["final_cli_sha256"]:
        raise SystemExit("CLI must match independently accepted main baseline")
    out = Path(a.out).resolve()
    out.mkdir(parents=True, exist_ok=False)
    commands, comparisons = [], []
    def run(label, args, area):
        r = subprocess.run(list(map(str, args)), capture_output=True)
        commands.append({"label": label, "argv": list(map(str, args)), "returncode": r.returncode})
        if r.stdout:
            (area / (label + ".stdout")).write_bytes(r.stdout)
        if r.stderr:
            (area / (label + ".stderr")).write_bytes(r.stderr)
        return r
    for leg, (jdk, release) in JDKS.items():
        for debug in ("debug", "nodebug"):
            for case in ("CallHold", "ExceptionHold"):
                input_dir = FROZEN / leg / debug / case
                area = out / leg / debug / case
                area.mkdir(parents=True)
                input_jar = input_dir / (case + ".jar")
                row = {"leg": leg, "debug": debug, "case": case,
                       "input_jar": str(input_jar), "input_sha256": sha(input_jar)}
                emitted = run("class-source", [cli, "class-source", "--input", input_jar, "--class", case,
                              "--policy", "plain-jar", "--release", "8", "--format", "text"], area)
                (area / (case + ".java")).write_bytes(emitted.stdout)
                source = emitted.stdout.decode()
                row["header"] = bool(source.strip()) and "presentation of `" + case + "`" in source
                row["cli_exit"] = emitted.returncode
                for flavor, source_file, qualified in (
                    ("original", input_dir / "source" / (case + ".java"), case),
                    ("jadx", input_dir / "jadx/sources/defpackage" / (case + ".java"), "defpackage." + case),
                    ("baseline", area / (case + ".java"), case),
                ):
                    row[flavor + "_source_sha256"] = sha(source_file)
                    with tempfile.TemporaryDirectory(prefix="jarde-call-adaptation-") as temp:
                        temp = Path(temp)
                        classes, empty = temp / "classes", temp / "empty"
                        classes.mkdir(); empty.mkdir()
                        driver = temp / "CallProbe.java"
                        driver.write_text(DRIVER)
                        compiled = run(flavor + "-javac", [jdk / "bin/javac", *release, "-classpath", empty,
                                       "-sourcepath", empty, "-d", classes, source_file, driver], area)
                        row[flavor + "_compile"] = compiled.returncode
                        if compiled.returncode == 0:
                            runtime = run(flavor + "-run", [jdk / "bin/java", "-Xverify:all", "-cp", classes,
                                          "CallProbe", qualified], area)
                            row[flavor + "_run"] = runtime.returncode
                            row[flavor + "_behavior"] = runtime.stdout.decode().strip()
                comparisons.append(row)
    (out / "manifest.json").write_text(json.dumps({"baseline": "564e22c1340d2d0901117012b986277fb5292313",
        "cli_sha256": sha(cli), "commands": commands, "comparisons": comparisons}, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(comparisons, ensure_ascii=False))

if __name__ == "__main__":
    main()
