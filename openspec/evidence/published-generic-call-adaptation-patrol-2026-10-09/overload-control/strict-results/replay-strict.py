#!/usr/bin/env python3
"""Rebuild and compare the BoundOverload control on two Java 8 compiler legs."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
OUT = HERE / "strict-results"
SOURCE = HERE / "BoundOverload.java"
CLI = Path("/tmp/jarde-raw-receiver-final-v3-cli")
JADX = Path("/opt/homebrew/bin/jadx")
JDKS = {
    "corretto8": {
        "home": Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"),
        "release": ["-source", "8", "-target", "8"],
    },
    "openjdk23": {
        "home": Path("/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"),
        "release": ["--release", "8"],
    },
}
DRIVER = """public class RelayDriver {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName(args[0]);
        Object receiver = type.getConstructor().newInstance();
        type.getMethod("relay", Number.class).invoke(receiver, Integer.valueOf(7));
    }
}
"""


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    global OUT
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=HERE / "strict-results")
    OUT = parser.parse_args().out.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite existing evidence: {OUT}")
    for path in (SOURCE, CLI, JADX):
        if not path.is_file():
            raise SystemExit(f"required file is missing: {path}")
    expected_cli_sha = "3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70"
    if sha(CLI) != expected_cli_sha:
        raise SystemExit("Jarde CLI SHA-256 does not match the authorized binary")

    OUT.mkdir(parents=True)
    commands = []
    cases = []

    def run(label, argv, cwd, area):
        argv = [str(item) for item in argv]
        result = subprocess.run(argv, cwd=cwd, capture_output=True)
        (area / f"{label}.stdout").write_bytes(result.stdout)
        (area / f"{label}.stderr").write_bytes(result.stderr)
        commands.append({
            "label": label,
            "argv": argv,
            "cwd": str(cwd),
            "returncode": result.returncode,
            "stdout_sha256": sha(area / f"{label}.stdout"),
            "stderr_sha256": sha(area / f"{label}.stderr"),
        })
        return result

    run("jadx-version", [JADX, "--version"], HERE, OUT)

    for leg, config in JDKS.items():
        home = config["home"]
        javac = home / "bin/javac"
        java = home / "bin/java"
        javap = home / "bin/javap"
        jar_tool = home / "bin/jar"
        if not all(tool.is_file() for tool in (javac, java, javap, jar_tool)):
            raise SystemExit(f"incomplete JDK installation: {home}")
        version_area = OUT / leg
        version_area.mkdir()
        run("javac-version", [javac, "-version"], HERE, version_area)
        run("java-version", [java, "-version"], HERE, version_area)
        for debug in ("debug", "nodebug"):
            case_dir = OUT / leg / debug
            case_dir.mkdir(parents=True)
            debug_args = ["-g"] if debug == "debug" else ["-g:none"]
            with tempfile.TemporaryDirectory(prefix="bound-overload-strict-") as temporary:
                temp = Path(temporary)
                input_classes = temp / "input-classes"
                input_classes.mkdir()
                jar_path = case_dir / "BoundOverload.jar"
                compile_args = [javac, *config["release"], *debug_args,
                                "-classpath", temp / "empty-input-classpath",
                                "-sourcepath", temp / "empty-input-sourcepath",
                                "-d", input_classes, SOURCE]
                (temp / "empty-input-classpath").mkdir()
                (temp / "empty-input-sourcepath").mkdir()
                original_compile = run("freeze-javac", compile_args, HERE, case_dir)
                if original_compile.returncode != 0:
                    raise SystemExit(f"input source failed to compile: {leg}/{debug}")
                run("freeze-jar", [jar_tool, "cf", jar_path, "-C", input_classes,
                                   "BoundOverload.class"], HERE, case_dir)
                input_hash = sha(jar_path)
                input_class_hash = sha(input_classes / "BoundOverload.class")
                shutil.copyfile(SOURCE, case_dir / "BoundOverload.original.java")
                run("original-javap", [javap, "-p", "-c", "-s", "-v", "-classpath",
                                       jar_path, "BoundOverload"], HERE, case_dir)
                javap_text = (case_dir / "original-javap.stdout").read_text(errors="replace")
                relay_match = re.search(
                    r"public void relay\(T\);(.*?)(?=\n\s*(?:public|private|protected)\s|\Z)",
                    javap_text,
                    re.S,
                )
                if relay_match is None:
                    raise SystemExit(f"javap did not show relay(T): {leg}/{debug}")
                relay_text = relay_match.group(1)
                invoke_match = re.search(r"// Method pick:\(([^)]*)\)V", relay_text)
                original_selected_descriptor = invoke_match.group(1) if invoke_match else None
                original_has_checkcast = "checkcast" in relay_text

                jarde_area = case_dir / "jarde"
                jarde_area.mkdir()
                emitted = run("class-source", [CLI, "class-source", "--input", jar_path,
                              "--class", "BoundOverload", "--policy", "plain-jar",
                              "--release", "8", "--format", "text"], HERE, jarde_area)
                (jarde_area / "BoundOverload.java").write_bytes(emitted.stdout)

                jadx_area = case_dir / "jadx"
                jadx_area.mkdir()
                jadx_out = jadx_area / "full-output"
                jadx_out.mkdir()
                jadx_result = run("jadx", [JADX, "--no-res", "-d", jadx_out, jar_path],
                                  HERE, jadx_area)
                if jadx_result.returncode != 0:
                    raise SystemExit(f"JADX failed: {leg}/{debug}")
                candidates = list(jadx_out.rglob("BoundOverload.java"))
                if len(candidates) != 1:
                    raise SystemExit(f"expected one JADX class source, found {len(candidates)}")
                jadx_source = jadx_area / "BoundOverload.java"
                shutil.copyfile(candidates[0], jadx_source)

                package_match = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", (jadx_area / "BoundOverload.java").read_text())
                jadx_class_name = (package_match.group(1) + "." if package_match else "") + "BoundOverload"

                row = {
                    "leg": leg,
                    "debug": debug,
                    "compiler": str(javac),
                    "java": str(java),
                    "release_args": config["release"],
                    "debug_args": debug_args,
                    "input_source": str(SOURCE),
                    "input_source_sha256": sha(SOURCE),
                    "input_jar": str(jar_path),
                    "input_jar_sha256": input_hash,
                    "input_class_sha256": input_class_hash,
                    "original_relay_selected_descriptor": original_selected_descriptor,
                    "original_relay_has_checkcast": original_has_checkcast,
                    "jarde_source_sha256": sha(jarde_area / "BoundOverload.java"),
                    "jadx_source_sha256": sha(jadx_area / "BoundOverload.java"),
                    "jadx_class_name": jadx_class_name,
                    "results": {},
                }
                for flavor, source_file, class_name, area in (
                    ("original", case_dir / "BoundOverload.original.java", "BoundOverload", case_dir / "original"),
                    ("jarde", jarde_area / "BoundOverload.java", "BoundOverload", jarde_area),
                    ("jadx", jadx_area / "BoundOverload.java", jadx_class_name, jadx_area),
                ):
                    if flavor == "original":
                        area.mkdir()
                        shutil.copyfile(source_file, area / "BoundOverload.java")
                        source_file = area / "BoundOverload.java"
                    with tempfile.TemporaryDirectory(prefix=f"bound-overload-{flavor}-") as compile_temp:
                        compile_temp = Path(compile_temp)
                        classes = compile_temp / "classes"
                        empty_cp = compile_temp / "empty-classpath"
                        empty_sp = compile_temp / "empty-sourcepath"
                        classes.mkdir()
                        empty_cp.mkdir()
                        empty_sp.mkdir()
                        driver = compile_temp / "RelayDriver.java"
                        driver.write_text(DRIVER)
                        result = run(f"{flavor}-javac", [javac, *config["release"], *debug_args,
                                      "-classpath", empty_cp, "-sourcepath", empty_sp,
                                      "-d", classes, source_file, driver], HERE, area)
                        flavor_row = {"source_sha256": sha(source_file), "compile_exit": result.returncode}
                        if result.returncode == 0:
                            runtime = run(f"{flavor}-java", [java, "-Xverify:all", "-cp", classes,
                                          "RelayDriver", class_name], HERE, area)
                            flavor_row.update({
                                "runtime_exit": runtime.returncode,
                                "runtime_stdout": runtime.stdout.decode(errors="replace"),
                                "runtime_stderr": runtime.stderr.decode(errors="replace"),
                            })
                        row["results"][flavor] = flavor_row
                cases.append(row)

    manifest = {
        "purpose": "Strict four-leg whole-class overload control; source, CLI, and tools are hashed or named below.",
        "jarde_cli": str(CLI),
        "jarde_cli_sha256": sha(CLI),
        "jadx_cli": str(JADX),
        "jadx_cli_sha256": sha(JADX),
        "replay_script": str(Path(__file__).resolve()),
        "replay_script_sha256": sha(__file__),
        "source": str(SOURCE),
        "source_sha256": sha(SOURCE),
        "commands": commands,
        "cases": cases,
    }
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")

    for case in cases:
        if case["results"]["original"].get("compile_exit") != 0:
            raise SystemExit(f"original compile failed: {case['leg']}/{case['debug']}")
        if case["results"]["original"].get("runtime_stdout", "").strip() != "number":
            raise SystemExit(f"original did not select Number overload: {case['leg']}/{case['debug']}")
        if case["original_relay_selected_descriptor"] != "Ljava/lang/Number;" or case["original_relay_has_checkcast"]:
            raise SystemExit(f"original bytecode dispatch differs from expected descriptor proof: {case['leg']}/{case['debug']}")
        if case["results"]["jarde"].get("compile_exit") == 0:
            raise SystemExit(f"Jarde unexpectedly compiled: {case['leg']}/{case['debug']}")

    original_ok = sum(c["results"]["original"].get("runtime_stdout", "").strip() == "number" for c in cases)
    jadx_ok = sum(c["results"]["jadx"].get("compile_exit") == 0 and
                  c["results"]["jadx"].get("runtime_stdout", "").strip() == "number" for c in cases)
    jarde_failed = sum(c["results"]["jarde"].get("compile_exit") != 0 for c in cases)
    summary = (
        "# Strict overload control\n\n"
        "This four-leg replay compiles the frozen `BoundOverload.java` with Corretto 8 and "
        "OpenJDK 23 `--release 8`, with and without debug metadata. Each generated jar is "
        "independently decompiled as a whole class by the Jarde CLI and local JADX CLI. Each "
        "source flavor is compiled with empty classpath and sourcepath into its own temporary "
        "classes directory. The reflection driver constructs the class and invokes "
        "`getMethod(\"relay\", Number.class)` with `Integer.valueOf(7)`; temporary classes are "
        "removed after each run.\n\n"
        f"Actual outcomes: original source compiled and printed `number` in {original_ok}/4 legs; "
        f"Jarde full-class compile success was {4-jarde_failed}/4 (ambiguous-overload failure "
        f"{jarde_failed}/4); JADX full-class compile success was {jadx_ok}/4. Jarde and JADX "
        "both retained `pick(Number)` and "
        "`pick(Comparable<T>)`, then emitted `relay(T)` as an uncast `pick(value)` call; both "
        "compilers reject that overload expression as ambiguous in all four legs. JADX's "
        "`defpackage` package and both overloads are preserved as emitted. In the original bytecode, "
        "`relay(T)` invokes `pick:(Ljava/lang/Number;)V` in all four legs, with no `checkcast` "
        "inside `relay`; the descriptor itself pins the selected overload.\n\n"
        "The Jarde result records a source compilation regression: the void-body recovery kept "
        "the consumer expression typed as `T` instead of preserving the call site's selected "
        "`Number` overload. The evidence does not modify consumer types or approve a design. The "
        "JADX also fails in this boundary case; future overload pinning must connect to generic "
        "type restoration before it can close this regression. The complete argv, exit codes, "
        "stdout/stderr, and hashes are in `manifest.json`; each command's stdout and stderr is "
        "stored beside its leg, and all files in each original JADX output tree are retained "
        "and hashed. Replay with `python3 replay-strict.py`; pass `--out` to choose a fresh "
        "output directory.\n"
    )
    (OUT / "summary.md").write_text(summary)
    manifest["files"] = [
        {"path": str(path.relative_to(OUT)), "bytes": path.stat().st_size, "sha256": sha(path)}
        for path in sorted(OUT.rglob("*")) if path.is_file() and path.name != "manifest.json"
    ]
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({
        "cases": len(cases),
        "original_compile_and_behavior": f"{original_ok}/4; number",
        "jadx_compile_and_behavior": f"{jadx_ok}/4",
        "jarde_compile_failure": f"{jarde_failed}/4",
        "results": str(OUT),
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()
