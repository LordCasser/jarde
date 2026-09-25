#!/usr/bin/env python3
"""Replay a verifier-valid direct call to javac's lambda implementation helper."""
from __future__ import annotations

import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
from tempfile import TemporaryDirectory

HERE = Path(__file__).resolve().parent
CLI_DEFAULT = "/tmp/jarde-generic-accepted-cli"
JADX_DEFAULT = "/opt/homebrew/bin/jadx"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(*args: object) -> subprocess.CompletedProcess[str]:
    result = run(*args)
    if result.returncode:
        raise RuntimeError(f"exit={result.returncode}: {' '.join(map(str, args))}\n{result.stdout}{result.stderr}")
    return result


def save(name: str, text: str) -> None:
    (HERE / name).write_text(text)


def utf8_entry(text: str) -> bytes:
    encoded = text.encode("ascii")
    return b"\x01" + struct.pack(">H", len(encoded)) + encoded


def patch_methodref(data: bytes) -> tuple[bytes, dict[str, object]]:
    """Retarget the directHelper Methodref's NameAndType name only."""
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a classfile")
    cp_count = struct.unpack_from(">H", data, 8)[0]
    entries: dict[int, tuple[int, int, object]] = {}
    offset = 10
    index = 1
    while index < cp_count:
        tag = data[offset]
        start = offset
        offset += 1
        if tag == 1:
            length = struct.unpack_from(">H", data, offset)[0]
            raw = data[offset + 2:offset + 2 + length]
            entries[index] = (tag, start, raw.decode("ascii"))
            offset += 2 + length
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entries[index] = (tag, start, struct.unpack_from(">H", data, offset)[0])
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            entries[index] = (tag, start, struct.unpack_from(">HH", data, offset))
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        index += 1

    def class_name(class_index: int) -> str:
        _, _, name_index = entries[class_index]
        return str(entries[int(name_index)][2])

    nat_indices = [
        i for i, entry in entries.items()
        if entry[0] == 12
        and entries[int(entry[2][0])][2] == "directHelper"
        and entries[int(entry[2][1])][2] == "(II)I"
    ]
    if len(nat_indices) != 1:
        raise RuntimeError(f"expected one directHelper:(II)I NameAndType; found {nat_indices}")
    methodrefs = []
    for i, entry in entries.items():
        if entry[0] != 10:
            continue
        owner_index, nat_index = entry[2]
        if class_name(owner_index) == "LambdaAlias" and nat_index == nat_indices[0]:
            methodrefs.append(i)
    if len(methodrefs) != 1:
        raise RuntimeError(f"expected one owning directHelper Methodref; found {methodrefs}")

    helper_name_indices = [i for i, entry in entries.items() if entry[0] == 1 and entry[2] == "lambda$build$0"]
    if len(helper_name_indices) != 1:
        raise RuntimeError(f"expected existing lambda helper Utf8 entry; found {helper_name_indices}")
    nat_index = nat_indices[0]
    _, nat_offset, _ = entries[nat_index]
    result = bytearray(data)
    struct.pack_into(">H", result, nat_offset + 1, helper_name_indices[0])
    patched = bytes(result)
    return patched, {
        "name_and_type_index": nat_index,
        "methodref_index": methodrefs[0],
        "old_name": "directHelper",
        "new_name": "lambda$build$0",
        "descriptor": "(II)I",
        "changed_classfile_byte_offsets": [i for i, (before, after) in enumerate(zip(data, patched)) if before != after],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, default=Path(os.environ.get("JARDE_CLI", CLI_DEFAULT)))
    parser.add_argument("--jadx", type=Path, default=Path(os.environ.get("JADX", JADX_DEFAULT)))
    args = parser.parse_args()
    cli, jadx = args.jarde_cli.resolve(), args.jadx.resolve()
    if not cli.is_file() or not jadx.is_file():
        parser.error(f"missing tool: CLI={cli} JADX={jadx}")

    generated = []
    versions = []
    for label, result in (("javac", checked("javac", "-version")), ("java", checked("java", "-version")),
                          ("jadx", checked(jadx, "--version")), ("jarde", checked(cli, "--version"))):
        versions.append(f"{label}: {(result.stderr or result.stdout).strip()}")
    save("tool-versions.txt", "\n".join(versions) + "\n")
    generated.append("tool-versions.txt")
    mode_summaries = {}

    with TemporaryDirectory(prefix="jarde-lambda-alias-") as temp:
        work = Path(temp)
        for mode, debug_flag, suffix in (("-g", "-g", ""), ("-g:none", "-g:none", "g-none")):
            mode_work = work / suffix if suffix else work / "g"
            mode_work.mkdir()
            original = mode_work / "original"
            patched = mode_work / "patched"
            original.mkdir()
            patched.mkdir()
            checked("javac", "--release", "8", "-Xlint:-options", debug_flag, "-d", original,
                    HERE / "LambdaAlias.java", HERE / "Runner.java")
            original_class = original / "LambdaAlias.class"
            original_bytes = original_class.read_bytes()
            patched_bytes, patch_info = patch_methodref(original_bytes)
            changed_offsets = [i for i, (before, after) in enumerate(zip(original_bytes, patched_bytes)) if before != after]
            if changed_offsets != patch_info["changed_classfile_byte_offsets"]:
                raise RuntimeError(f"{mode}: patch changed unexpected classfile bytes: {changed_offsets}")
            (patched / "LambdaAlias.class").write_bytes(patched_bytes)
            shutil.copy2(original / "Runner.class", patched / "Runner.class")

            source_run = checked("java", "-Xverify:all", "-cp", original, "Runner")
            patched_run = checked("java", "-Xverify:all", "-cp", patched, "Runner")
            if source_run.stdout != "lambda=24\ndirect=125\n" or patched_run.stdout != "lambda=24\ndirect=14\n":
                raise RuntimeError(f"{mode}: unexpected output source={source_run.stdout!r} patched={patched_run.stdout!r}")
            runtime_names = (f"original-source-run{('-' + suffix) if suffix else ''}.txt",
                             f"patched-class-run{('-' + suffix) if suffix else ''}.txt")
            for name, output in zip(runtime_names, (source_run.stdout, patched_run.stdout)):
                save(name, output)
                generated.append(name)

            patch_name = f"patch{('-' + suffix) if suffix else ''}.json"
            save(patch_name, json.dumps(patch_info, indent=2) + "\n")
            generated.append(patch_name)
            class_hash_name = f"classfile-sha256{('-' + suffix) if suffix else ''}.txt"
            save(class_hash_name, "\n".join((
                f"{sha256(original_class.read_bytes()).hexdigest()}  original/LambdaAlias.class",
                f"{sha256(patched_bytes).hexdigest()}  patched/LambdaAlias.class",
            )) + "\n")
            generated.append(class_hash_name)

            mode_outputs = {"source_runtime": source_run.stdout.strip().splitlines(),
                            "patched_runtime": patched_run.stdout.strip().splitlines(),
                            "patched_verified": True, "patch": patch_info, "compile_exit": {}, "outputs": []}
            for label, class_dir in (("original", original), ("patched", patched)):
                name_suffix = f"-{suffix}" if suffix else ""
                javap = checked("javap", "-classpath", class_dir, "-c", "-p", "-v", "LambdaAlias")
                filename = f"javap-{label}{name_suffix}.txt"
                save(filename, javap.stdout.replace(str(work), "<WORK>"))
                generated.append(filename)
                mode_outputs["outputs"].append(filename)

                report = mode_work / f"jarde-{label}.json"
                checked(cli, "class-source", "--input", class_dir / "LambdaAlias.class", "--class", "LambdaAlias",
                        "--policy", "single-class", "--release", "8", "--format", "json", "--output", report)
                data = json.loads(report.read_text())
                for filename, content in (
                    (f"jarde-{label}{name_suffix}.java", data["text"]),
                    (f"jarde-{label}{name_suffix}-report.json", json.dumps(data, indent=2) + "\n"),
                ):
                    save(filename, content)
                    generated.append(filename)
                    mode_outputs["outputs"].append(filename)

                jadx_out = mode_work / f"jadx-{label}"
                checked(jadx, "-d", jadx_out, class_dir / "LambdaAlias.class")
                source = jadx_out / "sources" / "defpackage" / "LambdaAlias.java"
                if not source.exists():
                    candidates = list((jadx_out / "sources").rglob("LambdaAlias.java"))
                    if len(candidates) != 1:
                        raise RuntimeError(f"could not locate JADX source: {candidates}")
                    source = candidates[0]
                jadx_name = f"jadx-{label}{name_suffix}.java"
                save(jadx_name, source.read_text())
                generated.append(jadx_name)
                mode_outputs["outputs"].append(jadx_name)

                for tool_label, source_text in (("jarde", data["text"]),
                                                ("jadx", source.read_text().replace("package defpackage;\n", ""))):
                    src = mode_work / f"{tool_label}-{label}-compile" / "LambdaAlias.java"
                    src.parent.mkdir()
                    src.write_text(source_text)
                    out = src.parent / "classes"
                    out.mkdir()
                    result = run("javac", "--release", "8", "-Xlint:-options", "-d", out, src)
                    mode_outputs["compile_exit"][f"{tool_label}-{label}"] = result.returncode
                    filename = f"{tool_label}-{label}{name_suffix}-javac.txt"
                    save(filename, f"exit={result.returncode}\n{result.stdout}{result.stderr}".replace(str(work), "<WORK>"))
                    generated.append(filename)
                    mode_outputs["outputs"].append(filename)
            mode_summaries[mode] = mode_outputs

    save("summary.json", json.dumps({
        "modes": mode_summaries,
        "temporary_build_directory_removed": True,
    }, indent=2) + "\n")
    generated.append("summary.json")
    inputs = ["LambdaAlias.java", "Runner.java", "analysis.md", "replay.py"]
    save("SHA256SUMS.txt", "\n".join(
        f"{sha256((HERE / name).read_bytes()).hexdigest()}  {name}"
        for name in sorted(inputs + generated)
    ) + "\n")
    print("lambda-helper direct-call evidence refreshed for -g and -g:none; compilation outputs were confined to TemporaryDirectory")

if __name__ == "__main__":
    main()
