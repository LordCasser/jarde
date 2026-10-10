#!/usr/bin/env python3
"""Read-only independent verifier for two typed-local whole-class replays.

This checks existing execution records and raw files only. It never runs a tool.
It intentionally writes no acceptance record.
"""

from __future__ import annotations

import hashlib
from blake3 import blake3
import json
import zipfile
from pathlib import Path


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
CLI = Path("/private/tmp/jarde-proved-local-source-types-cli-v1")
CLI_META = ROOT / "openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_SHA256 = "e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX = Path("/opt/homebrew/bin/jadx").resolve()

CASES = {
    "required": {
        "out": Path("/private/tmp/jarde-required-char-root-v1"),
        "execution_sha256": "a2b069ba2101052f73d9adf69d380587086a93f66793ca22adeb01d9ad0b9c28",
        "class_name": "RequiredConversions",
        "fixture": ROOT / "tests/fixtures/p3-required-conversions/v8/RequiredConversions.class",
        "fixture_sha256": "d71eeaa5dc3eb66eafffda9e0c509eae43e3dde566b67df0006348352778d1e4",
        "class_blake3": "96ed4aee49026f64179075ee3843709aa7e7a5cd02ed2d05f7a11e1b9c385d3d",
        "class_length": 1031,
        "source": ROOT / "tests/fixtures/p3-required-conversions/RequiredConversions.java",
        "source_sha256": "7f9705b16412242c732af0ad2cb7cfb61b02e0b26c95fe077d2797898339c66d",
        "method_rows": [
            ("<init>", "()V"),
            ("castPart", "(C)Ljava/lang/String;"),
            ("castPartLast", "(Ljava/lang/String;C)Ljava/lang/String;"),
            ("intPart", "(I)Ljava/lang/String;"),
            ("widen", "(I)I"),
            ("argued", "(C)I"),
            ("arguedByte", "(B)I"),
            ("returned", "(C)I"),
            ("returnedShort", "(S)I"),
            ("kept", "(C)C"),
            ("declared", "(C)I"),
            ("assigned", "(C)I"),
            ("written", "(C)I"),
        ],
        "runner": (
            'public class Runner { public static void main(String[] args) { long sum=0; '
            'for(int i=0;i<=65535;i++) { int value=RequiredConversions.declared((char)i); '
            'if(value!=i || RequiredConversions.assigned((char)i)!=i || '
            'RequiredConversions.written((char)i)!=i || '
            '!RequiredConversions.castPart((char)i).equals(i+"!") || '
            '!RequiredConversions.castPartLast("x",(char)i).equals("x"+i)) '
            'throw new AssertionError("char="+i+", result="+value); sum+=value; } '
            'System.out.println("chars=65536,sum="+sum); } }\n'
        ),
        "runtime": b"chars=65536,sum=2147450880\n",
        "retained_methods": 13,
    },
    "null_builder": {
        "out": Path("/private/tmp/jarde-null-builder-runtime-root-v1"),
        "execution_sha256": "9d4cbb606405c0928cc79b0820ad36db9bd100ad860e38a6ba9f06475e44052e",
        "class_name": "NullThenBuilder",
        "fixture": ROOT / "tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.class",
        "fixture_sha256": "93acdfe27b312e2c98666a00dbfcb5f61299b03f62ac0dbd916b385232f260b3",
        "class_blake3": "f24b5b45eb94733793e5ee6d7f52f9cfe710d89bdbd65140930822d38f84a77b",
        "class_length": 629,
        "source": None,
        "source_sha256": None,
        "method_rows": [
            ("<init>", "()V"),
            ("run", "(Z)Ljava/lang/Object;"),
            ("main", "([Ljava/lang/String;)V"),
        ],
        "runner": (
            'public class Runner { public static void main(String[] args) { '
            'System.out.println("true="+NullThenBuilder.run(true)); '
            'System.out.println("false="+NullThenBuilder.run(false)); } }\n'
        ),
        "runtime": b"true=7\nfalse=6\n",
        "retained_methods": 3,
    },
}

