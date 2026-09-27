#!/usr/bin/env python3
"""Replay the JADX DT-20 diamond example with and without local generic debug data."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile


HERE = Path(__file__).resolve().parent
JADX_REPO = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_REPO / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode:
        raise SystemExit(f"{label}: exit {result.returncode}: {result.stderr[-3000:]}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def u2(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 2], "big")


def u4(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 4], "big")


def debug_attribute_offsets(data: bytes | bytearray) -> tuple[
    dict[int, tuple[int, int]], int, int, int, int, int
]:
    """Return UTF-8 entries and the method's Code/LVTT attribute boundaries."""
    cp_count = u2(data, 8)
    cp: dict[int, tuple[int, int]] = {}
    offset = 10
    index = 1
    while index < cp_count:
        tag = data[offset]
        if tag == 1:
            size = u2(data, offset + 1)
            cp[index] = (offset + 3, size)
            offset += 3 + size
        elif tag in (3, 4):
            offset += 5
        elif tag in (5, 6):
            offset += 9
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 3
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 5
        elif tag == 15:
            offset += 4
        else:
            raise SystemExit(f"unexpected constant-pool tag {tag}")
        index += 1

    def utf8(constant: int) -> bytes:
        start, size = cp[constant]
        return bytes(data[start:start + size])

    offset += 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2

    def skip_members(offset: int, count: int) -> tuple[int, list[tuple[int, int]]]:
        methods = []
        for _ in range(count):
            name_index = u2(data, offset + 2)
            attribute_count = u2(data, offset + 6)
            offset += 8
            attributes = []
            for _ in range(attribute_count):
                attribute_start = offset
                length = u4(data, offset + 2)
                attributes.append((attribute_start, length))
                offset += 6 + length
            if utf8(name_index) == b"test":
                methods = attributes
        return offset, methods

    fields = u2(data, offset)
    offset, _ = skip_members(offset + 2, fields)
    methods = u2(data, offset)
    _, attributes = skip_members(offset + 2, methods)
    code = next((item for item in attributes if utf8(u2(data, item[0])) == b"Code"), None)
    if code is None:
        raise SystemExit("test method has no Code attribute")
    code_start, code_length = code
    info = code_start + 6
    bytecode_length = u4(data, info + 4)
    exceptions = u2(data, info + 8 + bytecode_length)
    nested_count_offset = info + 10 + bytecode_length + exceptions * 8
    nested_count = u2(data, nested_count_offset)
    nested = nested_count_offset + 2
    lvtt = None
    for _ in range(nested_count):
        length = u4(data, nested + 2)
        if utf8(u2(data, nested)) == b"LocalVariableTypeTable":
            lvtt = (nested, length)
        nested += 6 + length
    if lvtt is None:
        raise SystemExit("test method has no LocalVariableTypeTable")
    return cp, code_start, code_length, nested_count_offset, *lvtt


def mutate_debug_class(data: bytes, *, duplicate_lvtt: bool = False,
                       malformed_signature: bool = False,
                       wrong_range: bool = False) -> bytes:
    cp, code_start, code_length, nested_count_offset, lvtt_start, lvtt_length = debug_attribute_offsets(data)
    mutated = bytearray(data)
    if malformed_signature:
        info = lvtt_start + 6
        signature_index = u2(mutated, info + 8)
        start, length = cp[signature_index]
        signature = bytes(mutated[start:start + length])
        if not signature.endswith(b">;"):
            raise SystemExit(f"unexpected frozen LVTT signature spelling: {signature!r}")
        mutated[start + length - 1] = ord("!")
    if wrong_range:
        info = lvtt_start + 6
        mutated[info + 2:info + 4] = (9).to_bytes(2, "big")
        mutated[info + 4:info + 6] = (11).to_bytes(2, "big")
    if duplicate_lvtt:
        raw_attribute = bytes(mutated[lvtt_start:lvtt_start + 6 + lvtt_length])
        insert_at = nested_count_offset + 2
        mutated[insert_at:insert_at] = raw_attribute
        mutated[nested_count_offset:nested_count_offset + 2] = (u2(mutated, nested_count_offset) + 1).to_bytes(2, "big")
        mutated[code_start + 2:code_start + 6] = (code_length + len(raw_attribute)).to_bytes(4, "big")
    return bytes(mutated)


