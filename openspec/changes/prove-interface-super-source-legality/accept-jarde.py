#!/usr/bin/env python3
"""Independently recompile legal I.super calls and refuse three patched source-illegal calls."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZipFile


HERE = Path(__file__).resolve().parent
EVIDENCE = HERE.parent.parent / "evidence" / "java-syntax-2026-09-25"
DIRECT = EVIDENCE / "redundant-interface-super"
ABSTRACT = EVIDENCE / "abstract-interface-super"
SUPERCLASS = EVIDENCE / "superclass-redundant-interface-super"
PATCH = runpy.run_path(str(DIRECT / "replay.py"))["replace_interface_owner"]


def command(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def run(*args: object) -> str:
    result = command(*args)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, args))}\n{result.stdout}{result.stderr}")
    return result.stdout


def jar_of(classes: Path, target: Path) -> Path:
    with ZipFile(target, "w") as archive:
        for item in sorted(classes.rglob("*.class")):
            archive.write(item, item.relative_to(classes))
    return target


def class_source(cli: Path, jar: Path, binary: str, evidence: str = "all") -> dict:
    report = json.loads(run(cli, "class-source", "--input", jar, "--class", binary,
                            "--format", "json", "--evidence", evidence))
    if report["execution"]["status"] != "complete":
        raise RuntimeError(f"{binary}: unexpected request stop: {report['execution']}")
    return report


def recover(cli: Path, jar: Path, binary: str, method: str, evidence: str = "all") -> dict:
    document = json.loads(run(cli, "recover", "--input", jar, "--class-name", binary,
                              "--method-name", method, "--descriptor", "()I",
                              "--format", "json", "--evidence", evidence))
    if (document["outcome"] != "performed"
            or document["recovered"]["recovery"]["execution"]["status"] != "complete"):
        raise RuntimeError(f"{binary}.{method}: method-only request did not complete")
    return document["recovered"]["recovery"]


def has_bytecode_bci(text: str, bci: int) -> bool:
    return any(
        str(bci) in line.split("@bytecode", 1)[1].split()
        for line in text.splitlines()
        if "@bytecode" in line
    )


def compare_legal(cli: Path, work: Path, classes: Path, binary: str,
                  source_name: str, runner: str, expected: str, qualifiers: tuple[str, ...]) -> None:
    work.mkdir(parents=True)
    jar = jar_of(classes, work / f"{source_name}.jar")
    original = run("java", "-Xverify:all", "-cp", classes, runner)
    if original != expected:
        raise RuntimeError(f"{binary}: original {original!r}, expected {expected!r}")
    all_report = class_source(cli, jar, binary)
    essential = class_source(cli, jar, binary, "essential")
    source = all_report["text"]
    if essential["text"] != source or any(qualifier not in source for qualifier in qualifiers):
        raise RuntimeError(f"{binary}: source changed with evidence or lost qualifier")
    if binary == "InterfaceSuperProbe":
        expected_methods = {
            "value": ("DefaultLeft.super.value()",),
            "chooseRight": ("DefaultRight.super.value()",),
            "both": ("DefaultLeft.super.value()", "DefaultRight.super.value()"),
        }
    elif binary == "DefaultChild":
        expected_methods = {"value": ("DefaultParent.super.value()",)}
    else:
        expected_methods = {"value": ("Child.super.value()",)}
    for method, expected_calls in expected_methods.items():
        detailed = recover(cli, jar, binary, method)
        lean = recover(cli, jar, binary, method, "essential")
        if (detailed["text"] != lean["text"] or detailed["fallbacks"]
                or any(call not in detailed["text"] for call in expected_calls)):
            raise RuntimeError(f"{binary}.{method}: method-only dispatch or evidence differs")
    if binary == "InterfaceSuperProbe":
        ranged = command(cli, "class-source", "--input", jar, "--class", binary,
                         "--format", "json", "--evidence", "source_map",
                         "--evidence-bci", "0..2")
        range_report = json.loads(ranged.stdout)
        if (ranged.returncode != 4
                or range_report["execution"]["status"] != "partial"
                or range_report["execution"]["reason"]["code"]
                != "jre_evidence_range_invalid"):
            raise RuntimeError(f"{binary}: unsupported class-source BCI range was not an explicit stop")
    rebuilt_source = work / "source" / f"{source_name}.java"
    rebuilt_source.parent.mkdir(exist_ok=True)
    rebuilt_source.write_text(source)
    rebuilt = work / "rebuilt"
    rebuilt.mkdir()
    run("javac", "--release", "8", "-Xlint:-options", "-cp", classes,
        "-d", rebuilt, rebuilt_source)
    actual = run("java", "-Xverify:all", "-cp",
                 os.pathsep.join((str(rebuilt), str(classes))), runner)
    if actual != expected:
        raise RuntimeError(f"{binary}: rebuilt {actual!r}, expected {expected!r}")


def refuse_illegal(cli: Path, work: Path, classes: Path, binary: str,
                   invalid_qualifier: str, method: str, call_bci: int = 1) -> None:
    work.mkdir(parents=True)
    jar = jar_of(classes, work / "patched.jar")
    report = class_source(cli, jar, binary)
    source = report["text"]
    if class_source(cli, jar, binary, "essential")["text"] != source:
        raise RuntimeError(f"{binary}.{method}: evidence selection changed refusal text")
    if invalid_qualifier in source or not has_bytecode_bci(source, call_bci):
        raise RuntimeError(f"{binary}.{method}: source published unproved qualifier or lost BCI {call_bci}\n{source}")
    item = next(row for row in report["methods"] if row["item"]["name"]["escaped"] == method)
    outcome = item["outcome"]
    if (outcome["kind"] != "recovered"
            or outcome["report"]["quality"] != "fallback"
            or outcome["report"]["content"] != "explanation_only"):
        raise RuntimeError(f"{binary}.{method}: refusal not classified as a recovery gap")
    detailed = recover(cli, jar, binary, method)
    lean = recover(cli, jar, binary, method, "essential")
    if (invalid_qualifier in detailed["text"] or not has_bytecode_bci(detailed["text"], call_bci)
            or detailed["quality"] != "fallback"
            or detailed["content"] != "explanation_only"
            or detailed["text"] != lean["text"]):
        raise RuntimeError(f"{binary}.{method}: method-only refused call lost its gap or source identity")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        parser.error(f"Jarde CLI missing: {cli}")

    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        with TemporaryDirectory(prefix=f"jarde-interface-accept-{label}-") as temporary:
            root = Path(temporary)
            direct = root / "direct"
            direct.mkdir()
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", direct,
                DIRECT / "InterfaceSuperProbe.java", DIRECT / "InterfaceSuperRunner.java")
            compare_legal(cli, root / "direct-case", direct, "InterfaceSuperProbe",
                          "InterfaceSuperProbe", "InterfaceSuperRunner", "11\n22\n33\n",
                          ("DefaultLeft.super.value()", "DefaultRight.super.value()"))

            for variant, value in (("default", "4\n"), ("inherited", "3\n")):
                classes = root / variant
                classes.mkdir()
                run("javac", "--release", "8", "-Xlint:-options", debug, "-d", classes,
                    ABSTRACT / "Parent.java", ABSTRACT / variant / "Child.java",
                    ABSTRACT / "Probe.java")
                compare_legal(cli, root / f"{variant}-case", classes, "Probe", "Probe",
                              "Probe", value, ("Child.super.value()",))
                if variant == "inherited":
                    missing = root / "missing-default"
                    missing.mkdir()
                    shutil.copy2(classes / "Probe.class", missing / "Probe.class")
                    shutil.copy2(classes / "Child.class", missing / "Child.class")
                    refuse_illegal(cli, root / "missing-default-case", missing, "Probe",
                                   "Child.super.value()", "value")

            interface_default = root / "interface-default"
            interface_default.mkdir()
            interface_default_source = root / "DefaultInInterface.java"
            interface_default_source.write_text(
                "interface DefaultParent { default int value() { return 3; } }\n"
                "interface DefaultChild extends DefaultParent {\n"
                "  default int value() { return DefaultParent.super.value() + 1; }\n"
                "}\n"
                "class DefaultRunner implements DefaultChild {\n"
                "  public static void main(String[] args) {\n"
                "    System.out.println(new DefaultRunner().value());\n"
                "  }\n"
                "}\n"
            )
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", interface_default,
                interface_default_source)
            compare_legal(cli, root / "interface-default-case", interface_default,
                          "DefaultChild", "DefaultChild", "DefaultRunner", "4\n",
                          ("DefaultParent.super.value()",))

            overload = root / "overload"
            overload.mkdir()
            overload_source = root / "OverloadProbe.java"
            overload_source.write_text(
                "interface Overloaded {\n"
                "  default int value(int n) { return n + 9; }\n"
                "  default int value(long n) { return (int) n + 19; }\n"
                "}\n"
                "public final class OverloadProbe implements Overloaded {\n"
                "  public int value() { return Overloaded.super.value(1); }\n"
                "  public static void main(String[] args) {\n"
                "    System.out.println(new OverloadProbe().value());\n"
                "  }\n"
                "}\n"
            )
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", overload,
                overload_source)
            if run("java", "-Xverify:all", "-cp", overload, "OverloadProbe") != "10\n":
                raise RuntimeError("same-name overload control changed JVM behavior")
            refuse_illegal(cli, root / "overload-case", overload, "OverloadProbe",
                           "Overloaded.super.value(1)", "value", 2)

            redundant = root / "redundant"
            redundant.mkdir()
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", redundant,
                DIRECT / "RedundantDirectSuperProbe.java", DIRECT / "RedundantRunner.java")
            target = redundant / "RedundantDirectSuperProbe.class"
            target.write_bytes(PATCH(target.read_bytes(), "RedundantChild", "RedundantParent", "value"))
            if run("java", "-Xverify:all", "-cp", redundant, "RedundantRunner") != "3\n":
                raise RuntimeError("patched direct-interface control changed JVM behavior")
            refuse_illegal(cli, root / "redundant-case", redundant, "RedundantDirectSuperProbe",
                           "RedundantParent.super.value()", "value")

            effectful = root / "effectful"
            effectful.mkdir()
            effectful_source = root / "EffectfulProbe.java"
            effectful_source.write_text(
                "interface EffectParent { default int value(int x) { return x + 3; } }\n"
                "interface EffectChild extends EffectParent {}\n"
                "public final class EffectfulProbe implements EffectParent, EffectChild {\n"
                "  static int count;\n"
                "  static int tick() { count++; return 2; }\n"
                "  public int value() { return EffectChild.super.value(tick()); }\n"
                "  public static void main(String[] args) {\n"
                "    System.out.println(new EffectfulProbe().value() + \":\" + count);\n"
                "  }\n"
                "}\n"
            )
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", effectful,
                effectful_source)
            effectful_class = effectful / "EffectfulProbe.class"
            effectful_class.write_bytes(PATCH(effectful_class.read_bytes(), "EffectChild",
                                              "EffectParent", "value"))
            if run("java", "-Xverify:all", "-cp", effectful, "EffectfulProbe") != "5:1\n":
                raise RuntimeError("effectful patched interface-special changed JVM behavior")
            refuse_illegal(cli, root / "effectful-case", effectful, "EffectfulProbe",
                           "EffectParent.super.value(tick())", "value", 4)
            effect_jar = jar_of(effectful, root / "effectful-producer.jar")
            effect_class = class_source(cli, effect_jar, "EffectfulProbe")["text"]
            effect_method = recover(cli, effect_jar, "EffectfulProbe", "value")["text"]
            if not has_bytecode_bci(effect_class, 1) or not has_bytecode_bci(effect_method, 1):
                raise RuntimeError("refusal lost the effectful tick producer at BCI 1")

            valid = root / "valid"
            abstract = root / "abstract"
            valid.mkdir(); abstract.mkdir()
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", valid,
                ABSTRACT / "Parent.java", ABSTRACT / "default" / "Child.java", ABSTRACT / "Probe.java")
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", abstract,
                ABSTRACT / "Parent.java", ABSTRACT / "abstract" / "Child.java")
            shutil.copy2(abstract / "Child.class", valid / "Child.class")
            error = command("java", "-Xverify:all", "-cp", valid, "Probe")
            if error.returncode == 0 or "AbstractMethodError" not in error.stderr:
                raise RuntimeError("abstract-owner control no longer throws AbstractMethodError")
            refuse_illegal(cli, root / "abstract-case", valid, "Probe",
                           "Child.super.value()", "value")

            intermediate = root / "intermediate"
            intermediate.mkdir()
            intermediate_source = root / "IntermediateProbe.java"
            intermediate_source.write_text(
                "interface Ancestral { default int value() { return 3; } }\n"
                "interface Middle extends Ancestral {}\n"
                "interface Leaf extends Middle {}\n"
                "public final class IntermediateProbe implements Leaf {\n"
                "  public int value() { return Leaf.super.value(); }\n"
                "  public static void main(String[] args) {\n"
                "    System.out.println(new IntermediateProbe().value());\n"
                "  }\n"
                "}\n"
            )
            run("javac", "--release", "8", "-Xlint:-options", debug, "-d", intermediate,
                intermediate_source)
            if run("java", "-Xverify:all", "-cp", intermediate,
                   "IntermediateProbe") != "3\n":
                raise RuntimeError("unshadowed intermediate default no longer runs")
            abstract_middle_source = root / "Middle.java"
            abstract_middle_source.write_text(
                "interface Middle extends Ancestral { int value(); }\n"
            )
            abstract_middle = root / "abstract-middle"
            abstract_middle.mkdir()
            run("javac", "--release", "8", "-Xlint:-options", debug, "-cp", intermediate,
                "-d", abstract_middle, abstract_middle_source)
            shutil.copy2(abstract_middle / "Middle.class", intermediate / "Middle.class")
            intermediate_error = command("java", "-Xverify:all", "-cp", intermediate,
                                         "IntermediateProbe")
            if (intermediate_error.returncode == 0
                    or "AbstractMethodError" not in intermediate_error.stderr):
                raise RuntimeError("abstract intermediate owner no longer throws AbstractMethodError")
            refuse_illegal(cli, root / "intermediate-case", intermediate, "IntermediateProbe",
                           "Leaf.super.value()", "value")

            for variant in ("Probe", "TransitiveProbe"):
                classes = root / variant
                classes.mkdir()
                run("javac", "--release", "8", "-Xlint:-options", debug, "-d", classes,
                    SUPERCLASS / f"{variant}.java")
                target = classes / "p" / "Child.class"
                target.write_bytes(PATCH(target.read_bytes(), "p/B", "p/A", "m"))
                if run("java", "-Xverify:all", "-cp", classes, "p.Runner") != "1\n":
                    raise RuntimeError(f"{variant}: patched JVM behavior changed")
                refuse_illegal(cli, root / f"{variant}-case", classes, "p/Child",
                               "p.A.super.m()", "m")
            print(f"{label}: class and interface legal dispatch cases recompile; direct, effectful, abstract, intermediate, parent, ancestor, missing-default and overload controls refuse")


if __name__ == "__main__":
    main()
