#!/usr/bin/env python3
"""Root audit of unchanged complete shallow sources, hashes and actual command streams."""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
BASE = HERE / "shallow-comparison-root-v1"
CHECKS = 0


def check(condition, label):
    global CHECKS
    CHECKS += 1
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


def main():
    manifest_path = BASE / "manifest.json"
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
    inventory_path = BASE / "file-inventory.json"
    inventory_bytes = inventory_path.read_bytes()
    inventory = json.loads(inventory_bytes)
    actual_paths = {p.relative_to(BASE).as_posix() for p in BASE.rglob("*")
                    if p.is_file() and p != inventory_path}
    check(actual_paths == {r["path"] for r in inventory}, "closed file inventory")
    for row in inventory:
        read(row)
    input_path = Path(manifest["input_result"]["path"])
    check(sha(input_path.read_bytes()) == manifest["input_result"]["sha256"], "input result")
    input_result = json.loads(input_path.read_bytes())
    check(manifest["status"] == "comparison-complete-with-candidate-failures", "actual failure status")
    check(manifest["actual_counts"] == {"jarde": 2, "jadx-default": 2, "jadx-none": 2}, "six legs")
    check(manifest["candidate_success_counts"] == {"jarde": 0, "jadx-default": 2, "jadx-none": 2}, "actual success counts")
    commands = {c["label"]: c for c in manifest["commands"]}
    check(len(commands) == len(manifest["commands"]) == 15, "all fifteen commands, unique")
    for command in commands.values():
        read(command["stdout"])
        read(command["stderr"])
    version = manifest["jadx_version"]["command"]
    check(version["exit"] == 0 and read(version["stdout"]).strip() == b"1.5.6", "exact JADX version")
    for leg in ("javac8", "javac23"):
        for row in manifest["jdk_tools"][leg].values():
            read(row)
        oracle = manifest["original_oracles"][leg]
        check(oracle["exit"] == 0, leg + " original runtime")
        check(read(oracle["stdout"]) == b"run(0)=0 trace=1\nrun(40)=1 trace=2\n", "original values/trace")
        check(read(oracle["stderr"]) == b"", "original stderr")
        rows = manifest["input_inventory"][leg]
        original_runner = read(rows["runner"])
        copied_jarde = read(rows["jarde_source"])
        original_entry = next(c for c in input_result["classes"] if c["leg"] == leg and c["depth"] == 2)
        original_cli_path = input_path.parent / original_entry["cli"]["class_source_text"]["path"]
        check(copied_jarde == original_cli_path.read_bytes(), "verbatim complete Jarde source")
        document = json.loads(read(rows["jarde_document_json"]))
        # The complete report and source are preserved. Do not reclassify a fallback as Stop/depth.
        report_text = json.dumps(document)
        check(all(code in report_text for code in ("jre_guard_finally_copy", "jre_region_loop_shape", "jre_region_uncovered_blocks")), "three actual refusal diagnostics")
        for case in (c for c in manifest["cases"] if c["leg"] == leg):
            compile_command = case["compile"]
            argv = compile_command["argv"]
            empty = Path(argv[argv.index("-classpath") + 1])
            check(empty == Path(argv[argv.index("-sourcepath") + 1]) and not list(empty.iterdir()), "empty CP/SP")
            classes = Path(argv[argv.index("-d") + 1])
            check(argv[0] == manifest["jdk_tools"][leg]["javac"]["path"], "pinned compiler")
            for row in case["sources"]:
                read(row)
            if case["kind"] == "jarde":
                check(compile_command["exit"] != 0 and not case["compile_success"], "real whole-class compile failure")
                check(any(token in read(compile_command["stderr"]) for token in (b"missing return statement", "缺少返回语句".encode())), "actual missing return diagnostic")
                check(case["runtime"] is None and not case["success"], "no execution borrowed from original classes")
                check(read(case["sources"][0]) == copied_jarde and read(case["sources"][1]) == original_runner, "unchanged full input pair")
            else:
                decompile = case["decompile"]
                check(decompile["exit"] == 0, "fresh JADX extraction")
                check(("--rename-flags" in decompile["argv"]) == (case["kind"] == "jadx-none"), "profile separation")
                dest = Path(decompile["argv"][decompile["argv"].index("-d") + 1])
                generated = sorted(str(p) for p in dest.rglob("*.java"))
                source_paths = [str(BASE / row["path"]) for row in case["sources"]]
                check(len(generated) == 1 and source_paths[:-1] == generated, "all generated Java files compiled")
                runner = read(case["sources"][-1]).decode()
                check(re.sub(r"\Apackage [\w.]+;\n\n", "", runner).encode() == original_runner, "fixed complete Runner, only package adaptation")
                check(compile_command["exit"] == 0 and case["compile_success"], "JADX full compile")
                runtime = case["runtime"]
                check(runtime["argv"][:4] == [manifest["jdk_tools"][leg]["java"]["path"], "-Xverify:all", "-cp", str(classes)], "only fresh classes with verifier")
                check(runtime["exit"] == oracle["exit"], "same exit")
                for stream in ("stdout", "stderr"):
                    check(read(runtime[stream]) == read(oracle[stream]), "same raw " + stream)
                check(case["success"] and case["runtime_matches_original"], "correct actual success")
    output = HERE / "shallow-comparison-root-verification-v2.json"
    check(not output.exists(), "no overwrite")
    accepted = {"schema": "cf16-shallow-comparison-root-verification-v2", "checks": CHECKS,
                "errors": 0, "manifest_sha256": sha(manifest_bytes),
                "inventory_sha256": sha(inventory_bytes), "commands": 15,
                "original_legs_reused_from_hash_bound_root_probe": 2,
                "fresh_jadx_complete_legs": 4, "jadx_compile_runtime_matches": 4,
                "jarde_complete_legs": 2, "jarde_compile_failures": 2,
                "depth_limit_pass_claimed": False, "production_change_tested": False,
                "scope": "saved frozen CLI source unchanged, fresh JADX extraction; no recovery implementation acceptance"}
    output.write_text(json.dumps(accepted, indent=2) + "\n")
    print(json.dumps(accepted))


if __name__ == "__main__":
    main()
