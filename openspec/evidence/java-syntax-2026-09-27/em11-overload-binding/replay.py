#!/usr/bin/env python3
"""Replay the legal Java 8 overload-binding slice against fixed JADX and Jarde."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em11"
NAMES = ("OverloadCalls", "HBase", "HMid", "HLeaf", "HierarchyCalls", "InputHierarchyCalls", "Runner")
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
JADX_PINS = {
    "test/java/jadx/tests/integration/invoke/TestCastInOverloadedInvoke.java": "ce275a64584f006b8e6ee9e516cee134fe1d85a650880f7638cbade6bb4ebc0c",
    "test/java/jadx/tests/integration/invoke/TestHierarchyOverloadedInvoke.java": "3d878526b62265fd24ffb705bcb12e95c4d54fdc3eb89c82b8b4545569a35cde",
    "test/java/jadx/tests/integration/invoke/TestOverloadedInvoke.java": "a17d21222c3cdb07481e5516e486de9d255215767dc82671bb9dd655703293f9",
    "test/java/jadx/tests/integration/invoke/TestCastInOverloadedAccessor.java": "a673bdc37513ddd88326326642ac663c2c8c46b4ef98c2c5fa109a140c7eb716",
    "main/java/jadx/core/dex/visitors/MethodInvokeVisitor.java": "60ee5bd34ad003d48c85bc622f7ac160f4a197acb894999140fcedf1914129ff",
    "main/java/jadx/core/codegen/InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
EXPECTED = (
    "ArrayList/List/String/List/ArrayList/String/Object[][]/int[][]\n"
    "ArrayList/List/String/List/ArrayList/none/Object[][]/int[][]\n"
    "leaf-ArrayList/mid-List/base-String/mid-List/leaf-ArrayList/base-String/mid-List\n"
    "mid\n"
    "created=2\n"
)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(*args):
    return subprocess.run([str(arg) for arg in args], capture_output=True, text=True, timeout=180)


def checked(*args):
    result = run(*args)
    if result.returncode:
        raise RuntimeError(f"{args[0]} failed ({result.returncode}): {result.stderr[:1200]}")
    return result


def compile_and_run(label, sources, directory, output, main_class="em11.Runner"):
    classes = directory / f"{label}-classes"
    classes.mkdir()
    compile_result = run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *sources)
    (output / f"{label}-javac.log").write_text(compile_result.stdout + compile_result.stderr)
    result = {"javac_exit": compile_result.returncode}
    if compile_result.returncode == 0:
        execution = run("java", "-Xverify:all", "-cp", classes, main_class)
        (output / f"{label}-runtime.log").write_text(execution.stdout + execution.stderr)
        result.update(runtime_exit=execution.returncode, stdout=execution.stdout,
                      stdout_sha256=hashlib.sha256(execution.stdout.encode()).hexdigest())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, required=True)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("--out must be empty")
    output.mkdir(parents=True, exist_ok=True)

    revision = checked("git", "-C", args.jadx_checkout, "rev-parse", "HEAD").stdout.strip()
    if revision != JADX_REV:
        raise RuntimeError(f"JADX checkout moved: {revision}")
    for relative, sha in JADX_PINS.items():
        if digest(args.jadx_checkout / "jadx-core" / "src" / relative) != sha:
            raise RuntimeError(f"JADX source changed: {relative}")

    originals = [INPUT / f"{name}.java" for name in NAMES]
    with tempfile.TemporaryDirectory(prefix="jarde-em11-") as tmp:
        temporary = Path(tmp)
        original = compile_and_run("original", originals, temporary, output)
        if original.get("stdout") != EXPECTED or original["javac_exit"]:
            raise RuntimeError("original source did not compile and produce the pinned observations")
        jar = temporary / "input.jar"
        checked("jar", "cf", jar, "-C", temporary / "original-classes", ".")
        class_hashes = {name: digest(temporary / "original-classes" / "em11" / f"{name}.class")
                        for name in NAMES}
        jadx_tmp = temporary / "jadx"
        jadx_result = checked(args.jadx, "--no-res", "-d", jadx_tmp, jar)
        (output / "jadx.log").write_text("\n".join(
            line.rstrip() for line in (jadx_result.stdout + jadx_result.stderr).splitlines()) + "\n")
        jadx_sources = []
        jarde_sources = []
        jarde_status = {}
        for name in NAMES:
            jadx_source = output / "source" / "jadx" / "em11" / f"{name}.java"
            jadx_source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_tmp / "sources" / "em11" / f"{name}.java", jadx_source)
            jadx_sources.append(jadx_source)
            jarde_result = run(args.jarde, "class-source", "--input", jar, "--class", f"em11.{name}",
                               "--policy", "plain-jar", "--release", "8", "--format", "text")
            jarde_source = output / "source" / "jarde" / "em11" / f"{name}.java"
            jarde_source.parent.mkdir(parents=True, exist_ok=True)
            jarde_source.write_text(jarde_result.stdout)
            jarde_sources.append(jarde_source)
            jarde_status[name] = {"exit": jarde_result.returncode,
                                  "stderr_sha256": hashlib.sha256(jarde_result.stderr.encode()).hexdigest()}
        jadx = compile_and_run("jadx", jadx_sources, temporary, output)
        jarde = compile_and_run("jarde", jarde_sources, temporary, output)
        source_anchors = {}
        for name, marker, cast, call_bci in (
            ("OverloadCalls", "call((java.util.List) new java.util.ArrayList())",
             "(java.util.List) new java.util.ArrayList()", 18),
            ("HierarchyCalls", "call((java.util.List) new java.util.ArrayList())",
             "(java.util.List) new java.util.ArrayList()", 42),
            ("InputHierarchyCalls", "take((em11.HMid) new em11.HLeaf())",
             "(em11.HMid) new em11.HLeaf()", 7),
        ):
            source = output / "source" / "jarde" / "em11" / f"{name}.java"
            if marker not in source.read_text():
                raise RuntimeError(f"target overload cast missing from {name}")
            evidence = checked(args.jarde, "class-source", "--input", jar,
                               "--class", f"em11.{name}", "--policy", "plain-jar",
                               "--release", "8", "--format", "json", "--evidence", "source_map")
            report = json.loads(evidence.stdout)
            body = next(method["outcome"]["report"] for method in report["methods"]
                        if method["item"]["name"]["escaped"] == "run")
            anchors = [segment["origin"] for segment in body["source_map"]["segments"]
                       if body["text"][segment["start"]:segment["end"]] == cast]
            if not any(origin.get("primary") and
                       call_bci in [anchor["bci"] for anchor in origin["derived"]]
                       for origin in anchors):
                raise RuntimeError(f"cast source map lost its producer or call BCI: {name}")
            source_anchors[name] = {
                "producer_bci": anchors[0]["primary"]["bci"],
                "derived_bcis": [anchor["bci"] for anchor in anchors[0]["derived"]],
            }
        simple_names = ("NullArrayCalls", "Runner")
        simple_sources = [HERE / "input" / "em11simple" / f"{name}.java" for name in simple_names]
        simple_original = compile_and_run("simple-original", simple_sources, temporary, output,
                                          "em11simple.Runner")
        simple_jar = temporary / "simple.jar"
        checked("jar", "cf", simple_jar, "-C", temporary / "simple-original-classes", ".")
        simple_class_hashes = {
            name: digest(temporary / "simple-original-classes" / "em11simple" / f"{name}.class")
            for name in simple_names
        }
        simple_jadx_tmp = temporary / "simple-jadx"
        simple_jadx_result = checked(args.jadx, "--no-res", "-d", simple_jadx_tmp, simple_jar)
        (output / "simple-jadx.log").write_text("\n".join(
            line.rstrip() for line in (simple_jadx_result.stdout + simple_jadx_result.stderr).splitlines()) + "\n")
        simple_jadx_sources = []
        simple_jarde_sources = []
        for name in simple_names:
            source = output / "source" / "simple-jadx" / "em11simple" / f"{name}.java"
            source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(simple_jadx_tmp / "sources" / "em11simple" / f"{name}.java", source)
            simple_jadx_sources.append(source)
            jarde_result = run(args.jarde, "class-source", "--input", simple_jar,
                               "--class", f"em11simple.{name}", "--policy", "plain-jar",
                               "--release", "8", "--format", "text")
            if jarde_result.returncode:
                raise RuntimeError(f"Jarde simple class-source failed: {name}")
            source = output / "source" / "simple-jarde" / "em11simple" / f"{name}.java"
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text(jarde_result.stdout)
            simple_jarde_sources.append(source)
        simple_jadx = compile_and_run("simple-jadx", simple_jadx_sources, temporary, output,
                                      "em11simple.Runner")
        simple_jarde = compile_and_run("simple-jarde", simple_jarde_sources, temporary, output,
                                       "em11simple.Runner")

        negative_names = ("MissingBase", "MissingChild", "MissingCalls", "MissingIface",
                          "InterfaceChild", "MissingInterfaceCalls", "Runner")
        negative_sources = [HERE / "input" / "em11negative" / f"{name}.java"
                            for name in negative_names]
        negative_original = compile_and_run("negative-original", negative_sources,
                                            temporary, output, "em11negative.Runner")
        if negative_original.get("stdout") != "base\ninterface\n":
            raise RuntimeError("the complete negative input did not retain its original binding")
        negative_class_hashes = {
            name: digest(temporary / "negative-original-classes" / "em11negative" / f"{name}.class")
            for name in negative_names
        }
        missing_jar = temporary / "missing-base.jar"
        checked("jar", "cf", missing_jar,
                "-C", temporary / "negative-original-classes", "em11negative/MissingChild.class",
                "-C", temporary / "negative-original-classes", "em11negative/MissingCalls.class",
                "-C", temporary / "negative-original-classes", "em11negative/InterfaceChild.class",
                "-C", temporary / "negative-original-classes", "em11negative/MissingInterfaceCalls.class")
        missing_results = {}
        missing_sources = []
        for name in ("MissingCalls", "MissingInterfaceCalls"):
            missing_result = run(args.jarde, "class-source", "--input", missing_jar,
                                 "--class", f"em11negative.{name}", "--policy", "plain-jar",
                                 "--release", "8", "--format", "text")
            missing_source = output / "source" / "missing-hierarchy-jarde" / "em11negative" / f"{name}.java"
            missing_source.parent.mkdir(parents=True, exist_ok=True)
            missing_source.write_text(missing_result.stdout)
            if (missing_result.returncode != 0
                    or "invocation at BCI 7" not in missing_result.stdout
                    or "no safe reference conversion evidence" not in missing_result.stdout):
                raise RuntimeError(f"missing selected hierarchy was not refused: {name}")
            missing_results[name] = missing_result.returncode
            missing_sources.append(missing_source)

    if any(item.get("stdout") != EXPECTED for item in (original, jadx, jarde)):
        raise RuntimeError("full-source overload binding differs across the three compilers")
    expected_simple = "String/List/ArrayList/Object[][]/int[][]\n"
    if any(item.get("stdout") != expected_simple for item in
           (simple_original, simple_jadx, simple_jarde)):
        raise RuntimeError("null/array slice failed full-source overload comparison")
    result = {
        "jadx_revision": revision,
        "jadx_source_pins": JADX_PINS,
        "jarde_cli_sha256": digest(args.jarde),
        "input_source_sha256": {source.name: digest(source) for source in originals},
        "original_class_sha256": class_hashes,
        "generated_source_sha256": {
            label: {source.name: digest(source) for source in sources}
            for label, sources in (("jadx", jadx_sources), ("jarde", jarde_sources))
        },
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "jarde_class_source": jarde_status,
        "cast_source_anchors": source_anchors,
        "simple_input_source_sha256": {source.name: digest(source) for source in simple_sources},
        "simple_original_class_sha256": simple_class_hashes,
        "simple_generated_source_sha256": {
            label: {source.name: digest(source) for source in sources}
            for label, sources in (("jadx", simple_jadx_sources), ("jarde", simple_jarde_sources))
        },
        "simple_original": simple_original,
        "simple_jadx": simple_jadx,
        "simple_jarde": simple_jarde,
        "negative_input_source_sha256": {source.name: digest(source) for source in negative_sources},
        "negative_original_class_sha256": negative_class_hashes,
        "negative_original": negative_original,
        "missing_hierarchy_jarde_source_sha256": {source.name: digest(source)
                                                   for source in missing_sources},
        "missing_hierarchy_jarde_exit": missing_results,
    }
    (output / "results.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(f"original={original['javac_exit']}/{original.get('runtime_exit')} "
          f"jadx={jadx['javac_exit']}/{jadx.get('runtime_exit')} "
          f"jarde={jarde['javac_exit']}/{jarde.get('runtime_exit')} "
          f"simple={simple_original['javac_exit']}/{simple_jadx['javac_exit']}/{simple_jarde['javac_exit']}")


if __name__ == "__main__":
    main()
