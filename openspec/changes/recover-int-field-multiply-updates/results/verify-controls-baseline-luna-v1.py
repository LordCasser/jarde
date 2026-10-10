#!/usr/bin/env python3
"""Independently authenticate the full-class multiply-controls baseline."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import sys
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/recover-int-field-multiply-updates"
BASE = CHANGE / "results/controls-prepared-luna-v1/baseline-root-v1"
RESULT = CHANGE / "results/controls-baseline-independent-acceptance-luna-v1.json"
MANIFEST_SHA256 = "7538e33c5b687efbba53bf6efcee55be9ae2668519fd96fbc7b46b4a85832b75"
INVENTORY_SHA256 = "13528c4a32bfcf1d12d808c6a9d81f990d4bb7f2dfb989ed04bb94a854646ba9"
HELPER_PATH = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/results/verify-baseline-luna-v3.py"
HELPER_SHA256 = "efead211ceb2ecde7540a4d0a8a0493f74a47009899be0dff603f8afcbfb9161"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JARDE_PATH = Path("/private/tmp/jarde-int-array-names-cli-v2")
JARDE_SHA256 = "51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067"
JARDE_METADATA = ROOT / "openspec/changes/recover-int-array-constant-names/results/candidate-cli-v2.json"
JARDE_METADATA_SHA256 = "dd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_LAUNCHER = "/opt/homebrew/bin/jadx"
JADX_RESOLVED = "/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx"
JAVA8 = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"
JAVA23 = "/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"
SOURCE_SHA256 = "8ba481b7c116d9e8a2386bc6be3295642d28af69b58bd334b933e3b4828150a3"
SOURCE_BLAKE3 = "2ed0efc1bc77177abe5b225c8dee889cd96084cb984c69c8095d4758f22fc295"
RUNNER_SHA256 = "940bb4d04c0d1f125e0ba3ad33d1abab6fb92bfbaa5ddcc918a8041aaeb9fd9e"
RUNNER_BLAKE3 = "acae07bd84b50c097d184d5e375b36c54794e9bb9a1533a3256fceaa112c2b7d"
OUTER = "em23/InputFieldMultiplyControls.class"
CHILD = "em23/InputFieldMultiplyControls$A.class"
RUNNER_CLASS = "em23/Runner.class"
TARGETS = {OUTER, CHILD}
CLASS_PATHS = TARGETS | {RUNNER_CLASS}
EXPECTED_STDOUT = (
    b"explicit-add=8\n"
    b"multiply-normal=20\n"
    b"multiply-max=2147483645\n"
    b"multiply-min-neg1=-2147483648\n"
    b"multiply-zero=0\n"
    b"multiply-negative=-15\n"
    b"multiply-null=NullPointerException\n"
    b"multiply-divide-normal=12,field=12\n"
    b"multiply-divide-zero=ArithmeticException,field=17\n"
    b"multiply-divide-null-zero=NullPointerException\n"
)
EXPECTED_PHYSICAL = {
    OUTER: {
        "fields": [("a", "Lem23/InputFieldMultiplyControls$A;", 0x0001)],
        "methods": [
            ("<init>", "()V", 0x0001, [0, 1, 4]),
            ("test1", "(I)V", 0x0001, [0, 1, 4, 5, 8, 11, 12, 13, 16]),
            ("test2", "(I)V", 0x0001, [0, 1, 4, 5, 8, 9, 10, 13]),
            ("multiplyDivide", "(I)I", 0x0001, [0, 1, 4, 5, 8, 10, 11, 12, 13, 16, 17, 20, 23]),
        ],
    },
    CHILD: {
        "fields": [("f", "I", 0x0000)],
        "methods": [("<init>", "()V", 0x0002, [0, 1, 4, 5, 6, 9])],
    },
}


def load_accepted_helpers():
    require(sha256(HELPER_PATH.read_bytes()) == HELPER_SHA256, "accepted EM23 helper SHA-256 changed")
    spec = importlib.util.spec_from_file_location("accepted_em23_baseline_verifier_v3", HELPER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load the pinned EM23 helper module")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def close_inventory(h):
    manifest_path, inventory_path = BASE / "manifest.json", BASE / "file-inventory.json"
    require(sha256(manifest_path.read_bytes()) == MANIFEST_SHA256, "controls manifest SHA-256 mismatch")
    require(sha256(inventory_path.read_bytes()) == INVENTORY_SHA256, "controls inventory SHA-256 mismatch")
    manifest, inventory = read_json(manifest_path), read_json(inventory_path)
    require(manifest["schema"] == "em23-nested-field-multiply-controls-luna-v1"
            and manifest["status"] == "baseline-with-failures", "unexpected controls baseline identity/status")
    require(manifest["file_inventory"] == {
        "path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
        "excludes": ["file-inventory.json"],
    }, "controls inventory policy changed")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory) == 126, "controls inventory count or duplicate paths differ")
    actual = set()
    for path in BASE.rglob("*"):
        require(not path.is_symlink(), f"symlink in controls evidence tree: {path}")
        if path.is_file() and path != inventory_path:
            actual.add(path.relative_to(BASE).as_posix())
    require(actual == set(rows),
            f"controls inventory is not closed: missing={sorted(set(rows)-actual)}, extra={sorted(actual-set(rows))}")
    for row in inventory:
        h.recorded_bytes(BASE, row)
    prepared = manifest["prepared_script"]
    prepared_path = Path(prepared["path"])
    require(prepared_path.is_file() and sha256(prepared_path.read_bytes()) == prepared["sha256"],
            "controls collector source changed")
    return manifest, rows


def verify_tool_pins(manifest, h):
    jdk_ref = manifest["jdk_manifest"]
    jdk_bytes = Path(jdk_ref["path"]).read_bytes()
    require(sha256(jdk_bytes) == JDK_MANIFEST_SHA256 == jdk_ref["sha256"], "pinned JDK manifest changed")
    jdk = json.loads(jdk_bytes)
    toolchain_ref = jdk["toolchain_manifest"]
    toolchain_bytes = Path(toolchain_ref["path"]).read_bytes()
    require(sha256(toolchain_bytes) == toolchain_ref["sha256"], "JDK source toolchain manifest changed")
    toolchain = json.loads(toolchain_bytes)
    control_legs = {row["leg"]: row for row in jdk["legs"]}
    source_legs = {row["leg"]: row for row in toolchain["legs"]}
    expected_homes = {"javac8": JAVA8, "javac23": JAVA23}
    require(set(control_legs) == set(source_legs) == set(expected_homes), "JDK leg census changed")
    normalized = {}
    for leg, home in expected_homes.items():
        recorded_leg = manifest["jdk_legs"][leg]
        require(recorded_leg["home"] == home, f"unexpected JDK home: {leg}")
        control, source = control_legs[leg], source_legs[leg]
        require(set(control["jdk_tools"]) == set(source["tools"]) == {"javac", "java", "javap"},
                f"JDK tool census changed: {leg}")
        for tool in ("javac", "java", "javap"):
            expected = recorded_leg["tools"][tool]
            for pin in (control["jdk_tools"][tool], source["tools"][tool]):
                require((expected["path"], expected["sha256"]) == (pin["path"], pin["sha256"]),
                        f"JDK tool pin differs: {leg}/{tool}")
            require(str(Path(expected["path"]).parent.parent) == home, f"JDK binary outside home: {leg}/{tool}")
            require(sha256(Path(expected["path"]).read_bytes()) == expected["sha256"],
                    f"JDK binary hash mismatch: {leg}/{tool}")
        normalized[leg] = {"home": home, "tools": recorded_leg["tools"]}

    cli = manifest["frozen_jarde_cli"]
    require(cli["path"] == str(JARDE_PATH) and cli["sha256"] == JARDE_SHA256,
            "frozen Jarde CLI identity changed")
    require(sha256(JARDE_PATH.read_bytes()) == JARDE_SHA256, "frozen Jarde CLI binary hash mismatch")
    require(cli["metadata_path"] == str(JARDE_METADATA)
            and cli["metadata_sha256"] == JARDE_METADATA_SHA256, "frozen Jarde metadata identity changed")
    metadata_bytes = JARDE_METADATA.read_bytes()
    require(sha256(metadata_bytes) == JARDE_METADATA_SHA256, "frozen candidate metadata hash mismatch")
    require(h.recorded_bytes(BASE, cli["metadata_copy"]) == metadata_bytes,
            "archived candidate metadata differs from frozen metadata")
    metadata = json.loads(metadata_bytes)
    require(metadata["cli_path"] == str(JARDE_PATH) and metadata["cli_sha256"] == JARDE_SHA256,
            "candidate metadata does not identify the frozen CLI")

    jadx = manifest["jadx"]
    require(jadx["expected_version"] == "1.5.6" and jadx["sha256"] == JADX_SHA256,
            "JADX version/binary pin changed")
    require(jadx["launcher"] == JADX_LAUNCHER and jadx["resolved_launcher"] == JADX_RESOLVED,
            "JADX launcher resolution changed")
    require(sha256(Path(JADX_RESOLVED).read_bytes()) == JADX_SHA256, "JADX binary hash mismatch")
    preflight = {row["label"]: row for row in manifest["preflight"]}
    expected_preflight = {
        "fixed-jdk-manifest", "javac8:javac", "javac8:java", "javac8:javap",
        "javac23:javac", "javac23:java", "javac23:javap", "jdk-leg-set",
        "supplied-frozen-jarde-cli", "supplied-frozen-jarde-metadata", "fixed-jadx-launcher",
    }
    require(set(preflight) == expected_preflight and all(row["ok"] for row in preflight.values()),
            "recorded preflight pins failed or changed")
    return {"legs": normalized}


def verify_execution_policy(manifest):
    expected = {
        "removed_environment": ["JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"],
        "complete_compile": ["-source", "8", "-target", "8", "-g:none"],
        "empty_classpath_sourcepath": True,
        "runtime_verifier": "-Xverify:all",
        "JADX_input_is_exact_javac23_outer_and_static_nested_classes": True,
        "Jarde_plain_jar_selects_outer_and_records_full_member_family": True,
        "operator_presentation_and_member_family_state_are_observational_only": True,
        "generated_product_sources_unmodified": True,
        "only_runner_package_adaptation_allowed": True,
        "Jarde_default_and_all_root_texts_compiled_on_both_JDKs": True,
    }
    require(manifest["execution_policy"] == expected, "controls execution policy changed")


def verify_commands(manifest, inventory, h):
    commands = manifest["commands"]
    require(len(commands) == 33, "expected 33 controls commands")
    by_label = {row["label"]: row for row in commands}
    require(len(by_label) == len(commands), "duplicate controls command labels")
    expected = {"jadx-version", "jadx-default-decompile", "jadx-none-decompile"}
    for leg in ("javac8", "javac23"):
        expected |= {f"{leg}-{tool}-version" for tool in ("java", "javac", "javap")}
        expected |= {f"{leg}-original-{suffix}" for suffix in ("compile", "run")}
        expected |= {f"{leg}-javap-InputFieldMultiplyControls{suffix}" for suffix in ("", "$A")}
        for profile in ("default", "none"):
            expected |= {f"{leg}-jadx-{profile}-{suffix}" for suffix in ("compile", "run")}
        for profile in ("default", "all"):
            expected |= {f"{leg}-jarde-{profile}-{suffix}" for suffix in ("render", "compile")}
    require(set(by_label) == expected, "controls command label set differs from the complete 33-command plan")

    failed_compile_labels = {f"{leg}-jarde-{profile}-compile"
                             for leg in ("javac8", "javac23") for profile in ("default", "all")}
    for label, command in by_label.items():
        require(command["cwd"] == str(ROOT), f"unexpected command cwd: {label}")
        require(command["exit"] == (1 if label in failed_compile_labels else 0),
                f"unexpected command exit: {label}")
        for stream in ("stdout", "stderr"):
            raw = h.recorded_bytes(BASE, command["streams"][stream])
            require(command["streams"][stream]["path"] in inventory, f"un-inventoried command stream: {label}/{stream}")
            require(len(raw) == command["streams"][stream]["bytes"], f"stream byte count differs: {label}/{stream}")

    for leg, home in (("javac8", JAVA8), ("javac23", JAVA23)):
        for tool in ("java", "javac", "javap"):
            cmd = by_label[f"{leg}-{tool}-version"]
            require(cmd["argv"] == [f"{home}/bin/{tool}", "-version"] and cmd["java_home"] == home,
                    f"wrong tool version command: {leg}/{tool}")
    require(by_label["jadx-version"]["argv"] == [JADX_LAUNCHER, "--version"], "wrong JADX version argv")
    for profile in ("default", "none"):
        cmd = by_label[f"jadx-{profile}-decompile"]
        expected_argv = [JADX_LAUNCHER, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            expected_argv += ["--rename-flags", "none"]
        expected_argv += ["-d", str(BASE / f"jadx-output/{profile}"),
                          str(BASE / "jadx-input/InputFieldMultiplyControls-family.jar")]
        require(cmd["argv"] == expected_argv, f"wrong JADX {profile} argv")
        profile_row = next(row for row in manifest["jadx"]["profiles"] if row["profile"] == profile)
        require(profile_row["command"] == cmd and profile_row["decompile_success"],
                f"JADX {profile} command/status linkage differs")
    return by_label


def verify_case_compile_and_runtime(case, commands, source, runner, jdk, oracle, h):
    label, leg, kind = case["label"], case["jdk_leg"], case["kind"]
    class_dir = BASE / f"cases/{label}/classes"
    isolation = BASE / f"cases/{label}/empty-classpath-sourcepath"
    require(Path(case["class_output"]) == class_dir, f"class output differs: {label}")
    require(Path(case["empty_classpath_sourcepath"]) == isolation and isolation.is_dir()
            and not any(isolation.iterdir()), f"classpath/sourcepath not empty: {label}")
    require(case["expected_class_paths"] == sorted(CLASS_PATHS) and case["class_set_exact"] is True,
            f"whole class family census not exact: {label}")
    product = case["product_sources"][0]
    product_bytes = h.recorded_bytes(BASE, product)
    runner_bytes = h.recorded_bytes(BASE, case["runner_source"])
    require(runner_bytes == runner, f"Runner changed in compile case: {label}")
    if kind == "original":
        require(product_bytes == source and case["source_pins"] == {
            "InputFieldMultiplyControls.java": SOURCE_SHA256, "Runner.java": RUNNER_SHA256,
        }, f"original source identity differs: {label}")
    elif kind == "jadx":
        generated = case["decompilation"]["generated_sources"][0]
        require(product_bytes == h.recorded_bytes(BASE, generated), f"JADX source was edited: {label}")
        require(case["decompilation"]["profile"] == case["profile"]
                and case["decompilation"]["input_jar_exact"], f"JADX source profile/input mismatch: {label}")
    else:
        raise AssertionError(f"unexpected compile/runtime case kind: {kind}")
    require(product["path"].startswith(f"cases/{label}/"), f"source path escapes case: {label}")
    compile_cmd, run_cmd = case["compile"], case["runtime"]
    require(commands[compile_cmd["label"]] == compile_cmd and commands[run_cmd["label"]] == run_cmd,
            f"compile/runtime command linkage differs: {label}")
    home = jdk["legs"][leg]["home"]
    require(compile_cmd["java_home"] == home and run_cmd["java_home"] == home,
            f"compile/runtime JDK differs: {label}")
    javac, java = jdk["legs"][leg]["tools"]["javac"]["path"], jdk["legs"][leg]["tools"]["java"]["path"]
    compile_argv = [javac, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                    "-classpath", str(isolation), "-sourcepath", str(isolation), "-d", str(class_dir),
                    str(BASE / product["path"]), str(BASE / case["runner_source"]["path"])]
    require(compile_cmd["argv"] == compile_argv, f"full-source isolated javac argv differs: {label}")
    require(run_cmd["argv"] == [java, "-Xverify:all", "-cp", str(class_dir), "em23.Runner"],
            f"fresh JVM verification argv differs: {label}")
    expected_files = {f"cases/{label}/classes/{path}" for path in CLASS_PATHS}
    require({row["path"] for row in case["classes"]} == expected_files,
            f"compiled class record set differs: {label}")
    actual_files = {p.relative_to(class_dir).as_posix() for p in class_dir.rglob("*.class")}
    require(actual_files == CLASS_PATHS == set(case["actual_class_paths"]),
            f"actual generated class set differs: {label}")
    for row in case["classes"]:
        h.recorded_bytes(BASE, row)
    require(case["compile_success"] and case["runtime_success"] and case["success"],
            f"original/JADX compile or runtime failed: {label}")
    stdout = h.recorded_bytes(BASE, run_cmd["streams"]["stdout"])
    stderr = h.recorded_bytes(BASE, run_cmd["streams"]["stderr"])
    triple = {"exit": run_cmd["exit"], "stdout": stdout, "stderr": stderr}
    require(triple == oracle[leg], f"complete Runner raw triple differs from original oracle: {label}")
    require(stdout == EXPECTED_STDOUT and stderr == b"" and run_cmd["exit"] == 0,
            f"Runner raw output differs from fixed boundary/exception oracle: {label}")
    return {"label": label, "compile_exit": compile_cmd["exit"], "runtime_exit": run_cmd["exit"],
            "stdout_sha256": sha256(stdout), "stderr_sha256": sha256(stderr),
            "whole_source_class_set": True, "raw_oracle_equal": True}


def verify_failed_jarde_cases(manifest, commands, runner, h):
    cases = {case["label"]: case for case in manifest["cases"]}
    expected = {f"javac{version}-jarde-{profile}" for version in (8, 23) for profile in ("default", "all")}
    require(set(cases) == {"javac8-original", "javac23-original",
                           "javac8-jadx-default", "javac23-jadx-default",
                           "javac8-jadx-none", "javac23-jadx-none"} | expected,
            "controls case labels differ from 2 original + 4 JADX + 4 Jarde legs")
    failures = {}
    renders = {row["label"]: row for row in manifest["jarde_render_cases"]}
    for label in sorted(expected):
        case = cases[label]
        leg = case["jdk_leg"]
        require(case["kind"] == "jarde" and case["success"] is False
                and case["compile_success"] is False and case["runtime_success"] is False
                and case["runtime"] is None, f"Jarde failure boundary changed: {label}")
        require(case["expected_class_paths"] == sorted(CLASS_PATHS)
                and case["class_set_exact"] is False
                and case["actual_class_paths"] == [] and case["classes"] == [],
                f"failed Jarde compile emitted a class set: {label}")
        render = renders[f"{leg}-jarde-{case['evidence_mode']}"]
        product = h.recorded_bytes(BASE, case["product_sources"][0])
        require(product == h.recorded_bytes(BASE, render["generated_text"]),
                f"Jarde compile source differs from complete rendered output: {label}")
        require(h.recorded_bytes(BASE, case["runner_source"]) == runner
                and h.recorded_bytes(BASE, case["runner_adaptation"]) == runner,
                f"Jarde source or Runner adaptation changed: {label}")
        cmd = case["compile"]
        require(commands[cmd["label"]] == cmd and cmd["exit"] == 1,
                f"expected Jarde compiler failure absent: {label}")
        stderr = h.recorded_bytes(BASE, cmd["streams"]["stderr"]).decode("utf-8", errors="replace")
        require("jarde_refused_body" in stderr and ("找不到符号" in stderr or "cannot find symbol" in stderr),
                f"Jarde compile diagnostic does not identify the refused-body symbol: {label}")
        source_text = product.decode("utf-8")
        require("jarde_refused_body();" in source_text
                and "the copy at BCI 4 has no proved local assignment" in source_text,
                f"Jarde full source does not preserve the test2 fallback: {label}")
        class_dir = BASE / f"cases/{label}/classes"
        isolation = BASE / f"cases/{label}/empty-classpath-sourcepath"
        require(Path(case["class_output"]) == class_dir and class_dir.is_dir()
                and not any(class_dir.rglob("*.class")), f"failed Jarde compile produced classes: {label}")
        require(Path(case["empty_classpath_sourcepath"]) == isolation and isolation.is_dir()
                and not any(isolation.iterdir()), f"Jarde compiler environment not empty: {label}")
        javac = manifest["jdk_legs"][leg]["tools"]["javac"]["path"]
        require(cmd["java_home"] == manifest["jdk_legs"][leg]["home"],
                f"Jarde compile used the wrong JDK home: {label}")
        expected_argv = [javac, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                         "-classpath", str(isolation), "-sourcepath", str(isolation), "-d", str(class_dir),
                         str(BASE / case["product_sources"][0]["path"]),
                         str(BASE / case["runner_source"]["path"])]
        require(cmd["argv"] == expected_argv, f"Jarde whole-source compile/isolation argv differs: {label}")
        failures[label] = {"compile_exit": 1, "runtime_attempted": False,
                           "stderr_sha256": sha256(h.recorded_bytes(BASE, cmd["streams"]["stderr"])),
                           "fallback_symbol_unresolved": True}
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["expected_case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["success_counts"] == {"original": 2, "jadx": 4, "jarde": 0},
            "controls baseline counts hide or rewrite the four Jarde compile failures")
    return failures


def verify_jarde_reports(manifest, commands, original_jars, original_classes, census, h):
    rows = {row["label"]: row for row in manifest["jarde_render_cases"]}
    expected_labels = {f"javac{version}-jarde-{mode}" for version in (8, 23) for mode in ("default", "all")}
    require(set(rows) == expected_labels, "Jarde render rows incomplete")
    by_leg = {}
    for leg in ("javac8", "javac23"):
        root_owner = h.owner_identity(original_classes[leg][OUTER], original_jars[leg], OUTER)
        child_owner = h.owner_identity(original_classes[leg][CHILD], original_jars[leg], CHILD)
        modes = {}
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            row = rows[label]
            cmd = row["command"]
            require(row["success"] and cmd["exit"] == 0 and commands[cmd["label"]] == cmd,
                    f"Jarde render failed or command linkage changed: {label}")
            require(cmd["java_home"] == manifest["jdk_legs"][leg]["home"],
                    f"Jarde render recorded the wrong JDK home: {label}")
            argv = [str(JARDE_PATH), "class-source", "--input",
                    str(BASE / f"cases/{leg}-original/InputFieldMultiplyControls-family.jar"),
                    "--class", "em23/InputFieldMultiplyControls", "--policy", "plain-jar",
                    "--release", "8", "--format", "json"]
            if mode == "all":
                argv += ["--evidence", "all"]
            require(cmd["argv"] == argv, f"Jarde render argv differs: {label}")
            raw_doc = h.recorded_bytes(BASE, row["document"])
            require(raw_doc == h.recorded_bytes(BASE, cmd["streams"]["stdout"]),
                    f"Jarde JSON differs from command stdout: {label}")
            doc = json.loads(raw_doc)
            root_text = doc["text"].encode("utf-8")
            require(root_text == h.recorded_bytes(BASE, row["generated_text"]),
                    f"Jarde root text differs from recorded generated source: {label}")
            require(row["mode"] == mode and row["input_class_selection"] == "em23/InputFieldMultiplyControls",
                    f"Jarde mode or class selection differs: {label}")
            jar_bytes = original_jars[leg]
            require(row["input_family_jar"] == {
                "path": f"cases/{leg}-original/InputFieldMultiplyControls-family.jar",
                "bytes": len(jar_bytes), "sha256": sha256(jar_bytes),
                "blake3": h.blake3(jar_bytes).hexdigest(),
            }, f"Jarde input family jar differs: {label}")
            require(doc["outcome"] == "performed" and doc["execution"]["status"] == "complete",
                    f"Jarde source report incomplete: {label}")
            require(row["member_family_kind"] == "prepared_static"
                    and row["member_family_projected"] is True
                    and row["member_family_projection_state"] == "projected",
                    f"static nested member family was not completely projected: {label}")
            family = doc["member_family"]["members"]
            require(len(family) == 1, f"nested member-family census differs: {label}")
            child_doc = family[0]["child"]
            require(family[0]["relation"]["root"] == root_owner
                    and family[0]["relation"]["child"] == child_owner,
                    f"root/child archive owner identity differs: {label}")
            outer = h.class_members(doc, root_owner, census[leg][OUTER], f"{label}/outer")
            child = h.class_members(child_doc, child_owner, census[leg][CHILD], f"{label}/child")
            require(set(outer["fields"]) == {"a:Lem23/InputFieldMultiplyControls$A;"}
                    and set(outer["methods"]) == {"<init>()V", "test1(I)V", "test2(I)V", "multiplyDivide(I)I"},
                    f"outer physical member identity set differs: {label}")
            require(set(child["fields"]) == {"f:I"} and set(child["methods"]) == {"<init>()V"},
                    f"nested physical member identity set differs: {label}")
            require("this.a.f = this.a.f + arg1;" in outer["methods"]["test1(I)V"]["text"],
                    f"positive two-receiver assignment absent: {label}")
            test2 = outer["methods"]["test2(I)V"]
            require(test2["content"] == "explanation_only" and "jarde_refused_body();" in test2["text"]
                    and "the copy at BCI 4 has no proved local assignment" in test2["text"],
                    f"test2 refusal/failure evidence changed: {label}")
            divide = outer["methods"]["multiplyDivide(I)I"]
            require(divide["content"] == "explanation_only"
                    and "the copy at BCI 4 has no proved local assignment" in divide["text"],
                    f"multiplyDivide refusal explanation changed: {label}")
            require({20, 23}.issubset(set(divide["mapped_bcis"])),
                    f"multiplyDivide post-write field read/ireturn source-map anchors absent: {label}")
            modes[mode] = {"root_text": root_text, "child_text": child_doc["text"],
                           "outer": outer, "child": child, "doc": doc}
        require(modes["default"]["root_text"] == modes["all"]["root_text"]
                and modes["default"]["child_text"] == modes["all"]["child_text"],
                f"default/all source text differs: {leg}")
        for group in ("outer", "child"):
            for identity, default_method in modes["default"][group]["methods"].items():
                all_method = modes["all"][group]["methods"][identity]
                require(default_method["text"] == all_method["text"]
                        and default_method["source_map"] == all_method["source_map"],
                        f"default/all method text or source map differs: {leg}/{identity}")
        divide_mapped = modes["all"]["outer"]["methods"]["multiplyDivide(I)I"]["mapped_bcis"]
        require(divide_mapped == EXPECTED_PHYSICAL[OUTER]["methods"][3][3],
                f"multiplyDivide full physical source-map BCI set differs: {leg}")
        by_leg[leg] = {
            "default_all_root_child_text_and_method_maps_equal": True,
            "outer_physical_members": sorted(modes["all"]["outer"]["methods"]),
            "child_physical_members": sorted(modes["all"]["child"]["methods"]),
            "multiplyDivide": {"content": "explanation_only", "mapped_bcis": divide_mapped,
                               "post_write_getfield_bci": 20, "ireturn_bci": 23,
                               "runtime_claim": "none; all Jarde whole-class compiles failed"},
        }
    return by_leg


def verify_multiply_divide_bytecode(text):
    active_method = False
    in_code = False
    instructions = {}
    for line in text.splitlines():
        if re.fullmatch(r"  public int multiplyDivide\(int\);", line):
            active_method = True
            in_code = False
            continue
        if active_method and re.fullmatch(r"  \S.*;", line):
            break
        if not active_method:
            continue
        if line.strip() == "Code:":
            in_code = True
            continue
        match = re.match(r"^\s+(\d+):\s+(.+)$", line)
        if in_code and match:
            instructions[int(match.group(1))] = match.group(2)
    expected_prefixes = {
        4: "dup", 8: "bipush", 11: "idiv", 12: "imul", 13: "putfield",
        16: "aload_0", 17: "getfield", 20: "getfield", 23: "ireturn",
    }
    for bci, prefix in expected_prefixes.items():
        require(instructions.get(bci, "").startswith(prefix),
                f"multiplyDivide physical opcode at BCI {bci} is not {prefix}: {instructions.get(bci)}")
    require("Field em23/InputFieldMultiplyControls$A.f:I" in instructions[20],
            "multiplyDivide BCI 20 is not the nested field post-read")
    require("Field em23/InputFieldMultiplyControls$A.f:I" in instructions[13]
            and "Field a:Lem23/InputFieldMultiplyControls$A;" in instructions[17],
            "multiplyDivide write and receiver reload do not match the physical field sequence")
    return instructions


def main():
    require(not RESULT.exists(), f"refusing to overwrite controls acceptance result: {RESULT}")
    h = load_accepted_helpers()
    manifest, inventory = close_inventory(h)
    jdk = verify_tool_pins(manifest, h)
    verify_execution_policy(manifest)
    commands = verify_commands(manifest, inventory, h)
    source_row, runner_row = manifest["prepared_inputs"]["source"], manifest["prepared_inputs"]["runner"]
    source, runner = h.recorded_bytes(BASE, source_row), h.recorded_bytes(BASE, runner_row)
    require(sha256(source) == SOURCE_SHA256 and h.blake3(source).hexdigest() == SOURCE_BLAKE3,
            "prepared controls source SHA/BLAKE3 changed")
    require(sha256(runner) == RUNNER_SHA256 and h.blake3(runner).hexdigest() == RUNNER_BLAKE3,
            "prepared full Runner SHA/BLAKE3 changed")
    require(manifest["prepared_inputs"]["sha256"] == {"source": SOURCE_SHA256, "runner": RUNNER_SHA256},
            "manifest source/Runner pins differ")
    require(manifest["targets"] == sorted(TARGETS), "target nested class family changed")
    require(manifest["expected_original_stdout"] == EXPECTED_STDOUT.decode("ascii"),
            "fixed Runner output oracle changed")

    cases = {case["label"]: case for case in manifest["cases"]}
    original_jars, original_classes, census, oracle = {}, {}, {}, {}
    for leg in ("javac8", "javac23"):
        case = cases[f"{leg}-original"]
        require(case["kind"] == "original" and case["success"] and case["compile_success"]
                and case["runtime_success"], f"original source full-class leg incomplete: {leg}")
        jar_path = BASE / f"cases/{leg}-original/InputFieldMultiplyControls-family.jar"
        jar_bytes = jar_path.read_bytes()
        with zipfile.ZipFile(jar_path) as archive:
            require(archive.namelist() == [CHILD, OUTER], f"original two-class JAR order/membership changed: {leg}")
        original_jars[leg] = jar_bytes
        class_rows = {row["path"].split("/classes/", 1)[1]: row for row in case["classes"]
                      if row["path"].endswith(".class") and not row["path"].endswith("/Runner.class")}
        require(set(class_rows) == TARGETS, f"original physical target classes differ: {leg}")
        original_classes[leg] = {name: h.recorded_bytes(BASE, row) for name, row in class_rows.items()}
        with zipfile.ZipFile(jar_path) as archive:
            for name in TARGETS:
                require(archive.read(name) == original_classes[leg][name], f"original JAR class differs: {leg}/{name}")
        runtime = case["runtime"]
        stdout, stderr = (h.recorded_bytes(BASE, runtime["streams"][stream]) for stream in ("stdout", "stderr"))
        require(runtime["exit"] == 0 and stdout == EXPECTED_STDOUT and stderr == b"",
                f"original Runner raw triple differs from fixed exceptional/overflow oracle: {leg}")
        oracle[leg] = {"exit": 0, "stdout": stdout, "stderr": stderr}
        physical = manifest["original_physical_classes"][leg]
        require(set(physical) == TARGETS, f"original javap physical class set differs: {leg}")
        census[leg] = {}
        for name, row in physical.items():
            class_bytes = original_classes[leg][name]
            require(h.recorded_bytes(BASE, row["class_bytes"]) == class_bytes
                    and h.blake3(class_bytes).hexdigest() == row["blake3"],
                    f"original class hash differs: {leg}/{name}")
            require(row["javap_success"] is True, f"javap failed: {leg}/{name}")
            cmd = row["command"]
            require(commands[cmd["label"]] == cmd and cmd["exit"] == 0,
                    f"javap command linkage/exit differs: {leg}/{name}")
            expected_path = BASE / f"cases/{leg}-original/classes/{name}"
            expected_argv = [jdk["legs"][leg]["tools"]["javap"]["path"], "-p", "-c", "-s", "-v", str(expected_path)]
            require(cmd["argv"] == expected_argv, f"javap argv differs: {leg}/{name}")
            require(cmd["java_home"] == jdk["legs"][leg]["home"], f"javap JDK home differs: {leg}/{name}")
            javap_text = h.recorded_bytes(BASE, row["text"]).decode("utf-8")
            require(javap_text.encode("utf-8") == h.recorded_bytes(BASE, cmd["streams"]["stdout"]),
                    f"javap report differs from raw stdout: {leg}/{name}")
            simple_name = "InputFieldMultiplyControls$A" if name == CHILD else "InputFieldMultiplyControls"
            census[leg][name] = h.parse_javap(javap_text, simple_name)
            expected = EXPECTED_PHYSICAL[name]
            got_fields = [(x["name"], x["descriptor"], x["flags"]) for x in census[leg][name]["fields"]]
            got_methods = [(x["name"], x["descriptor"], x["flags"], sorted(x["bcis"]))
                           for x in census[leg][name]["methods"]]
            require(got_fields == expected["fields"] and got_methods == expected["methods"],
                    f"physical member/flags/BCI census differs: {leg}/{name}")
            if name == OUTER:
                verify_multiply_divide_bytecode(javap_text)

    require(oracle["javac8"] == oracle["javac23"], "original dual-JDK Runner raw triples differ")
    require(manifest["original_physical_classes"]["javac8"][OUTER]["class_bytes"]["sha256"]
            != manifest["original_physical_classes"]["javac23"][OUTER]["class_bytes"]["sha256"],
            "unexpected byte-identical compiler outputs")

    jadx = manifest["jadx"]
    jar_bytes = h.recorded_bytes(BASE, jadx["input_jar"])
    with zipfile.ZipFile(BASE / jadx["input_jar"]["path"]) as archive:
        require(archive.namelist() == [CHILD, OUTER], "JADX input JAR is not exactly the two original targets")
        for name in TARGETS:
            require(archive.read(name) == original_classes["javac23"][name], f"JADX input class changed: {name}")
    require(jadx["input_jar_exact"] is True and [row["name"] for row in jadx["jar_members"]] == [CHILD, OUTER],
            "JADX input JAR manifest census changed")
    for row in jadx["jar_members"]:
        with zipfile.ZipFile(BASE / jadx["input_jar"]["path"]) as archive:
            raw = archive.read(row["name"])
        require(len(raw) == row["bytes"] and sha256(raw) == row["sha256"]
                and h.blake3(raw).hexdigest() == row["blake3"], f"JADX JAR member hash differs: {row['name']}")
    generated = {}
    for profile in ("default", "none"):
        profile_row = next(row for row in jadx["profiles"] if row["profile"] == profile)
        require(profile_row["source_count"] == 1 and profile_row["package_set"] == ["em23"],
                f"JADX {profile} source inventory differs")
        row = profile_row["generated_sources"][0]
        generated[profile] = h.recorded_bytes(BASE, row)
        source_root = BASE / f"jadx-output/{profile}/sources"
        actual = {p.relative_to(source_root).as_posix() for p in source_root.rglob("*.java")}
        require(actual == {"em23/InputFieldMultiplyControls.java"}, f"JADX {profile} source files differ")
        require((source_root / "em23/InputFieldMultiplyControls.java").read_bytes() == generated[profile],
                f"JADX {profile} generated source bytes differ")
    require(generated["default"] == generated["none"], "JADX default/none source bytes differ")

    jadx_results = []
    for case in sorted((c for c in manifest["cases"] if c["kind"] == "jadx"), key=lambda c: c["label"]):
        jadx_results.append(verify_case_compile_and_runtime(case, commands, source, runner, jdk, oracle, h))
        require(h.recorded_bytes(BASE, case["product_sources"][0]) == generated[case["profile"]],
                f"JADX compile case source differs from generated profile: {case['label']}")
    original_results = [verify_case_compile_and_runtime(cases[f"javac{leg}-original"], commands,
                                                         source, runner, jdk, oracle, h)
                        for leg in (8, 23)]
    failures = verify_failed_jarde_cases(manifest, commands, runner, h)
    jarde_results = verify_jarde_reports(manifest, commands, original_jars, original_classes, census, h)

    summary = read_json(BASE / "summary.json")
    require(summary["schema"] == "em23-nested-field-multiply-summary-v1"
            and summary["status"] == "baseline-with-failures"
            and summary["case_counts"] == manifest["case_counts"]
            and summary["success_counts"] == manifest["success_counts"]
            and summary["original_cross_jdk_raw_equal"] is True
            and summary["failures"] == manifest["failures"], "controls summary differs from manifest")
    result = {
        "schema": "em23-nested-field-multiply-controls-independent-acceptance-luna-v1",
        "status": "accepted-baseline-with-recorded-product-gap",
        "manifest_sha256": MANIFEST_SHA256,
        "inventory_sha256": INVENTORY_SHA256,
        "closed_file_count": len(inventory),
        "command_count": len(commands),
        "cases": {"original": original_results, "jadx": jadx_results, "jarde_compile_failures": failures},
        "original_runner_oracle": {leg: {"exit": 0, "stdout_sha256": sha256(data["stdout"]),
                                         "stderr_sha256": sha256(data["stderr"])}
                                   for leg, data in oracle.items()},
        "exception_and_overflow_boundary": {
            "negative_multiply_overflow": "2147483645",
            "minimum_times_negative_one": "-2147483648",
            "divide_zero_mutation_preserves_field": "ArithmeticException,field=17",
            "null_receiver_precedes_zero_division": "NullPointerException",
        },
        "multiplyDivide_physical_method": {
            "descriptor": "(I)I", "flags": "ACC_PUBLIC", "bci_20": "getfield nested receiver field after putfield",
            "bci_23": "ireturn", "source_maps_cover_all_physical_instruction_bcis": True,
            "default_all_maps_equal": True,
        },
        "jarde_reports": jarde_results,
        "product_gap": {
            "whole_class_compile_legs": 0, "expected_legs": 4, "runtime_legs": 0,
            "test2_refused_copy_remains_unresolved": True,
            "multiplyDivide_explanation_only_and_return_postread_evidence_preserved": True,
        },
        "claim_boundary": "This accepts the recorded controls baseline evidence. All four Jarde whole-class compiles failed; no Jarde runtime comparison is claimed.",
    }
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "files": len(inventory), "commands": len(commands),
                      "original": len(original_results), "jadx": len(jadx_results),
                      "jarde_compile_failures": len(failures), "jarde_runtimes": 0}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"controls baseline acceptance failed: {type(exc).__name__}: {exc}", file=sys.stderr)
        raise SystemExit(1)
