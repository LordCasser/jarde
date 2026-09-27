#!/usr/bin/env python3
"""Replay the isolated positive and private-field DT-29 Java 8 audits."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
JARDE_BASE = "85117144069a5ab6901792b2c0a9f73951f2f3b8"
JARDE = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-dt29-audit-target/debug/jarde-cli"))
OUT = HERE / "outputs"
EXPECTED_INTERFACE = "runnable:ClassCastException\n"
EXPECTED_FIELDS = "true:false\n"


def run(*args: object, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], cwd=cwd, text=True, capture_output=True)


def checked(*args: object, cwd: Path | None = None) -> str:
    result = run(*args, cwd=cwd)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-3000:]}")
    return result.stdout


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalized(text: str, work: Path) -> str:
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def normalize_report_timing(value: object) -> None:
    if isinstance(value, dict):
        usage = value.get("usage")
        if isinstance(usage, dict) and "elapsed_millis" in usage:
            usage["elapsed_millis"] = "<elapsed>"
        for child in value.values():
            normalize_report_timing(child)
    elif isinstance(value, list):
        for child in value:
            normalize_report_timing(child)


def compile_run(sources: list[Path], runner: Path, main_class: str,
                work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                   "-g:none", "-Xlint:-options", "-d", classes, *sources, runner)
    result: dict[str, object] = {
        "compile_exit": compiled.returncode,
        "compile_stderr": normalized(compiled.stderr, work),
    }
    if compiled.returncode:
        return result
    execution = run("java", "-Xverify:all", "-cp", classes, main_class)
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout,
                  run_stderr=normalized(execution.stderr, work))
    return result


def clean_outputs() -> None:
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)


def private_field_negative_cases(work: Path) -> dict[str, str]:
    """Change one selected definition at a time; no rejected write may become source."""
    classes = work / "private-field-original-classes" / "dt29"
    originals = {path.name: path.read_bytes()
                 for path in classes.glob("PrivateFieldFamily*.class")}

    def jar(name: str, entries: dict[str, bytes]) -> Path:
        path = work / f"{name}.jar"
        with ZipFile(path, "w", ZIP_DEFLATED) as archive:
            for filename, data in sorted(entries.items()):
                archive.writestr(f"dt29/{filename}", data)
        return path

    def synthetic_accessors(data: bytes) -> bytes:
        """Set only explicit variant access$002 methods' synthetic flags in class bytes."""
        result = bytearray(data)
        def u2(at: int) -> int:
            return int.from_bytes(result[at:at + 2], "big")
        def u4(at: int) -> int:
            return int.from_bytes(result[at:at + 4], "big")
        require(result[:4] == b"\xca\xfe\xba\xbe", "variant is not a class file")
        pool: dict[int, bytes] = {}
        count = u2(8)
        at = 10
        index = 1
        while index < count:
            tag = result[at]
            at += 1
            if tag == 1:
                length = u2(at)
                pool[index] = bytes(result[at + 2:at + 2 + length])
                at += 2 + length
            elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
                at += 4
            elif tag in (5, 6):
                at += 8
                index += 1
            elif tag in (7, 8, 16, 19, 20):
                at += 2
            elif tag == 15:
                at += 3
            else:
                raise RuntimeError(f"unexpected class pool tag {tag}")
            index += 1
        at += 6  # class flags, this_class, super_class
        interfaces = u2(at)
        at += 2 + 2 * interfaces
        def skip_member(position: int) -> int:
            attributes = u2(position + 6)
            position += 8
            for _ in range(attributes):
                position += 6 + u4(position + 2)
            return position
        fields = u2(at)
        at += 2
        for _ in range(fields):
            at = skip_member(at)
        methods = u2(at)
        at += 2
        marked = 0
        for _ in range(methods):
            if pool.get(u2(at + 2)) == b"access$002":
                result[at:at + 2] = (u2(at) | 0x1000).to_bytes(2, "big")
                marked += 1
            at = skip_member(at)
        require(marked > 0, "variant has no access$002 method")
        return bytes(result)

    def variant_a(name: str, methods: str, extra_field: str = "") -> bytes:
        source_dir = work / name
        source_dir.mkdir()
        source = source_dir / "PrivateFieldFamily.java"
        source.write_text("package dt29; public class PrivateFieldFamily { "
                          "public static class A { public boolean visible; "
                          f"private boolean hidden; {extra_field} {methods} }} "
                          "public static class B extends A {} }\n")
        classes_dir = work / f"{name}-classes"
        classes_dir.mkdir()
        checked("javac", "--release", "8", "-Xlint:-options", "-d", classes_dir, source)
        return synthetic_accessors(
            (classes_dir / "dt29" / "PrivateFieldFamily$A.class").read_bytes())

    def method_text(path: Path, class_name: str, method_name: bytes,
                    budget: str | None = None) -> tuple[str, dict[str, object]]:
        args: list[object] = [JARDE, "class-source", "--input", path,
                              "--class", class_name, "--policy", "plain-jar",
                              "--release", "8", "--format", "json"]
        if budget:
            args.extend(("--budget", budget))
        result = run(*args)
        require(result.returncode == (4 if budget else 0),
                f"negative replay exited {result.returncode}: {result.stderr[-1000:]}")
        report = json.loads(result.stdout)
        for method in report["methods"]:
            if method["item"]["identity"]["name"] == list(method_name):
                return method["outcome"]["report"].get("text", ""), method
        raise RuntimeError(f"{class_name}.{method_name!r} absent from negative replay")

    original_jar = jar("private-field-negative-base", originals)
    wrong_owner = dict(originals)
    b_name = "PrivateFieldFamily$B.class"
    old = b"dt29/PrivateFieldFamily$A"
    require(wrong_owner[b_name].count(old) == 2,
            "unexpected B constant-pool owner shape before relation mutation")
    wrong_owner[b_name] = wrong_owner[b_name].replace(old, b"dt29/PrivateFieldFamily$X")
    owner_text, _ = method_text(jar("private-field-wrong-owner", wrong_owner),
                                "dt29/PrivateFieldFamily$B", b"set")
    require("@bytecode 2" in owner_text and "@bytecode 7" in owner_text
            and ".visible =" not in owner_text and "access$002(" not in owner_text,
            "missing parent relation must refuse both physical operations")

    variant_source = work / "descriptor-variant" / "PrivateFieldFamily.java"
    variant_source.parent.mkdir()
    variant_source.write_text("package dt29; public class PrivateFieldFamily { "
                              "public static class A { public int visible; "
                              "private boolean hidden; } }\n")
    variant_classes = work / "descriptor-variant-classes"
    variant_classes.mkdir()
    checked("javac", "--release", "8", "-Xlint:-options", "-d",
            variant_classes, variant_source)
    wrong_descriptor = dict(originals)
    wrong_descriptor["PrivateFieldFamily$A.class"] = (
        variant_classes / "dt29" / "PrivateFieldFamily$A.class").read_bytes()
    descriptor_text, _ = method_text(jar("private-field-wrong-descriptor", wrong_descriptor),
                                     "dt29/PrivateFieldFamily$B", b"set")
    require("@bytecode 2" in descriptor_text and ".visible =" not in descriptor_text,
            "selected parent field descriptor mismatch must refuse the store")

    ambiguous = dict(originals)
    ambiguous["PrivateFieldFamily$A.class"] = variant_a(
        "ambiguous-accessor",
        "static boolean access$002(A a, boolean v) { return a.hidden = v; } "
        "static boolean access$002(B b, boolean v) { return ((A) b).hidden = v; }")
    ambiguous_text, _ = method_text(jar("private-field-ambiguous-accessor", ambiguous),
                                    "dt29/PrivateFieldFamily$B", b"set")
    require("@bytecode 7" in ambiguous_text and "access$002(" not in ambiguous_text,
            "multiple selected accessor overloads must refuse the call cast")

    extra_effect = dict(originals)
    extra_effect["PrivateFieldFamily$A.class"] = variant_a(
        "extra-effect-accessor",
        "static boolean access$002(A a, boolean v) { a.other = v; return a.hidden = v; }",
        "private boolean other;")
    extra_text, _ = method_text(jar("private-field-extra-effect", extra_effect),
                                "dt29/PrivateFieldFamily$A", b"access$002")
    require("@bytecode" in extra_text and "arg0.hidden = arg1;" not in extra_text,
            "an accessor with a second field write must refuse the closed helper proof")

    budget_text, budget_method = method_text(original_jar,
        "dt29/PrivateFieldFamily$A", b"access$002", "method_bodies=1")
    require(not budget_text and "stopped" in budget_method["outcome"]["report"]["outcome"],
            "a stopped helper may not publish a partial body")
    return {"wrong_owner": "refused at BCI 2 and 7",
            "wrong_descriptor": "refused at BCI 2",
            "ambiguous_accessor": "call cast refused at BCI 7",
            "extra_effect_accessor": "helper body refused",
            "method_budget": "helper stopped without source"}


