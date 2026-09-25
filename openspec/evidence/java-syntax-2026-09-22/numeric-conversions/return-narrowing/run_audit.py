from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
OUT = ROOT / "openspec/evidence/java-syntax-2026-09-22/numeric-conversions/return-narrowing"
WORK = Path("/tmp/jarde-return-narrowing-0923")
CLI = ROOT / "target/debug/jarde-cli"
VALUES = [
    -2**31,
    2**31 - 1,
    -65537,
    -32769,
    -129,
    -2,
    -1,
    0,
    1,
    2,
    128,
    32768,
    65535,
]

CASES = {
    "ReturnByte": ("B", "byte", "byte"),
    "ReturnChar": ("C", "char", "char"),
    "ReturnShort": ("S", "short", "short"),
    "ReturnBoolean": ("Z", "boolean", "boolean"),
}


def run(args: list[str], log: Path, *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=60, cwd=cwd)
    log.write_text(result.stdout + result.stderr, encoding="utf-8")
    return result


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def u1(data: bytes, offset: int) -> tuple[int, int]:
    return data[offset], offset + 1


def u2(data: bytes, offset: int) -> tuple[int, int]:
    return int.from_bytes(data[offset : offset + 2], "big"), offset + 2


def u4(data: bytes, offset: int) -> tuple[int, int]:
    return int.from_bytes(data[offset : offset + 4], "big"), offset + 4


def skip_attributes(data: bytes, offset: int, count: int) -> tuple[int, list[tuple[int, int, int]]]:
    attributes = []
    for _ in range(count):
        name_index, offset = u2(data, offset)
        length, offset = u4(data, offset)
        start = offset
        attributes.append((name_index, start, length))
        offset += length
    return offset, attributes


def code_digests(data: bytes) -> list[str]:
    """Return method Code attribute payload hashes from a small class-file reader."""
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    offset = 8
    cp_count, offset = u2(data, offset)
    cp: list[bytes | None] = [None] * cp_count
    index = 1
    while index < cp_count:
        tag, offset = u1(data, offset)
        if tag == 1:
            length, offset = u2(data, offset)
            cp[index] = data[offset : offset + length]
            offset += length
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
            raise ValueError(f"unknown constant-pool tag {tag} at {index}")
        index += 1

    offset += 6  # access_flags, this_class, super_class
    interfaces, offset = u2(data, offset)
    offset += 2 * interfaces

    def skip_members(offset: int) -> tuple[int, list[tuple[int, int, int]]]:
        count, offset = u2(data, offset)
        all_attributes: list[tuple[int, int, int]] = []
        for _ in range(count):
            offset += 6  # access_flags, name_index, descriptor_index
            attribute_count, offset = u2(data, offset)
            offset, attributes = skip_attributes(data, offset, attribute_count)
            all_attributes.extend(attributes)
        return offset, all_attributes

    offset, _ = skip_members(offset)  # fields
    offset, method_attributes = skip_members(offset)
    class_attribute_count, offset = u2(data, offset)
    _, class_attributes = skip_attributes(data, offset, class_attribute_count)
    code_name = b"Code"
    hashes: list[str] = []
    for name_index, start, length in method_attributes:
        if cp[name_index] == code_name:
            hashes.append(hashlib.sha256(data[start : start + length]).hexdigest())
    # Class-level Code is impossible, but keeping this parser's boundary explicit avoids silently
    # treating a malformed attribute table as method code.
    if class_attributes:
        pass
    return hashes


def patch_descriptor(original: Path, patched: Path, descriptor: str, log: Path) -> dict[str, object]:
    before = original.read_bytes()
    needle = b"(I)I"
    replacement = f"(I){descriptor}".encode("ascii")
    if len(replacement) != len(needle):
        raise AssertionError("descriptor replacement changed class-file width")
    count = before.count(needle)
    if count != 1:
        raise AssertionError(f"expected one unique (I)I Utf8 descriptor, found {count}")
    after = before.replace(needle, replacement, 1)
    patched.write_bytes(after)
    changed = [index for index, (left, right) in enumerate(zip(before, after)) if left != right]
    if changed != list(range(changed[0], changed[-1] + 1)):
        raise AssertionError("descriptor patch changed disjoint byte ranges")
    before_codes = code_digests(before)
    after_codes = code_digests(after)
    if before_codes != after_codes:
        raise AssertionError("Code attribute payload changed during descriptor patch")
    result = {
        "descriptor_before": "(I)I",
        "descriptor_after": f"(I){descriptor}",
        "descriptor_occurrences": count,
        "changed_byte_range": [changed[0], changed[-1] + 1],
        "changed_byte_count": len(changed),
        "original_sha256": hashlib.sha256(before).hexdigest(),
        "patched_sha256": hashlib.sha256(after).hexdigest(),
        "code_attribute_sha256_before": before_codes,
        "code_attribute_sha256_after": after_codes,
    }
    log.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


