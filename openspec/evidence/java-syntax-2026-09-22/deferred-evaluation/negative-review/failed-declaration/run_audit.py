from __future__ import annotations

import hashlib
import json
import re
import struct
import subprocess
from pathlib import Path


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/negative-review/failed-declaration"
WORK = Path("/tmp/jarde-deferred-negative-failed-declaration")
CLI = ROOT / "target/debug/jarde-cli"


def run(args: list[str], log: Path) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=30)
    log.write_text(result.stdout + result.stderr)
    return result.returncode


def cp(data: bytes) -> tuple[list[object | None], int]:
    count = struct.unpack_from(">H", data, 8)[0]
    entries: list[object | None] = [None] * count
    offset = 10
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = struct.unpack_from(">H", data, offset)[0]
            offset += 2
            entries[index] = (tag, data[offset:offset + length].decode("utf-8"))
            offset += length
        elif tag in (3, 4):
            entries[index] = (tag, data[offset:offset + 4])
            offset += 4
        elif tag in (5, 6):
            entries[index] = (tag, data[offset:offset + 8])
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            value = struct.unpack_from(">H", data, offset)[0]
            offset += 2
            entries[index] = (tag, value)
        elif tag in (9, 10, 11, 12, 17, 18):
            left, right = struct.unpack_from(">HH", data, offset)
            offset += 4
            entries[index] = (tag, left, right)
        elif tag == 15:
            kind = data[offset]
            reference = struct.unpack_from(">H", data, offset + 1)[0]
            offset += 3
            entries[index] = (tag, kind, reference)
        else:
            raise AssertionError(f"unsupported CP tag {tag}")
        index += 1
    return entries, offset


def text(entries: list[object | None], index: int) -> str:
    entry = entries[index]
    assert entry is not None and entry[0] == 1
    return entry[1]


def support_ref(entries: list[object | None]) -> int:
    for index, entry in enumerate(entries):
        if not entry or entry[0] != 10:
            continue
        owner = entries[entry[1]]
        name_type = entries[entry[2]]
        assert owner is not None and owner[0] == 7
        assert name_type is not None and name_type[0] == 12
        if (
            text(entries, owner[1]).replace("/", ".") == "Support"
            and text(entries, name_type[1]) == "mark"
            and text(entries, name_type[2]) == "()V"
        ):
            return index
    raise AssertionError("missing Support.mark()V constant-pool reference")


def skip_member(data: bytes, offset: int) -> int:
    offset += 6
    count = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    for _ in range(count):
        length = struct.unpack_from(">I", data, offset + 2)[0]
        offset += 6 + length
    return offset


