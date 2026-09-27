#!/usr/bin/env python3
"""Replay the fixed DT-17 wildcard parameter forms against Java 8 output."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import textwrap
import tempfile
import zipfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_REPO = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_REPO / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
NAMES = ("any", "ext", "sup", "extArray", "supArray", "raw")
EXPECTED = (
    "any=java.util.List<?>\n"
    "ext=java.util.List<? extends java.lang.Number>\n"
    "sup=java.util.List<? super java.lang.String>\n"
    "extArray=java.util.List<? extends byte[]>\n"
    "supArray=java.util.List<? super int[]>\n"
    "raw=java.util.List\n"
)
BASELINE_JARDE = "".join(f"{name}=java.util.List\n" for name in NAMES)


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode:
        raise SystemExit(f"{label}: exit {result.returncode}: {result.stderr[-3000:]}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_utf8_constant(class_bytes: bytes, old: bytes, new: bytes) -> bytes:
    """Change one constant-pool UTF8 value while preserving the rest of a classfile."""
    if class_bytes[:4] != b"\xca\xfe\xba\xbe":
        raise SystemExit("signature mutation input is not a classfile")
    count = int.from_bytes(class_bytes[8:10], "big")
    cursor = 10
    output = bytearray(class_bytes[:10])
    replaced = False
    index = 1
    while index < count:
        start = cursor
        tag = class_bytes[cursor]
        cursor += 1
        if tag == 1:
            length = int.from_bytes(class_bytes[cursor:cursor + 2], "big")
            cursor += 2
            value = class_bytes[cursor:cursor + length]
            cursor += length
            if value == old and not replaced:
                value = new
                replaced = True
            output.append(tag)
            output.extend(len(value).to_bytes(2, "big"))
            output.extend(value)
        else:
            width = {
                3: 4, 4: 4, 5: 8, 6: 8, 7: 2, 8: 2, 9: 4, 10: 4, 11: 4,
                12: 4, 15: 3, 16: 2, 17: 4, 18: 4, 19: 2, 20: 2,
            }.get(tag)
            if width is None:
                raise SystemExit(f"signature mutation found unsupported constant-pool tag {tag}")
            cursor += width
            output.extend(class_bytes[start:cursor])
            if tag in (5, 6):
                index += 1
        index += 1
    if not replaced:
        raise SystemExit("expected wildcard method Signature was absent")
    output.extend(class_bytes[cursor:])
    return bytes(output)


def truncate_method_code(class_bytes: bytes, method_name: bytes) -> bytes:
    """Make a single Code body structurally incomplete without changing its method descriptor."""
    data = bytearray(class_bytes)
    count = int.from_bytes(data[8:10], "big")
    cursor = 10
    utf8: dict[int, bytes] = {}
    index = 1
    while index < count:
        tag = data[cursor]
        cursor += 1
        if tag == 1:
            length = int.from_bytes(data[cursor:cursor + 2], "big")
            cursor += 2
            utf8[index] = bytes(data[cursor:cursor + length])
            cursor += length
        else:
            width = {
                3: 4, 4: 4, 5: 8, 6: 8, 7: 2, 8: 2, 9: 4, 10: 4, 11: 4,
                12: 4, 15: 3, 16: 2, 17: 4, 18: 4, 19: 2, 20: 2,
            }.get(tag)
            if width is None:
                raise SystemExit(f"Code mutation found unsupported constant-pool tag {tag}")
            cursor += width
            if tag in (5, 6):
                index += 1
        index += 1

    def u2(at: int) -> int:
        return int.from_bytes(data[at:at + 2], "big")

    def skip_members(at: int, member_count: int, target: bool) -> tuple[int, bool]:
        found = False
        for _ in range(member_count):
            member_name = utf8.get(u2(at + 2))
            descriptor = utf8.get(u2(at + 4))
            attr_count = u2(at + 6)
            attribute = at + 8
            for _ in range(attr_count):
                attribute_name = utf8.get(u2(attribute))
                length = int.from_bytes(data[attribute + 2:attribute + 6], "big")
                info = attribute + 6
                if (target and member_name == method_name and descriptor == b"(Ljava/util/List;)V"
                        and attribute_name == b"Code"):
                    code_length_at = info + 4
                    code_length = int.from_bytes(data[code_length_at:code_length_at + 4], "big")
                    if code_length == 0:
                        raise SystemExit("Code mutation target was already empty")
                    data[code_length_at:code_length_at + 4] = (0).to_bytes(4, "big")
                    found = True
                    break
                attribute = info + length
            if found:
                break
            at = attribute
        return at, found

    cursor += 6
    interface_count = u2(cursor)
    cursor += 2 + interface_count * 2
    field_count = u2(cursor)
    cursor += 2
    cursor, _ = skip_members(cursor, field_count, False)
    method_count = u2(cursor)
    cursor += 2
    _, found = skip_members(cursor, method_count, True)
    if not found:
        raise SystemExit("Code mutation target method was absent")
    return bytes(data)


def compile_sources(classes: Path, *sources: Path) -> None:
    classes.mkdir()
    checked(run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                "-g:none", "-Xlint:-options", "-d", classes, *sources), "javac")


def verified_run(classes: Path, label: str, main: str = "dt17.Runner") -> str:
    result = run("java", "-Xverify:all", "-cp", classes, main)
    checked(result, label)
    return result.stdout


NEGATIVE_SOURCE = textwrap.dedent("""\
    package dt17;

    import java.lang.annotation.ElementType;
    import java.lang.annotation.Retention;
    import java.lang.annotation.RetentionPolicy;
    import java.lang.annotation.Target;
    import java.util.List;

    @Target(ElementType.TYPE_USE)
    @Retention(RetentionPolicy.CLASS)
    @interface Mark {}

    public class NegativeWildcards {
        static class Bound { static class Inner {} }

        public static void used(List<?> list) { list.size(); }
        public static void assigned(List<?> list) { list = null; }
        public static void called(List<?> list) { helper(list); }
        public static void extraInstruction(List<?> list) {
            if (System.nanoTime() == 0) return;
        }
        public static void helper(List<?> list) {}
        public static void handled(List<?> list) {
            try { throw new IllegalStateException(); }
            catch (IllegalStateException ignored) {}
        }
        public static void extra(List<?> list, int ignored) {}
        public static void nested(List<? extends Bound.Inner> list) {}
        public static void annotated(List<@Mark ? extends Number> list) {}
        public static void same(List<?> list) {}
        public static void trigger(List<?> list) { same(list); }
        public static void raw(List list) {}
    }
