#!/usr/bin/env python3
"""Root 独立重算完整源集、物理身份、隔离执行和原始双流；不导入 replay runner。"""
import hashlib
import json
from pathlib import Path
import sys
import zipfile

RESULTS = Path(__file__).resolve().parent
ROOT = RESULTS.parents[3]
OUT = RESULTS / "candidate-root-v1"
BASE = RESULTS / "baseline-v1"
checks = []


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(label, condition):
    checks.append({"label": label, "ok": bool(condition)})


def signatures(document):
    return {(bytes(m["item"]["name"]["raw"]), bytes(m["item"]["descriptor"]["raw"]))
            for m in document["methods"]}


def main():
    destination = RESULTS / "candidate-root-verification-v1.json"
    if destination.exists():
        raise SystemExit("refuse to overwrite root verification")
    manifest = json.loads((OUT / "manifest.json").read_bytes())
    baseline = json.loads((BASE / "manifest.json").read_bytes())
    cli = json.loads((RESULTS / "candidate-cli-v1.json").read_bytes())
    check("frozen CLI", sha(cli["cli_path"]) == cli["cli_sha256"] == manifest["cli_sha256"])
    for path, digest in cli["candidate_sources"].items():
        check("current product identity " + path, sha(ROOT / path) == digest)
    check("baseline identity", sha(BASE / "manifest.json") == manifest["baseline_manifest_sha256"])
    for item in manifest["files"]:
        path = OUT / item["path"]
        check("recorded file " + item["path"], path.is_file() and
              path.stat().st_size == item["bytes"] and sha(path) == item["sha256"])
    actual_files = {str(p.relative_to(OUT)) for p in OUT.rglob("*") if p.is_file() and p.name != "manifest.json"}
    check("closed result files", actual_files == {x["path"] for x in manifest["files"]})
    for command in manifest["commands"]:
        for stream in ("stdout", "stderr"):
            p = OUT / command[stream]
            check(command["label"] + " " + stream, sha(p) == command[stream + "_sha256"] and
                  p.stat().st_size == command[stream + "_bytes"])
    expected_sources = {"ConstructorPrimitiveConversionControls.java", "PrimitiveLongPair.java"}
    for leg, detail in manifest["jdk_legs"].items():
        original = baseline["jdk_legs"][leg]
        work = OUT / "generated" / leg
        sources, classes, empty = [work / x for x in ("sources", "classes", "empty-classpath-sourcepath")]
        check(leg + " empty CP/SP", empty.is_dir() and not list(empty.iterdir()))
        check(leg + " complete source closure", {p.name for p in sources.rglob("*.java")} == expected_sources)
        check(leg + " private class closure", {p.name for p in classes.rglob("*.class")} ==
              {p.replace(".java", ".class") for p in expected_sources})
        check(leg + " JDK identity", sha(Path(original["jdk_home"]) / "bin/javac") == original["javac_sha256"] and
              sha(Path(original["jdk_home"]) / "bin/java") == original["java_sha256"])
        compile_command = detail["candidate_compile"]
        argv = compile_command["argv"]
        source_argv = {str(p) for p in sources.rglob("*.java")}
        check(leg + " all-source compile argv", set(argv[-2:]) == source_argv and compile_command["exit"] == 0)
        check(leg + " compile flags", argv[1:1 + len(original["compiler_flags_from_fixture_v1_run003"])] ==
              original["compiler_flags_from_fixture_v1_run003"])
        check(leg + " isolated compiler", argv[argv.index("-classpath") + 1] == str(empty) and
              argv[argv.index("-sourcepath") + 1] == str(empty) and argv[argv.index("-d") + 1] == str(classes))
        runtime = detail["candidate_runtime"]
        check(leg + " private verified runtime argv", runtime["argv"] ==
              [str(Path(original["jdk_home"]) / "bin/java"), "-Xverify:all", "-cp", str(classes),
               "ConstructorPrimitiveConversionControls"])
        check(leg + " exact exit", runtime["exit"] == original["original_runtime"]["exit"] == 0)
        for stream in ("stdout", "stderr"):
            check(leg + " original raw " + stream, (OUT / runtime[stream]).read_bytes() ==
                  (BASE / original["original_runtime"][stream]).read_bytes())
        all_sites = []
        for record in detail["candidate_source_reports"]:
            document = json.loads((OUT / record["command"]["stdout"]).read_bytes())
            name = record["class_name"]
            old_cmd = next(x for x in baseline["commands"] if x["label"] == leg + "-jarde-cli2-render-" + name)
            old_doc = json.loads((BASE / old_cmd["stdout"]).read_bytes())
            check(leg + "/" + name + " physical class identity", document["class"] == old_doc["class"])
            check(leg + "/" + name + " complete members", signatures(document) == signatures(old_doc))
            check(leg + "/" + name + " exact source text", (sources / (name + ".java")).read_text() == document["text"])
            check(leg + "/" + name + " whole source without refusal", "@bytecode" not in document["text"] and
                  "jarde_refused_body" not in document["text"])
            for member in document["methods"]:
                body = member["outcome"].get("report")
                check(leg + "/" + name + "/" + str(member["item"]["name"]["raw"]) + " complete body",
                      body is not None and body["quality"] == "structured" and body["representation"] == "java" and
                      "@bytecode" not in body["text"] and not member["markers"])
                if not body:
                    continue
                bcis = {v["bci"] for segment in body["source_map"]["segments"]
                        for v in [segment["origin"]["primary"], *segment["origin"]["derived"]]}
                for site in body["news"]:
                    check(leg + " closed site " + str(site["head"]), site["presented"] and not site["refusal"] and
                          {site["head"], site["dup"], site["constructor"], *site["arguments"]} <= bcis)
                    all_sites.append((name, tuple(member["item"]["name"]["raw"]), site["head"]))
        check(leg + " exactly 32 unique physical sites", len(all_sites) == len(set(all_sites)) == 32)
    errors = [x for x in checks if not x["ok"]]
    result = {"schema": "numeric-candidate-independent-root-v1", "runner_sha256": sha(__file__),
              "candidate_manifest_sha256": sha(OUT / "manifest.json"), "checks": checks,
              "check_count": len(checks), "errors": errors, "accepted": not errors}
    destination.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"checks": len(checks), "errors": len(errors)}))
    return int(bool(errors))


if __name__ == "__main__":
    sys.exit(main())
