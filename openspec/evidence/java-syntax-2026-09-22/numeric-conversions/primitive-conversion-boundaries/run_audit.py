#!/usr/bin/env python3
"""Reproduce JVM-verified primitive-conversion storage/effect boundaries."""

from __future__ import annotations

import hashlib
import json
import struct
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = [
    "BoundaryProbe.java",
    "BoundaryEffects.java",
    "BooleanDescriptor.java",
    "BoundaryRunner.java",
    "BooleanRunner.java",
    "DiscardedConversion.java",
    "DiscardRunner.java",
]
CLASSES = ["BoundaryProbe", "BooleanDescriptor", "DiscardedConversion"]


def run(args: list[str], output: Path, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    output.write_text(result.stdout + result.stderr)
    return result


def read_u2(data: bytearray, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def read_u4(data: bytearray, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def patch_method_code(class_path: Path, method: str, wanted_descriptor: str, old_code: bytes, new_code: bytes) -> None:
    """Replace a straight-line method body while preserving its valid classfile envelope."""
    data = bytearray(class_path.read_bytes())
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise AssertionError("not a class file")
    cp_count = read_u2(data, 8)
    offset = 10
    utf8: dict[int, str] = {}
    index = 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = read_u2(data, offset)
            offset += 2
            utf8[index] = bytes(data[offset : offset + size]).decode("utf-8", errors="replace")
            offset += size
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise AssertionError(f"unknown constant-pool tag {tag}")
        index += 1

    offset += 6
    interfaces = read_u2(data, offset)
    offset += 2 + interfaces * 2

    def skip_members(cursor: int, count: int) -> int:
        for _ in range(count):
            cursor += 6
            attributes = read_u2(data, cursor)
            cursor += 2
            for _ in range(attributes):
                length = read_u4(data, cursor + 2)
                cursor += 6 + length
        return cursor

    fields = read_u2(data, offset)
    offset = skip_members(offset + 2, fields)
    methods = read_u2(data, offset)
    offset += 2
    patched = False
    for _ in range(methods):
        name = utf8[read_u2(data, offset + 2)]
        descriptor = utf8[read_u2(data, offset + 4)]
        attributes = read_u2(data, offset + 6)
        cursor = offset + 8
        for _ in range(attributes):
            attr_name = utf8[read_u2(data, cursor)]
            attr_length = read_u4(data, cursor + 2)
            info = cursor + 6
            if name == method and descriptor == wanted_descriptor and attr_name == "Code":
                code_length = read_u4(data, info + 4)
                code_start = info + 8
                code = bytes(data[code_start : code_start + code_length])
                if code != old_code:
                    raise AssertionError(f"unexpected javac code for {method}{descriptor}: {code.hex()}")
                data[code_start : code_start + code_length] = new_code
                delta = len(new_code) - code_length
                if 0x1A in new_code:
                    struct.pack_into(">H", data, info, max(read_u2(data, info), 1))
                struct.pack_into(">I", data, info + 4, len(new_code))
                struct.pack_into(">I", data, cursor + 2, attr_length + delta)
                patched = True
            cursor += 6 + attr_length
        offset = cursor
    if not patched:
        raise AssertionError(f"method {method}{descriptor} Code attribute not found")
    class_path.write_bytes(data)


def hash_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    cli_path = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "target/debug/jarde-cli"
    if not cli_path.is_file():
        raise SystemExit(f"Jarde CLI missing: {cli_path}; pass its built path as the first argument")
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    artifact_dir = EVIDENCE / "original-classes"
    artifact_dir.mkdir(exist_ok=True)
    sources = [EVIDENCE / source for source in SOURCES]
    source_hashes = {source.name: hash_file(source) for source in sources}
    cli_hash_start = hash_file(cli_path)
    (EVIDENCE / "cli-hash-input.txt").write_text(f"{cli_hash_start}  {cli_path}\n")

    with tempfile.TemporaryDirectory(prefix="jarde-primitive-boundaries-") as tmp:
        work = Path(tmp)
        original = work / "original"
        original.mkdir()
        source_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(original), *(str(path) for path in sources)],
            EVIDENCE / "source-javac.log",
        )
        (EVIDENCE / "source-javac-status.txt").write_text(f"exit={source_compile.returncode}\n")
        if source_compile.returncode:
            raise SystemExit(source_compile.returncode)

        boolean_class = original / "BooleanDescriptor.class"
        patch_method_code(boolean_class, "convert", "(Z)Z", bytes((0x1A, 0xAC)), bytes((0x1A, 0x91, 0xAC)))
        discarded_class = original / "DiscardedConversion.class"
        patch_method_code(discarded_class, "drop", "(I)V", bytes((0xB1,)), bytes((0x1A, 0x91, 0x57, 0xB1)))
        for name in CLASSES:
            target = original / f"{name}.class"
            (artifact_dir / target.name).write_bytes(target.read_bytes())

        original_verify = run(
            ["java", "-Xverify:all", "-cp", str(original), "BoundaryRunner"],
            EVIDENCE / "original.txt",
            cwd=original,
        )
        original_boolean = run(
            ["java", "-Xverify:all", "-cp", str(original), "BooleanRunner"],
            EVIDENCE / "original-boolean.txt",
            cwd=original,
        )
        original_discarded = run(
            ["java", "-Xverify:all", "-cp", str(original), "DiscardRunner"],
            EVIDENCE / "original-discarded.txt",
            cwd=original,
        )
        (EVIDENCE / "original-status.txt").write_text(f"exit={original_verify.returncode}\n")
        (EVIDENCE / "original-boolean-status.txt").write_text(f"exit={original_boolean.returncode}\n")
        (EVIDENCE / "original-discarded-status.txt").write_text(f"exit={original_discarded.returncode}\n")
        if original_verify.returncode or original_boolean.returncode or original_discarded.returncode:
            raise SystemExit("original JVM verification/execution failed")
        for name in CLASSES:
            run(["javap", "-p", "-c", "-v", str(original / f"{name}.class")], EVIDENCE / f"{name}-javap.txt")

        stages: dict[str, dict[str, object]] = {}
        for name in CLASSES:
            cli = subprocess.run(
                [
                    str(cli_path), "class-source", "--input", str(original / f"{name}.class"),
                    "--class", name, "--policy", "single-class", "--release", "8",
                    "--format", "text", "--evidence", "all",
                ],
                capture_output=True,
                text=True,
                timeout=90,
            )
            (EVIDENCE / f"jarde-{name}.java.txt").write_text(cli.stdout)
            (EVIDENCE / f"jarde-{name}-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
            stages[name] = {"cli_exit": cli.returncode, "quote_count": cli.stdout.count("@bytecode")}

        jarde_dir = work / "jarde"
        jarde_dir.mkdir()
        for source in sources:
            (jarde_dir / source.name).write_text(source.read_text())
        for name in CLASSES:
            (jarde_dir / f"{name}.java").write_text((EVIDENCE / f"jarde-{name}.java.txt").read_text())
        jarde_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(jarde_dir / "classes"),
             *(str(jarde_dir / source) for source in ("BoundaryProbe.java", "BoundaryEffects.java", "BoundaryRunner.java"))],
            EVIDENCE / "jarde-javac.log",
        )
        (EVIDENCE / "jarde-javac-status.txt").write_text(f"exit={jarde_compile.returncode}\n")
        jarde_run = None
        if jarde_compile.returncode == 0:
            jarde_run = run(["java", "-Xverify:all", "-cp", str(jarde_dir / "classes"), "BoundaryRunner"], EVIDENCE / "jarde.txt", cwd=jarde_dir)
            (EVIDENCE / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")
        jarde_boolean_compile = run(
            ["javac", "--release", "8", "-g:none", "-cp", str(original), "-d", str(jarde_dir / "boolean-classes"),
             str(jarde_dir / "BooleanDescriptor.java"), str(jarde_dir / "BooleanRunner.java")],
            EVIDENCE / "jarde-boolean-javac.log",
        )
        (EVIDENCE / "jarde-boolean-javac-status.txt").write_text(f"exit={jarde_boolean_compile.returncode}\n")
        jarde_boolean_run = None
        if jarde_boolean_compile.returncode == 0:
            jarde_boolean_run = run(["java", "-Xverify:all", "-cp", f"{jarde_dir / 'boolean-classes'}:{original}", "BooleanRunner"], EVIDENCE / "jarde-boolean.txt", cwd=jarde_dir)
            (EVIDENCE / "jarde-boolean-status.txt").write_text(f"exit={jarde_boolean_run.returncode}\n")
        jarde_discard_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(jarde_dir / "discard-classes"),
             str(jarde_dir / "DiscardedConversion.java"), str(jarde_dir / "DiscardRunner.java")],
            EVIDENCE / "jarde-discard-javac.log",
        )
        (EVIDENCE / "jarde-discard-javac-status.txt").write_text(f"exit={jarde_discard_compile.returncode}\n")
        jarde_discard_run = None
        if jarde_discard_compile.returncode == 0:
            jarde_discard_run = run(["java", "-Xverify:all", "-cp", str(jarde_dir / "discard-classes"), "DiscardRunner"], EVIDENCE / "jarde-discard.txt", cwd=jarde_dir)
            (EVIDENCE / "jarde-discard-status.txt").write_text(f"exit={jarde_discard_run.returncode}\n")

        jadx_outcomes: dict[str, object] = {}
        jadx_probe_run = None
        jadx_boolean_run = None
        jadx_discard_run = None
        for name in CLASSES:
            generated = work / f"jadx-{name}"
            jadx = run(["jadx", "--no-res", "-d", str(generated), str(original / f"{name}.class")], EVIDENCE / f"jadx-{name}.log")
            jadx_source = next(generated.rglob(f"{name}.java"), None)
            jadx_outcomes[name] = {"jadx_exit": jadx.returncode, "source_found": jadx_source is not None}
            if jadx_source is None:
                jadx_outcomes[name]["javac_exit"] = None
                continue
            (EVIDENCE / f"jadx-{name}.java.txt").write_text(jadx_source.read_text())
            package_line = next((line for line in jadx_source.read_text().splitlines() if line.startswith("package ")), "")
            package_prefix = package_line + "\n" if package_line else ""
            package_name = package_line.removeprefix("package ").removesuffix(";")
            support = work / f"jadx-support-{name}"
            support.mkdir()
            runner_name = "BoundaryRunner.java" if name == "BoundaryProbe" else "BooleanRunner.java"
            if name == "DiscardedConversion":
                runner_name = "DiscardRunner.java"
            for source_name in ("BoundaryEffects.java", runner_name):
                source = EVIDENCE / source_name
                if source_name == "BoundaryEffects.java" and name != "BoundaryProbe":
                    continue
                (support / source_name).write_text(package_prefix + source.read_text())
            compile_dir = work / f"jadx-classes-{name}"
            compile_args = ["javac", "--release", "8", "-g:none", "-d", str(compile_dir), str(jadx_source)]
            compile_args.extend(str(support / n) for n in (runner_name,))
            if name == "BoundaryProbe":
                compile_args.append(str(support / "BoundaryEffects.java"))
            javac = run(compile_args, EVIDENCE / f"jadx-{name}-javac.log")
            (EVIDENCE / f"jadx-{name}-javac-status.txt").write_text(f"exit={javac.returncode}\n")
            jadx_outcomes[name]["javac_exit"] = javac.returncode
            if javac.returncode == 0:
                main_name = {"BoundaryProbe": "BoundaryRunner", "BooleanDescriptor": "BooleanRunner", "DiscardedConversion": "DiscardRunner"}[name]
                main_class = f"{package_name}.{main_name}" if package_name else main_name
                run_log = EVIDENCE / {"BoundaryProbe": "jadx.txt", "BooleanDescriptor": "jadx-boolean.txt", "DiscardedConversion": "jadx-discard.txt"}[name]
                outcome = run(["java", "-Xverify:all", "-cp", str(compile_dir), main_class], run_log, cwd=work)
                status_name = {"BoundaryProbe": "jadx-status.txt", "BooleanDescriptor": "jadx-boolean-status.txt", "DiscardedConversion": "jadx-discard-status.txt"}[name]
                (EVIDENCE / status_name).write_text(f"exit={outcome.returncode}\n")
                if name == "BoundaryProbe":
                    jadx_probe_run = outcome
                elif name == "BooleanDescriptor":
                    jadx_boolean_run = outcome
                else:
                    jadx_discard_run = outcome

        cli_hash_end = hash_file(cli_path)
        (EVIDENCE / "cli-hash-end.txt").write_text(f"{cli_hash_end}  {cli_path}\n")
        summary = {
            "java_version": subprocess.run(["java", "-version"], capture_output=True, text=True).stderr.splitlines()[0],
            "release": 8,
            "source_sha256": source_hashes,
            "patched_boolean_descriptor": {
                "descriptor": "(Z)Z",
                "method_code": "iload_0; i2b; ireturn",
                "verification": "java -Xverify:all exited 0",
                "class_sha256": hash_file(artifact_dir / "BooleanDescriptor.class"),
            },
            "discarded_conversion": {
                "method_descriptor": "(I)V",
                "method_code": "iload_0; i2b; pop; return",
                "verification": "java -Xverify:all exited 0",
                "class_sha256": hash_file(artifact_dir / "DiscardedConversion.class"),
            },
            "original_class_sha256": {name: hash_file(artifact_dir / f"{name}.class") for name in CLASSES},
            "original_exit": original_verify.returncode,
            "original_cases": len((EVIDENCE / "original.txt").read_text().splitlines()),
            "original_boolean_output": (EVIDENCE / "original-boolean.txt").read_text().strip(),
            "original_discarded_output": (EVIDENCE / "original-discarded.txt").read_text().strip(),
            "jarde_classes": stages,
            "jarde_javac_exit": jarde_compile.returncode,
            "jarde_run_exit": jarde_run.returncode if jarde_run else None,
            "jarde_boolean_javac_exit": jarde_boolean_compile.returncode,
            "jarde_boolean_run_exit": jarde_boolean_run.returncode if jarde_boolean_run else None,
            "jarde_discard_javac_exit": jarde_discard_compile.returncode,
            "jarde_discard_run_exit": jarde_discard_run.returncode if jarde_discard_run else None,
            "original_jarde_discard_equal": (EVIDENCE / "jarde-discard.txt").read_bytes() == (EVIDENCE / "original-discarded.txt").read_bytes() if jarde_discard_run else None,
            "jadx": jadx_outcomes,
            "jadx_run_exit": jadx_probe_run.returncode if jadx_probe_run else None,
            "jadx_boolean_run_exit": jadx_boolean_run.returncode if jadx_boolean_run else None,
            "jadx_discard_run_exit": jadx_discard_run.returncode if jadx_discard_run else None,
            "original_jarde_equal": (EVIDENCE / "jarde.txt").read_bytes() == (EVIDENCE / "original.txt").read_bytes() if jarde_run else None,
            "original_jadx_discard_equal": (EVIDENCE / "jadx-discard.txt").read_bytes() == (EVIDENCE / "original-discarded.txt").read_bytes() if jadx_discard_run else None,
            "original_jadx_equal": (EVIDENCE / "jadx.txt").read_bytes() == (EVIDENCE / "original.txt").read_bytes() if jadx_probe_run else None,
            "cli_sha256_before": cli_hash_start,
            "cli_sha256_after": cli_hash_end,
            "cli_unchanged": cli_hash_start == cli_hash_end,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        manifest = []
        for path in sorted(EVIDENCE.rglob("*")):
            if path.is_file() and path.name != "sha256sums.txt":
                manifest.append(f"{hash_file(path)}  {path.relative_to(EVIDENCE)}")
        (EVIDENCE / "sha256sums.txt").write_text("\n".join(manifest) + "\n")
        if cli_hash_start != cli_hash_end:
            raise SystemExit("Jarde CLI changed during evidence collection")


if __name__ == "__main__":
    main()
