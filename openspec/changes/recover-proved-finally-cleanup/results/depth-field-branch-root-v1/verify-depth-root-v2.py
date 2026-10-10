#!/usr/bin/env python3
"""Independent root audit of the real Region recursion stop and unpublished method."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE / "run-v2"
checks = 0


def check(condition, label):
    global checks
    checks += 1
    if not condition:
        raise AssertionError(label)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(row):
    path = Path(row["path"])
    if not path.is_absolute():
        path = BASE / path
    data = path.read_bytes()
    check(len(data) == row["bytes"] and sha(data) == row["sha256"], str(path))
    return data


manifest_path = BASE / "result.json"
manifest_bytes = manifest_path.read_bytes()
manifest = json.loads(manifest_bytes)
inventory_path = BASE / "file-inventory.json"
inventory_bytes = inventory_path.read_bytes()
inventory = json.loads(inventory_bytes)
check({row["path"] for row in inventory} == {
    path.relative_to(BASE).as_posix() for path in BASE.rglob("*")
    if path.is_file() and path != inventory_path}, "closed complete file inventory")
for row in inventory:
    read(row)
check(manifest["schema"] == "cf16-depth-field-branch-root-v2", "schema")
check(manifest["status"] == "prepared-observations-complete", "complete observation")
check({(c["leg"], c["depth"]) for c in manifest["depths"]} == {
    (leg, depth) for leg in ("javac8", "javac23") for depth in (2, 33)}, "four exact cases")
commands = {c["label"]: c for c in manifest["commands"]}
check(len(commands) == len(manifest["commands"]) == 12, "twelve actual commands")
for command in commands.values():
    read(command["stdout"])
    read(command["stderr"])
for leg, tools in manifest["jdk_tools"].items():
    for row in tools.values():
        read(row)
for case in manifest["depths"]:
    leg, depth = case["leg"], case["depth"]
    label = f"{leg}-depth-{depth:02d}"
    compiler = commands[label + "-original-compile"]
    runtime = commands[label + "-original-verify-run"]
    argv = compiler["argv"]
    empty = Path(argv[argv.index("-classpath") + 1])
    classes = Path(argv[argv.index("-d") + 1])
    check(compiler["exit"] == 0, "original complete compile")
    check(empty == Path(argv[argv.index("-sourcepath") + 1]) and not list(empty.iterdir()), "empty CP/SP")
    check(argv[0] == manifest["jdk_tools"][leg]["javac"]["path"], "pinned compiler")
    source = read(case["source"])
    runner = read(case["runner"])
    check(source.count(b"if (") == depth and source.count(b"finally {") == 1, "actual structural source depth")
    check(b"if (trace + x > 1)" in source and b"return x;" in source, "field-entry single saved return")
    check(len(argv) == 15 and Path(argv[-2]).read_bytes() == source and Path(argv[-1]).read_bytes() == runner, "complete original source pair")
    read(case["class_file"])
    read(case["runner_class"])
    check(runtime["argv"] == [manifest["jdk_tools"][leg]["java"]["path"], "-Xverify:all", "-cp", str(classes), "Runner"], "only fresh classes with verifier")
    check(runtime["exit"] == 0, "actual JVM success")
    expected = f"run(0)=0 trace=0\nrun(40)=40 trace={1 + depth * (depth + 1) // 2}\n".encode()
    check(read(runtime["stdout"]) == expected and read(runtime["stderr"]) == b"", "exact two original paths")
    cli_command = commands[label + "-class-source-all-json"]
    cli = case["cli"]
    doc_bytes = read(cli["document_json"])
    check(doc_bytes == read(cli_command["stdout"]), "uncut CLI JSON")
    document = json.loads(doc_bytes)
    methods = [m for m in document["methods"] if m["item"]["name"]["escaped"] == "run"
               and m["item"]["descriptor"]["escaped"] == "(I)I"]
    check(len(methods) == 1, "exact run(I)I physical member")
    report = methods[0]["outcome"]["report"]
    check(read(cli["class_source_text"]).decode() == document["text"], "complete saved class text")
    check(read(cli["run_body_text"]).decode() == report["text"], "verbatim method text")
    if depth == 33:
        check(cli_command["exit"] == cli["cli_exit"] == 4, "actual partial CLI exit")
        check(report["outcome"] == {"stopped": {"interrupted": {"at": 450, "code": "jre_recursion_bound"}}}, "real hard Region bound")
        check(report["content"] == "not_produced", "no method publication")
        check(report["text"] == "" and report["source_map"]["segments"] == [], "no half text or source map")
        check(report["rules"] == report["regions"] == [] and report["init"] is None, "no half claims or initializer")
        check(report["execution"]["status"] == "partial" and report["execution"]["reason"]["code"] == "jre_recursion_bound", "accurate execution reason")
        check(document["execution"]["status"] == "partial", "aggregate partial")
    else:
        check(cli_command["exit"] == 0 and report["outcome"] == "produced", "shallow actual report")
        check(report["content"] == "explanation_only" and report["quality"] == "fallback", "shallow not a recovery success")
        check(report["fallbacks"] == ["jre_guard_finally_copy"], "shallow bounded-child refusal")
        check("the proved finally body has no complete bounded structure" in report["text"], "real shallow explanation")
output = HERE / "root-depth-verification-v2.json"
check(not output.exists(), "no overwrite")
accepted = {"checks": checks, "errors": 0, "original_complete_class_runs": 4,
            "original_paths_per_run": 2, "actual_region_depth_stops": 2,
            "stop_code": "jre_recursion_bound", "stop_bci": 450,
            "not_produced_empty_text_map_rules_regions_init": True,
            "shallow_recovery_success_claimed": False, "shallow_fallback_cases": 2,
            "internal_builder_mid_child_rollback_claimed": False,
            "manifest_sha256": sha(manifest_bytes), "inventory_sha256": sha(inventory_bytes),
            "scope": "Region hard-depth publication contract; no general branch/finally recovery acceptance"}
output.write_text(json.dumps(accepted, indent=2) + "\n")
print(json.dumps(accepted))