def compile_sources(classes: Path, debug: str, *sources: Path) -> None:
    classes.mkdir(parents=True)
    checked(run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                debug, "-Xlint:-options", "-d", classes, *sources), "javac")


def verified_run(classes: Path, label: str) -> str:
    result = run("java", "-Xverify:all", "-cp", classes, "dt20.Runner")
    checked(result, label)
    if result.stdout != "true\n":
        raise SystemExit(f"{label}: unexpected runtime output {result.stdout!r}")
    return result.stdout


if len(sys.argv) != 2 or not Path(sys.argv[1]).is_file() or not JADX.is_file():
    raise SystemExit("usage: replay.py /absolute/path/to/jarde-cli")
cli = Path(sys.argv[1])
head = run("git", "-C", JADX_REPO, "rev-parse", "HEAD")
status = run("git", "-C", JADX_REPO, "status", "--porcelain")
if head.returncode or head.stdout.strip() != JADX_HEAD or status.returncode or status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen commit")

output = HERE / "outputs"
output.mkdir(exist_ok=True)
results = {"jadx_head": JADX_HEAD, "javac_version": run("javac", "-version").stdout.strip(),
           "source_sha256": {name: sha(HERE / name) for name in ("Diamond.java", "Runner.java")},
           "variants": {}}