""")

NEGATIVE_NAMES = (
    "used", "assigned", "called", "extraInstruction", "helper", "handled", "extra", "nested",
    "annotated", "same", "trigger", "raw",
)
NEGATIVE_ORIGINAL = (
    "used=java.util.List<?>\n"
    "assigned=java.util.List<?>\n"
    "called=java.util.List<?>\n"
    "extraInstruction=java.util.List<?>\n"
    "helper=java.util.List<?>\n"
    "handled=java.util.List<?>\n"
    "extra=java.util.List<?>\n"
    "nested=java.util.List<? extends dt17.NegativeWildcards$Bound$Inner>\n"
    "annotated=java.util.List<? extends java.lang.Number>\n"
    "same=java.util.List<?>\n"
    "trigger=java.util.List<?>\n"
    "raw=java.util.List\n"
)
NEGATIVE_JARDE = "".join(f"{name}=java.util.List\n" for name in NEGATIVE_NAMES)

NEGATIVE_RUNNER = textwrap.dedent("""\
    package dt17;

    import java.lang.reflect.Method;
    import java.util.List;

    public class NegativeRunner {
        public static void main(String[] args) throws Exception {
            for (String name : new String[] {"used", "assigned", "called", "extraInstruction",
                    "helper", "handled", "extra", "nested", "annotated", "same", "trigger",
                    "raw"}) {
                Method method = NegativeWildcards.class.getMethod(name,
                        name.equals("extra") ? new Class<?>[] {List.class, int.class}
                                : new Class<?>[] {List.class});
                System.out.println(name + "="
                        + method.getGenericParameterTypes()[0].getTypeName());
            }
        }
    }