def source_for(class_name: str) -> str:
    return f"public class {class_name} {{\n    public static int run(int value) {{ return value; }}\n}}\n"


def runner_for(class_name: str, kind: str) -> str:
    lines = [
        f"public class {class_name}Runner {{",
        "    public static void main(String[] args) {",
        f"        int[] values = new int[] {{{', '.join(str(value) for value in VALUES)}}};",
        "        for (int value : values) {",
    ]
    if kind == "C":
        expression = f"(int) {class_name}.run(value)"
    else:
        expression = f"{class_name}.run(value)"
    lines.append(f'            System.out.println(value + ":" + {expression});')
    lines.extend(["        }", "    }", "}"])
    return "\n".join(lines) + "\n"


def expected(kind: str) -> str:
    rows = []
    for value in VALUES:
        if kind == "B":
            result = ((value + 128) % 256) - 128
            rendered = str(result)
        elif kind == "C":
            rendered = str(value & 0xFFFF)
        elif kind == "S":
            result = ((value + 32768) % 65536) - 32768
            rendered = str(result)
        else:
            rendered = "true" if value & 1 else "false"
        rows.append(f"{value}:{rendered}")
    return "\n".join(rows) + "\n"


def package_prefix(source: str) -> tuple[str, str]:
    for line in source.splitlines():
        if line.startswith("package "):
            package = line[len("package ") :].rstrip(";")
            return line, package + "."
    return "", ""


if WORK.exists():
    raise SystemExit(f"refusing to overwrite existing work directory: {WORK}")
existing_output = [entry for entry in OUT.iterdir() if entry.name != "run_audit.py"] if OUT.exists() else []
if existing_output:
    raise SystemExit(f"refusing to overwrite non-empty evidence directory: {OUT}")
if not CLI.is_file():
    raise SystemExit(f"missing CLI: {CLI}")

WORK.mkdir(parents=True)
OUT.mkdir(parents=True, exist_ok=True)
(OUT / "sources").mkdir()

cli_before = sha256(CLI)
(OUT / "cli-sha-before.txt").write_text(cli_before + "  jarde-cli\n", encoding="utf-8")
(OUT / "environment.txt").write_text(
    subprocess.run(["javac", "-version"], capture_output=True, text=True).stderr
    + subprocess.run(["java", "-version"], capture_output=True, text=True).stderr
    + subprocess.run(["jadx", "--version"], capture_output=True, text=True).stdout,
    encoding="utf-8",
)

summary: dict[str, object] = {
    "release": 8,
    "inputs": VALUES,
    "cli_sha256_before": cli_before,
    "cases": {},
}

