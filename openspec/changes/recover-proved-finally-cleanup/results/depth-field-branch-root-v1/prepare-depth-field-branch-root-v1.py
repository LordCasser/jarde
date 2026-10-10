#!/usr/bin/env python3
"""Prepare the shallow/deep nested-branch finally probe using the pinned toolchain."""
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
CLI_MANIFEST_SHA256 = "ddfbe127890d95ced5c4ed2a89d67d42deb7eba19e6c1e4ce12707221bd8f769"
CLI_SHA256 = "71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110"
DEPTHS = (2, 33)
INPUTS = (0, 40)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def file_record(path: Path, base: Path | None = None) -> dict:
    data = path.read_bytes()
    try:
        shown = path.relative_to(base).as_posix() if base is not None else str(path)
    except ValueError:
        shown = str(path)
    return {"path": shown, "bytes": len(data), "sha256": sha256_bytes(data)}


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def check_tool(record: dict, label: str) -> dict:
    path = Path(record["path"])
    identity = {"path": str(path), "bytes": path.stat().st_size, "sha256": sha256_file(path)}
    if identity["bytes"] != record["bytes"] or identity["sha256"] != record["sha256"]:
        raise RuntimeError(f"{label} differs from pinned JDK manifest: {identity}")
    return identity


def make_sources(depth: int) -> tuple[str, str, bytes]:
    if depth < 1:
        raise ValueError("depth must be positive")
    lines = [
        "public class BranchFinally {",
        f"    static final int DEPTH = {depth};",
        "    static int trace;",
        "",
        "    private static void cleanup() {",
        "        trace++;",
        "    }",
        "",
        "    static int run(int x) {",
        "        try {",
    ]
    for index in range(1, depth + 1):
        indent = "            " + "    " * (index - 1)
        condition = "trace + x > 1" if index == 1 else f"x > {index}"
        lines.extend([
            f"{indent}if ({condition}) {{",
            f"{indent}    trace += {index};",
        ])
    for index in reversed(range(1, depth + 1)):
        indent = "            " + "    " * (index - 1)
        lines.extend([
            f"{indent}}} else {{",
            f"{indent}    trace -= {index};",
            f"{indent}}}",
        ])
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
        if (BranchFinally.trace != 0) {
            throw new AssertionError("trace did not start at its default value");
        }
        int zero = BranchFinally.run(0);
        if (zero != 0 || BranchFinally.trace != 0) {
            throw new AssertionError("run(0): result=" + zero + " trace=" + BranchFinally.trace);
        }
        System.out.println("run(0)=" + zero + " trace=" + BranchFinally.trace);

        BranchFinally.trace = 0;
        int positive = BranchFinally.run(40);
        int expectedTrace = 1;
        for (int index = 1; index <= BranchFinally.DEPTH; index++) {
            expectedTrace += index;
        }
        if (positive != 40 || BranchFinally.trace != expectedTrace) {
            throw new AssertionError("run(40): result=" + positive + " trace=" + BranchFinally.trace
                    + " expectedTrace=" + expectedTrace);
        }
        System.out.println("run(40)=" + positive + " trace=" + BranchFinally.trace);
    }
}
"""
    expected_stdout = (
        f"run(0)=0 trace=0\nrun(40)=40 trace={1 + depth * (depth + 1) // 2}\n"
    ).encode("ascii")
    return class_source, runner_source, expected_stdout


def command(label: str, argv: list[str], cwd: Path, env: dict[str, str]) -> dict:
    process = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, check=False)
    raw_dir = RUN_DIR / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = raw_dir / f"{label}.stdout"
    stderr_path = raw_dir / f"{label}.stderr"
    stdout_path.write_bytes(process.stdout)
    stderr_path.write_bytes(process.stderr)
    return {
        "label": label,
        "argv": argv,
        "cwd": str(cwd),
        "exit": process.returncode,
        "stdout": file_record(stdout_path, RUN_DIR),
        "stderr": file_record(stderr_path, RUN_DIR),
        "_stdout": process.stdout,
        "_stderr": process.stderr,
    }


def public_command(row: dict) -> dict:
    return {key: value for key, value in row.items() if not key.startswith("_")}


def require_success(row: dict) -> None:
    if row["exit"] != 0:
        raise RuntimeError(
            f"{row['label']} failed with exit {row['exit']}; "
            f"inspect {row['stdout']['path']} and {row['stderr']['path']}"
        )


def main() -> int:
    if RUN_DIR.exists():
        raise SystemExit(f"refusing to overwrite existing result directory: {RUN_DIR}")
    if sha256_file(BASELINE_MANIFEST) != BASELINE_MANIFEST_SHA256:
        raise SystemExit("pinned returned-array JDK tools manifest changed")
    baseline = json.loads(BASELINE_MANIFEST.read_text(encoding="utf-8"))
    if baseline.get("schema") != "returned-array-update-next-baseline-v1":
        raise SystemExit("unexpected JDK manifest schema")
    if sha256_file(CLI_MANIFEST) != CLI_MANIFEST_SHA256:
        raise SystemExit("frozen CLI metadata identity changed")
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
        identities[leg] = {name: check_tool(row[name], f"{leg} {name}")
                            for name in ("javac", "java", "javap")}
    if set(tools) != {"javac8", "javac23"}:
        raise SystemExit("manifest must provide exactly javac8 and javac23")

    RUN_DIR.mkdir(parents=True)
    result = {
        "schema": "cf16-depth-field-branch-root-v1",
        "status": "running",
        "scope": "complete original BranchFinally plus Runner only; two branch depths, no recovered source compiled",
        "baseline_manifest": {"path": str(BASELINE_MANIFEST), "sha256": BASELINE_MANIFEST_SHA256},
        "cli": {"manifest": str(CLI_MANIFEST), "manifest_sha256": CLI_MANIFEST_SHA256,
                "path": str(cli_path), "sha256": CLI_SHA256},
        "jdk_tools": identities,
        "depths": [],
        "commands": [],
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

        for depth in DEPTHS:
            label = f"{leg}-depth-{depth:02d}"
            case_dir = RUN_DIR / leg / f"depth-{depth:02d}"
            source_dir = case_dir / "source"
            empty_dir = case_dir / "empty-classpath-sourcepath"
            classes_dir = case_dir / "classes"
            cli_dir = case_dir / "cli"
            for directory in (source_dir, empty_dir, classes_dir, cli_dir):
                directory.mkdir(parents=True)

            class_source, runner_source, expected_stdout = make_sources(depth)
            class_java = source_dir / "BranchFinally.java"
            runner_java = source_dir / "Runner.java"
            class_java.write_text(class_source, encoding="utf-8")
            runner_java.write_text(runner_source, encoding="utf-8")
            case_record = {
                "leg": leg,
                "depth": depth,
                "source": file_record(class_java, RUN_DIR),
                "runner": file_record(runner_java, RUN_DIR),
                "expected_stdout": {"bytes": len(expected_stdout), "sha256": sha256_bytes(expected_stdout),
                                    "text": expected_stdout.decode("ascii")},
            }

            compile_row = command(
                label + "-original-compile",
                [jdk["javac"]["path"], "-source", "8", "-target", "8", "-g:none",
                 "-Xlint:-options", "-classpath", str(empty_dir), "-sourcepath", str(empty_dir),
                 "-d", str(classes_dir), str(class_java), str(runner_java)],
                case_dir, env,
            )
            result["commands"].append(public_command(compile_row))
            write_json(RUN_DIR / "start.json", result)
            require_success(compile_row)

            class_file = classes_dir / "BranchFinally.class"
            runner_class = classes_dir / "Runner.class"
            if not class_file.is_file() or not runner_class.is_file():
                raise RuntimeError(f"{label}: javac did not produce both full classes")
            case_record["class_file"] = file_record(class_file, RUN_DIR)
            case_record["runner_class"] = file_record(runner_class, RUN_DIR)
            run_row = command(
                label + "-original-verify-run",
                [jdk["java"]["path"], "-Xverify:all", "-cp", str(classes_dir), "Runner"],
                case_dir, env,
            )
            result["commands"].append(public_command(run_row))
            write_json(RUN_DIR / "start.json", result)
            require_success(run_row)
            if run_row["_stdout"] != expected_stdout:
                raise RuntimeError(f"{label}: original Runner output differs from calculated trace oracle")
            case_record["oracle"] = {
                "exit": run_row["exit"],
                "stdout": run_row["stdout"],
                "stderr": run_row["stderr"],
            }

            cli_row = command(
                label + "-class-source-all-json",
                [str(cli_path), "class-source", "--input", str(class_file), "--class",
                 "BranchFinally", "--policy", "single-class", "--release", "8",
                 "--evidence", "all", "--format", "json"],
                case_dir, env,
            )
            result["commands"].append(public_command(cli_row))
            write_json(RUN_DIR / "start.json", result)
            require_success(cli_row)
            document_bytes = cli_row["_stdout"]
            document = json.loads(document_bytes)
            document_path = cli_dir / "document.json"
            document_path.write_bytes(document_bytes)
            full_source_path = cli_dir / "class-source.txt"
            full_source = document.get("text", "")
            full_source_path.write_text(full_source, encoding="utf-8")

            matching = [method for method in document.get("methods", [])
                        if method.get("item", {}).get("name", {}).get("escaped") == "run"
                        and method.get("item", {}).get("descriptor", {}).get("escaped") == "(I)I"]
            if len(matching) != 1:
                raise RuntimeError(f"{label}: expected one run(I)I record, got {len(matching)}")
            method = matching[0]
            outcome = method.get("outcome", {})
            report = outcome.get("report", {})
            body_text = report.get("text", "")
            body_path = cli_dir / "run-body.txt"
            body_path.write_text(body_text, encoding="utf-8")
            case_record["cli"] = {
                "document_json": file_record(document_path, RUN_DIR),
                "class_source_text": file_record(full_source_path, RUN_DIR),
                "run_body_text": file_record(body_path, RUN_DIR),
                "document_outcome": document.get("outcome"),
                "document_diagnostics": document.get("diagnostics"),
                "execution": document.get("execution"),
                "method_outcome_kind": outcome.get("kind"),
                "report_outcome": report.get("outcome"),
                "report_stop": report.get("stop"),
                "quality": report.get("quality"),
                "fallbacks": report.get("fallbacks"),
                "diagnostics": report.get("diagnostics"),
                "regions": report.get("regions"),
                "rules": report.get("rules"),
                "cli_raw_stdout": cli_row["stdout"],
                "cli_raw_stderr": cli_row["stderr"],
                "cli_text_executed_or_compiled": False,
            }
            result["depths"].append(case_record)
            write_json(RUN_DIR / "start.json", result)

    result["status"] = "prepared-observations-complete"
    result["note"] = (
        "depth-02 is the shallow branch control and depth-33 is the deeper nested-branch probe. "
        "Use the recorded method outcome, stop, produced report, and diagnostics as observed; "
        "do not infer a hard depth-bound hit without its explicit diagnostic."
    )
    write_json(RUN_DIR / "result.json", result)
    inventory = [file_record(path, RUN_DIR) for path in sorted(RUN_DIR.rglob("*"))
                 if path.is_file() and path != RUN_DIR / "file-inventory.json"]
    write_json(RUN_DIR / "file-inventory.json", inventory)
    print(json.dumps({"status": result["status"], "observations": len(result["depths"])}, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"depth-branch preparation failed: {error}", file=sys.stderr)
        raise
