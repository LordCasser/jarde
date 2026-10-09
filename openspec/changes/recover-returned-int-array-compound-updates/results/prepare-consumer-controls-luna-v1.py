#!/usr/bin/env python3
"""Create verifier-only scalar consumer controls; never invoke scalar or another target method.

Root runs this script after reviewing it. It reuses the frozen child-control class parser,
mutates only scalar's Code bytes and (for long-return) its same-length descriptor constant,
and records every raw input/output and -Xverify:all load-only command in a fresh directory.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import traceback
from typing import Any

REPO = Path("/Users/lordcasser/workspace/projects/jarde")
RESULTS = REPO / "openspec/changes/recover-returned-int-array-compound-updates/results"
BASE = REPO / "openspec/changes/recover-nested-int-array-compound-updates/results/returned-next-baseline-v1"
TYPED = RESULTS / "typed-controls-root-v1"
BASELINE_MANIFEST = BASE / "manifest.json"
TYPED_MANIFEST = TYPED / "manifest.json"
PARSER_SCRIPT = REPO / "openspec/changes/recover-covariant-child-array-initializers/results/prepare-child-jvm-controls-v1.py"
VARIANTS = ("sum-extra-consumer", "returned-copy-extra-consumer", "long-return")
LEGS = (("v8", "javac8"), ("v23", "javac23"))


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def load_parser() -> Any:
    require(PARSER_SCRIPT.is_file(), f"shared class parser is missing: {PARSER_SCRIPT}")
    spec = importlib.util.spec_from_file_location("jarde_child_jvm_class_parser", PARSER_SCRIPT)
    require(spec is not None and spec.loader is not None, "cannot load shared class parser")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def locate_scalar(parser: Any, raw: bytes, descriptor: str = "([III)I") -> tuple[dict[str, Any], dict[str, Any]]:
    parsed = parser.parse_class(raw)
    require(sum(method["descriptor"] == descriptor for method in parsed["methods"]) == 1,
            f"expected scalar descriptor {descriptor} to belong to one method only")
    code = parser.find_method_code(raw, parsed, "scalar", descriptor)
    instructions = parser.decode_instructions(raw, code)
    by_bci = {instruction["bci"]: instruction for instruction in instructions}
    require(len(instructions) == code["code_length"], "scalar instruction count is inconsistent")
    require(code["max_stack"] == 4, f"unexpected scalar max_stack={code['max_stack']}")
    require(code["exception_count"] == 0, "scalar Code unexpectedly has exception handlers")
    require(code["subattribute_count"] == 0, "scalar Code unexpectedly has StackMap/debug subattributes")
    require(code["code_length"] == 9, f"unexpected scalar code_length={code['code_length']}")
    require(by_bci.get(5, {}).get("opcode") == 0x60, "scalar iadd is not at BCI 5")
    require(by_bci.get(6, {}).get("opcode") == 0x5B, "scalar dup_x2 is not at BCI 6")
    require(by_bci.get(7, {}).get("opcode") == 0x4F, "scalar iastore is not at BCI 7")
    require(by_bci.get(8, {}).get("opcode") == 0xAC, "scalar ireturn is not at BCI 8")
    return parsed, code



def non_scalar_method_facts(parser: Any, raw: bytes) -> list[dict[str, Any]]:
    parsed = parser.parse_class(raw)
    facts = []
    for method in parsed["methods"]:
        if method["name"] == "scalar":
            continue
        attributes = []
        for attribute in method["attributes"]:
            contents = raw[attribute["content_start"]:attribute["content_end"]]
            attributes.append({"name": attribute["name"], "length": attribute["length"],
                               "sha256": sha256_bytes(contents)})
        facts.append({"name": method["name"], "descriptor": method["descriptor"],
                      "access": method["access"], "attributes": attributes})
    return facts


def ensure_descriptor_only_scalar(parser: Any, raw: bytes, parsed: dict[str, Any]) -> None:
    matches = [index for index, entry in enumerate(parsed["cp"])
               if entry is not None and entry["tag"] == 1 and entry["raw"] == b"([III)I"]
    require(len(matches) == 1, "expected one scalar descriptor Utf8 constant")
    target_index = matches[0]
    name_and_type_refs = [index for index, entry in enumerate(parsed["cp"])
                          if entry is not None and entry["tag"] == 12
                          and entry["descriptor_index"] == target_index]
    require(not name_and_type_refs,
            f"scalar descriptor constant is shared by a NameAndType entry: {name_and_type_refs}")
    require(sum(method["descriptor"] == "([III)I" for method in parsed["methods"]) == 1,
            "scalar descriptor is shared by another method")

def insert_code_bytes(parser: Any, raw: bytes, code: dict[str, Any], at_bci: int, inserted: bytes) -> bytes:
    parser.require(0 <= at_bci <= code["code_length"], "insertion BCI is outside Code")
    result = bytearray(raw)
    absolute = code["code_start"] + at_bci
    result[absolute:absolute] = inserted
    parser.put_u4(result, code["code_length_field"], code["code_length"] + len(inserted))
    parser.put_u4(result, code["attribute"]["length_field"], code["attribute"]["length"] + len(inserted))
    return bytes(result)


def mutate_variant(parser: Any, raw: bytes, variant: str) -> tuple[bytes, dict[str, Any]]:
    parsed, code = locate_scalar(parser, raw)
    original = parser.decode_instructions(raw, code)
    if variant == "sum-extra-consumer":
        # A category-1 stack-neutral read of the iadd result; iadd no longer feeds dup_x2 directly.
        at_bci, inserted = 6, bytes((0x59, 0x57))  # dup; pop
        output = insert_code_bytes(parser, raw, code, at_bci, inserted)
        expected_code_length = code["code_length"] + 2
        expected_return_descriptor = "([III)I"
        change = "insert dup; pop after iadd@5, before the original dup_x2"
        instruction_bcis = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    elif variant == "returned-copy-extra-consumer":
        # The retained bottom sum is duplicated after iastore, so ireturn is no longer its only use.
        at_bci, inserted = 8, bytes((0x59, 0x57))  # dup; pop
        output = insert_code_bytes(parser, raw, code, at_bci, inserted)
        expected_code_length = code["code_length"] + 2
        expected_return_descriptor = "([III)I"
        change = "insert dup; pop after iastore@7, before the original ireturn"
        instruction_bcis = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    elif variant == "long-return":
        # Keep the same array update and widen its result legally for the descriptor's long return.
        ensure_descriptor_only_scalar(parser, raw, parsed)
        needle = b"\x01\x00\x07([III)I"
        require(raw.count(needle) == 1, "expected one scalar descriptor Utf8 constant")
        output = bytearray(raw.replace(needle, b"\x01\x00\x07([III)J", 1))
        parser.put_u4(output, code["code_length_field"], code["code_length"] + 1)
        parser.put_u4(output, code["attribute"]["length_field"], code["attribute"]["length"] + 1)
        return_absolute = code["code_start"] + 8
        require(output[return_absolute] == 0xAC, "scalar terminal opcode changed unexpectedly")
        output[return_absolute:return_absolute + 1] = bytes((0x85, 0xAD))  # i2l; lreturn
        output = bytes(output)
        expected_code_length = code["code_length"] + 1
        expected_return_descriptor = "([III)J"
        change = "change scalar descriptor I to J and replace ireturn@8 with i2l@8; lreturn@9"
        instruction_bcis = list(range(10))
    else:
        raise RuntimeError(f"unknown control variant: {variant}")

    parsed_after = parser.parse_class(output)
    descriptor_after = "([III)J" if variant == "long-return" else "([III)I"
    code_after = parser.find_method_code(output, parsed_after, "scalar", descriptor_after)
    after_instructions = parser.decode_instructions(output, code_after)
    require(code_after["code_length"] == expected_code_length, f"{variant}: wrong Code length")
    require(code_after["attribute"]["length"] == code["attribute"]["length"] + expected_code_length - code["code_length"],
            f"{variant}: wrong Code attribute length")
    require(code_after["max_stack"] == code["max_stack"], f"{variant}: max_stack changed")
    require(code_after["max_locals"] == code["max_locals"], f"{variant}: max_locals changed")
    require(code_after["exception_count"] == 0 and code_after["subattribute_count"] == 0,
            f"{variant}: exception/StackMap substructure changed")
    after_by_bci = {instruction["bci"]: instruction["opcode"] for instruction in after_instructions}
    if variant == "sum-extra-consumer":
        require([after_by_bci.get(bci) for bci in (5, 6, 7, 8)] == [0x60, 0x59, 0x57, 0x5B],
                "sum-extra-consumer does not place dup;pop after iadd")
        require(after_by_bci.get(9) == 0x4F and after_by_bci.get(10) == 0xAC,
                "sum-extra-consumer lost its shifted store/return")
    elif variant == "returned-copy-extra-consumer":
        require([after_by_bci.get(bci) for bci in (6, 7, 8, 9, 10)] == [0x5B, 0x4F, 0x59, 0x57, 0xAC],
                "returned-copy-extra-consumer does not place dup;pop after iastore")
    else:
        require([after_by_bci.get(bci) for bci in (5, 6, 7, 8, 9)] == [0x60, 0x5B, 0x4F, 0x85, 0xAD],
                "long-return does not widen the updated int before lreturn")
    require(code_after["code_length"] == len(after_instructions), "scalar BCI list is not a complete instruction boundary")
    return output, {
        "variant": variant,
        "change": change,
        "descriptor_before": "([III)I",
        "descriptor_after": expected_return_descriptor,
        "code_length_before": code["code_length"],
        "code_length_after": code_after["code_length"],
        "code_attribute_length_before": code["attribute"]["length"],
        "code_attribute_length_after": code_after["attribute"]["length"],
        "max_stack_before": code["max_stack"],
        "max_stack_after": code_after["max_stack"],
        "max_locals_before": code["max_locals"],
        "max_locals_after": code_after["max_locals"],
        "exception_count": code_after["exception_count"],
        "code_subattribute_count": code_after["subattribute_count"],
        "scalar_instruction_bcis": instruction_bcis,
    }


def create_output(path_arg: str | None) -> Path:
    if path_arg:
        output = Path(path_arg).expanduser().resolve()
    else:
        stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        output = Path(f"/private/tmp/jarde-returned-array-consumer-controls-{stamp}-{os.getpid()}")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir(exist_ok=False)
    return output


def file_record(path: Path) -> dict[str, Any]:
    return {"path": str(path), "bytes": path.stat().st_size, "sha256": sha256_file(path)}


def inventory(output: Path) -> list[dict[str, Any]]:
    return [file_record(path) | {"relative_path": str(path.relative_to(output))}
            for path in sorted(p for p in output.rglob("*") if p.is_file() and p.name != "manifest.json")]


def main() -> int:
    parser_arg = argparse.ArgumentParser(description=__doc__)
    parser_arg.add_argument("--output", help="fresh evidence directory; existing paths are rejected")
    args = parser_arg.parse_args()
    out = create_output(args.output)
    runner_path = Path(__file__).resolve()
    baseline = json.loads(BASELINE_MANIFEST.read_text(encoding="utf-8"))
    typed_manifest = json.loads(TYPED_MANIFEST.read_text(encoding="utf-8"))
    parser = load_parser()
    manifest: dict[str, Any] = {
        "schema": "returned-int-array-consumer-controls-luna-v1",
        "status": "preparing",
        "created_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "runner": file_record(runner_path),
        "shared_parser": file_record(PARSER_SCRIPT),
        "baseline_manifest": file_record(BASELINE_MANIFEST),
        "typed_control_manifest": file_record(TYPED_MANIFEST),
        "verify_only_source": typed_manifest["verify_only_source"],
        "scope": "three scalar bytecode/descriptor controls per real JDK; class-load/reflection only; scalar and all target methods are never invoked",
        "input_classes": {},
        "controls": [],
        "commands": [],
        "failures": [],
        "files": [],
        "manifest_self_excluded": True,
    }
    shutil.copyfile(runner_path, out / runner_path.name)
    (out / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def persist() -> None:
        (out / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def run_verify(label: str, java_path: Path, java_home: Path, verifier_dir: Path, control_dir: Path, expected: bytes) -> dict[str, Any]:
        command_env = os.environ.copy()
        filtered = []
        for variable in ("CLASSPATH", "JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS"):
            if variable in command_env:
                command_env.pop(variable)
                filtered.append(variable)
        command_env["JAVA_HOME"] = str(java_home)
        argv = [str(java_path), "-Xverify:all", "-cp", os.pathsep.join((str(verifier_dir), str(control_dir))), "VerifyOnly"]
        try:
            completed = subprocess.run(argv, cwd=str(REPO), env=command_env, capture_output=True, timeout=60, check=False)
            stdout, stderr, exit_code, launch_error = completed.stdout, completed.stderr, completed.returncode, None
        except Exception as error:
            stdout, stderr, exit_code, launch_error = b"", traceback.format_exc().encode("utf-8"), None, repr(error)
        safe = label.replace("/", "_")
        stdout_path, stderr_path = out / "logs" / f"{safe}.stdout", out / "logs" / f"{safe}.stderr"
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        stdout_path.write_bytes(stdout)
        stderr_path.write_bytes(stderr)
        row = {
            "label": label, "argv": argv, "cwd": str(REPO), "java_home": str(java_home),
            "filtered_environment": filtered, "exit_code": exit_code, "launch_error": launch_error,
            "stdout": file_record(stdout_path), "stderr": file_record(stderr_path),
            "expected_stdout": expected.decode("ascii"), "target_method_invoked": False,
        }
        manifest["commands"].append(row)
        persist()
        require(exit_code == 0 and stdout == expected and stderr == b"", f"{label} verifier/reflection check failed")
        return row

    try:
        require(BASELINE_MANIFEST.is_file() and TYPED_MANIFEST.is_file(), "frozen baseline or typed-control manifest is missing")
        require(sha256_file(VERIFY_SOURCE := Path(typed_manifest["verify_only_source"]["path"])) == typed_manifest["verify_only_source"]["sha256"],
                "VerifyOnly source differs from typed-control manifest")
        verify_source = VERIFY_SOURCE.read_bytes()
        require(b"Class.forName(\"ReturnedIntArrayUpdates\")" in verify_source and b"m.invoke" not in verify_source,
                "existing VerifyOnly source is not load/reflect-only")
        require(typed_manifest["baseline_manifest_sha256"] == sha256_file(BASELINE_MANIFEST),
                "typed controls were built from a different returned-array baseline")
        javac_tools = baseline["jdk_tools"]
        require(len(javac_tools) == 2, "expected exactly two recorded JDK legs")
        tool_by_name = {"javac8": javac_tools[0], "javac23": javac_tools[1]}
        controls_root = out / "controls"
        inputs_root = out / "inputs"
        verifier_root = out / "verifier"
        controls_root.mkdir()
        inputs_root.mkdir()
        verifier_root.mkdir()

        for fixture_leg, tool_leg in LEGS:
            tool = tool_by_name[tool_leg]
            java_path = Path(tool["java"]["path"])
            java_home = java_path.parent.parent
            require(java_path.is_file() and sha256_file(java_path) == tool["java"]["sha256"],
                    f"recorded java binary identity differs for {tool_leg}")
            base_row = next(row for row in baseline["inputs"] if tool_leg in row["path"] and row["path"].endswith("ReturnedIntArrayUpdates.class"))
            source_class = BASE / base_row["path"]
            raw = source_class.read_bytes()
            parsed_input = parser.parse_class(raw)
            ensure_descriptor_only_scalar(parser, raw, parsed_input)
            other_method_facts = non_scalar_method_facts(parser, raw)
            require(len(raw) == base_row["bytes"] and sha256_bytes(raw) == base_row["sha256"],
                    f"frozen baseline class differs for {tool_leg}")
            verifier_dir = TYPED / f"{tool_leg}-verifier"
            verifier_class = verifier_dir / "VerifyOnly.class"
            require(verifier_class.is_file(), f"precompiled VerifyOnly class is missing for {tool_leg}")
            typed_command = next((row for row in typed_manifest["commands"] if row["label"] == f"{tool_leg}-compile-verifier"), None)
            require(typed_command is not None and typed_command["exit_code"] == 0,
                    f"typed-control manifest does not prove existing VerifyOnly compilation for {tool_leg}")
            saved_verifier_dir = verifier_root / fixture_leg
            saved_verifier_dir.mkdir()
            shutil.copyfile(verifier_class, saved_verifier_dir / "VerifyOnly.class")
            input_path = inputs_root / fixture_leg / "ReturnedIntArrayUpdates.class"
            input_path.parent.mkdir()
            input_path.write_bytes(raw)
            manifest["input_classes"][fixture_leg] = {
                "baseline_record": base_row, "saved_raw_input": file_record(input_path),
                "java": {"path": str(java_path), "sha256": sha256_file(java_path), "home": str(java_home)},
                "verify_only_source": typed_manifest["verify_only_source"],
                "verify_only_class": file_record(saved_verifier_dir / "VerifyOnly.class"),
                "verify_only_compile_record": typed_command,
            }
            leg_controls = controls_root / fixture_leg
            leg_controls.mkdir()
            for variant in VARIANTS:
                mutated, details = mutate_variant(parser, raw, variant)
                require(non_scalar_method_facts(parser, mutated) == other_method_facts,
                        f"{fixture_leg}/{variant} changed a non-scalar method member")
                descriptor = details["descriptor_after"]
                parsed_mutated = parser.parse_class(mutated)
                code_mutated = parser.find_method_code(mutated, parsed_mutated, "scalar", descriptor)
                control_path = leg_controls / f"{variant}.class"
                control_path.write_bytes(mutated)
                expected = ("ok:int:int:int\n" if descriptor == "([III)I" else "ok:int:int:long\n").encode("ascii")
                control = {
                    **details, "leg": fixture_leg, "jdk_leg": tool_leg,
                    "input": file_record(input_path), "output": file_record(control_path),
                    "non_scalar_methods_unchanged": True,
                    "scalar_descriptor": descriptor,
                    "max_stack_preserved": code_mutated["max_stack"] == 4,
                    "no_exception_handlers": code_mutated["exception_count"] == 0,
                    "no_code_subattributes": code_mutated["subattribute_count"] == 0,
                    "verify_only_class": file_record(saved_verifier_dir / "VerifyOnly.class"),
                }
                manifest["controls"].append(control)
                persist()
                run_verify(f"{fixture_leg}/{variant}", java_path, java_home,
                           saved_verifier_dir, leg_controls, expected)
                control["verified_load_only"] = True
                control["target_method_invoked"] = False
                persist()

        manifest["status"] = "prepared-and-verified"
        manifest["interpretation"] = (
            "Only -Xverify:all class loading and reflection through the precompiled VerifyOnly main was checked. "
            "No scalar method or other target method was invoked; this is not evidence of Jarde recovery or behavior."
        )
    except Exception as error:
        manifest["status"] = "failed"
        manifest["failures"].append({"error": repr(error), "traceback": traceback.format_exc()})
        persist()
        print(f"FAILED: {error}; raw evidence retained at {out}", file=sys.stderr)
        return 1
    finally:
        manifest["files"] = inventory(out)
        persist()
    print(f"Prepared verifier-only consumer controls: {out}")
    print("No scalar or other target method was invoked.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
