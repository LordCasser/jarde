#!/usr/bin/env python3
"""Rebuild verifier-valid generic member-class metadata controls."""
from __future__ import annotations

import hashlib
from pathlib import Path
import struct
import subprocess
from tempfile import TemporaryDirectory
import zipfile

HERE = Path(__file__).resolve().parent
BASE = HERE.parent
FIXTURE = BASE / "fixture"
OUTER_SOURCE = FIXTURE / "Outer.java"
CALLER_SOURCE = FIXTURE / "UseObject.java"
EXPECTED = "minimal.Outer$Inner:1\nnull:1\n"
FIXED_ZIP_DATE = (2000, 1, 1, 0, 0, 0)


def run(*args: object, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run([str(arg) for arg in args], cwd=HERE,
                            text=True, capture_output=True)
    if check and result.returncode:
        raise RuntimeError(f"{args}: exit {result.returncode}\n"
                           f"stdout={result.stdout}\nstderr={result.stderr}")
    return result


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(name: str, content: str) -> None:
    (HERE / name).write_text(content)


def u2(data: bytes, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def p2(value: int) -> bytes:
    return struct.pack(">H", value)


def constant_pool(data: bytes) -> tuple[int, list[tuple[int, bytes, str | None]]]:
    count = u2(data, 8)
    pos = 10
    entries: list[tuple[int, bytes, str | None]] = []
    index = 1
    while index < count:
        start = pos
        tag = data[pos]
        pos += 1
        value: str | None = None
        if tag == 1:
            size = u2(data, pos)
            pos += 2
            value = data[pos:pos + size].decode("utf-8")
            pos += size
        elif tag in (3, 4):
            pos += 4
        elif tag in (5, 6):
            pos += 8
        elif tag in (7, 8, 16, 19, 20):
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag == 15:
            pos += 3
        else:
            raise ValueError(f"unsupported constant pool tag {tag}")
        entries.append((tag, data[start:pos], value))
        if tag in (5, 6):
            entries.append((0, b"", None))
            index += 1
        index += 1
    return pos, entries


def replace_utf8(data: bytes, old: str, new: str, *, once: bool = True) -> bytes:
    cp_end, entries = constant_pool(data)
    found = 0
    rebuilt = bytearray(data[:10])
    for tag, raw, value in entries:
        if tag == 0:
            continue
        if tag == 1 and value == old:
            payload = new.encode("utf-8")
            raw = bytes((1,)) + p2(len(payload)) + payload
            found += 1
        rebuilt.extend(raw)
    assert found == (1 if once else found) and found > 0, (old, found)
    rebuilt.extend(data[cp_end:])
    return bytes(rebuilt)


def class_name_indexes(data: bytes) -> dict[str, int]:
    _, entries = constant_pool(data)
    utf8 = {i: val for i, (tag, _, val) in enumerate(entries, 1) if tag == 1}
    return {utf8[u2(raw, 1)]: index
            for index, (tag, raw, _) in enumerate(entries, 1) if tag == 7}


def skip_attributes(data: bytes, pos: int) -> int:
    count = u2(data, pos)
    pos += 2
    _, entries = constant_pool(data)
    utf8 = {i: val for i, (tag, _, val) in enumerate(entries, 1) if tag == 1}
    for _ in range(count):
        pos += 2
        length = struct.unpack_from(">I", data, pos)[0]
        pos += 4 + length
    return pos


def replace_inner_outer(data: bytes, new_outer_index: int) -> bytes:
    cp_end, entries = constant_pool(data)
    utf8 = {i: val for i, (tag, _, val) in enumerate(entries, 1) if tag == 1}
    pos = cp_end + 6  # access_flags, this_class, super_class
    interfaces = u2(data, pos)
    pos += 2 + 2 * interfaces
    for _table in ("fields", "methods"):
        count = u2(data, pos)
        pos += 2
        for _ in range(count):
            pos += 6  # access, name, descriptor
            pos = skip_attributes(data, pos)
    class_attrs = u2(data, pos)
    pos += 2
    result = bytearray(data)
    found = False
    for _ in range(class_attrs):
        name_index = u2(data, pos)
        length = struct.unpack_from(">I", data, pos + 2)[0]
        start = pos + 6
        end = start + length
        if utf8[name_index] == "InnerClasses":
            n = u2(data, start)
            entry = start + 2
            for _ in range(n):
                inner_index = u2(data, entry)
                inner_name_index = u2(data, entry + 4)
                if inner_name_index and utf8.get(inner_name_index) == "Inner":
                    # Point the InnerClasses row at java/lang/Object as its owner.
                    result[entry + 2:entry + 4] = p2(new_outer_index)
                    found = True
                entry += 8
        pos = end
    assert found, "InnerClasses row for simple name Inner not found"
    return bytes(result)


def read_jar(path: Path) -> dict[str, bytes]:
    with zipfile.ZipFile(path) as jar:
        return {name: jar.read(name) for name in jar.namelist() if not name.endswith("/")}


def write_jar(path: Path, entries: dict[str, bytes]) -> None:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as jar:
        for name, payload in sorted(entries.items()):
            info = zipfile.ZipInfo(name, FIXED_ZIP_DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            jar.writestr(info, payload)


def compile_sources(out: Path, *sources: Path, classpath: Path | None = None,
                    debug: str = "-g:none") -> subprocess.CompletedProcess[str]:
    out.mkdir()
    args: list[object] = ["javac", "--release", "8", "-Xlint:-options", debug]
    if classpath is not None:
        args += ["-classpath", classpath]
    args += ["-d", out, *sources]
    return run(*args)


def javap(label: str, jar: Path) -> None:
    result = run("javap", "-classpath", jar, "-p", "-s", "-v", "minimal.Outer$Inner")
    write(f"javap-{label}.txt", result.stdout)


def execute(label: str, caller_jar: Path, target_jar: Path, expected: str = EXPECTED) -> None:
    result = run("java", "-Xverify:all", "-cp", f"{caller_jar}:{target_jar}",
                 "minimal.UseObject", check=False)
    write(f"run-{label}.txt",
          f"exit={result.returncode}\nstdout:\n{result.stdout}stderr:\n{result.stderr}")
    assert result.returncode == 0, (label, result.stdout, result.stderr)
    assert result.stdout == expected, (label, result.stdout)


with TemporaryDirectory(prefix="jarde-inner-invalid-controls-") as temporary:
    work = Path(temporary)
    original_classes = work / "original"
    compile_sources(original_classes, OUTER_SOURCE, CALLER_SOURCE)
    original_entries = {
        "minimal/Outer.class": (original_classes / "minimal/Outer.class").read_bytes(),
        "minimal/Outer$Inner.class": (original_classes / "minimal/Outer$Inner.class").read_bytes(),
    }
    original_jar = HERE / "original-target.jar"
    write_jar(original_jar, original_entries)

    caller_jar = HERE / "caller-use-object.jar"
    with zipfile.ZipFile(caller_jar, "w", compression=zipfile.ZIP_DEFLATED) as jar:
        info = zipfile.ZipInfo("minimal/UseObject.class", FIXED_ZIP_DATE)
        info.compress_type = zipfile.ZIP_DEFLATED
        jar.writestr(info, (original_classes / "minimal/UseObject.class").read_bytes())

    original_run = run("java", "-Xverify:all", "-cp", f"{caller_jar}:{original_jar}",
                       "minimal.UseObject")
    assert original_run.returncode == 0 and original_run.stdout == EXPECTED
    write("original-run.txt", original_run.stdout)
    javap("original", original_jar)

    inner_path = "minimal/Outer$Inner.class"
    original_inner = original_entries[inner_path]
    class_sig_variant = replace_utf8(
        original_inner,
        "<V:Ljava/lang/Object;>Ljava/lang/Object;",
        "<W:Ljava/lang/Object;>Ljava/lang/Object;",
    )
    ctor_sig_variant = replace_utf8(original_inner, "(TV;)V", "(Ljava/lang/Long;)V")
    cp_names = class_name_indexes(original_inner)
    object_index = cp_names["java/lang/Object"]
    inner_relation_variant = replace_inner_outer(original_inner, object_index)

    variants = {
        "wrong-class-type-variable": class_sig_variant,
        "wrong-constructor-signature-erasure": ctor_sig_variant,
        "wrong-innerclasses-owner": inner_relation_variant,
    }
    mutated_jars: dict[str, Path] = {}
    for label, mutated_inner in variants.items():
        assert mutated_inner != original_inner
        entries = dict(original_entries)
        entries[inner_path] = mutated_inner
        jar_path = HERE / f"{label}.jar"
        write_jar(jar_path, entries)
        mutated_jars[label] = jar_path
        javap(label, jar_path)
        execute(label, caller_jar, jar_path)

    # Missing-target control retains Outer.class but omits the referenced member class.
    no_target_jar = HERE / "missing-inner-target.jar"
    write_jar(no_target_jar, {"minimal/Outer.class": original_entries["minimal/Outer.class"]})
    missing = run("java", "-Xverify:all", "-cp", f"{caller_jar}:{no_target_jar}",
                  "minimal.UseObject", check=False)
    write("run-missing-inner-target.txt",
          f"exit={missing.returncode}\nstdout:\n{missing.stdout}stderr:\n{missing.stderr}")
    assert missing.returncode != 0 and "NoClassDefFoundError" in missing.stderr

    # Additional source-compiled overload: keep the generic Object-erasure constructor
    # and add a String constructor, then compile the original caller against that jar.
    overloaded_source = work / "overload-src/minimal/Outer.java"
    overloaded_source.parent.mkdir(parents=True)
    overloaded_source.write_text('''package minimal;\n\npublic class Outer {\n    public class Inner<V> {\n        private final V value;\n        public Inner(V value) { this.value = value; }\n        public Inner(String ignored) { this.value = null; }\n        public V value() { return value; }\n    }\n}\n''')
    overloaded_classes = work / "overload-classes"
    compile_sources(overloaded_classes, overloaded_source, CALLER_SOURCE)
    overload_jar = HERE / "second-constructor-overload.jar"
    write_jar(overload_jar, {
        "minimal/Outer.class": (overloaded_classes / "minimal/Outer.class").read_bytes(),
        "minimal/Outer$Inner.class": (overloaded_classes / "minimal/Outer$Inner.class").read_bytes(),
    })
    overload_caller = work / "overload-caller"
    overload_compile = compile_sources(overload_caller, CALLER_SOURCE,
                                       classpath=overload_jar)
    write("second-overload-caller-javac.txt",
          f"exit={overload_compile.returncode}\nstdout:{overload_compile.stdout}"
          f"stderr:{overload_compile.stderr}")
    overload_caller_jar = HERE / "second-overload-caller.jar"
    with zipfile.ZipFile(overload_caller_jar, "w", compression=zipfile.ZIP_DEFLATED) as jar:
        info = zipfile.ZipInfo("minimal/UseObject.class", FIXED_ZIP_DATE)
        info.compress_type = zipfile.ZIP_DEFLATED
        jar.writestr(info, (overload_caller / "minimal/UseObject.class").read_bytes())
    javap("second-constructor-overload", overload_jar)
    overload_caller_javap = run("javap", "-classpath",
                                f"{overload_caller_jar}:{overload_jar}",
                                "-p", "-s", "-c", "minimal.UseObject")
    write("javap-second-overload-caller.txt", overload_caller_javap.stdout)
    execute("second-constructor-overload", overload_caller_jar, overload_jar)

    # Save raw mutated classfiles alongside the jars for direct inspection.
    for label, payload in variants.items():
        (HERE / f"{label}-Outer$Inner.class").write_bytes(payload)

# Hash every replay input and frozen binary in a stable, path-sorted manifest.
hash_paths = [*FIXTURE.glob("*.java"), *HERE.glob("*.jar"),
              *HERE.glob("*.class"), *HERE.glob("javap-*.txt"),
              *HERE.glob("run-*.txt"), *HERE.glob("*-javac.txt"),
              HERE / "original-run.txt"]
lines = [f"{path.relative_to(BASE)} {digest(path)}"
         for path in sorted(set(hash_paths))]
write("sha256.txt", "\n".join(lines) + "\n")
versions = run("javac", "-version")
write("tool-versions.txt",
      f"{(versions.stderr or versions.stdout).strip()}\n"
      f"{run('jar', '--version').stdout.strip()}\n")
print(f"Invalid-control evidence refreshed in {HERE}")
