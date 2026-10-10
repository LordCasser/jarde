#!/usr/bin/env python3
"""Prepare the CF16 depth-boundary oracle and collect frozen all-evidence reports.

This script is not run during preparation. It invokes only the pinned JDK tools and frozen CLI
after a reviewer chooses to execute it.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT = SCRIPT_DIR.parents[4]
RUN_DIR = SCRIPT_DIR / "run"
BASELINE_MANIFEST = ROOT / (
    "openspec/changes/recover-nested-int-array-compound-updates/results/"
    "returned-next-baseline-v1/manifest.json"
)
BASELINE_MANIFEST_SHA256 = "10a6ea368ba5304594909d0318d650b7ad7c580904d0897c2cf7ba13232db4a3"
CLI_MANIFEST = ROOT / (
    "openspec/changes/recover-returned-int-array-compound-updates/results/"
    "candidate-cli-v2.json"
)
CLI_SHA256 = "71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110"
EXPECTED_RUNNER_STDOUT = b"run(0)=0 trace=1\nrun(40)=1 trace=2\n"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def check_tool(record: dict, label: str) -> dict:
    path = Path(record["path"])
    identity = {"path": str(path), "bytes": path.stat().st_size, "sha256": sha256_file(path)}
    if identity["bytes"] != record["bytes"] or identity["sha256"] != record["sha256"]:
        raise RuntimeError(f"{label} differs from pinned JDK manifest: {identity}")
    return identity


def make_sources(depth: int) -> tuple[str, str]:
    if depth < 1:
        raise ValueError("depth must be positive")
    lines = [
        "public class DeepFinally {",
        "    static int trace;",
        "",
        "    private static void cleanup() {",
        "        trace++;",
        "    }",
        "",
        "    static int run(int x) {",
        "        try {",
    ]
    for index, threshold in enumerate(range(depth, 0, -1)):
        indent = "            " + "    " * index
        lines.extend([f"{indent}while (x > {threshold}) {{", f"{indent}    x--;"])
    for index in reversed(range(depth)):
        lines.append("            " + "    " * index + "}")
    lines.extend([
        "            return x;",
        "        } finally {",
        "            cleanup();",
        "        }",
        "    }",
        "}",
        "",
    ])
    class_source = "\n".join(lines)
    runner_source = """public class Runner {
    public static void main(String[] args) {
        if (DeepFinally.trace != 0) {
            throw new AssertionError("trace did not start at its default value");
        }
        int zero = DeepFinally.run(0);
        if (zero != 0 || DeepFinally.trace != 1) {
            throw new AssertionError("run(0): result=" + zero + " trace=" + DeepFinally.trace);
        }
        System.out.println("run(0)=" + zero + " trace=" + DeepFinally.trace);

        int positive = DeepFinally.run(40);
        if (positive != 1 || DeepFinally.trace != 2) {
            throw new AssertionError("run(40): result=" + positive + " trace=" + DeepFinally.trace);
        }
        System.out.println("run(40)=" + positive + " trace=" + DeepFinally.trace);
    }
}
"""
    return class_source, runner_source


def run_command(label: str, argv: list[str], cwd: Path, env: dict[str, str]) -> dict:
    process = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, check=False)
    raw_dir = RUN_DIR / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = raw_dir / f"{label}.stdout"
    stderr_path = raw_dir / f"{label}.stderr"
    stdout_path.write_bytes(process.stdout)
    stderr_path.write_bytes(process.stderr)
    return {
        "label": label, "argv": argv, "cwd": str(cwd), "exit": process.returncode,
        "stdout": {"path": stdout_path.relative_to(RUN_DIR).as_posix(),
                   "bytes": len(process.stdout), "sha256": sha256_bytes(process.stdout)},
        "stderr": {"path": stderr_path.relative_to(RUN_DIR).as_posix(),
                   "bytes": len(process.stderr), "sha256": sha256_bytes(process.stderr)},
        "_stdout": process.stdout,
    }


def public_command(record: dict) -> dict:
    return {key: value for key, value in record.items() if not key.startswith("_")}


def require_success(record: dict) -> None:
    if record["exit"] != 0:
        raise RuntimeError(
            f"{record['label']} failed with exit {record['exit']}; "
            f"inspect {record['stdout']['path']} and {record['stderr']['path']}"
        )


def main() -> int:
    if RUN_DIR.exists():
        raise SystemExit(f"refusing to overwrite existing result directory: {RUN_DIR}")
    if sha256_file(BASELINE_MANIFEST) != BASELINE_MANIFEST_SHA256:
        raise SystemExit("pinned returned-array JDK tools manifest changed")
    baseline = json.loads(BASELINE_MANIFEST.read_text(encoding="utf-8"))
    if baseline.get("schema") != "returned-array-update-next-baseline-v1":
        raise SystemExit("unexpected JDK manifest schema")
    cli_doc = json.loads(CLI_MANIFEST.read_text(encoding="utf-8"))
    cli_path = Path(cli_doc["cli_path"])
    if cli_doc.get("cli_sha256") != CLI_SHA256 or sha256_file(cli_path) != CLI_SHA256:
        raise SystemExit("frozen returned-array-v2 CLI identity does not match")

    tools = {}
    identities = {}
    for row in baseline["jdk_tools"]:
        java_path = row["java"]["path"]
        if "corretto-1.8.0_432" in java_path:
            leg = "javac8"
        elif "openjdk-23.0.1" in java_path:
            leg = "javac23"
        else:
            raise SystemExit(f"unrecognized JDK path in pinned manifest: {java_path}")
        if leg in tools:
            raise SystemExit(f"duplicate JDK leg: {leg}")
        tools[leg] = row
        identities[leg] = {
            name: check_tool(row[name], f"{leg} {name}")
            for name in ("javac", "java", "javap")
        }
    if set(tools) != {"javac8", "javac23"}:
        raise SystemExit("manifest must provide exactly javac8 and javac23")

    RUN_DIR.mkdir()
    result = {
        "schema": "cf16-depth-boundary-root-v1",
        "status": "running",
        "scope": "original generated classes only are compiled/run; CLI source is saved, never compiled or executed",
        "baseline_manifest": {"path": str(BASELINE_MANIFEST), "sha256": BASELINE_MANIFEST_SHA256},
        "cli": {"manifest": str(CLI_MANIFEST), "path": str(cli_path), "sha256": CLI_SHA256},
        "jdk_tools": identities,
        "classes": [],
        "commands": [],
        "boundary_interpretation": (
            "record the exact report; do not assume depth 33 reaches recursion_bound or any "
            "particular early-refusal path"
        ),
    }
    write_json(RUN_DIR / "start.json", result)

    for leg in ("javac8", "javac23"):
        jdk = tools[leg]
        java_home = Path(jdk["java"]["path"]).parent.parent
        env = os.environ.copy()
        for option in ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"):
            env.pop(option, None)
        env["JAVA_HOME"] = str(java_home)
        env["PATH"] = str(java_home / "bin") + os.pathsep + env.get("PATH", "")
        for depth in (2, 33):
            case = f"depth-{depth:02d}"
            case_dir = RUN_DIR / leg / case
            source_dir = case_dir / "source"
            empty_dir = case_dir / "empty"
            classes_dir = case_dir / "classes"
            cli_dir = case_dir / "cli"
            for directory in (source_dir, empty_dir, classes_dir, cli_dir):
                directory.mkdir(parents=True)

            class_source, runner_source = make_sources(depth)
            class_java = source_dir / "DeepFinally.java"
            runner_java = source_dir / "Runner.java"
            class_java.write_text(class_source, encoding="utf-8")
            runner_java.write_text(runner_source, encoding="utf-8")
            compile_row = run_command(
                f"{leg}-{case}-original-compile",
                [jdk["javac"]["path"], "-source", "8", "-target", "8", "-g:none",
                 "-Xlint:-options", "-classpath", str(empty_dir), "-sourcepath", str(empty_dir),
                 "-d", str(classes_dir), str(class_java), str(runner_java)],
                case_dir, env,
            )
            result["commands"].append(public_command(compile_row))
            require_success(compile_row)

            class_file = classes_dir / "DeepFinally.class"
            runner_class = classes_dir / "Runner.class"
            if not class_file.is_file() or not runner_class.is_file():
                raise RuntimeError(f"{leg}/{case}: javac did not produce both classes")
            run_row = run_command(
                f"{leg}-{case}-original-verify-run",
                [jdk["java"]["path"], "-Xverify:all", "-cp", str(classes_dir), "Runner"],
                case_dir, env,
            )
            result["commands"].append(public_command(run_row))
            require_success(run_row)
            if run_row["_stdout"] != EXPECTED_RUNNER_STDOUT:
                raise RuntimeError(f"{leg}/{case}: oracle stdout differs from expected values")

            class_record = {
                "leg": leg, "depth": depth,
                "class_file": {"path": str(class_file), "bytes": class_file.stat().st_size,
                               "sha256": sha256_file(class_file)},
                "runner_class": {"path": str(runner_class), "bytes": runner_class.stat().st_size,
                                 "sha256": sha256_file(runner_class)},
                "oracle_stdout": next(row["stdout"] for row in result["commands"]
                                      if row["label"] == run_row["label"]),
            }
            cli_row = run_command(
                f"{leg}-{case}-class-source-all-json",
                [str(cli_path), "class-source", "--input", str(class_file), "--class",
                 "DeepFinally", "--policy", "single-class", "--release", "8",
                 "--evidence", "all", "--format", "json"],
                case_dir, env,
            )
            result["commands"].append(public_command(cli_row))
            require_success(cli_row)
            raw_json = cli_row["_stdout"]
            document_path = cli_dir / "document.json"
            document_path.write_bytes(raw_json)
            document = json.loads(raw_json)
            full_text_path = cli_dir / "class-source.txt"
            full_text_path.write_text(document.get("text", ""), encoding="utf-8")

            matches = [
                method for method in document.get("methods", [])
                if method.get("item", {}).get("name", {}).get("escaped") == "run"
                and method.get("item", {}).get("descriptor", {}).get("escaped") == "(I)I"
            ]
            if len(matches) != 1:
                raise RuntimeError(f"{leg}/{case}: expected one run(I)I record, got {len(matches)}")
            method = matches[0]
            method_outcome = method.get("outcome", {})
            report = method_outcome.get("report", {})
            body_text_path = cli_dir / "run-body.txt"
            body_text_path.write_text(report.get("text", ""), encoding="utf-8")
            class_record["cli"] = {
                "document_json": {"path": document_path.relative_to(RUN_DIR).as_posix(),
                                  "bytes": len(raw_json), "sha256": sha256_bytes(raw_json)},
                "class_source_text": {"path": full_text_path.relative_to(RUN_DIR).as_posix(),
                                      "bytes": full_text_path.stat().st_size,
                                      "sha256": sha256_file(full_text_path)},
                "run_body_text": {"path": body_text_path.relative_to(RUN_DIR).as_posix(),
                                  "bytes": body_text_path.stat().st_size,
                                  "sha256": sha256_file(body_text_path)},
                "method_outcome_kind": method_outcome.get("kind"),
                "report_outcome": report.get("outcome"),
                "execution": report.get("execution"),
                "quality": report.get("quality"),
                "rules": report.get("rules"),
                "fallbacks": report.get("fallbacks"),
                "regions": report.get("regions"),
                "raw_stdout": cli_row["stdout"],
                "raw_stderr": cli_row["stderr"],
                "cli_text_executed_or_compiled": False,
            }
            result["classes"].append(class_record)
            write_json(RUN_DIR / "start.json", result)

    result["status"] = "prepared-observations-complete"
    result["note"] = (
        "depth 2 is the shallow control and depth 33 is a probe of the existing bounded outcome. "
        "Review its explicit report before claiming a recursion-bound hit. Recompile the depth-2 "
        "recovered candidate independently."
    )
    write_json(RUN_DIR / "result.json", result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"preparation failed: {error}", file=sys.stderr)
        raise
