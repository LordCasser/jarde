#!/usr/bin/env python3
"""Build verifier-valid Java 8 boolean return boundary classes."""
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
PATCHER = HERE.parent / "p3-narrow-array-stores" / "patch_array_stores.py"
SPEC = importlib.util.spec_from_file_location("array_patch", PATCHER)
array_patch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(array_patch)


def patch(source: Path, output: Path, methods: dict[str, tuple[str, str]]) -> None:
    data = bytearray(source.read_bytes())
    code_before = code_digest(data)
    entries, cp_end = array_patch.parse_cp(data)
    resolved = {}
    for name, (original, narrowed) in methods.items():
        if original.startswith("("):
            original_parameters = original[:original.rfind(")") + 1]
            narrow_descriptor = original_parameters + narrowed
        else:
            raise ValueError(f"invalid method descriptor {original}")
        data, entries, cp_end, descriptor_index = array_patch.cp_add_utf8(
            data, entries, cp_end, narrow_descriptor
        )
        resolved[name] = (original, narrow_descriptor, descriptor_index)

    found = set()
    for method in array_patch.find_methods(data, entries):
        name = method["name"]
        if name not in resolved:
            continue
        original, narrowed, descriptor_index = resolved[name]
        if method["descriptor"] != original:
            raise ValueError(f"{name} descriptor was {method['descriptor']}, expected {original}")
        array_patch.put_u2(data, method["info"] + 4, descriptor_index)
        found.add(name)
    if found != set(resolved):
        raise ValueError(f"method mismatch: found {sorted(found)}")

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    report = {
        "input_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "output_sha256": hashlib.sha256(data).hexdigest(),
        "code_sha256_before": code_before,
        "code_sha256_after": code_digest(data),
        "patched_methods": {
            name: {"from": old, "to": new}
            for name, (old, new, _) in resolved.items()
        },
    }
    if report["code_sha256_before"] != report["code_sha256_after"]:
        raise ValueError("return descriptor patch changed Code bytes")
    output.with_suffix(".patch.json").write_text(json.dumps(report, indent=2) + "\n")


def code_digest(data: bytes) -> str:
    entries, _ = array_patch.parse_cp(data)
    code = bytearray()
    for method in array_patch.find_methods(data, entries):
        for name_index, attr_start, _ in method["attrs"]:
            if array_patch.cp_utf8(entries, name_index) != "Code":
                continue
            _, cursor = array_patch.u2(data, attr_start)
            _, cursor = array_patch.u2(data, cursor)
            code_length, cursor = array_patch.u4(data, cursor)
            code.extend(data[cursor : cursor + code_length])
    return hashlib.sha256(code).hexdigest()


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_return_boundaries.py COMPILED_DIR OUTPUT_DIR")
    compiled, output = map(Path, sys.argv[1:])
    patch(
        compiled / "BooleanReturnBoundaries.class",
        output / "BooleanReturnBoundaries.class",
        {
            "booleanAsByte": ("(Z)Z", "B"),
            "booleanAsChar": ("(Z)Z", "C"),
            "booleanAsShort": ("(Z)Z", "S"),
            "integerAsBoolean": ("(I)I", "Z"),
        },
    )
    caller_source = HERE / "BooleanRawTwoCaller.java"
    compile_caller = subprocess.run(
        [
            "javac", "--release", "8", "-g:none", "-cp", str(output), "-d",
            str(output), str(caller_source),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if compile_caller.returncode != 0:
        raise SystemExit(compile_caller.stdout + compile_caller.stderr)
    caller_class = output / "BooleanRawTwoCaller.class"
    data = bytearray(caller_class.read_bytes())
    caller_before = bytes(data)
    entries, _ = array_patch.parse_cp(data)
    seen = set()
    for method in array_patch.find_methods(data, entries):
        if method["name"] not in {"byteValue", "charValue", "shortValue"}:
            continue
        code_attrs = [
            attr for attr in method["attrs"]
            if array_patch.cp_utf8(entries, attr[0]) == "Code"
        ]
        if len(code_attrs) != 1:
            raise ValueError(f"{method['name']} requires one Code attribute")
        _, code_start, _ = code_attrs[0]
        _, code_start = array_patch.u2(data, code_start)  # max_stack
        _, code_start = array_patch.u2(data, code_start)  # max_locals
        code_length, code_start = array_patch.u4(data, code_start)
        if code_length < 1 or data[code_start] != 0x03:
            raise ValueError(f"{method['name']} no longer starts with iconst_0")
        data[code_start] = 0x05  # Push raw int 2 to a descriptor-typed Z argument.
        seen.add(method["name"])
    if seen != {"byteValue", "charValue", "shortValue"}:
        raise ValueError(f"raw-two caller methods missing: {seen}")
    caller_class.write_bytes(data)
    caller_class.with_suffix(".patch.json").write_text(
        json.dumps(
            {
                "source_sha256": hashlib.sha256(caller_before).hexdigest(),
                "output_sha256": hashlib.sha256(data).hexdigest(),
                "code_changes": {
                    name: {"bci": 0, "from": "iconst_0", "to": "iconst_2"}
                    for name in sorted(seen)
                },
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    main()