PROFILES = ("original", "default", "all", "jadx")
LEGS = tuple((jdk, profile) for jdk in ("javac8", "javac23") for profile in PROFILES)
METHOD_COMPARE_KEYS = (
    "text",
    "source_map",
    "method",
    "quality",
    "representation",
    "fallbacks",
    "diagnostics",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def raw_path(out: Path, index: int, stream: str) -> Path:
    return out / f"{index}.{stream}.raw"


def check_recorded_raw(out: Path, row: dict, index: int) -> None:
    require(row["exit_code"] == 0, f"command {index} did not exit 0")
    for stream in ("stdout", "stderr"):
        record = row[stream]
        path = Path(record["path"])
        require(path == raw_path(out, index, stream), f"command {index} {stream} path drift")
        data = path.read_bytes()
        require(len(data) == record["bytes"], f"command {index} {stream} byte count drift")
        require(hashlib.sha256(data).hexdigest() == record["sha256"], f"command {index} {stream} SHA drift")


def bytes_from_raw(value: dict) -> bytes:
    return bytes(value["raw"])


def class_report(out: Path, index: int) -> dict:
    row = json.loads(raw_path(out, index, "stdout").read_text())
    require(row["outcome"] == "performed", f"report {index} not performed")
    return row


def verify_reports(case: dict, d: dict) -> tuple[dict, dict]:
    out = case["out"]
    default = class_report(out, 0)
    all_evidence = class_report(out, 1)
    require(len(default["methods"]) == case["retained_methods"], "default method count")
    require(len(all_evidence["methods"]) == case["retained_methods"], "all method count")
    require(default["text"] == all_evidence["text"], "default/all whole-class text differs")
    require(default["fields"] == all_evidence["fields"], "default/all field table differs")
    require(default["class"] == all_evidence["class"], "default/all class identity differs")
    require(default["class"]["class_bytes"] == {"digest": case["class_blake3"], "length": case["class_length"]}, "class report BLAKE3/length mismatch")

    expected_rows = case["method_rows"]
    for report in (default, all_evidence):
        methods = report["methods"]
        actual_rows = []
        for index, item in enumerate(methods):
            method = item["item"]
            name = bytes_from_raw(method["name"]).decode("ascii")
            descriptor = bytes_from_raw(method["descriptor"]).decode("ascii")
            actual_rows.append((name, descriptor))
            require(method["index"] == index, f"member order/index drift at {name}{descriptor}")
            identity = method["identity"]
            require(bytes(identity["name"]).decode("ascii") == name, "identity name mismatch")
            require(bytes(identity["descriptor"]).decode("ascii") == descriptor, "identity descriptor mismatch")
            owner_bytes = identity["owner"]["class_bytes"]
            require(owner_bytes == {"digest": case["class_blake3"], "length": case["class_length"]}, "method owner BLAKE3/length mismatch")
            require(item["markers"] == [], f"method has markers: {name}{descriptor}")
            require(item["outcome"]["kind"] == "recovered", f"method not recovered: {name}{descriptor}")
            result = item["outcome"]["report"]
            require(result["outcome"] == "produced", f"method not produced: {name}{descriptor}")
            require(result["quality"] == "structured", f"method quality: {name}{descriptor}")
            require(result["representation"] == "java", f"method representation: {name}{descriptor}")
            require(result["fallbacks"] == [], f"method fallback: {name}{descriptor}")
            require(isinstance(result["source_map"], dict) and isinstance(result["source_map"].get("segments"), list), f"source map absent: {name}{descriptor}")
            require(bool(result["source_map"]["segments"]), f"source map has no segments: {name}{descriptor}")
            for segment in result["source_map"]["segments"]:
                require(0 <= segment["start"] < segment["end"] <= len(result["text"].encode()), "map span invalid")
                for origin in [segment["origin"]["primary"], *segment["origin"]["derived"]]:
                    require(origin["method"] == identity, "origin owner mismatch")
        require(actual_rows == expected_rows, f"member identity/order mismatch: {actual_rows!r}")

    # The field is part of the complete RequiredConversions class contract.
    if case["class_name"] == "RequiredConversions":
        fields = default["fields"]
        require(len(fields) == 1, "RequiredConversions field count")
        field = fields[0]
        require(field["item"]["name"]["escaped"] == "field", "field name mismatch")
        require(field["item"]["descriptor"]["escaped"] == "I", "field descriptor mismatch")
        require(field["item"]["identity"]["member"] == {"descriptor": [73], "kind": "field", "name": [102, 105, 101, 108, 100]}, "field identity mismatch")
        require(field["item"]["identity"]["owner"]["class_bytes"] == {"digest": case["class_blake3"], "length": case["class_length"]}, "field owner BLAKE3/length mismatch")
        require(field["declaration"] == "static int field", "field declaration mismatch")
        required_text = default["text"]
        require("char local1 = arg0;" in required_text, "declared char source local missing")
        require("char local1 = '\\u0000';" in required_text and "local1 = arg0;" in required_text, "assigned char source local missing")
    else:
        require(default["fields"] == [], "NullThenBuilder should have no fields")
        run_text = next(m["outcome"]["report"]["text"] for m in default["methods"] if m["item"]["name"]["escaped"] == "run")
        require("java.lang.StringBuilder local2;" in run_text, "null-leading local was not recovered as StringBuilder")
        require("local2 = null;" in run_text and 'local2 = new java.lang.StringBuilder("second");' in run_text, "null-leading assignment body changed")

    # Evidence selection may change auxiliary analysis data, but these public method facts must not.
    for left, right in zip(default["methods"], all_evidence["methods"]):
        require(left["item"] == right["item"], "default/all item differs")
        require(left["outcome"]["kind"] == right["outcome"]["kind"], "default/all outcome kind differs")
        lreport, rreport = left["outcome"]["report"], right["outcome"]["report"]
        for key in METHOD_COMPARE_KEYS:
            require(lreport[key] == rreport[key], f"default/all {key} differs for {lreport['method']}")
    return default, all_evidence


def verify_case_tools_and_commands(case: dict, d: dict, metadata: dict, tools: dict) -> None:
    out = case["out"]
    require(d["schema"] == "meet-char-int-full-class-comparison-root-v1", "execution schema")
    require(d["status"] == "passed", "collector did not report passed")
    require(len(d["commands"]) == 19, "expected exactly 19 commands")
    require(len(d["legs"]) == 8, "expected exactly 8 runtime legs")
    require({(leg["jdk"], leg["profile"]) for leg in d["legs"]} == set(LEGS), "runtime matrix differs")
    require(d["fixture_sha256"] == case["fixture_sha256"], "fixture SHA in execution differs")
    require(sha256(case["fixture"]) == case["fixture_sha256"], "fixture SHA on disk differs")
    require(blake3(case["fixture"].read_bytes()).hexdigest() == case["class_blake3"], "independent BLAKE3 mismatch")
    require(d["cli_sha256"] == CLI_SHA256 == metadata["cli_sha256"], "CLI SHA provenance differs")
    require(metadata["cli_path"] == str(CLI), "metadata CLI path differs")
    require(sha256(CLI) == CLI_SHA256, "live CLI SHA differs")
    require((CLI.stat().st_mode & 0o777) == 0o555, "frozen CLI mode differs")
    require(d["runner_sha256"] == hashlib.sha256(case["runner"].encode()).hexdigest(), "runner source SHA differs")
    require(d["full_class_methods_retained"] == case["retained_methods"], "retained member count differs")
    if case["class_name"] == "RequiredConversions":
        require(d["original_source_sha256"] == case["source_sha256"], "original Java source SHA in execution differs")
        require(sha256(case["source"]) == case["source_sha256"], "original Java source SHA on disk differs")
        require(d["characters_per_runtime_leg"] == 65536, "character domain count differs")
    else:
        require(d["original_source_available"] is False, "NullThenBuilder must state original source unavailable")
        require(all(leg["source_sha256"] is None for leg in d["legs"] if leg["profile"] == "original"), "original NullThenBuilder source must be absent")

    for index, row in enumerate(d["commands"]):
        check_recorded_raw(out, row, index)
    require([row["argv"][0] for row in d["commands"][:2]] == [str(CLI), str(CLI)], "class-source CLI path/order")
    fixture_path = str(case["fixture"])
    base_cli = [str(CLI), "class-source", "--input", fixture_path, "--class", case["class_name"], "--policy", "single-class", "--release", "8", "--format", "json"]
    require(d["commands"][0]["argv"] == base_cli, "default CLI argv differs")
    require(d["commands"][1]["argv"] == base_cli + ["--evidence", "all"], "all-evidence CLI argv differs")
    require(d["commands"][2]["argv"] == [str(JADX), "--no-res", "--config", "none", "--threads-count", "1", "-d", str(out / "jadx-output"), str(out / "input.jar")], "JADX argv differs")
    require(sha256(JADX) == JADX_SHA256, "live JADX SHA differs")
    with zipfile.ZipFile(out / "input.jar") as jar:
        require(jar.namelist() == [case["class_name"] + ".class"], "JADX input jar contents differ")
        require(jar.read(jar.namelist()[0]) == case["fixture"].read_bytes(), "JADX input class bytes differ")

    for leg_index, (jdk, profile) in enumerate(LEGS):
        compile_index = 3 + leg_index * 2
        runtime_index = compile_index + 1
        compile_row, runtime_row = d["commands"][compile_index], d["commands"][runtime_index]
        leg = next(item for item in d["legs"] if (item["jdk"], item["profile"]) == (jdk, profile))
        require(leg["runtime"] == runtime_row, f"runtime record differs for {jdk}/{profile}")
        tool = tools[jdk]
        work = out / jdk / profile
        classes = work / "classes"
        empty = work / "empty"
        package = profile == "jadx"
        source = work / (case["class_name"] + ".java")
        runner_path = work / "Runner.java"
        expected_runner = ("package defpackage;\n" if package else "") + case["runner"]
        require(runner_path.read_text() == expected_runner, f"Runner source changed for {jdk}/{profile}")
        require(sha256(runner_path) == leg["runner_sha256"], f"Runner SHA differs for {jdk}/{profile}")

        if profile == "original":
            if case["source"] is None:
                require(not source.exists(), "original NullThenBuilder source unexpectedly exists")
                require(leg["source_sha256"] is None, "original NullThenBuilder source SHA must be null")
                require((classes / (case["class_name"] + ".class")).read_bytes() == case["fixture"].read_bytes(), "original NullThenBuilder class is not canonical fixture bytes")
                source_args = [str(runner_path)]
                classpath = classes
            else:
                require(source.read_bytes() == case["source"].read_bytes(), "original Java source bytes differ")
                require(sha256(source) == leg["source_sha256"] == case["source_sha256"], "original source SHA differs")
                source_args = [str(source), str(runner_path)]
                classpath = empty
        else:
            expected_source = (
                (out / "jadx-output" / "sources" / "defpackage" / (case["class_name"] + ".java")).read_bytes()
                if profile == "jadx"
                else class_report(out, 0)["text"].encode()
            )
            require(source.read_bytes() == expected_source, f"full class source bytes differ for {jdk}/{profile}")
            require(sha256(source) == leg["source_sha256"], f"full class source SHA differs for {jdk}/{profile}")
            source_args = [str(source), str(runner_path)]
            classpath = empty

        flags = ["-source", "8", "-target", "8"] if jdk == "javac8" else ["--release", "8"]
        expected_compile = [
            str(tool["javac"]), "-J-Duser.language=en", "-J-Duser.country=US", "-encoding", "UTF-8",
            *flags, "-g:none", "-proc:none", "-classpath", str(classpath), "-sourcepath", str(empty),
            "-d", str(classes), *source_args,
        ]
        require(compile_row["argv"] == expected_compile, f"compile argv differs for {jdk}/{profile}")
        main = "defpackage.Runner" if package else "Runner"
        require(runtime_row["argv"] == [str(tool["java"]), "-Xverify:all", "-cp", str(classes), main], f"runtime argv differs for {jdk}/{profile}")
        require(runtime_row["exit_code"] == 0, f"runtime did not exit 0 for {jdk}/{profile}")
        require(Path(runtime_row["stdout"]["path"]).read_bytes() == case["runtime"], f"runtime stdout differs for {jdk}/{profile}")
        require(Path(runtime_row["stderr"]["path"]).read_bytes() == b"", f"runtime stderr is not empty for {jdk}/{profile}")
        class_file = classes / ("defpackage/" if package else "") / (case["class_name"] + ".class")
        runner_class = classes / ("defpackage/" if package else "") / "Runner.class"
        require(class_file.is_file() and runner_class.is_file(), f"compiled whole class/runner missing for {jdk}/{profile}")


def main() -> None:
    require(sha256(CLI_META) == "6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943", "metadata SHA drift")
    metadata = json.loads(CLI_META.read_text())
    require(metadata["schema"] == "recover-proved-local-source-types-candidate-cli-root-v1", "CLI metadata schema")
    require(metadata["cli_sha256"] == CLI_SHA256 and metadata["cli_mode"] == "0o555", "metadata CLI freeze differs")
    require(metadata["uncommitted_local_source_types_product"] is True, "metadata product state differs")
    require(sha256(JDK_MANIFEST) == JDK_MANIFEST_SHA256, "JDK manifest SHA differs")
    manifest = json.loads(JDK_MANIFEST.read_text())
    tools = {}
    for row in manifest["legs"]:
        name = row["leg"]
        tools[name] = {}
        for kind, fact in row["jdk_tools"].items():
            path = Path(fact["path"]).resolve(strict=True)
            require(path.is_file() and sha256(path) == fact["sha256"], f"live {name}/{kind} SHA differs")
            tools[name][kind] = path

    results = {}
    for label, case in CASES.items():
        execution_path = case["out"] / "execution.json"
        require(sha256(execution_path) == case["execution_sha256"], f"{label} execution SHA differs")
        d = json.loads(execution_path.read_text())
        reports = verify_reports(case, d)
        verify_case_tools_and_commands(case, d, metadata, tools)
        results[label] = {
            "execution_sha256": sha256(execution_path),
            "fixture_sha256": case["fixture_sha256"],
            "class_blake3_from_reports": case["class_blake3"],
            "commands": len(d["commands"]),
            "runtime_legs": len(d["legs"]),
            "members": len(reports[0]["methods"]),
        }
    print(json.dumps({"verified_observations": results, "acceptance_record_written": False}, indent=2))


if __name__ == "__main__":
    main()
