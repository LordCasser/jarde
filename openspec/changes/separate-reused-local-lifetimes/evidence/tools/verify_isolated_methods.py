#!/usr/bin/env python3
import json
import pathlib
import subprocess
import sys

run_root = pathlib.Path(sys.argv[1])
evidence_dir = pathlib.Path(sys.argv[2])
specs = {
    "sameType": ["sameType(4)"],
    "exclusiveBranch": ["exclusiveBranch(true)", "exclusiveBranch(false)"],
    "loopBodyReuse": ["loopBodyReuse(2)"],
    "handlerReuse": ["handlerReuse(false)", "handlerReuse(true)"],
    "category2Adjacent": ["category2Adjacent()"],
}

lines = []
for mode in ("g", "none"):
    report = json.load(open(run_root / f"boundaries-jarde-{mode}.json"))
    methods = {
        bytes(method["item"]["identity"]["name"]).decode(): method
        for method in report["methods"]
    }
    for name, calls in specs.items():
        method_dir = run_root / f"isolated-{mode}-{name}"
        method_dir.mkdir(parents=True, exist_ok=True)
        method_text = methods[name]["text"]
        (method_dir / "SlotReuseBoundaries.java").write_text(
            "public class SlotReuseBoundaries {\n" + method_text + "\n}\n"
        )
        runner_text = (
            "public class BoundaryMethodRunner { public static void main(String[] args) {\n"
            + "".join(f"System.out.println(SlotReuseBoundaries.{call});\n" for call in calls)
            + "} }\n"
        )
        runner_path = method_dir / "BoundaryMethodRunner.java"
        runner_path.write_text(runner_text)

        original_classes = method_dir / "original-classes"
        original_classes.mkdir(exist_ok=True)
        original_compile = subprocess.run(
            [
                "javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                str(original_classes), str(evidence_dir / "fixtures/SlotReuseBoundaries.java"),
                str(runner_path),
            ],
            capture_output=True,
            text=True,
        )
        if original_compile.returncode:
            raise SystemExit(original_compile.stderr)
        original_run = subprocess.run(
            ["java", "-Xverify:all", "-cp", str(original_classes), "BoundaryMethodRunner"],
            capture_output=True,
            text=True,
        )
        if original_run.returncode:
            raise SystemExit(original_run.stderr)

        classes = method_dir / "jarde-classes"
        classes.mkdir(exist_ok=True)
        compile_result = subprocess.run(
            [
                "javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                str(classes), str(method_dir / "SlotReuseBoundaries.java"), str(runner_path),
            ],
            capture_output=True,
            text=True,
        )
        lines.append(f"{mode} {name} javac_exit={compile_result.returncode}")
        if compile_result.stderr:
            lines.append(compile_result.stderr.rstrip())
        if compile_result.returncode == 0:
            actual = subprocess.run(
                ["java", "-Xverify:all", "-cp", str(classes), "BoundaryMethodRunner"],
                capture_output=True,
                text=True,
            )
            matches = actual.returncode == 0 and actual.stdout == original_run.stdout
            lines.append(f"java_exit={actual.returncode} output_matches_original={matches}")
            lines.append(actual.stdout.rstrip())
            if actual.stderr:
                lines.append(actual.stderr.rstrip())
        else:
            lines.append("java=not-run (method source did not compile)")
(run_root / "boundary-isolated-results.txt").write_text("\n".join(lines) + "\n")
print((run_root / "boundary-isolated-results.txt").read_text(), end="")
