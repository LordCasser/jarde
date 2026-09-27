#!/usr/bin/env python3
"""Replay the fixed DT-29 whole family on one Jarde CLI, without touching old snapshots."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo


HERE = Path(__file__).resolve().parent
INPUT = HERE.parents[1] / "dt29-reference-cast-audit/combined/inputs"
JADX_CHECKOUT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX_CLI = JADX_CHECKOUT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/types/TestFieldCast.java": "a57ca7a3f4571dca914f7f39fb8dbdee929da51720f59bf37bebdece1074fc5f",
    "jadx-core/src/test/java/jadx/tests/integration/types/TestInterfacesCast.java": "446576f103a4026919e7cd0b683ecb0fff4de6bbf83006cbcde7524bbe4370ee",
    "jadx-core/src/main/java/jadx/core/dex/visitors/ModVisitor.java": "2d208158695097fd5ee4bfb833fa865b4b75e0f784fc9b8fde1ffbb3e49cf69f",
    "jadx-core/src/main/java/jadx/core/dex/visitors/ShadowFieldVisitor.java": "50811e321fd841b298b96f3f12bad7d481a0207de5b21c2386d78f24e9d713d5",
    "jadx-core/src/main/java/jadx/core/dex/visitors/SimplifyVisitor.java": "71aede9230b70a162d65b782c9e4c09e1a6d03925927787a6de6603a838798e2",
}
EXPECTED_CLASSES = [
    "Both", "CloseOnly", "FieldCast", "FieldCast$1", "FieldCast$A",
    "FieldCast$B", "FieldCast$C", "FieldCast$D", "InterfaceCast",
]
EXPECTED = "runnable:1111:0000:1111:ClassCastException\n"
EXPECTED_REFLECTION = "1:T:dt29.FieldCast$B:T:dt29.FieldCast$B\n"
REFLECTION_SOURCE = """package dt29;
import java.lang.reflect.Method;
import java.lang.reflect.TypeVariable;
public class GenericRunner {
    public static void main(String[] args) throws Exception {
        Class<?> owner = Class.forName("dt29.FieldCast$D");
        Method method = owner.getDeclaredMethod("set", Class.forName("dt29.FieldCast$B"), boolean.class);
        TypeVariable<Method>[] vars = method.getTypeParameters();
        System.out.println(vars.length + ":" + vars[0].getName() + ":"
            + vars[0].getBounds()[0].getTypeName() + ":"
            + method.getGenericParameterTypes()[0].getTypeName() + ":"
            + method.getParameterTypes()[0].getTypeName());
    }
}
"""


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, log):
    result = subprocess.run([str(arg) for arg in args], text=True, capture_output=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    return result


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def compile_and_run(label, sources, runner, out, work):
    classes = work / f"{label}-classes"
    classes.mkdir()
    reflection = work / f"{label}-reflection" / "GenericRunner.java"
    reflection.parent.mkdir()
    reflection.write_text(REFLECTION_SOURCE)
    result = run(["javac", "--release", "8", "-g:none", "-Xlint:-options",
                  "-d", classes, *sources, runner, reflection], out / label / "javac.log")
    answer = {"compile_exit": result.returncode,
              "compile_stderr": result.stderr.replace(str(work), "<TMP>")}
    if result.returncode == 0:
        executed = run(["java", "-Xverify:all", "-cp", classes, "dt29.Runner"],
                       out / label / "runtime.log")
        answer.update(run_exit=executed.returncode, stdout=executed.stdout,
                      run_stderr=executed.stderr)
        reflected = run(["java", "-Xverify:all", "-cp", classes, "dt29.GenericRunner"],
                        out / label / "reflection.log")
        answer.update(reflection_exit=reflected.returncode,
                      reflection_stdout=reflected.stdout,
                      reflection_stderr=reflected.stderr)
    return answer, classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--require-jarde", action="store_true")
    args = parser.parse_args()
    out = args.out.resolve()
    require(not out.exists() or not any(out.iterdir()), "--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    require(args.jarde.is_file() and JADX_CLI.is_file(), "one CLI is missing")
    rev = run(["git", "-C", JADX_CHECKOUT, "rev-parse", "HEAD"], out / "jadx-revision.log")
    require(rev.returncode == 0 and rev.stdout.strip() == JADX_REV, "JADX checkout moved")
    for path, pinned in PINS.items():
        require(digest(JADX_CHECKOUT / path) == pinned, f"JADX pin changed: {path}")
    sources = sorted(path for path in INPUT.glob("*.java") if path.name != "Runner.java")
    runner = INPUT / "Runner.java"
    frozen = json.loads((INPUT.parent / "outputs/results.json").read_text())
    for source in [*sources, runner]:
        destination = out / "source" / "original" / source.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    with tempfile.TemporaryDirectory(prefix="jarde-dt29-family-") as temporary:
        work = Path(temporary)
        original, original_classes = compile_and_run("original", sources, runner, out, work)
        require(original.get("stdout") == EXPECTED and original.get("run_exit") == 0
                and original.get("reflection_stdout") == EXPECTED_REFLECTION
                and original.get("reflection_exit") == 0,
                "original fixture changed")
        class_files = sorted(path for path in (original_classes / "dt29").glob("*.class")
                             if path.stem not in ("Runner", "GenericRunner"))
        class_names = [path.stem for path in class_files]
        require(sorted(class_names) == sorted(EXPECTED_CLASSES),
                f"physical class set changed: {class_names}")
        class_hashes = {"dt29/" + path.stem: digest(path) for path in class_files}
        require(class_hashes == frozen["input_class_sha256"],
                "fixed physical class hashes changed")
        jar = work / "input.jar"
        with ZipFile(jar, "w", ZIP_DEFLATED) as archive:
            for path in class_files:
                entry = ZipInfo("dt29/" + path.name, date_time=(2000, 1, 1, 0, 0, 0))
                entry.compress_type = ZIP_DEFLATED
                archive.writestr(entry, path.read_bytes())
        jadx_root = work / "jadx"
        jadx_run = run([JADX_CLI, "-d", jadx_root, jar], out / "jadx.log")
        require(jadx_run.returncode == 0, "fixed JADX failed")
        jadx_sources = sorted(jadx_root.rglob("*.java"))
        for source in jadx_sources:
            destination = out / "source" / "jadx" / source.relative_to(jadx_root)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        jadx, _ = compile_and_run("jadx", jadx_sources, runner, out, work)
        require(jadx.get("stdout") == EXPECTED and jadx.get("run_exit") == 0
                and jadx.get("reflection_stdout") == EXPECTED_REFLECTION
                and jadx.get("reflection_exit") == 0,
                "fixed JADX whole family changed")
        jarde_source = work / "jarde-source" / "dt29"
        jarde_source.mkdir(parents=True)
        methods = {}
        for class_name in class_names:
            physical = "dt29/" + class_name
            cmd = [args.jarde, "class-source", "--input", jar, "--class", physical,
                   "--policy", "plain-jar", "--release", "8"]
            source = run([*cmd, "--format", "text"], out / "jarde-logs" / f"{class_name}.log")
            require(source.returncode == 0, f"Jarde text failed for {physical}")
            (jarde_source / f"{class_name}.java").write_text(source.stdout)
            archived = out / "source" / "jarde" / f"{class_name}.java"
            archived.parent.mkdir(parents=True, exist_ok=True)
            archived.write_text(source.stdout)
            report_run = run([*cmd, "--format", "json"],
                             out / "jarde-logs" / f"{class_name}-json.log")
            require(report_run.returncode == 0, f"Jarde report failed for {physical}")
            report = json.loads(report_run.stdout)
            methods[physical] = [
                {"name": bytes(member["item"]["identity"]["name"]).decode(errors="replace"),
                 "descriptor": bytes(member["item"]["identity"]["descriptor"]).decode(errors="replace"),
                 "quality": member.get("outcome", {}).get("report", {}).get("quality"),
                 "quoted_bcis": [int(value) for value in re.findall(
                     r"@bytecode\s+(\d+)", member.get("outcome", {}).get("report", {}).get("text", ""))]}
                for member in report.get("methods", [])
            ]
        jarde_sources = sorted(jarde_source.glob("*.java"))
        for class_name in ("FieldCast$C", "FieldCast$D", "FieldCast"):
            disassembly = run(["javap", "-classpath", jar, "-v", "-c", "-p",
                               "dt29." + class_name], out / "javap" / f"{class_name}.txt")
            require(disassembly.returncode == 0, f"javap failed for {class_name}")
        jarde, _ = compile_and_run("jarde", jarde_sources, runner, out, work)
        summary = {
            "jadx_revision": JADX_REV, "jadx_pins": PINS,
            "jarde_cli_sha256": digest(args.jarde),
            "input_sha256": {path.name: digest(path) for path in [*sources, runner]},
            "class_sha256": class_hashes,
            "classes": class_names, "original": original, "jadx": jadx, "jarde": jarde,
            "jarde_methods": methods,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n")
        if args.require_jarde:
            require(jarde.get("compile_exit") == 0 and jarde.get("run_exit") == 0
                    and jarde.get("stdout") == EXPECTED
                    and jarde.get("reflection_exit") == 0
                    and jarde.get("reflection_stdout") == EXPECTED_REFLECTION,
                    "Jarde complete family or generic metadata differs")
            require(all("@bytecode" not in path.read_text() for path in jarde_sources),
                    "Jarde still quotes a physical method")
        print(json.dumps({key: summary[key] for key in ("classes", "original", "jadx", "jarde")},
                         indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
