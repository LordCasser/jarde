#!/usr/bin/env python3
"""Collect guarded baseline full-class observations for five conditional-switch boundaries."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
import zipfile


ROOT = Path("/Users/lordcasser/workspace/projects/jarde").resolve()
CHANGE = "recover-proved-conditional-switch-fallthrough"
RESULTS = ROOT / "openspec/changes" / CHANGE / "results"
FIXTURE = RESULTS / "private-conditional-switch-boundaries-luna-v2"
SOURCE = FIXTURE / "ConditionalSwitchBoundaries.java"
RUNNER = FIXTURE / "ConditionalSwitchBoundariesRunner.java"
SOURCE_SHA256 = "72a8b7716d0867bf6c543baab64e24a2e8a1c3a4d6f3f64249c45d8c8784e636"
RUNNER_SHA256 = "de1955dc08a09068625035fac104b255ce48699aeb6270cae0f4f372ce26e75f"
GUARD_PATH = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
GUARD_SHA256 = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX_PATH = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
TYPED_RESULTS = ROOT / "openspec/changes/recover-proved-local-source-types/results"
TYPED_CLI = Path("/private/tmp/jarde-proved-local-source-types-cli-v1")
TYPED_CLI_SHA256 = "e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403"
TYPED_METADATA = TYPED_RESULTS / "candidate-cli-typed-root-v1.json"
TYPED_METADATA_SHA256 = "6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943"
TYPED_BUILD = TYPED_RESULTS / "validation-build-root-v2/execution.json"
TYPED_BUILD_SHA256 = "18a9750cdeb0d8bc45786f5d0766cbb256f1c8a87b1daf350965e910535a6413"
TYPED_SOURCE_BASE = "5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02"
TYPED_UNCOMMITTED_KEY = "uncommitted_local_source_types_product"
TYPED_SCHEMA = "recover-proved-local-source-types-candidate-cli-root-v1"
TYPED_BUILD_SCHEMA = "recover-proved-local-source-types-validation-build-root-v1"
TYPED_RUNNER_SHA256 = "75e3b3494bf6a2f3179fc1d61e01865382cbdcb4bcefc527f9bc97f012687b87"
TYPED_RUNNER = TYPED_RESULTS / "run-validation-build-root-v2.py"
TYPED_GUARD_ABSOLUTE = "/Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
TYPED_BUILD_SHA_PINS = {"metadata": TYPED_METADATA_SHA256, "execution": TYPED_BUILD_SHA256,
                        "cli": TYPED_CLI_SHA256, "runner": TYPED_RUNNER_SHA256}
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
EXPECTED_CLASSES = {"ConditionalSwitchBoundaries.class", "ConditionalSwitchBoundariesRunner.class"}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical(path: Path) -> Path:
    return path.expanduser().resolve(strict=True)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def live_pins(md: dict) -> dict:
    result = {}
    for group, key in (("candidate_sources", "candidate_sources"),
                       ("test_sources", "test_sources"), ("canonical_files", "canonical_files")):
        expected = md.get(key)
        require(isinstance(expected, dict) and expected, f"metadata {key} pins are missing")
        live = {}
        for relative, digest in sorted(expected.items()):
            path = ROOT / relative
            require(path.is_file(), f"pinned input is missing: {relative}")
            actual = file_sha(path)
            require(actual == digest, f"pinned input SHA differs: {relative}")
            live[relative] = actual
        result[group] = live
    return result


def bind_frozen_inputs() -> tuple[dict, dict, dict]:
    for path, expected in ((TYPED_CLI, TYPED_CLI_SHA256), (TYPED_METADATA, TYPED_METADATA_SHA256),
                           (TYPED_BUILD, TYPED_BUILD_SHA256), (TYPED_RUNNER, TYPED_RUNNER_SHA256),
                           (GUARD_PATH, GUARD_SHA256)):
        require(path.is_file() and file_sha(path) == expected, f"typed baseline frozen input SHA mismatch: {path}")
    cli = canonical(TYPED_CLI)
    md = read_json(TYPED_METADATA)
    execution = read_json(TYPED_BUILD)
    require(md.get("schema") == TYPED_SCHEMA and execution.get("schema") == TYPED_BUILD_SCHEMA,
            "typed frozen metadata/build schema mismatch")
    require(md.get("cli_path") == str(TYPED_CLI) and md.get("cli_sha256") == TYPED_CLI_SHA256,
            "typed metadata does not bind frozen CLI")
    require(stat.S_IMODE(cli.stat().st_mode) == 0o555 and md.get("cli_mode") == "0o555",
            "typed frozen CLI mode mismatch")
    require(md.get("metadata_path") == str(TYPED_METADATA) and
            md.get("build_result_sha256") == TYPED_BUILD_SHA256,
            "typed metadata path/build SHA binding mismatch")
    require(md.get("source_commit_base") == TYPED_SOURCE_BASE and
            md.get(TYPED_UNCOMMITTED_KEY) is True, "typed metadata source base/product marker mismatch")
    require(execution.get("status") == "validation-passed-cli-frozen" and
            execution.get("source_commit_base_expected") == TYPED_SOURCE_BASE and
            execution.get(TYPED_UNCOMMITTED_KEY) is True, "typed validation execution identity mismatch")
    require(execution.get("validation_runner") == {"path": str(TYPED_RUNNER), "sha256": TYPED_RUNNER_SHA256},
            "typed validation runner identity mismatch")
    guard_identity = {"path": str(GUARD_PATH.resolve()), "sha256": GUARD_SHA256}
    require(execution.get("guarded_runner_template") == guard_identity,
            "typed build does not bind absolute pinned v9 guard path/SHA")
    freeze = execution.get("freeze", {})
    for key in ("cli_path", "cli_sha256", "cli_mode", "metadata_path", "source_commit_base",
                TYPED_UNCOMMITTED_KEY, "validation_runner", "guarded_runner_template"):
        require(freeze.get(key) == md.get(key) if key in md else freeze.get(key) == execution.get(key),
                f"typed freeze field differs: {key}")
    require(md.get("validation_runner") == execution.get("validation_runner") and
            md.get("guarded_runner_template") == execution.get("guarded_runner_template"),
            "typed metadata and build runner/guard records differ")
    pins = live_pins(md)
    preflight = execution.get("preflight", {})
    require(preflight.get("source_pins_before") == pins and preflight.get("source_pins_after") == pins,
            "typed build source pins do not equal current live inputs")
    require(md.get("product_paths") == sorted(pins["candidate_sources"]) and
            md.get("test_paths") == sorted(pins["test_sources"]) and
            md.get("canonical_paths") == sorted(pins["canonical_files"]),
            "typed frozen path sets differ from pinned maps")
    build_pins = {"metadata": file_sha(TYPED_METADATA), "execution": file_sha(TYPED_BUILD),
                  "cli": file_sha(cli), "runner": file_sha(TYPED_RUNNER), "guard": file_sha(GUARD_PATH)}
    require(build_pins == {**TYPED_BUILD_SHA_PINS, "guard": GUARD_SHA256}, "typed baseline pin snapshot mismatch")
    return md, execution, {"cli": cli, "source_pins": pins, "sha_pins": build_pins}


def load_tools(out: Path) -> tuple[dict, dict, dict]:
    require(file_sha(GUARD_PATH) == GUARD_SHA256, "v9 guard source SHA mismatch")
    require(file_sha(JDK_MANIFEST) == JDK_MANIFEST_SHA256, "JDK controls manifest SHA mismatch")
    manifest = read_json(JDK_MANIFEST)
    legs = {}
    for name in ("javac8", "javac23"):
        row = next((item for item in manifest.get("legs", []) if item.get("leg") == name), None)
        require(row is not None, f"manifest lacks {name}")
        home = canonical(Path(row.get("jdk_tools", {}).get("java", {}).get("path", "")).parent.parent)
        tools = {}
        tool_hashes = {}
        for tool in ("java", "javac", "javap"):
            record = row.get("jdk_tools", {}).get(tool, {})
            tool_path = canonical(Path(record.get("path", "")))
            expected_sha = record.get("sha256")
            require(isinstance(expected_sha, str) and re.fullmatch(r"[0-9a-f]{64}", expected_sha) and
                    file_sha(tool_path) == expected_sha, f"{name} {tool} manifest/live SHA mismatch")
            require(tool_path.parent == home / "bin", f"{name} {tool} is outside supplied JAVA_HOME")
            tools[tool] = str(tool_path)
            tool_hashes[tool] = expected_sha
        legs[name] = {"java_home": str(home), "tools": tools, "tool_sha256": tool_hashes}

    jadx = canonical(JADX_PATH)
    require(jadx == JADX_PATH.resolve(strict=True) and file_sha(jadx) == JADX_SHA256,
            "JADX fixed path/SHA mismatch")
    require(file_sha(SOURCE) == SOURCE_SHA256 and file_sha(RUNNER) == RUNNER_SHA256,
            "boundary source or Runner SHA changed")
    require(file_sha(GUARD_PATH) == GUARD_SHA256, "guard changed while preparing tools")
    spec = importlib.util.spec_from_file_location("conditional_boundary_guard", GUARD_PATH)
    require(spec is not None and spec.loader is not None, "cannot load pinned v9 guard module")
    module = importlib.util.module_from_spec(spec)
    previous_dont_write_bytecode = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec.loader.exec_module(module)
    finally:
        sys.dont_write_bytecode = previous_dont_write_bytecode
    module.ROOT = ROOT
    module.OUT = out / "guard-streams"
    module.OUT.mkdir()
    module.command_stream = lambda path: {
        "path": str(path), "bytes": path.stat().st_size, "sha256": file_sha(path)
    }
    guard_globals = module.run_command.__globals__
    guard_globals["ROOT"] = ROOT
    guard_globals["OUT"] = module.OUT
    guard_globals["command_stream"] = module.command_stream
    return legs, {"path": str(jadx), "sha256": JADX_SHA256, "version": JADX_VERSION}, module


def exact_class_set(directory: Path, package: str | None = None) -> list[str]:
    actual = {p.relative_to(directory).as_posix() for p in directory.rglob("*.class")}
    prefix = package.replace(".", "/") + "/" if package else ""
    expected = {prefix + name for name in EXPECTED_CLASSES}
    require(actual == expected, f"complete class set differs: {sorted(actual)}")
    return sorted(actual)


def adapt_runner(package: str | None, destination: Path) -> bytes:
    raw = RUNNER.read_bytes()
    if package:
        require(not re.search(rb"(?m)^\s*package\s+", raw), "original Runner unexpectedly has a package")
        raw = f"package {package};\n".encode() + raw
    destination.write_bytes(raw)
    return raw


def package_of(text: str) -> str | None:
    match = re.search(r"(?m)^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;", text)
    return match.group(1) if match else None


def report_method_key(row: dict) -> tuple:
    item = row["item"]
    identity = item["identity"]
    owner = identity["owner"]
    name = bytes(identity["name"]).decode("ascii")
    descriptor = bytes(identity["descriptor"]).decode("ascii")
    require(identity["name"] == item["name"]["raw"] and
            identity["descriptor"] == item["descriptor"]["raw"],
            "report method identity differs from its raw declaration")
    ordinal = item.get("index")
    require(type(ordinal) is int and ordinal >= 0, "report method has no physical member ordinal")
    digest = owner.get("class_bytes", {}).get("digest")
    snapshot = owner.get("location", {}).get("snapshot")
    require(digest and snapshot, "report method has no physical owner digest/snapshot")
    return (name, descriptor, digest, snapshot, ordinal, row.get("declaration"))


def report_source_maps(methods: list) -> list:
    rows = []
    for method in methods:
        key = report_method_key(method)
        report = method.get("outcome", {}).get("report")
        require(isinstance(report, dict) and isinstance(report.get("source_map"), dict),
                "method report is missing its source_map")
        source_map = report["source_map"]
        segments = source_map.get("segments")
        require(isinstance(segments, list), "source map segments schema is missing")
        for segment in segments:
            origin = segment.get("origin", {})
            primary = origin.get("primary")
            derived = origin.get("derived")
            require(isinstance(primary, dict) and isinstance(derived, list),
                    "source map primary/derived origin schema mismatch")
            for source in (primary, *derived):
                source_method = source.get("method", {})
                source_owner = source_method.get("owner", {})
                source_identity = (
                    bytes(source_method.get("name", [])).decode("ascii"),
                    bytes(source_method.get("descriptor", [])).decode("ascii"),
                    source_owner.get("class_bytes", {}).get("digest"),
                    source_owner.get("location", {}).get("snapshot"),
                )
                require(source_identity == key[:4] and type(source.get("bci")) is int,
                        "source-map origin method/owner/BCI differs from enclosing physical method")
        # Preserve the full source-map payload, including each origin's actual `method` identity.
        rows.append((key, source_map))
    return sorted(rows, key=lambda row: row[0])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path,
                        help="new exclusive output directory; must not exist")
    args = parser.parse_args()
    out = args.out.expanduser().resolve()
    require(not out.exists(), f"refusing to overwrite output: {out}")
    require(out != Path("/private/tmp/jarde-conditional-boundary-preflight-luna-v1").resolve(),
            "v2 output must not reuse the v1 draft directory")
    require(shutil.disk_usage(ROOT).free >= FREE_LIMIT, "5 GiB free-space guard before output creation")
    target = ROOT / "target"
    require(not target.exists() or
            sum(p.stat().st_size for p in target.rglob("*") if p.is_file()) <= TARGET_LIMIT,
            "1 GiB target-size guard before output creation")
    require(file_sha(SOURCE) == SOURCE_SHA256 and file_sha(RUNNER) == RUNNER_SHA256,
            "frozen boundary source/Runner SHA mismatch")
    md, build, frozen = bind_frozen_inputs()
    out.mkdir(parents=True)
    (out / "cases").mkdir()
    (out / "streams").mkdir()
    try:
        legs, jadx, guard = load_tools(out)
    except BaseException as error:
        failed = {"schema": CHANGE + "-boundary-baseline-observation-v2",
                  "status": "failed-preflight-incomplete",
                  "scope": "unpatched conditional-switch product baseline; no command was started",
                  "failure": f"{type(error).__name__}: {error}",
                  "baseline": {"metadata_path": str(TYPED_METADATA),
                               "metadata_sha256": TYPED_METADATA_SHA256,
                               "build_path": str(TYPED_BUILD), "build_sha256": TYPED_BUILD_SHA256,
                               "source_base": TYPED_SOURCE_BASE},
                  "fixture": {"source_sha256": SOURCE_SHA256, "runner_sha256": RUNNER_SHA256},
                  "commands": []}
        write_json(out / "execution.json", failed)
        inventory = [{"path": path.relative_to(out).as_posix(), "bytes": path.stat().st_size,
                      "sha256": file_sha(path)} for path in sorted(out.rglob("*")) if path.is_file()]
        write_json(out / "file-inventory.json", inventory)
        return 1
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    env.update({"LC_ALL": "C", "TZ": "UTC"})
    commands: list[dict] = []
    failures: list[str] = []
    oracles: dict[str, tuple[int, bytes, bytes]] = {}
    observations: dict[str, dict] = {}
    command_index = 1
    manifest = {
        "schema": CHANGE + "-boundary-baseline-observation-v2",
        "status": "running",
        "scope": "unpatched conditional-switch product baseline; complete-class runtime/source observation only",
        "product_acceptance": False,
        "collector": {"path": str(Path(__file__).resolve()), "sha256": file_sha(Path(__file__).resolve())},
        "fixture": {"source": {"path": str(SOURCE), "sha256": SOURCE_SHA256},
                    "runner": {"path": str(RUNNER), "sha256": RUNNER_SHA256},
                    "methods": ["partialBreak", "innerLoopBreak", "innerSwitchBreak", "terminalCase",
                                "caughtExceptionThenFallthrough"]},
        "baseline_product": {"cli_path": str(frozen["cli"]), "cli_sha256": TYPED_CLI_SHA256,
                             "metadata_path": str(TYPED_METADATA), "metadata_sha256": TYPED_METADATA_SHA256,
                             "build_path": str(TYPED_BUILD), "build_sha256": TYPED_BUILD_SHA256,
                             "source_base": TYPED_SOURCE_BASE,
                             "metadata_schema": md["schema"], "build_schema": build["schema"],
                             "uncommitted_local_source_types_product": True,
                             "source_pins_before": frozen["source_pins"]},
        "guard": {"path": TYPED_GUARD_ABSOLUTE, "sha256": GUARD_SHA256,
                  "minimum_free_bytes": FREE_LIMIT, "maximum_target_bytes": TARGET_LIMIT},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
        "jadx": jadx, "jdk": legs, "commands": commands, "legs": observations,
        "comparisons": {}, "failures": failures,
    }

    def checkpoint() -> None:
        manifest["commands"] = commands
        manifest["legs"] = observations
        manifest["failures"] = failures
        write_json(out / "execution.json", manifest)

    def run(label: str, argv: list[str], leg: str | None = None) -> dict:
        nonlocal command_index
        cmd_env = dict(env)
        if leg:
            home = legs[leg]["java_home"]
            cmd_env["JAVA_HOME"] = home
            cmd_env["PATH"] = str(Path(home) / "bin") + os.pathsep + cmd_env.get("PATH", "")
        row = guard.run_command(command_index, [str(value) for value in argv], cmd_env)
        row["label"] = label
        row["environment_removed"] = list(STRIPPED_ENV)
        commands.append(row)
        command_index += 1
        checkpoint()
        return row

    write_json(out / "execution.json", manifest)
    try:
        version = run("jadx-version", [jadx["path"], "--version"], "javac23")
        version_bytes = (out / "guard-streams" / f"{version['index']}.stdout.raw").read_bytes().strip()
        manifest["jadx"]["version_command"] = version
        manifest["jadx"]["version_observed"] = version_bytes.decode("utf-8", errors="replace")
        if version["exit_code"] != 0 or version.get("guard_stop") or version_bytes != JADX_VERSION.encode():
            failures.append("pinned JADX version command failed or did not report 1.5.6")
            raise RuntimeError("JADX version preflight failed")
        checkpoint()

        for leg in ("javac8", "javac23"):
            tools = legs[leg]["tools"]
            leg_root = out / "cases" / leg
            original = leg_root / "original"
            classes = original / "classes"
            empty = original / "empty"
            classes.mkdir(parents=True)
            empty.mkdir()
            source_copy = original / SOURCE.name
            runner_copy = original / RUNNER.name
            shutil.copyfile(SOURCE, source_copy)
            shutil.copyfile(RUNNER, runner_copy)
            compile_row = run(f"{leg}-original-compile", [tools["javac"], "-encoding", "UTF-8",
                "-source", "8", "-target", "8", "-g:none", "-Xlint:-options", "-proc:none",
                "-classpath", str(empty), "-sourcepath", str(empty), "-d", str(classes),
                str(source_copy), str(runner_copy)], leg)
            original_row = {"compile": compile_row}
            observations[leg] = {"original": original_row}
            if compile_row["exit_code"] != 0 or compile_row.get("guard_stop"):
                failures.append(f"{leg}: original compile failed or was guard-stopped")
                checkpoint()
                continue

            class_set = exact_class_set(classes)
            target_class = classes / "ConditionalSwitchBoundaries.class"
            runtime = run(f"{leg}-original-runtime", [tools["java"], "-Xverify:all", "-cp",
                str(classes), "ConditionalSwitchBoundariesRunner"], leg)
            runtime_out = out / "guard-streams" / f"{runtime['index']}.stdout.raw"
            runtime_err = out / "guard-streams" / f"{runtime['index']}.stderr.raw"
            oracle = (runtime["exit_code"], runtime_out.read_bytes(), runtime_err.read_bytes())
            oracles[leg] = oracle
            shutil.copyfile(runtime_out, out / "streams" / f"{leg}-original.stdout.raw")
            shutil.copyfile(runtime_err, out / "streams" / f"{leg}-original.stderr.raw")
            javap = run(f"{leg}-original-javap", [tools["javap"], "-p", "-c", "-s", str(target_class)], leg)
            original_row.update({"class_files": class_set, "classes_sha256":
                                 {name: file_sha(classes / name) for name in class_set},
                                 "runtime": runtime, "javap": javap})
            if runtime["exit_code"] != 0 or runtime.get("guard_stop") or oracle[2]:
                failures.append(f"{leg}: original runtime failed, guard-stopped, or wrote stderr")
                checkpoint()
                continue
            if javap["exit_code"] != 0 or javap.get("guard_stop"):
                failures.append(f"{leg}: original javap failed or was guard-stopped")
                checkpoint()
                continue

            # JADX is an independent leg: preserve its failure, then continue to candidate profiles.
            jar = leg_root / "input.jar"
            with zipfile.ZipFile(jar, "w", compression=zipfile.ZIP_STORED) as archive:
                info = zipfile.ZipInfo("ConditionalSwitchBoundaries.class", date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_STORED
                archive.writestr(info, target_class.read_bytes())
            jdest = leg_root / "jadx-output"
            jadx_row = run(f"{leg}-jadx-full-class", [jadx["path"], "--no-res", "--config", "none",
                "--threads-count", "1", "-d", str(jdest), str(jar)], leg)
            jrow = {"decompile": jadx_row, "compile": None, "runtime": None}
            observations[leg]["jadx"] = jrow
            if jadx_row["exit_code"] != 0 or jadx_row.get("guard_stop"):
                failures.append(f"{leg}: JADX decompile failed or was guard-stopped")
            else:
                generated_rows = list(jdest.rglob("ConditionalSwitchBoundaries.java"))
                if len(generated_rows) != 1:
                    failures.append(f"{leg}: JADX did not produce exactly one complete target source")
                else:
                    jcase = leg_root / "jadx-complete"
                    jclasses = jcase / "classes"
                    jempty = jcase / "empty"
                    jclasses.mkdir(parents=True)
                    jempty.mkdir()
                    jsource = jcase / "ConditionalSwitchBoundaries.java"
                    jtext = generated_rows[0].read_text(encoding="utf-8")
                    jsource.write_text(jtext, encoding="utf-8")
                    jrunner = jcase / "ConditionalSwitchBoundariesRunner.java"
                    jrunner_bytes = adapt_runner(package_of(jtext), jrunner)
                    jcompile = run(f"{leg}-jadx-compile", [tools["javac"], "-encoding", "UTF-8",
                        "-source", "8", "-target", "8", "-g:none", "-Xlint:-options", "-proc:none",
                        "-classpath", str(jempty), "-sourcepath", str(jempty), "-d", str(jclasses),
                        str(jsource), str(jrunner)], leg)
                    jrow.update({"source_path": str(jsource), "source_sha256": file_sha(jsource),
                                 "runner_sha256": sha256(jrunner_bytes),
                                 "runner_bytes_identical": jrunner_bytes == RUNNER.read_bytes(),
                                 "runner_package_adaptation_only": package_of(jtext) is not None,
                                 "compile": jcompile})
                    if jcompile["exit_code"] != 0 or jcompile.get("guard_stop"):
                        failures.append(f"{leg}: JADX complete-class compile failed or was guard-stopped")
                    else:
                        jclass_set = exact_class_set(jclasses, package_of(jtext))
                        jruntime = run(f"{leg}-jadx-runtime", [tools["java"], "-Xverify:all", "-cp",
                            str(jclasses), (package_of(jtext) + "." if package_of(jtext) else "") +
                            "ConditionalSwitchBoundariesRunner"], leg)
                        jout = (out / "guard-streams" / f"{jruntime['index']}.stdout.raw").read_bytes()
                        jerr = (out / "guard-streams" / f"{jruntime['index']}.stderr.raw").read_bytes()
                        jmatch = (jruntime["exit_code"] == 0 and not jruntime.get("guard_stop") and
                                  (jruntime["exit_code"], jout, jerr) == oracle)
                        jrow.update({"class_files": jclass_set, "runtime": jruntime,
                                     "runtime_matches_original": jmatch})
                        if not jmatch:
                            failures.append(f"{leg}: JADX runtime differs from original oracle")

            for profile in ("default", "all"):
                case = leg_root / f"jarde-{profile}"
                classes_dir = case / "classes"
                empty = case / "empty"
                classes_dir.mkdir(parents=True)
                empty.mkdir()
                argv = [str(frozen["cli"]), "class-source", "--input", str(target_class),
                        "--class", "ConditionalSwitchBoundaries", "--policy", "single-class",
                        "--release", "8", "--format", "json"]
                if profile == "all":
                    argv.extend(["--evidence", "all"])
                render = run(f"{leg}-candidate-{profile}-render", argv, leg)
                doc_path = case / "report.json"
                report_raw = (out / "guard-streams" / f"{render['index']}.stdout.raw").read_bytes()
                doc_path.write_bytes(report_raw)
                row = {"render": render, "report_path": str(doc_path), "report_sha256": sha256(report_raw),
                       "compile": None, "runtime": None}
                observations[leg].setdefault("candidate", {})[profile] = row
                if render["exit_code"] != 0 or render.get("guard_stop"):
                    failures.append(f"{leg}/{profile}: candidate render failed or was guard-stopped")
                    checkpoint()
                    continue
                try:
                    doc = json.loads(report_raw)
                    generated_text = doc["text"]
                    methods = doc["methods"]
                    require(isinstance(generated_text, str) and generated_text,
                            "candidate report has no full source text")
                    require(isinstance(methods, list) and methods,
                            "candidate report has no complete method report list")
                except Exception as error:
                    failures.append(f"{leg}/{profile}: report schema rejected ({type(error).__name__}: {error})")
                    checkpoint()
                    continue
                map_signature = None
                map_hash = None
                try:
                    map_signature = report_source_maps(methods)
                    map_hash = sha256(json.dumps(map_signature, sort_keys=True, separators=(",", ":")).encode())
                except Exception as error:
                    failures.append(f"{leg}/{profile}: method-map schema rejected ({type(error).__name__}: {error})")
                generated = case / "ConditionalSwitchBoundaries.java"
                generated.write_text(generated_text, encoding="utf-8")
                package = package_of(generated_text)
                gen_runner = case / "ConditionalSwitchBoundariesRunner.java"
                runner_bytes = adapt_runner(package, gen_runner)
                compiled = run(f"{leg}-candidate-{profile}-compile", [tools["javac"], "-encoding", "UTF-8",
                    "-source", "8", "-target", "8", "-g:none", "-Xlint:-options", "-proc:none",
                    "-classpath", str(empty), "-sourcepath", str(empty), "-d", str(classes_dir),
                    str(generated), str(gen_runner)], leg)
                row.update({"source_path": str(generated), "source_sha256": file_sha(generated),
                            "runner_path": str(gen_runner), "runner_sha256": sha256(runner_bytes),
                            "runner_bytes_identical": runner_bytes == RUNNER.read_bytes(),
                            "runner_package_adaptation_only": package is not None,
                            "method_report_count": len(methods),
                            "method_source_maps_sha256": map_hash,
                            "compile": compiled})
                if map_signature is not None:
                    row["method_identities"] = [list(report_method_key(method)) for method in methods]
                if compiled["exit_code"] != 0 or compiled.get("guard_stop"):
                    failures.append(f"{leg}/{profile}: candidate complete-class compile failed or was guard-stopped")
                    checkpoint()
                    continue
                class_set = exact_class_set(classes_dir, package)
                runtime_row = run(f"{leg}-candidate-{profile}-runtime", [tools["java"], "-Xverify:all",
                    "-cp", str(classes_dir), (package + "." if package else "") +
                    "ConditionalSwitchBoundariesRunner"], leg)
                stdout = (out / "guard-streams" / f"{runtime_row['index']}.stdout.raw").read_bytes()
                stderr = (out / "guard-streams" / f"{runtime_row['index']}.stderr.raw").read_bytes()
                same = (runtime_row["exit_code"] == 0 and not runtime_row.get("guard_stop") and
                        (runtime_row["exit_code"], stdout, stderr) == oracle)
                row.update({"class_files": class_set, "runtime": runtime_row,
                            "runtime_matches_original": same})
                if not same:
                    failures.append(f"{leg}/{profile}: candidate runtime differs from original oracle")
                checkpoint()

            pair = observations[leg].get("candidate", {})
            if set(pair) == {"default", "all"} and all("source_path" in pair[key] for key in pair):
                same_text = Path(pair["default"]["source_path"]).read_bytes() == Path(pair["all"]["source_path"]).read_bytes()
                same_maps = (pair["default"].get("method_identities") == pair["all"].get("method_identities") and
                             pair["default"].get("method_source_maps_sha256") ==
                             pair["all"].get("method_source_maps_sha256"))
                manifest["comparisons"][leg] = {"default_all_text_equal": same_text,
                                                "default_all_method_source_maps_equal": same_maps}
                if not same_text or not same_maps:
                    failures.append(f"{leg}: default/all whole-class source or method maps differ")
                checkpoint()

        if set(oracles) == {"javac8", "javac23"}:
            cross_equal = oracles["javac8"] == oracles["javac23"]
            manifest["comparisons"]["original_cross_jdk_raw_equal"] = cross_equal
            if not cross_equal:
                failures.append("original runtime raw differs between JDK legs")

        expected_labels = {"jadx-version"}
        for leg in ("javac8", "javac23"):
            expected_labels |= {f"{leg}-original-{suffix}" for suffix in ("compile", "runtime", "javap")}
            expected_labels |= {f"{leg}-jadx-{suffix}" for suffix in ("full-class", "compile", "runtime")}
            for profile in ("default", "all"):
                expected_labels |= {f"{leg}-candidate-{profile}-{suffix}" for suffix in ("render", "compile", "runtime")}
        actual_labels = {row["label"] for row in commands}
        if actual_labels != expected_labels:
            failures.append(f"command matrix incomplete: missing={sorted(expected_labels-actual_labels)} extra={sorted(actual_labels-expected_labels)}")

        require(file_sha(SOURCE) == SOURCE_SHA256 and file_sha(RUNNER) == RUNNER_SHA256,
                "boundary source/Runner SHA changed during replay")
        live_after = live_pins(md)
        require(live_after == frozen["source_pins"], "typed product/test/canonical source pins changed during replay")
        ending_pins = {"cli": file_sha(TYPED_CLI), "metadata": file_sha(TYPED_METADATA),
                       "build": file_sha(TYPED_BUILD), "runner": file_sha(TYPED_RUNNER),
                       "guard": file_sha(GUARD_PATH), "jadx": file_sha(jadx["path"]),
                       "jdk_manifest": file_sha(JDK_MANIFEST),
                       "jdk_tools": {leg: {tool: file_sha(path) for tool, path in row["tools"].items()}
                                     for leg, row in legs.items()},
                       "source": file_sha(SOURCE), "runner_fixture": file_sha(RUNNER)}
        if ending_pins != {"cli": TYPED_CLI_SHA256, "metadata": TYPED_METADATA_SHA256,
                           "build": TYPED_BUILD_SHA256, "runner": TYPED_RUNNER_SHA256,
                           "guard": GUARD_SHA256, "jadx": JADX_SHA256,
                           "jdk_manifest": JDK_MANIFEST_SHA256,
                           "jdk_tools": {leg: row["tool_sha256"] for leg, row in legs.items()},
                           "source": SOURCE_SHA256, "runner_fixture": RUNNER_SHA256}:
            failures.append("one or more frozen baseline/input/tool pins changed during replay")
        manifest["baseline_product"]["source_pins_after"] = live_after
        manifest["ending_sha_pins"] = ending_pins
    except KeyboardInterrupt:
        failures.append("collector interrupted; evidence remains incomplete")
        manifest["status"] = "interrupted-incomplete"
    except BaseException as error:
        failures.append(f"collector stopped: {type(error).__name__}: {error}")
        manifest["status"] = "failed-incomplete"
    else:
        manifest["status"] = "complete-class-boundary-baseline-observed" if not failures else "observations-recorded-with-failures"
    manifest["finished_at"] = dt.datetime.now(dt.timezone.utc).isoformat()
    checkpoint()
    inventory = []
    for path in sorted(out.rglob("*")):
        if path.is_file() and path.name != "file-inventory.json":
            inventory.append({"path": path.relative_to(out).as_posix(), "bytes": path.stat().st_size,
                              "sha256": file_sha(path)})
    write_json(out / "file-inventory.json", inventory)
    print(json.dumps({"status": manifest["status"], "output": str(out),
                      "commands": len(commands), "failures": failures}, ensure_ascii=False, indent=2))
    return 0 if manifest["status"] == "complete-class-boundary-baseline-observed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