""")


if len(sys.argv) != 3 or sys.argv[1] not in ("baseline", "fixed"):
    raise SystemExit("usage: replay.py baseline|fixed /absolute/path/to/jarde-cli")
mode, cli = sys.argv[1:]
if not Path(cli).is_file() or not JADX.is_file():
    raise SystemExit("Jarde CLI or pinned JADX executable is absent")
head = run("git", "-C", JADX_REPO, "rev-parse", "HEAD")
status = run("git", "-C", JADX_REPO, "status", "--porcelain")
if head.returncode or head.stdout.strip() != JADX_HEAD or status.returncode or status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen commit")
output = HERE / ("outputs" if mode == "baseline" else "fixed")
output.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt17-audit-") as temporary:
    work = Path(temporary)
    original = work / "original"
    compile_sources(original, HERE / "Wildcards.java", HERE / "Runner.java")
    original_run = verified_run(original, "original")
    if original_run != EXPECTED:
        raise SystemExit("original wildcard runtime differs from frozen result")
    javap = run("javap", "-c", "-p", "-s", "-classpath", original, "dt17.Wildcards")
    checked(javap, "original javap")
    (output / "original-javap.txt").write_text(javap.stdout)
    jar = work / "input.jar"
    with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.write(original / "dt17/Wildcards.class", "dt17/Wildcards.class")

    jadx_root = work / "jadx"
    checked(run(JADX, "-d", jadx_root, jar), "JADX")
    jadx_source = jadx_root / "sources/dt17/Wildcards.java"
    if not jadx_source.is_file():
        raise SystemExit("JADX did not produce the complete type")
    (output / "jadx-Wildcards.java.txt").write_text(jadx_source.read_text())
    jadx_classes = work / "jadx-classes"
    compile_sources(jadx_classes, jadx_source, HERE / "Runner.java")
    jadx_run = verified_run(jadx_classes, "JADX")
    if jadx_run != EXPECTED:
        raise SystemExit("JADX wildcard runtime differs from original")

    jarde = run(cli, "class-source", "--input", jar, "--class", "dt17/Wildcards",
                "--policy", "plain-jar", "--release", "8", "--format", "text")
    checked(jarde, "Jarde class-source")
    jarde_source = work / "jarde/dt17/Wildcards.java"
    jarde_source.parent.mkdir(parents=True)
    jarde_source.write_text(jarde.stdout)
    (output / "jarde-Wildcards.java.txt").write_text(jarde.stdout)
    jarde_classes = work / "jarde-classes"
    compile_sources(jarde_classes, jarde_source, HERE / "Runner.java")
    jarde_run = verified_run(jarde_classes, "Jarde")
    (output / "jarde-run.log").write_text(jarde_run)
    if mode == "baseline":
        if (jarde_run != BASELINE_JARDE
                or jarde.stdout.count("generic Signature projection refused") != 5):
            raise SystemExit("baseline Jarde wildcard refusal or erasure output changed")
    elif jarde_run != EXPECTED:
        raise SystemExit("fixed Jarde wildcard runtime differs from original")

    negative_source = work / "NegativeWildcards.java"
    negative_source.write_text(NEGATIVE_SOURCE)
    negative_runner = work / "NegativeRunner.java"
    negative_runner.write_text(NEGATIVE_RUNNER)
    negative_original = work / "negative-original"
    compile_sources(negative_original, negative_source, negative_runner)
    negative_original_run = verified_run(
        negative_original, "negative original", "dt17.NegativeRunner"
    )
    if negative_original_run != NEGATIVE_ORIGINAL:
        raise SystemExit(
            "verifier-valid negative fixture differs from its reflection controls:\n"
            + negative_original_run
        )
    negative_jar = work / "negative-input.jar"
    with zipfile.ZipFile(negative_jar, "w", zipfile.ZIP_DEFLATED) as archive:
        for class_file in sorted((negative_original / "dt17").glob("*.class")):
            archive.write(class_file, class_file.relative_to(negative_original).as_posix())
    negative_jarde = run(cli, "class-source", "--input", negative_jar,
                         "--class", "dt17/NegativeWildcards", "--policy", "plain-jar",
                         "--release", "8", "--format", "text")
    checked(negative_jarde, "negative Jarde class-source")
    negative_jarde_source = work / "jarde-negative/dt17/NegativeWildcards.java"
    negative_jarde_source.parent.mkdir(parents=True)
    negative_jarde_source.write_text(negative_jarde.stdout)
    (output / "jarde-NegativeWildcards.java.txt").write_text(negative_jarde.stdout)
    negative_jarde_classes = work / "negative-jarde-classes"
    compile_sources(negative_jarde_classes, negative_jarde_source, negative_runner)
    negative_jarde_run = verified_run(
        negative_jarde_classes, "negative Jarde", "dt17.NegativeRunner"
    )
    if negative_jarde_run != NEGATIVE_JARDE:
        raise SystemExit(
            "Jarde projected a verifier-valid negative wildcard parameter:\n"
            + negative_jarde_run
        )

    # Signature is metadata rather than verifier input. Alter only `any`'s generic type so the
    # class remains loadable while its Signature erasure conflicts with the physical List slot.
    wrong_erasure_class = replace_utf8_constant(
        (original / "dt17/Wildcards.class").read_bytes(),
        b"(Ljava/util/List<*>;)V",
        b"(Ljava/util/Other<*>;)V",
    )
    wrong_erasure_jar = work / "wrong-erasure.jar"
    with zipfile.ZipFile(wrong_erasure_jar, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("dt17/Wildcards.class", wrong_erasure_class)
    wrong_erasure_jarde = run(
        cli, "class-source", "--input", wrong_erasure_jar, "--class", "dt17/Wildcards",
        "--policy", "plain-jar", "--release", "8", "--format", "text",
    )
    checked(wrong_erasure_jarde, "wrong-erasure Jarde class-source")
    wrong_erasure_source = work / "wrong-erasure/dt17/Wildcards.java"
    wrong_erasure_source.parent.mkdir(parents=True)
    wrong_erasure_source.write_text(wrong_erasure_jarde.stdout)
    (output / "jarde-Wildcards-wrong-erasure.java.txt").write_text(wrong_erasure_jarde.stdout)
    if ("generic Signature projection refused for `any" not in wrong_erasure_jarde.stdout
            or "public static void any(java.util.List arg0)" not in wrong_erasure_jarde.stdout):
        raise SystemExit("Signature erasure mismatch did not keep the physical List declaration")
    wrong_erasure_classes = work / "wrong-erasure-classes"
    compile_sources(wrong_erasure_classes, wrong_erasure_source, HERE / "Runner.java")
    wrong_erasure_run = verified_run(wrong_erasure_classes, "wrong-erasure Jarde")
    wrong_erasure_expected = EXPECTED.replace("any=java.util.List<?>\n", "any=java.util.List\n")
    if wrong_erasure_run != wrong_erasure_expected:
        raise SystemExit("erasure refusal changed an unrelated wildcard method")

    incomplete_class = truncate_method_code((original / "dt17/Wildcards.class").read_bytes(), b"any")
    incomplete_jar = work / "incomplete-code.jar"
    with zipfile.ZipFile(incomplete_jar, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("dt17/Wildcards.class", incomplete_class)
    incomplete = run(
        cli, "class-source", "--input", incomplete_jar, "--class", "dt17/Wildcards",
        "--policy", "plain-jar", "--release", "8", "--format", "text",
    )
    (output / "incomplete-code-Wildcards.java.txt").write_text(incomplete.stdout)
    if (incomplete.returncode not in (0, 4)
            or "public static void any(java.util.List arg0)" not in incomplete.stdout
            or "public static void any(java.util.List<?>" in incomplete.stdout):
        raise SystemExit("incomplete Code did not retain the physical method declaration")

    # A low analysis-step budget can stop between method proof and declaration commit. The class
    # operation may return an incomplete physical report, but it must never publish half a type.
    budgeted = run(cli, "class-source", "--input", jar, "--class", "dt17/Wildcards",
                   "--policy", "plain-jar", "--release", "8", "--format", "text",
                   "--budget", "analysis_steps=100")
    (output / "budgeted-Wildcards.java.txt").write_text(budgeted.stdout)
    if budgeted.returncode != 4:
        raise SystemExit("the low analysis-step budget did not report a stopped class-source run")
    if ("public static void any(java.util.List<?> arg0)" not in budgeted.stdout
            or "public static void ext(java.util.List arg0)" not in budgeted.stdout
            or "public static void ext(java.util.List<?" in budgeted.stdout):
        raise SystemExit("budget stop did not retain complete prior and physical stopped declarations")
    allowed_budgeted_wildcards = {
        "java.util.List<?> arg0",
        "java.util.List<? extends java.lang.Number> arg0",
        "java.util.List<? super java.lang.String> arg0",
        "java.util.List<? extends byte[]> arg0",
        "java.util.List<? super int[]> arg0",
    }
    for line in budgeted.stdout.splitlines():
        match = re.search(r"java\.util\.List<[^)]*?\s+arg0", line)
        if match and match.group(0) not in allowed_budgeted_wildcards:
            raise SystemExit("budget stop exposed a partial wildcard parameter spelling")

    results = {
        "mode": mode,
        "jadx_head": JADX_HEAD,
        "javac": run("javac", "-version").stdout.strip(),
        "source_sha256": {name: sha(HERE / name) for name in ("Wildcards.java", "Runner.java")},
        "original_class_sha256": sha(original / "dt17/Wildcards.class"),
        "original_verified_run": original_run,
        "jadx_verified_run": jadx_run,
        "jarde_verified_run": jarde_run,
        "negative_original_verified_run": negative_original_run,
        "negative_jarde_verified_run": negative_jarde_run,
        "wrong_erasure_jarde_verified_run": wrong_erasure_run,
        "incomplete_code_exit": incomplete.returncode,
        "budget_exit": budgeted.returncode,
        "budget_stdout_sha256": hashlib.sha256(budgeted.stdout.encode()).hexdigest(),
    }
    (output / "results.json").write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n")
    print(f"DT-17 {mode} replay complete")
