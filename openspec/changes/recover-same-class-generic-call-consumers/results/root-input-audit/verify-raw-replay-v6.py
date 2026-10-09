#!/usr/bin/env python3
"""Independently verify saved raw-receiver replay transcripts and their inputs."""
import argparse
import csv
import hashlib
import json
import shlex
from collections import Counter
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(path):
    return list(csv.DictReader(path.open(), delimiter="\t"))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--frozen", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    root, frozen = args.results.resolve(), args.frozen.resolve()
    errors, counts = [], Counter()

    def require(ok, message):
        if not ok:
            errors.append(message)

    metadata = json.loads((root / "outer-run-metadata.json").read_text())
    preflight = json.loads((root / "outer-preflight-metadata.json").read_text())
    require(metadata["exit_code"] == 0, "runner exit")
    require(preflight["output_was_absent_before_run"], "fresh output")
    require(preflight["label"] == "gc09-raw-64-candidate-v6-cli6", "candidate label")
    require(preflight["cli"]["sha256"] == "c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8", "candidate identity")
    require(sha(root / "replay-executed.py") == preflight["runner"]["snapshot_sha256"] == metadata["runner_sha256"], "runner snapshot")
    require(sha(root / "adapter-executed.py") == preflight["adapter"]["sha256"] == metadata["adapter_sha256"], "adapter snapshot")
    require(metadata["runner_source_unchanged"] and metadata["source_and_input_hashes_unchanged"], "input integrity")
    frozen_before = preflight["source_and_input_hashes_before_run"]
    require(frozen_before["counts"] == {"runner_source_files":18,"fixture_source_files":23,"frozen_input_jars":64}, "frozen source/jar census")
    require(frozen_before == metadata["source_and_input_hashes_after_run"], "source/jar before-after")
    for path, digest in frozen_before["sha256"].items():
        require(sha(Path(path)) == digest, "frozen hash: " + path)
    for key in ("cli", "jadx", "adapter"):
        require(sha(Path(preflight[key]["path"])) == preflight[key]["sha256"], key + " preflight hash")
    require(len(preflight["tools"]["tools"]) == 59 and len(preflight["tools"]["jdk_tools"]) == 8, "tool census")
    for tool in preflight["tools"]["tools"] + preflight["tools"]["jdk_tools"]:
        require(sha(Path(tool["path"])) == tool["sha256"], "preflight tool " + tool["path"])
        if "version_exit_code" in tool:
            require(tool["version_exit_code"] == 0, "tool version exit")
            for stream in ("stdout", "stderr"):
                require(sha(Path(tool["version_" + stream])) == tool["version_" + stream + "_sha256"], "tool version stream")
    for fact in preflight["tools"]["jdk_release_facts"]:
        for kind in ("release", "package"):
            if fact[kind + "_file"] != "absent":
                require(sha(Path(fact[kind + "_file"])) == fact[kind + "_file_sha256"], "JDK release fact")
    actual_commands = metadata["subprocess_records"]
    require(len(actual_commands) == metadata["subprocess_count"], "actual command count")
    for command in actual_commands:
        require(isinstance(command["argv"], list) and command["argv"], "actual argv")
        for stream in ("stdout", "stderr"):
            require(sha(Path(command[stream])) == command[stream + "_sha256"], "actual " + stream)
    for record in rows(root / "result-file-hashes.tsv"):
        require(sha(root / record["path"]) == record["sha256"], "result manifest: " + record["path"])
    metadata["label"] = preflight["label"]
    tools = rows(root / "versions/tool-hashes.tsv")
    for row in tools:
        require(sha(Path(row["path"])) == row["sha256"], "tool hash: " + row["tool"])
    for row in rows(root / "versions/frozen-input-jar-hashes.tsv"):
        require(sha(frozen / row["input_jar"]) == row["sha256"], "frozen jar: " + row["input_jar"])
    repo = frozen.parents[2]
    for row in rows(root / "versions/source-hashes.tsv"):
        name = row["source"]
        if name.startswith("source/"):
            path = frozen / name
        elif name.startswith("tests/"):
            path = repo / name
        else:
            path = root / Path(name).relative_to(metadata["label"])
        require(sha(path) == row["sha256"], "source: " + name)
    commands = rows(root / "commands.tsv")
    by_scope = {row["scope"]: row for row in commands}
    require(len(by_scope) == len(commands), "duplicate command scope")
    actual_by_scope = {record["scope"]: record for record in actual_commands}
    process_rows = [row for row in commands if not row["scope"].endswith("source-header")]
    require(len(actual_by_scope) == len(actual_commands) == len(process_rows), "actual subprocess census")
    for row in process_rows:
        actual = actual_by_scope.get(row["scope"], {})
        require(shlex.split(row["command"]) == actual.get("argv") and int(row["exit_code"]) == actual.get("exit_code"), row["scope"] + " actual argv/exit")
    for row in commands:
        suffix = row["scope"].split("/")[-1]
        if suffix in ("original-javac", "original-probe-javac", "jarde-javac", "jadx-javac"):
            argv = shlex.split(row["command"])
            require("-classpath" in argv and "-sourcepath" in argv, row["scope"] + " compile isolation")
            if "-classpath" in argv and "-sourcepath" in argv:
                require(argv[argv.index("-classpath") + 1].endswith("/empty-classpath"), row["scope"] + " empty classpath")
                require(argv[argv.index("-sourcepath") + 1].endswith("/empty-sourcepath"), row["scope"] + " empty sourcepath")
            require(not any(value.endswith((".jar", ".class")) for value in argv), row["scope"] + " borrowed compiled input")
        if suffix in ("original-run", "jarde-verify-run", "jadx-verify-run"):
            argv = shlex.split(row["command"])
            require("-Xverify:all" in argv and "-cp" in argv, row["scope"] + " JVM verification")
            cp = argv[argv.index("-cp") + 1]
            require(not ".jar" in cp and "/input/" not in cp, row["scope"] + " runtime isolation")
            if suffix == "jarde-verify-run":
                require(cp.endswith("/jarde-classes") and ":" not in cp, row["scope"] + " candidate runtime classpath")
    cases = rows(root / "runtime-comparisons.tsv")
    require(len(cases) == 64, "64 cases")
    require(len({(r["leg"], r["variant"], r["case"]) for r in cases}) == 64, "unique cases")
    categories = {"field_api": ("field.type=", "field.identity="), "method_api": ("method=",), "class_api": ("class.type-parameter=",)}
    baseline_regressions = []
    for row in cases:
        scope = "/".join(row[k] for k in ("leg", "variant", "case"))
        area = root / row["leg"] / row["variant"] / "runtime"
        transcripts = {flavor: (area / (row["case"] + "." + flavor + ".txt")).read_text().splitlines()
                       for flavor in ("original", "jarde")}
        baseline_area = args.baseline / row["leg"] / row["variant"] / "runtime"
        previous = (baseline_area / (row["case"] + ".jarde.txt")).read_text().splitlines()
        for category, prefixes in categories.items():
            selected = lambda lines: sorted(line for line in lines if line.startswith(prefixes))
            expected, actual = selected(transcripts["original"]), selected(transcripts["jarde"])
            require(bool(expected) or category != "field_api", scope + " field API transcript")
            matches = expected == actual
            counts[category] += matches
            if selected(previous) == expected and not matches:
                baseline_regressions.append(scope + "/" + category)
        def behavior(lines):
            return [line if not line.startswith("behavior.value=") else "behavior.value=" + line.partition("=")[2].rsplit(".", 1)[-1]
                    for line in lines if line.startswith("behavior.")]
        expected, actual = behavior(transcripts["original"]), behavior(transcripts["jarde"])
        require(len(expected) == 7 and expected == actual, scope + " behavior")
        counts["behavior"] += expected == actual
        require(row["jarde_behavior_vs_original"] == str(expected == actual).lower(), scope + " reported behavior")
        for suffix in ("original-javac", "original-probe-javac", "original-run", "jarde-javac", "jarde-verify-run"):
            require(by_scope.get(scope + "/" + suffix, {}).get("exit_code") == "0", scope + "/" + suffix)
    require(not baseline_regressions, "API regressions versus accepted baseline")
    require(not list(root.rglob("*.class")), "no generated class residue")
    file_index = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p)}
                  for p in sorted(root.rglob("*")) if p.is_file()]
    result = {"scope": "root独立核对GC09 raw64候选；固定v38 source snapshot CLI6；完整片仍须其他矩阵和门禁", "passed": not errors,
              "rows": len(cases), "commands_checked": len(commands), "actual_subprocesses_checked": len(actual_commands), "synthetic_header_checks": len(commands)-len(actual_commands), "tool_hashes_checked": len(tools),
              "recomputed_matches": dict(counts), "new_api_regressions": baseline_regressions,
              "limitations": "v2 outer adapter补运行前工具/版本与全部成功stderr；临时编译class字节以runner hash记录，结果不保留生成class。",
              "files": file_index, "verifier_sha256": sha(Path(__file__)), "errors": errors}
    with args.out.open("x") as stream:
        json.dump(result, stream, indent=2, ensure_ascii=False)
        stream.write("\n")
    print(json.dumps({k: v for k, v in result.items() if k != "files"}, ensure_ascii=False))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