with tempfile.TemporaryDirectory(prefix="jarde-dt20-audit-") as temporary:
    work = Path(temporary)
    for variant, debug in (("debug", "-g"), ("no-debug", "-g:none")):
        folder = output / variant
        folder.mkdir(exist_ok=True)
        original = work / variant / "original"
        compile_sources(original, debug, HERE / "Diamond.java", HERE / "Runner.java")
        original_run = verified_run(original, f"{variant} original")
        javap = run("javap", "-v", "-c", "-p", "-classpath", original, "dt20.Diamond")
        checked(javap, f"{variant} javap")
        (folder / "original-javap.txt").write_text(javap.stdout.replace(str(original), "<CLASSES>"))
        if ("LocalVariableTypeTable" in javap.stdout) != (variant == "debug"):
            raise SystemExit(f"{variant}: unexpected local generic debug attributes")
        if variant == "debug" and not (
            "7: astore_1" in javap.stdout
            and "8: aload_1" in javap.stdout
            and re.search(r"\s8\s+12\s+1\s+map\s+Ljava/util/Map;", javap.stdout)
            and re.search(
                r"\s8\s+12\s+1\s+map\s+Ljava/util/Map<Ljava/lang/String;Ljava/lang/String;>;",
                javap.stdout,
            )
        ):
            raise SystemExit("debug local scope/store fixture changed from the frozen BCI facts")

        jar = work / variant / "input.jar"
        with zipfile.ZipFile(jar, "w") as archive:
            info = zipfile.ZipInfo("dt20/Diamond.class", date_time=(2000, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, (original / "dt20/Diamond.class").read_bytes())
        jadx_root = work / variant / "jadx"
        checked(run(JADX, "-d", jadx_root, jar), f"{variant} JADX")
        jadx_source = jadx_root / "sources/dt20/Diamond.java"
        if not jadx_source.is_file():
            raise SystemExit(f"{variant}: JADX did not produce complete source")
        jadx_text = jadx_source.read_text()
        (folder / "jadx-Diamond.java.txt").write_text(jadx_text)
        jadx_classes = work / variant / "jadx-classes"
        compile_sources(jadx_classes, debug, jadx_source, HERE / "Runner.java")
        jadx_run = verified_run(jadx_classes, f"{variant} JADX")

        jarde = run(cli, "class-source", "--input", jar, "--class", "dt20/Diamond",
                    "--policy", "plain-jar", "--release", "8", "--format", "text")
        checked(jarde, f"{variant} Jarde")
        jarde_text = jarde.stdout
        (folder / "jarde-Diamond.java.txt").write_text(jarde_text)
        jarde_source = work / variant / "jarde/dt20/Diamond.java"
        jarde_source.parent.mkdir(parents=True)
        jarde_source.write_text(jarde_text)
        jarde_classes = work / variant / "jarde-classes"
        compile_sources(jarde_classes, debug, jarde_source, HERE / "Runner.java")
        jarde_run = verified_run(jarde_classes, f"{variant} Jarde")
        if variant == "debug":
            if "Map<String, String> map = new HashMap<>();" not in jadx_text:
                raise SystemExit("JADX debug source lost its asserted diamond and generic local")
            if "java.util.Map<java.lang.String, java.lang.String> map = new java.util.HashMap<>();" not in jarde_text:
                raise SystemExit("Jarde did not recover the proved generic local and its exact diamond")
        else:
            if "new HashMap().get(\"test\")" not in jadx_text:
                raise SystemExit("JADX no-debug raw/cast control changed")
            if "java.util.Map<java.lang.String" in jarde_text or "new java.util.HashMap<>" in jarde_text:
                raise SystemExit("Jarde inferred generic local spelling without debug metadata")
            if "(java.lang.String) local1.get((java.lang.Object) \"test\")" not in jarde_text:
                raise SystemExit("Jarde no-debug raw/cast control changed")
        results["variants"][variant] = {
            "original_class_sha256": sha(original / "dt20/Diamond.class"),
            "original_verified_run": original_run,
            "jadx_verified_run": jadx_run,
            "jarde_verified_run": jarde_run,
        }

    controls = {
        "second-write": """package dt20;
import java.util.HashMap;
import java.util.Map;
public class Diamond {
  public String test() {
    Map<String, String> map = new HashMap<>();
    map = new HashMap<>();
    return map.get("test");
  }
}
""",
        "slot-reuse": """package dt20;
import java.util.HashMap;
import java.util.Map;
public class Diamond {
  public String test() {
    { Map<String, String> map = new HashMap<>(); map.get("first"); }
    Map<String, String> map = new HashMap<>();
    return map.get("test");
  }
}
""",
        "different-signature": """package dt20;
import java.util.HashMap;
import java.util.Map;
public class Diamond {
  public String test() {
    Map<Integer, Integer> map = new HashMap<>();
    return map.get(7) == null ? null : "present";
  }
}
""",
    }
    unrelated_allocation_source = work / "unrelated-allocation/dt20/Diamond.java"
    unrelated_allocation_source.parent.mkdir(parents=True)
    unrelated_allocation_source.write_text("""package dt20;
import java.util.HashMap;
import java.util.Map;
public class Diamond {
  public String test() {
    Map<String, String> map = new HashMap<>();
    Object other = new HashMap();
    other.toString();
    return map.get("test");
  }
}
""")
    unrelated_classes = work / "unrelated-allocation/classes"
    compile_sources(unrelated_classes, "-g", unrelated_allocation_source, HERE / "Runner.java")
    verified_run(unrelated_classes, "unrelated allocation original")
    unrelated_jar = work / "unrelated-allocation/input.jar"
    with zipfile.ZipFile(unrelated_jar, "w") as archive:
        for class_file in sorted((unrelated_classes / "dt20").glob("*.class")):
            archive.write(class_file, class_file.relative_to(unrelated_classes).as_posix())
    unrelated_jarde = run(cli, "class-source", "--input", unrelated_jar, "--class", "dt20/Diamond",
                          "--policy", "plain-jar", "--release", "8", "--format", "text")
    checked(unrelated_jarde, "unrelated allocation Jarde")
    expected_local = "java.util.Map<java.lang.String, java.lang.String> map = new java.util.HashMap<>();"
    raw_other = "java.util.HashMap other = new java.util.HashMap();"
    if expected_local not in unrelated_jarde.stdout or raw_other not in unrelated_jarde.stdout:
        raise SystemExit(f"the exact allocation/local pair was not isolated:\n{unrelated_jarde.stdout}")
    unrelated_source = work / "unrelated-allocation/jarde/Diamond.java"
    unrelated_source.parent.mkdir(parents=True)
    unrelated_source.write_text(unrelated_jarde.stdout)
    unrelated_recompiled = work / "unrelated-allocation/jarde-classes"
    compile_sources(unrelated_recompiled, "-g", unrelated_source, HERE / "Runner.java")
    verified_run(unrelated_recompiled, "unrelated allocation Jarde")
    results["allocation_isolation"] = "proved local uses diamond; unrelated HashMap remains raw"

    results["refusal_controls"] = {}
    for name, source in controls.items():
        control_dir = work / "controls" / name
        source_file = control_dir / "dt20/Diamond.java"
        source_file.parent.mkdir(parents=True)
        source_file.write_text(source)
        classes = control_dir / "classes"
        compile_sources(classes, "-g", source_file, HERE / "Runner.java")
        verified_run(classes, f"{name} original")
        javap = run("javap", "-v", "-c", "-p", "-classpath", classes, "dt20.Diamond")
        checked(javap, f"{name} javap")
        if name == "slot-reuse":
            same_slot = re.findall(r"\s\d+\s+\d+\s+1\s+map\s+Ljava/util/Map;", javap.stdout)
            if len(same_slot) != 2:
                raise SystemExit("slot-reuse fixture did not produce two slot-1 map lifetimes")
        jar = control_dir / "input.jar"
        with zipfile.ZipFile(jar, "w") as archive:
            for class_file in sorted((classes / "dt20").glob("*.class")):
                archive.write(class_file, class_file.relative_to(classes).as_posix())
        jarde = run(cli, "class-source", "--input", jar, "--class", "dt20/Diamond",
                    "--policy", "plain-jar", "--release", "8", "--format", "text")
        checked(jarde, f"{name} Jarde")
        if "java.util.Map<java.lang.String, java.lang.String>" in jarde.stdout or "new java.util.HashMap<>" in jarde.stdout:
            raise SystemExit(f"{name}: unproved local generic projection was published")
        recovered = control_dir / "jarde/Diamond.java"
        recovered.parent.mkdir(parents=True)
        recovered.write_text(jarde.stdout)
        recovered_classes = control_dir / "jarde-classes"
        compile_sources(recovered_classes, "-g", recovered, HERE / "Runner.java")
        verified_run(recovered_classes, f"{name} Jarde")
        results["refusal_controls"][name] = "raw local and constructor retained; Java 8 verifier passed"

    original_classes = work / "debug/original"
    original_diamond = original_classes / "dt20/Diamond.class"
    original_runner = original_classes / "dt20/Runner.class"
    attribute_mutations = {
        "malformed-lvtt": {"malformed_signature": True},
        "wrong-lvtt-range": {"wrong_range": True},
        "duplicate-lvtt": {"duplicate_lvtt": True},
    }
    for name, options in attribute_mutations.items():
        control_dir = work / "attribute-controls" / name
        classes = control_dir / "classes/dt20"
        classes.mkdir(parents=True)
        mutated_class = mutate_debug_class(original_diamond.read_bytes(), **options)
        (classes / "Diamond.class").write_bytes(mutated_class)
        shutil.copy2(original_runner, classes / "Runner.class")
        if name not in ("duplicate-lvtt", "wrong-lvtt-range"):
            verified_run(classes.parent, f"{name} original")
        jar = control_dir / "input.jar"
        with zipfile.ZipFile(jar, "w") as archive:
            archive.write(classes / "Diamond.class", "dt20/Diamond.class")
        jarde = run(cli, "class-source", "--input", jar, "--class", "dt20/Diamond",
                    "--policy", "plain-jar", "--release", "8", "--format", "text")
        checked(jarde, f"{name} Jarde")
        if "java.util.Map<java.lang.String, java.lang.String>" in jarde.stdout or "new java.util.HashMap<>" in jarde.stdout:
            raise SystemExit(f"{name}: ambiguous/malformed debug metadata was published")
        recovered = control_dir / "jarde/Diamond.java"
        recovered.parent.mkdir(parents=True)
        recovered.write_text(jarde.stdout)
        recovered_classes = control_dir / "jarde-classes"
        compile_sources(recovered_classes, "-g", recovered, HERE / "Runner.java")
        verified_run(recovered_classes, f"{name} Jarde")
        results["refusal_controls"][name] = (
            "raw local retained; recovered Java 8 verifier passed; JVM rejects mutated input metadata"
            if name in ("duplicate-lvtt", "wrong-lvtt-range")
            else "raw local retained; original and recovered Java 8 verifiers passed"
        )

(output / "results.json").write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n")
print("DT-20 replay and refusal controls complete")
