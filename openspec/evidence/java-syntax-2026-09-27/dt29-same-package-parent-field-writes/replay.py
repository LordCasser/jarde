#!/usr/bin/env python3
"""Replay the Java 8 same-package parent-field write proof."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JARDE = Path(os.environ.get("JARDE_CLI", ""))
OUT = HERE / "outputs"
EXPECTED = "true:true:false:false\n"


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


def normalize(text: str, work: Path) -> str:
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def compile_run(sources: list[Path], runner: Path, work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                   "-g:none", "-Xlint:-options", "-d", classes, *sources, runner)
    result: dict[str, object] = {
        "compile_exit": compiled.returncode,
        "compile_stderr": normalize(compiled.stderr, work),
    }
    if compiled.returncode:
        return result
    execution = run("java", "-Xverify:all", "-cp", classes, "dt29.SamePackageParentRunner")
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout,
                  run_stderr=normalize(execution.stderr, work))
    return result


def class_names(classes: Path) -> list[str]:
    return sorted(path.relative_to(classes).with_suffix("").as_posix()
                  for path in classes.rglob("*.class")
                  if path.name != "SamePackageParentRunner.class")


def make_jar(classes: Path, work: Path, label: str) -> tuple[Path, list[str]]:
    names = class_names(classes)
    jar_path = work / f"{label}.jar"
    with ZipFile(jar_path, "w", ZIP_DEFLATED) as archive:
        for name in names:
            path = classes / f"{name}.class"
            entry = ZipInfo(f"{name}.class", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            archive.writestr(entry, path.read_bytes())
    return jar_path, names


def classfile_pool(data: bytearray) -> tuple[dict[int, dict[str, object]], int]:
    pool: dict[int, dict[str, object]] = {}
    count = int.from_bytes(data[8:10], "big")
    at = 10
    index = 1
    while index < count:
        start = at
        tag = data[at]
        at += 1
        entry: dict[str, object] = {"tag": tag, "start": start}
        if tag == 1:
            length = int.from_bytes(data[at:at + 2], "big")
            entry["text"] = bytes(data[at + 2:at + 2 + length]).decode("utf-8")
            at += 2 + length
        elif tag in (3, 4):
            at += 4
        elif tag in (5, 6):
            at += 8
            pool[index] = entry
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entry["index"] = int.from_bytes(data[at:at + 2], "big")
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            entry["first"] = int.from_bytes(data[at:at + 2], "big")
            entry["second"] = int.from_bytes(data[at + 2:at + 4], "big")
            at += 4
        elif tag == 15:
            at += 3
        else:
            raise RuntimeError(f"unexpected constant-pool tag {tag}")
        pool[index] = entry
        index += 1
    return pool, at


def cp_text(pool: dict[int, dict[str, object]], index: int) -> str:
    entry = pool[index]
    if entry["tag"] == 7:
        return cp_text(pool, int(entry["index"]))
    return str(entry["text"])


def fieldref_entry(pool: dict[int, dict[str, object]], name: str) -> tuple[int, dict[str, object], dict[str, object]]:
    for index, entry in pool.items():
        if entry["tag"] != 9:
            continue
        owner = pool[int(entry["first"])]
        name_type = pool[int(entry["second"])]
        field_name = cp_text(pool, int(name_type["first"]))
        if cp_text(pool, int(owner["index"])) == "dt29/SamePackageParentFamily$A" \
                and field_name == name:
            return index, entry, name_type
    raise RuntimeError(f"field reference not found: {name}")


def mutate_fieldref(data: bytes, field_name: str, *, name: str | None = None,
                    descriptor: str | None = None) -> bytes:
    result = bytearray(data)
    pool, _ = classfile_pool(result)
    _, entry, name_type = fieldref_entry(pool, field_name)
    if name is not None:
        name_index = next(index for index, item in pool.items()
                          if item["tag"] == 1 and item["text"] == name)
        result[int(name_type["start"]) + 1:int(name_type["start"]) + 3] = name_index.to_bytes(2, "big")
    if descriptor is not None:
        descriptor_index = next(index for index, item in pool.items()
                                if item["tag"] == 1 and item["text"] == descriptor)
        result[int(name_type["start"]) + 3:int(name_type["start"]) + 5] = (
            descriptor_index.to_bytes(2, "big"))
    return bytes(result)


def mutate_field_flags(data: bytes, name: str, flags: int) -> bytes:
    result = bytearray(data)
    pool, at = classfile_pool(result)
    at += 6
    interfaces = int.from_bytes(result[at:at + 2], "big")
    at += 2 + interfaces * 2
    field_count = int.from_bytes(result[at:at + 2], "big")
    at += 2
    for _ in range(field_count):
        access_at = at
        name_index = int.from_bytes(result[at + 2:at + 4], "big")
        attr_count = int.from_bytes(result[at + 6:at + 8], "big")
        at += 8
        for _ in range(attr_count):
            length = int.from_bytes(result[at + 2:at + 6], "big")
            at += 6 + length
        if cp_text(pool, name_index) == name:
            result[access_at:access_at + 2] = flags.to_bytes(2, "big")
            return bytes(result)
    raise RuntimeError(f"field declaration not found: {name}")


def mutate_field_descriptor(data: bytes, name: str, descriptor: str) -> bytes:
    result = bytearray(data)
    pool, at = classfile_pool(result)
    descriptor_index = next(index for index, item in pool.items()
                            if item["tag"] == 1 and item["text"] == descriptor)
    at += 6
    interfaces = int.from_bytes(result[at:at + 2], "big")
    at += 2 + interfaces * 2
    field_count = int.from_bytes(result[at:at + 2], "big")
    at += 2
    for _ in range(field_count):
        member_start = at
        name_index = int.from_bytes(result[at + 2:at + 4], "big")
        attr_count = int.from_bytes(result[at + 6:at + 8], "big")
        at += 8
        for _ in range(attr_count):
            length = int.from_bytes(result[at + 2:at + 6], "big")
            at += 6 + length
        if cp_text(pool, name_index) == name:
            result[member_start + 4:member_start + 6] = descriptor_index.to_bytes(2, "big")
            return bytes(result)
    raise RuntimeError(f"field declaration not found: {name}")


def jar_variant(work: Path, label: str, base: dict[str, bytes],
                overrides: dict[str, bytes]) -> Path:
    jar_path = work / f"{label}.jar"
    with ZipFile(jar_path, "w", ZIP_DEFLATED) as archive:
        for name, data in sorted((base | overrides).items()):
            entry = ZipInfo(name, date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            archive.writestr(entry, data)
    return jar_path


def jarde_method(jar_path: Path, class_name: str, method: str) -> tuple[str, int]:
    result = run(JARDE, "class-source", "--input", jar_path, "--class", class_name,
                 "--policy", "plain-jar", "--release", "8", "--format", "text")
    if result.returncode not in (0, 4):
        raise RuntimeError(f"class-source exited {result.returncode}: {result.stderr[-1000:]}")
    text = result.stdout
    marker = f"@method {method}"
    start = text.find(marker)
    if start < 0:
        raise RuntimeError(f"{class_name}.{method} missing from report")
    next_method = text.find("@method ", start + len(marker))
    return text[start: next_method if next_method >= 0 else len(text)], result.returncode


def boundary_cases(jar_path: Path, names: list[str], work: Path) -> dict[str, str]:
    base = {name + ".class": (work / "original-classes" / (name + ".class")).read_bytes()
            for name in names}
    parent_name = "dt29/SamePackageParentFamily$A.class"
    child_name = "dt29/SamePackageParentFamily$B.class"
    parent = base[parent_name]
    child = base[child_name]
    refused: dict[str, str] = {}

    for label, changed_parent in (
        ("private", mutate_field_flags(parent, "protectedField", 0x0002)),
        ("static", mutate_field_flags(parent, "protectedField", 0x0004 | 0x0008)),
        ("final", mutate_field_flags(parent, "protectedField", 0x0004 | 0x0010)),
        ("volatile", mutate_field_flags(parent, "protectedField", 0x0004 | 0x0040)),
    ):
        method, _ = jarde_method(jar_variant(work, label, base, {parent_name: changed_parent}),
                                 "dt29/SamePackageParentFamily$B", "set(ZZ)V")
        require("@bytecode 7" in method and ").protectedField = arg1;" not in method,
                f"{label} parent field must retain BCI 7 as refused source")
        refused[label] = "refused at BCI 7"

    for label, changed_child in (
        ("wrong_name", mutate_fieldref(child, "protectedField", name="set")),
    ):
        method, _ = jarde_method(jar_variant(work, label, base, {child_name: changed_child}),
                                 "dt29/SamePackageParentFamily$B", "set(ZZ)V")
        require("@bytecode 7" in method and ").protectedField = arg1;" not in method,
                f"{label} parent field must retain BCI 7 as refused source")
        refused[label] = "refused at BCI 7"

    wrong_descriptor = mutate_field_descriptor(parent, "protectedField", "I")
    method, _ = jarde_method(jar_variant(work, "wrong_descriptor", base,
                                         {parent_name: wrong_descriptor}),
                             "dt29/SamePackageParentFamily$B", "set(ZZ)V")
    require("@bytecode 7" in method and ").protectedField = arg1;" not in method,
            "parent field descriptor mismatch must retain BCI 7 as refused source")
    refused["wrong_descriptor"] = "refused at BCI 7"

    ancestor = work / "non-direct-parent"
    ancestor_source = ancestor / "src" / "n" / "Family.java"
    ancestor_source.parent.mkdir(parents=True)
    ancestor_source.write_text("package n; public class Family { "
                               "public static class G { protected boolean value; } "
                               "public static class A extends G { protected boolean value; } "
                               "public static class B extends A { "
                               "public void set(boolean value) { ((G) this).value = value; } } }\n")
    ancestor_classes = ancestor / "classes"
    ancestor_classes.mkdir()
    ancestor_compile = run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                           ancestor_classes, ancestor_source)
    require(ancestor_compile.returncode == 0,
            f"non-direct-parent fixture did not compile: {ancestor_compile.stderr}")
    ancestor_jar, _ = make_jar(ancestor_classes, work, "non-direct-parent")
    method, _ = jarde_method(ancestor_jar, "n/Family$B", "set(Z)V")
    require("@bytecode 2" in method and ".value = arg1;" not in method,
            "non-direct ancestor field must retain BCI 2 as refused source")
    refused["non_direct_parent_wrong_owner"] = "refused at BCI 2"

    receiver = work / "non_B_receiver"
    receiver_source = receiver / "src" / "n" / "Family.java"
    receiver_source.parent.mkdir(parents=True)
    receiver_source.write_text("package n; public class Family { "
                               "public static class A { protected boolean value; } "
                               "public static class B extends A { "
                               "public void set(C receiver, boolean value) { "
                               "((A) receiver).value = value; } } "
                               "public static class C extends A {} }\n")
    receiver_classes = receiver / "classes"
    receiver_classes.mkdir()
    receiver_compile = run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                           receiver_classes, receiver_source)
    require(receiver_compile.returncode == 0,
            f"non-B receiver fixture did not compile: {receiver_compile.stderr}")
    receiver_jar, _ = make_jar(receiver_classes, work, "non-B-receiver")
    method, _ = jarde_method(receiver_jar, "n/Family$B", "set(Ln/Family$C;Z)V")
    require("@bytecode" in method and ".value = arg2;" not in method,
            f"a receiver not typed as the method class must retain its BCI: {method}")
    bytecode = re.search(r"@bytecode ([0-9 ]+)", method)
    require(bytecode is not None, "non-B receiver refusal has no physical BCI")
    refused["non_B_receiver"] = f"refused at BCI {bytecode.group(1).strip()}"

    cross = work / "cross-package"
    source_a = cross / "src" / "p" / "A.java"
    source_b = cross / "src" / "q" / "B.java"
    source_a.parent.mkdir(parents=True)
    source_b.parent.mkdir(parents=True)
    source_a.write_text("package p; public class A { protected boolean value; }\n")
    source_b.write_text("package q; public class B extends p.A { "
                        "protected boolean value; "
                        "public void set(boolean value) { super.value = value; } }\n")
    cross_classes = cross / "classes"
    cross_classes.mkdir()
    cross_compile = run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                        cross_classes, source_a, source_b)
    require(cross_compile.returncode == 0,
            f"cross-package protected Java source did not compile: {cross_compile.stderr}")
    cross_jar, cross_names = make_jar(cross_classes, work, "cross-package")
    cross_method, _ = jarde_method(cross_jar, "q/B", "set(Z)V")
    require("@bytecode 2" in cross_method and ".value = arg1;" not in cross_method,
            "cross-package protected field must retain BCI 2 as refused source")
    invalid_cast = cross / "src" / "q" / "IllegalCast.java"
    invalid_cast.write_text("package q; class IllegalCast extends p.A { "
                            "void set() { ((p.A) this).value = true; } }\n")
    invalid_dir = cross / "invalid-classes"
    invalid_dir.mkdir()
    invalid = run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-cp",
                  cross_classes, "-d", invalid_dir, invalid_cast)
    require(invalid.returncode != 0 and "protected" in invalid.stderr,
            "the cross-package A owner cast unexpectedly became legal Java source: "
            f"exit={invalid.returncode} stderr={invalid.stderr}")
    refused["cross_package_protected"] = "refused at BCI 2; explicit A cast rejected by javac"

    public_final = mutate_field_flags(parent, "publicField", 0x0001 | 0x0010)
    method, _ = jarde_method(jar_variant(work, "public-final-control", base,
                                         {parent_name: public_final}),
                             "dt29/SamePackageParentFamily$B", "set(ZZ)V")
    require("@bytecode 2" not in method and ").publicField = arg1;" in method,
            "existing public-field certificate changed for public final declaration")

    method, _ = jarde_method(jar_path, "dt29/SamePackageParentFamily$B", "set(ZZ)V")
    require("access$002((dt29.SamePackageParentFamily$A) this, arg2)" in method,
            "private accessor control changed")
    budget = run(JARDE, "class-source", "--input", jar_path, "--class",
                 "dt29/SamePackageParentFamily$B", "--policy", "plain-jar", "--release", "8",
                 "--format", "json", "--budget", "method_bodies=1")
    require(budget.returncode == 4 and "stopped" in budget.stdout,
            "a body-budget stop must remain explicit")
    return refused | {"public_final_control": "existing public path retained",
                      "private_accessor_control": "physical accessor call retained",
                      "method_body_budget": "request stopped explicitly"}


def combined_b_case(work: Path) -> dict[str, object]:
    source = HERE.parent / "dt29-reference-cast-audit" / "combined" / "inputs" / "FieldCast.java"
    classes = work / "combined-classes"
    classes.mkdir()
    compile_result = run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                         classes, source)
    require(compile_result.returncode == 0,
            f"combined FieldCast source did not compile: {compile_result.stderr}")
    jar_path, _ = make_jar(classes, work, "combined-fieldcast")
    result = run(JARDE, "class-source", "--input", jar_path, "--class", "dt29/FieldCast$B",
                 "--policy", "plain-jar", "--release", "8", "--format", "json",
                 "--evidence", "all")
    require(result.returncode == 0,
            f"combined B report did not complete: {result.stderr[-1000:]}")
    report = json.loads(result.stdout)
    method = next(member for member in report["methods"]
                  if member["item"]["identity"]["name"] == list(b"self"))
    body = method["outcome"]["report"]
    require(body["quality"] == "structured" and "@bytecode 7" not in body["text"]
            and "@bytecode 12" not in body["text"],
            "combined B.self did not recover BCI 7 and 12 as one complete body")
    for bci, field_name in ((7, "protectedField"), (12, "packagePrivateField")):
        require(any(segment["origin"]["primary"]["bci"] == bci
                    and f".{field_name} = arg1;" in body["text"][segment["start"]:segment["end"]]
                    for segment in body["source_map"]["segments"]),
                f"combined B.self lost field-write source at BCI {bci}")
    javap = checked("javap", "-classpath", jar_path, "-v", "-c", "dt29.FieldCast$B")
    require("Field dt29/FieldCast$A.protectedField:Z" in javap
            and "Field dt29/FieldCast$A.packagePrivateField:Z" in javap,
            "combined BCI 7/12 do not retain the physical A field owner")
    (OUT / "reports" / "combined-FieldCast$B.json").write_text(
        json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    shutil.copyfile(jar_path, OUT / "combined-input.jar")
    (OUT / "combined-javap.txt").write_text(javap)
    return {
        "class": "dt29/FieldCast$B",
        "method": "self(Z)V",
        "source_map_bcis": [7, 12],
        "physical_cp_owner": "dt29/FieldCast$A",
        "remaining_scope": ["C", "D", "root string concatenation/run/bits"],
    }


def run_jarde(jar_path: Path, names: list[str], work: Path) -> tuple[dict[str, str], dict[str, object]]:
    source_dir = work / "jarde-source" / "dt29"
    source_dir.mkdir(parents=True)
    source_texts: dict[str, str] = {}
    reports: dict[str, object] = {}
    for name in names:
        text = checked(JARDE, "class-source", "--input", jar_path, "--class", name,
                       "--policy", "plain-jar", "--release", "8", "--format", "text")
        file_name = name.rsplit("/", 1)[-1] + ".java"
        source_texts[name] = text
        (source_dir / file_name).write_text(text)
        report_text = checked(JARDE, "class-source", "--input", jar_path, "--class", name,
                              "--policy", "plain-jar", "--release", "8", "--format", "json")
        report = json.loads(report_text)
        for method in report.get("methods", []):
            usage = method.get("outcome", {}).get("report", {}).get("usage")
            if isinstance(usage, dict) and "elapsed_millis" in usage:
                usage["elapsed_millis"] = "<elapsed>"
        reports[name] = report
    return source_texts, reports


def main() -> None:
    require(JARDE.is_file(), "set JARDE_CLI to the built jarde-cli executable")
    require(checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip() == JADX_HEAD,
            "fixed JADX checkout HEAD changed")
    require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
            "fixed JADX checkout is not clean")
    require(JADX.is_file(), f"fixed JADX executable is missing: {JADX}")

    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)
    for name in ("original", "jadx", "jarde", "reports"):
        (OUT / name).mkdir()
    fixture = HERE / "fixtures"
    sources = [fixture / "SamePackageParentFamily.java"]
    runner = fixture / "SamePackageParentRunner.java"
    with tempfile.TemporaryDirectory(prefix="jarde-parent-field-") as temporary:
        work = Path(temporary)
        original = compile_run(sources, runner, work, "original")
        require(original.get("compile_exit") == 0 and original.get("run_exit") == 0
                and original.get("run_stdout") == EXPECTED and original.get("run_stderr") == "",
                f"original Java 8 run failed: {original}")
        for path in sources + [runner]:
            shutil.copyfile(path, OUT / "original" / path.name)
        original_classes = work / "original-classes"
        jar_path, names = make_jar(original_classes, work, "fixture")

        jadx_dir = work / "jadx"
        checked(JADX, "-d", jadx_dir, jar_path)
        jadx_sources = sorted(jadx_dir.rglob("*.java"))
        jadx = compile_run(jadx_sources, runner, work, "jadx-rebuild")
        require(jadx.get("compile_exit") == 0 and jadx.get("run_exit") == 0
                and jadx.get("run_stdout") == EXPECTED and jadx.get("run_stderr") == "",
                f"JADX Java 8 run failed: {jadx}")
        for path in jadx_sources:
            shutil.copyfile(path, OUT / "jadx" / path.name)

        jarde_sources, reports = run_jarde(jar_path, names, work)
        require("@bytecode" not in "\n".join(jarde_sources.values()),
                "the positive complete class family contains an unrecovered method")
        jarde_dir = work / "jarde-source" / "dt29"
        jarde_paths = sorted(path for path in jarde_dir.glob("*.java")
                             if path.name != "SamePackageParentRunner.java")
        shutil.copyfile(runner, jarde_dir / runner.name)
        jarde = compile_run(jarde_paths, jarde_dir / runner.name, work, "jarde-rebuild")
        require(jarde.get("compile_exit") == 0 and jarde.get("run_exit") == 0
                and jarde.get("run_stdout") == EXPECTED and jarde.get("run_stderr") == "",
                f"Jarde Java 8 run failed: {jarde}")
        b_text = jarde_sources["dt29/SamePackageParentFamily$B"]
        require("((dt29.SamePackageParentFamily$A) this).protectedField = arg1;" in b_text
                and "((dt29.SamePackageParentFamily$A) this).packagePrivateField = arg1;" in b_text,
                "Jarde did not preserve the selected parent field owner cast")
        b_report = reports["dt29/SamePackageParentFamily$B"]
        setter = next(method for method in b_report["methods"]
                      if method["item"]["identity"]["name"] == list(b"set"))
        body = setter["outcome"]["report"]
        require(body["quality"] == "structured", "Jarde setter did not remain structured")
        for bci, fragment in ((7, ".protectedField = arg1;"),
                              (12, ".packagePrivateField = arg1;")):
            require(any(segment["origin"]["primary"]["bci"] == bci
                        and fragment in body["text"][segment["start"]:segment["end"]]
                        for segment in body["source_map"]["segments"]),
                    f"BCI {bci} lost its source origin")
        boundaries = boundary_cases(jar_path, names, work)
        combined = combined_b_case(work)

        for path in sources + [runner]:
            shutil.copyfile(path, OUT / "original" / path.name)
        for path in sorted(jadx_dir.rglob("*.java")):
            shutil.copyfile(path, OUT / "jadx" / path.name)
        for name, source in jarde_sources.items():
            file_name = name.rsplit("/", 1)[-1] + ".java"
            (OUT / "jarde" / file_name).write_text(source)
            (OUT / "reports" / (name.rsplit("/", 1)[-1] + ".json")).write_text(
                json.dumps(reports[name], indent=2, ensure_ascii=False, sort_keys=True) + "\n")
        shutil.copyfile(runner, OUT / "jarde" / runner.name)

        (OUT / "results.json").write_text(json.dumps({
            "expected_stdout": EXPECTED,
            "fixed_jadx_head": JADX_HEAD,
            "input_classes": names,
            "input_class_sha256": {name: sha(original_classes / f"{name}.class") for name in names},
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
            "jarde_parent_casts": [
                "BCI 7 -> ((dt29.SamePackageParentFamily$A) this).protectedField",
                "BCI 12 -> ((dt29.SamePackageParentFamily$A) this).packagePrivateField",
            ],
            "boundary_cases": boundaries,
            "combined_B": combined,
            "source_map_bcis": [7, 12],
        }, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
        (OUT / "javap.txt").write_text(checked(
            "javap", "-classpath", original_classes, "-v", "-c",
            "dt29.SamePackageParentFamily$B"))
        shutil.copyfile(jar_path, OUT / "input.jar")
    files = sorted(path for path in OUT.rglob("*") if path.is_file()
                   and path.name != "SHA256SUMS")
    (OUT / "SHA256SUMS").write_text("".join(
        f"{sha(path)}  {path.relative_to(OUT)}\n" for path in files))
    print((OUT / "results.json").read_text(), end="")


if __name__ == "__main__":
    main()