def export_fixture(label: str, source_dir: Path, source_names: tuple[str, ...],
                   runner_name: str, main_class: str, expected: str,
                   jadx_assertions: tuple[str, ...], jarde_assertions: tuple[str, ...],
                   work: Path) -> dict[str, object]:
    destination = OUT / label
    for child in ("original", "jadx", "jarde", "reports"):
        (destination / child).mkdir(parents=True, exist_ok=True)
    inputs = [source_dir / name for name in source_names]
    runner = source_dir / runner_name
    original = compile_run(inputs, runner, main_class, work, f"{label}-original")
    require(original.get("compile_exit") == 0 and original.get("run_exit") == 0
            and original.get("run_stdout") == expected and original.get("run_stderr") == "",
            f"{label}: original Java 8 verification failed: {original}")
    for path in inputs + [runner]:
        shutil.copyfile(path, destination / "original" / path.name)

    class_root = work / f"{label}-original-classes"
    entries = sorted(path for path in (class_root / "dt29").rglob("*.class")
                     if path.name != runner_name.replace(".java", ".class"))
    class_names = [path.relative_to(class_root).with_suffix("").as_posix() for path in entries]
    jar_path = work / f"{label}-input.jar"
    with ZipFile(jar_path, "w", ZIP_DEFLATED) as jar:
        for path, name in zip(entries, class_names, strict=True):
            entry = ZipInfo(f"{name}.class", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, path.read_bytes())
    class_hashes = {name: sha(path) for name, path in zip(class_names, entries, strict=True)}

    jadx_dir = work / f"{label}-jadx"
    checked(JADX, "-d", jadx_dir, jar_path)
    jadx_paths = sorted(jadx_dir.rglob("*.java"))
    jadx_text = "\n".join(path.read_text() for path in jadx_paths)
    for assertion in jadx_assertions:
        require(assertion in jadx_text, f"{label}: JADX source assertion missing: {assertion}")
    for path in jadx_paths:
        shutil.copyfile(path, destination / "jadx" / path.name)
    jadx = compile_run(jadx_paths, runner, main_class, work, f"{label}-jadx-rebuild")
    require(jadx.get("compile_exit") == 0 and jadx.get("run_exit") == 0
            and jadx.get("run_stdout") == expected and jadx.get("run_stderr") == "",
            f"{label}: JADX rebuild behavior changed: {jadx}")

    compile_dir = work / f"{label}-jarde-src" / "dt29"
    compile_dir.mkdir(parents=True)
    jarde_paths: list[Path] = []
    jarde_texts: dict[str, str] = {}
    reports: dict[str, object] = {}
    for name in class_names:
        source = checked(JARDE, "class-source", "--input", jar_path, "--class", name,
                         "--policy", "plain-jar", "--release", "8", "--format", "text")
        file_name = name.rsplit("/", 1)[-1] + ".java"
        (destination / "jarde" / file_name).write_text(source)
        compile_path = compile_dir / file_name
        compile_path.write_text(source)
        jarde_paths.append(compile_path)
        jarde_texts[name] = source
        report = json.loads(checked(JARDE, "class-source", "--input", jar_path,
                                    "--class", name, "--policy", "plain-jar",
                                    "--release", "8", "--format", "json"))
        normalize_report_timing(report)
        reports[name] = report
        report_name = name.rsplit("/", 1)[-1] + ".json"
        (destination / "reports" / report_name).write_text(
            json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    complete_jarde = "\n".join(jarde_texts.values())
    for assertion in jarde_assertions:
        require(assertion in complete_jarde, f"{label}: Jarde evidence missing: {assertion}")
    if label == "private-field":
        for class_name, method_name, bci, fragment in (
            ("dt29/PrivateFieldFamily$B", b"set", 2, ").visible = arg1;"),
            ("dt29/PrivateFieldFamily$B", b"set", 7, "access$002((dt29.PrivateFieldFamily$A) this, arg2)"),
            ("dt29/PrivateFieldFamily$A", b"access$002", 3, "arg0.hidden = arg1;"),
            ("dt29/PrivateFieldFamily$A", b"access$002", 6, "return arg1;"),
        ):
            method = next(method for method in reports[class_name]["methods"]
                          if method["item"]["identity"]["name"] == list(method_name))
            body = method["outcome"]["report"]
            require(body["quality"] == "structured", f"{class_name}.{method_name!r} fell back")
            require(any(segment["origin"]["primary"]["bci"] == bci
                        and fragment in body["text"][segment["start"]:segment["end"]]
                        for segment in body["source_map"]["segments"]),
                    f"{class_name}.{method_name!r} lost physical BCI {bci}")
    jarde = compile_run(jarde_paths, runner, main_class, work, f"{label}-jarde-rebuild")

    results = {
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "expected_stdout": expected,
        "input_classes": class_names,
        "input_class_sha256": class_hashes,
        "jarde_report_classes": sorted(reports),
    }
    (destination / "results.json").write_text(
        json.dumps(results, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    (destination / "input.jar.sha256").write_text(f"{sha(jar_path)}  input.jar\n")
    return results


def main() -> None:
    require(checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip() == JADX_HEAD,
            "pinned JADX checkout changed")
    require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
            "pinned JADX checkout is not clean")
    require(JADX.is_file(), f"pinned JADX executable is missing: {JADX}")
    require(JARDE.is_file(), f"Jarde CLI is missing: {JARDE}")
    clean_outputs()
    with tempfile.TemporaryDirectory(prefix="jarde-dt29-") as temporary:
        work = Path(temporary)
        interfaces = export_fixture(
            "interfaces", HERE / "fixtures/interfaces",
            ("Both.java", "CloseOnly.java", "InterfaceCast.java"),
            "InterfaceRunner.java", "dt29.InterfaceRunner", EXPECTED_INTERFACE,
            ("return (Runnable) closeable;", "choose((Runnable) closeable)"),
            ("return (java.lang.Runnable) arg0;",
             "choose((java.lang.Runnable) arg0)"), work)
        fields = export_fixture(
            "private-field", HERE / "fixtures/private-field",
            ("PrivateFieldFamily.java",), "PrivateFieldRunner.java",
            "dt29.PrivateFieldRunner", EXPECTED_FIELDS,
            ("((A) this).hidden",),
            (".visible =", "access$002("), work)
        require(fields["jarde"].get("compile_exit") == 0
                and fields["jarde"].get("run_exit") == 0
                and fields["jarde"].get("run_stdout") == EXPECTED_FIELDS,
                f"private field Jarde closure must pass: {fields['jarde']}")
        negative_cases = private_field_negative_cases(work)
        require(interfaces["jarde"].get("compile_exit") == 0
                and interfaces["jarde"].get("run_exit") == 0,
                f"isolated interface Jarde closure must pass: {interfaces['jarde']}")
        results = {
            "fixed_jadx_head": JADX_HEAD,
            "jarde_source_baseline": JARDE_BASE,
            "interfaces": interfaces,
            "private_field": fields,
            "private_field_remaining_failures": [],
            "private_field_negative_cases": negative_cases,
            "combined_testfieldcast_scope": "retained as combined pending evidence under combined/",
        }
        (OUT / "results.json").write_text(
            json.dumps(results, indent=2, ensure_ascii=False, sort_keys=True) + "\n")

    entries = sorted(path for path in HERE.rglob("*") if path.is_file()
                     and path != OUT / "SHA256SUMS")
    manifest = "".join(f"{sha(path)}  {path.relative_to(HERE)}\n" for path in entries)
    (OUT / "SHA256SUMS").write_text(manifest)
    print(json.dumps(results, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