for class_name, (descriptor, java_type, _) in CASES.items():
    case = OUT / class_name
    case.mkdir()
    source = source_for(class_name)
    runner = runner_for(class_name, descriptor)
    source_path = OUT / "sources" / f"{class_name}.java"
    runner_path = OUT / "sources" / f"{class_name}Runner.java"
    source_path.write_text(source, encoding="utf-8")
    runner_path.write_text(runner, encoding="utf-8")

    source_dir = WORK / class_name / "source"
    source_dir.mkdir(parents=True)
    source_result = run(
        ["javac", "--release", "8", "-g:none", "-d", str(source_dir), str(source_path)],
        case / "source-javac.log",
    )
    if source_result.returncode:
        raise SystemExit(f"source javac failed for {class_name}")
    original_class = source_dir / f"{class_name}.class"
    run(["javap", "-v", "-c", "-p", str(original_class)], case / "javap-before.txt")

    patched_dir = WORK / class_name / "patched"
    patched_dir.mkdir()
    patched_class = patched_dir / f"{class_name}.class"
    patch = patch_descriptor(original_class, patched_class, descriptor, case / "patch.json")
    run(["javap", "-v", "-c", "-p", str(patched_class)], case / "javap.txt")
    (case / "patched-class-sha256.txt").write_text(
        patch["patched_sha256"] + f"  {class_name}.class\n", encoding="utf-8"
    )

    original_runner_dir = WORK / class_name / "original-runner"
    original_runner_dir.mkdir()
    original_compile = run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(patched_dir),
            "-d",
            str(original_runner_dir),
            str(runner_path),
        ],
        case / "runner-javac.log",
    )
    if original_compile.returncode:
        raise SystemExit(f"patched runner javac failed for {class_name}")
    original_run = run(
        [
            "java",
            "-Xverify:all",
            "-cp",
            f"{original_runner_dir}:{patched_dir}",
            f"{class_name}Runner",
        ],
        case / "original.txt",
    )
    if original_run.returncode:
        raise SystemExit(f"patched class execution failed for {class_name}")
    expected_text = expected(descriptor)
    (case / "expected.txt").write_text(expected_text, encoding="utf-8")
    if original_run.stdout != expected_text:
        raise SystemExit(f"patched class did not match JVMS expected output for {class_name}")

    jadx_dir = WORK / class_name / "jadx"
    jadx_result = run(["jadx", "--no-res", "-d", str(jadx_dir), str(patched_class)], case / "jadx.log")
    jadx_source_path = next(jadx_dir.rglob(f"{class_name}.java"), None) if jadx_result.returncode == 0 else None
    jadx_compile_code = None
    jadx_run_code = None
    if jadx_source_path is not None:
        jadx_source = jadx_source_path.read_text(encoding="utf-8")
        (case / "jadx.java.txt").write_text(jadx_source, encoding="utf-8")
        package_line, package_prefix_text = package_prefix(jadx_source)
        jadx_runner = package_line + "\n" + runner if package_line else runner
        jadx_runner_path = WORK / class_name / f"{class_name}Runner.java"
        jadx_runner_path.write_text(jadx_runner, encoding="utf-8")
        jadx_classes = WORK / class_name / "jadx-classes"
        jadx_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(jadx_classes), str(jadx_source_path), str(jadx_runner_path)],
            case / "jadx-javac.log",
        )
        jadx_compile_code = jadx_compile.returncode
        if jadx_compile.returncode == 0:
            jadx_run = run(
                ["java", "-Xverify:all", "-cp", str(jadx_classes), package_prefix_text + f"{class_name}Runner"],
                case / "jadx.txt",
            )
            jadx_run_code = jadx_run.returncode
            if jadx_run.returncode == 0 and (case / "jadx.txt").read_text(encoding="utf-8") != expected_text:
                raise SystemExit(f"JADX output mismatch for {class_name}")
    else:
        (case / "jadx.java.txt").write_text("<no JADX source>\n", encoding="utf-8")
        (case / "jadx-javac.log").write_text("<not run: JADX did not produce source>\n", encoding="utf-8")
        (case / "jadx.txt").write_text("<not run: JADX did not produce source>\n", encoding="utf-8")

    jarde = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--input",
            str(patched_class),
            "--class",
            class_name,
            "--policy",
            "single-class",
            "--release",
            "8",
            "--format",
            "text",
        ],
        capture_output=True,
        text=True,
        timeout=60,
    )
    (case / "jarde.java.txt").write_text(jarde.stdout, encoding="utf-8")
    (case / "jarde-report.txt").write_text(jarde.stderr, encoding="utf-8")
    jarde_compile_code = None
    jarde_run_code = None
    if jarde.returncode == 0:
        jarde_source = jarde.stdout
        jarde_package_line, jarde_package_prefix = package_prefix(jarde_source)
        jarde_runner = jarde_package_line + "\n" + runner if jarde_package_line else runner
        jarde_runner_path = WORK / class_name / f"{class_name}Runner.java"
        jarde_runner_path.write_text(jarde_runner, encoding="utf-8")
        jarde_source_path = WORK / class_name / f"{class_name}.java"
        jarde_source_path.write_text(jarde_source, encoding="utf-8")
        jarde_classes = WORK / class_name / "jarde-classes"
        jarde_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(jarde_classes), str(jarde_source_path), str(jarde_runner_path)],
            case / "jarde-javac.log",
        )
        jarde_compile_code = jarde_compile.returncode
        if jarde_compile.returncode == 0:
            jarde_run = run(
                ["java", "-Xverify:all", "-cp", str(jarde_classes), jarde_package_prefix + f"{class_name}Runner"],
                case / "jarde.txt",
            )
            jarde_run_code = jarde_run.returncode
            if jarde_run.returncode == 0 and (case / "jarde.txt").read_text(encoding="utf-8") != expected_text:
                raise SystemExit(f"jarde output mismatch for {class_name}")
    else:
        (case / "jarde-javac.log").write_text("<not run: jarde class-source failed>\n", encoding="utf-8")
        (case / "jarde.txt").write_text("<not run: jarde class-source failed>\n", encoding="utf-8")

    javap = (case / "javap.txt").read_text(encoding="utf-8")
    summary["cases"][class_name] = {
        "descriptor": f"(I){descriptor}",
        "java_type": java_type,
        "bytes": patched_class.stat().st_size,
        "source_class_sha256": patch["original_sha256"],
        "class_sha256": sha256(patched_class),
        "code_attribute_count": len(patch["code_attribute_sha256_after"]),
        "code_attribute_sha256": patch["code_attribute_sha256_after"],
        "runner_cases": len(VALUES),
        "ireturn_count": len(re.findall(r"^\s+\d+:\s+ireturn\b", javap, re.MULTILINE)),
        "jadx_javac": jadx_compile_code,
        "jadx_run": jadx_run_code,
        "jarde_exit": jarde.returncode,
        "jarde_javac": jarde_compile_code,
        "jarde_run": jarde_run_code,
        "jarde_quotes": jarde.stdout.count("@bytecode"),
    }

cli_after = sha256(CLI)
(OUT / "cli-sha-after.txt").write_text(cli_after + "  jarde-cli\n", encoding="utf-8")
summary["cli_sha256_after"] = cli_after
summary["cli_unchanged"] = cli_before == cli_after
(OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary, indent=2))
