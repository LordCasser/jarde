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
    require(sha(root / "replay-executed.py") == metadata["runner_snapshot_sha256"], "runner snapshot hash")
    require(metadata["launcher_exit"] == 0, "runner exit")
    for key in ("candidate_cli", "baseline_cli", "jadx_cli", "runner_stdout", "runner_stderr"):
        require(sha(Path(metadata[key])) == metadata[key + "_sha256"], key + " hash")
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
    reported = json.loads((root / "comparison-summary.json").read_text())
    for category, count in counts.items():
        require(reported["metrics"][category]["pass"] == count, "reported " + category)
    file_index = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p)}
                  for p in sorted(root.rglob("*")) if p.is_file()]
    result = {"scope": "root独立核对GC09 raw64候选；只证明固定CLI，不证明当前正在编辑源码", "passed": not errors,
              "rows": len(cases), "commands_checked": len(commands), "tool_hashes_checked": len(tools),
              "recomputed_matches": dict(counts), "new_api_regressions": baseline_regressions,
              "limitations": "runner未保存成功命令的stderr及运行前JDK二进制hash；检查保存命令、JDK版本、实际运行转录。文件索引为本次核对生成，不伪称原runner manifest。",
              "files": file_index, "errors": errors}
    with args.out.open("x") as stream:
        json.dump(result, stream, indent=2, ensure_ascii=False)
        stream.write("\n")
    print(json.dumps({k: v for k, v in result.items() if k != "files"}, ensure_ascii=False))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