def patch_run(class_path: Path) -> dict[str, object]:
    original = class_path.read_bytes()
    entries, offset = cp(original)
    mark = support_ref(entries)
    cursor = offset + 6
    interfaces = struct.unpack_from(">H", original, cursor)[0]
    cursor += 2 + 2 * interfaces
    fields = struct.unpack_from(">H", original, cursor)[0]
    cursor += 2
    for _ in range(fields):
        cursor = skip_member(original, cursor)
    methods = struct.unpack_from(">H", original, cursor)[0]
    cursor += 2
    found: tuple[int, int, int, bytes, bytes] | None = None
    for _ in range(methods):
        method_name_start = cursor + 2
        name_index = struct.unpack_from(">H", original, method_name_start)[0]
        descriptor_index = struct.unpack_from(">H", original, cursor + 4)[0]
        attrs = struct.unpack_from(">H", original, cursor + 6)[0]
        cursor += 8
        for _ in range(attrs):
            attr_start = cursor
            attr_name, length = struct.unpack_from(">HI", original, cursor)
            info_start = cursor + 6
            cursor = info_start + length
            if text(entries, name_index) != "run" or text(entries, descriptor_index) != "(I)I":
                continue
            if text(entries, attr_name) != "Code":
                continue
            info = original[info_start:info_start + length]
            max_stack, max_locals, code_length = struct.unpack_from(">HHI", info, 0)
            code_start = 8
            old_code = info[code_start:code_start + code_length]
            assert old_code[-1] == 0xAC
            new_code = old_code[:-1] + bytes([0xB8]) + struct.pack(">H", mark) + old_code[-1:]
            p = code_start + code_length
            exception_count = struct.unpack_from(">H", info, p)[0]
            p += 2
            exceptions = info[p:p + 8 * exception_count]
            p += 8 * exception_count
            nested_count = struct.unpack_from(">H", info, p)[0]
            p += 2
            nested: list[bytes] = []
            for _ in range(nested_count):
                nested_name, nested_length = struct.unpack_from(">HI", info, p)
                p += 6
                nested_payload = info[p:p + nested_length]
                p += nested_length
                nested.append(struct.pack(">HI", nested_name, nested_length) + nested_payload)
            assert p == len(info)
            new_info = (
                struct.pack(">HHI", max_stack, max_locals, len(new_code))
                + new_code
                + struct.pack(">H", exception_count)
                + exceptions
                + struct.pack(">H", len(nested))
                + b"".join(nested)
            )
            found = (attr_start, 6 + length, attr_name, old_code, new_code, new_info)  # type: ignore[assignment]
    assert found is not None
    attr_start, old_attr_size, attr_name, old_code, new_code, new_info = found
    patched = bytearray(original)
    patched[attr_start:attr_start + old_attr_size] = struct.pack(">HI", attr_name, len(new_info)) + new_info
    class_path.write_bytes(patched)
    return {
        "original_sha256": hashlib.sha256(original).hexdigest(),
        "patched_sha256": hashlib.sha256(patched).hexdigest(),
        "original_code_hex": old_code.hex(),
        "patched_code_hex": new_code.hex(),
        "old_code_length": len(old_code),
        "new_code_length": len(new_code),
        "inserted_opcode": "invokestatic Support.mark()V before ireturn",
        "class_major": struct.unpack_from(">H", original, 6)[0],
    }


def main() -> None:
    if WORK.exists():
        raise RuntimeError(f"refusing to remove existing work directory: {WORK}")
    WORK.mkdir()
    original = WORK / "original"
    original.mkdir()
    sources = [EVIDENCE / name for name in ("FailedDeclaration.java", "Support.java", "FailedDeclarationRunner.java")]
    assert run(["javac", "--release", "8", "-d", str(original), *(str(source) for source in sources)], EVIDENCE / "source-javac.log") == 0
    class_path = original / "FailedDeclaration.class"
    patch = patch_run(class_path)
    assert run(["java", "-Xverify:all", "-cp", str(original), "FailedDeclarationRunner"], EVIDENCE / "original.txt") == 0
    run(["javap", "-v", "-c", str(class_path)], EVIDENCE / "patched-javap.txt")
    (EVIDENCE / "patched-class.hex").write_text(class_path.read_bytes().hex() + "\n")

    before_cli = hashlib.sha256(CLI.read_bytes()).hexdigest()
    stable = Path("/tmp/jarde-cli-deferred-first-feed5c")
    stable_hash = hashlib.sha256(stable.read_bytes()).hexdigest() if stable.exists() else None
    result = subprocess.run(
        [str(CLI), "class-source", "--input", str(class_path), "--class", "FailedDeclaration", "--policy", "single-class", "--release", "8", "--format", "text"],
        capture_output=True,
        text=True,
        timeout=30,
    )
    (EVIDENCE / "jarde.java.txt").write_text(result.stdout)
    (EVIDENCE / "jarde-report.txt").write_text(result.stderr)
    stable_result = None
    if stable.exists():
        stable_result = subprocess.run(
            [str(stable), "class-source", "--input", str(class_path), "--class", "FailedDeclaration", "--policy", "single-class", "--release", "8", "--format", "text"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        (EVIDENCE / "stable-jarde.java.txt").write_text(stable_result.stdout)
        (EVIDENCE / "stable-jarde-report.txt").write_text(stable_result.stderr)
    full = subprocess.run(
        [str(CLI), "class-source", "--input", str(class_path), "--class", "FailedDeclaration", "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all"],
        capture_output=True,
        text=True,
        timeout=30,
    )
    (EVIDENCE / "jarde-full.json").write_text(full.stdout)
    (EVIDENCE / "jarde-full-report.txt").write_text(full.stderr)
    jarde = WORK / "jarde"
    jarde.mkdir()
    jarde_source = jarde / "FailedDeclaration.java"
    jarde_source.write_text(result.stdout)
    jarde_compile = run(
        ["javac", "--release", "8", "-d", str(jarde / "classes"), str(jarde_source), *(str(source) for source in sources[1:])],
        EVIDENCE / "jarde-javac.log",
    )
    if jarde_compile == 0:
        run(["java", "-Xverify:all", "-cp", str(jarde / "classes"), "FailedDeclarationRunner"], EVIDENCE / "jarde.txt")

    jadx_root = WORK / "jadx"
    jadx_rc = run(["jadx", "--no-res", "-d", str(jadx_root), str(class_path)], EVIDENCE / "jadx.log")
    jadx_compile = None
    if jadx_rc == 0:
        generated = next(jadx_root.rglob("FailedDeclaration.java"))
        generated_text = generated.read_text()
        (EVIDENCE / "jadx.java.txt").write_text(generated_text)
        package = next((line for line in generated_text.splitlines() if line.startswith("package ")), "")
        package_name = package.removeprefix("package ").rstrip(";")
        support = WORK / "jadx-support"
        support.mkdir()
        for source in sources[1:]:
            (support / source.name).write_text(package + "\n" + source.read_text())
        classes = WORK / "jadx-classes"
        jadx_compile = run(["javac", "--release", "8", "-d", str(classes), str(generated), *(str(path) for path in support.glob("*.java"))], EVIDENCE / "jadx-javac.log")
        if jadx_compile == 0:
            main_class = f"{package_name}.FailedDeclarationRunner" if package_name else "FailedDeclarationRunner"
            assert run(["java", "-Xverify:all", "-cp", str(classes), main_class], EVIDENCE / "jadx.txt") == 0

    after_cli = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert before_cli == after_cli
    run_body = re.search(r"public static int run\(int arg0\) \{(?P<body>.*?)\n    \}", result.stdout, re.S)
    body = run_body.group("body") if run_body else ""
    body_code = "\n".join(line for line in body.splitlines() if not line.strip().startswith("//"))
    run_source_map_bcis: list[dict[str, object]] = []
    if full.returncode == 0:
        full_document = json.loads(full.stdout)
        run_method = next(method for method in full_document["methods"] if method["item"]["name"]["escaped"] == "run")
        for segment in run_method["outcome"]["report"]["source_map"]["segments"]:
            origin = segment["origin"]
            run_source_map_bcis.append(
                {
                    "start": segment["start"],
                    "end": segment["end"],
                    "primary": origin["primary"]["bci"],
                    "derived": [item["bci"] for item in origin.get("derived", [])],
                }
            )
    summary = {
        **patch,
        "cli_sha256_before": before_cli,
        "cli_sha256_after": after_cli,
        "stable_cli_sha256": stable_hash,
        "jarde_cli_returncode": result.returncode,
        "stable_jarde_cli_returncode": stable_result.returncode if stable_result else None,
        "stable_source_equal_current": stable_result is None or stable_result.stdout == result.stdout,
        "jarde_full_evidence_returncode": full.returncode,
        "jarde_quotes": result.stdout.count("@bytecode"),
        "jarde_javac_returncode": jarde_compile,
        "jadx_returncode": jadx_rc,
        "jadx_javac_returncode": jadx_compile,
        "original_lines": len((EVIDENCE / "original.txt").read_text().splitlines()),
        "run_body_executable": body_code,
        "run_body_take_count": body_code.count("Support.take("),
        "run_body_saved_name": bool(re.search(r"\bsaved\d*\b", body_code)),
        "run_body_local_name": bool(re.search(r"\blocal\d*\b", body_code)),
        "run_source_map_bcis": run_source_map_bcis,
    }
    if jarde_compile == 0:
        summary["jarde_equal"] = (EVIDENCE / "original.txt").read_bytes() == (EVIDENCE / "jarde.txt").read_bytes()
    if jadx_compile == 0:
        summary["jadx_equal"] = (EVIDENCE / "original.txt").read_bytes() == (EVIDENCE / "jadx.txt").read_bytes()
    (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
