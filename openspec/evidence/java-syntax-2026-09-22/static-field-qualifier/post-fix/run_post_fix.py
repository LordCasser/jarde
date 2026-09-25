"""Replay whole-class behavior against the rebuilt Jarde CLI after the qualifier fix."""

from pathlib import Path
import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
BASE = HERE.parent
FIXTURE = ROOT / "tests/fixtures/p3-popped-static-qualifier"
CLI_DEFAULT = ROOT / "target/debug/jarde-cli"
EVIDENCE = HERE


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, label):
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (EVIDENCE / f"{label}.stdout").write_text(result.stdout)
    (EVIDENCE / f"{label}.stderr").write_text(result.stderr)
    (EVIDENCE / f"{label}.status").write_text(f"{result.returncode}\n")
    return result


def require_ok(result, label):
    assert result.returncode == 0, f"{label} exited {result.returncode}: {result.stderr}"


def class_source(cli, class_file, class_name, label):
    result = run(
        [
            str(cli),
            "class-source",
            "--input",
            str(class_file),
            "--class",
            class_name,
            "--policy",
            "single-class",
            "--release",
            "8",
            "--format",
            "text",
        ],
        label,
    )
    require_ok(result, label)
    source = EVIDENCE / f"{label}.java.txt"
    source.write_text(result.stdout)
    return source


def decompile(class_file, class_name, work, label):
    output = work / f"{label}-jadx"
    result = run(["jadx", "--no-res", "-d", str(output), str(class_file)], f"{label}-jadx")
    require_ok(result, f"{label}-jadx")
    source = next(output.rglob(f"{class_name}.java"))
    saved = EVIDENCE / f"{label}-jadx.java.txt"
    shutil.copy2(source, saved)
    return source


def compile_and_run(work, label, class_name, class_source_path, helper_sources, runner_source):
    source_dir = work / f"{label}-sources"
    source_dir.mkdir()
    probe = source_dir / f"{class_name}.java"
    shutil.copy2(class_source_path, probe)
    probe_text = probe.read_text()
    package_match = re.search(r"^package\s+([^;]+);", probe_text, re.MULTILINE)
    package_name = package_match.group(1) if package_match else None
    copied = [probe]
    for path in helper_sources:
        target = source_dir / path.name
        shutil.copy2(path, target)
        if package_name and not re.search(r"^package\s", target.read_text(), re.MULTILINE):
            target.write_text(f"package {package_name};\n\n" + target.read_text())
        copied.append(target)
    runner = source_dir / runner_source.name
    shutil.copy2(runner_source, runner)
    if package_name and not re.search(r"^package\s", runner.read_text(), re.MULTILINE):
        runner.write_text(f"package {package_name};\n\n" + runner.read_text())
    copied.append(runner)
    classes = work / f"{label}-classes"
    classes.mkdir()
    compiled = run(
        ["javac", "--release", "8", "-g:none", "-d", str(classes), *map(str, copied)],
        f"{label}-javac",
    )
    require_ok(compiled, f"{label}-javac")
    runner_class = f"{package_name}.{runner_source.stem}" if package_name else runner_source.stem
    executed = run(
        ["java", "-Xverify:all", "-cp", str(classes), runner_class],
        f"{label}-runtime",
    )
    require_ok(executed, f"{label}-runtime")
    return executed.stdout.splitlines()


def compare_class(cli, work, class_file, class_name, source_paths, runner, label, expected_hash=None):
    source_class, *helpers = source_paths
    if expected_hash is not None:
        assert sha256(class_file) == expected_hash, f"unexpected {class_name} fixture hash"
    source_run = compile_and_run(work, f"{label}-original", class_name, source_class, helpers, runner)
    source_classes = work / f"{label}-original-classes"
    compiled_class = source_classes / f"{class_name}.class"
    assert compiled_class.read_bytes() == class_file.read_bytes(), f"javac output differs for {class_name}"

    jarde_source = class_source(cli, class_file, class_name, f"{label}-jarde-cli")
    jarde_run = compile_and_run(work, f"{label}-jarde", class_name, jarde_source, helpers, runner)
    jadx_source = decompile(class_file, class_name, work, label)
    jadx_run = compile_and_run(work, f"{label}-jadx", class_name, jadx_source, helpers, runner)
    assert jarde_run == source_run, f"Jarde runtime differs for {class_name}: {jarde_run} != {source_run}"
    assert jadx_run == source_run, f"JADX runtime differs for {class_name}: {jadx_run} != {source_run}"
    return {
        "class_bytes": class_file.stat().st_size,
        "class_sha256": sha256(class_file),
        "javap_code_methods": run(["javap", "-c", "-p", str(class_file)], f"{label}-javap").stdout.count("    Code:"),
        "source": source_run,
        "jarde": jarde_run,
        "jadx": jadx_run,
        "all_match": True,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, default=CLI_DEFAULT)
    args = parser.parse_args()
    cli = args.cli.resolve()
    assert cli.is_file(), f"build the current CLI first: {cli}"
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="jarde-static-qualifier-post-fix-") as raw:
        work = Path(raw)

        main_class = FIXTURE / "v8/StaticQualifierProbe.class"
        main_probe = FIXTURE / "StaticQualifierProbe.java"
        main_runner = FIXTURE / "StaticQualifierRunner.java"
        main_result = compare_class(
            cli,
            work,
            main_class,
            "StaticQualifierProbe",
            [main_probe],
            main_runner,
            "fixture",
            "21cdace20a84accd06e52567c250b0359ef76e059df993cc6c00a9fe1d6b7266",
        )

        boundary_class = BASE / "boundary/QualifierBoundaryProbe.class"
        boundary_probe = BASE / "boundary/QualifierBoundaryProbe.java"
        boundary_runner = BASE / "boundary/QualifierBoundaryRunner.java"
        boundary_result = compare_class(
            cli,
            work,
            boundary_class,
            "QualifierBoundaryProbe",
            [boundary_probe],
            boundary_runner,
            "boundary",
        )

        interface_class = BASE / "boundary/InterfaceBoundaryProbe.class"
        interface_probe = BASE / "boundary/InterfaceBoundaryProbe.java"
        interface_owner = BASE / "boundary/InterfaceStaticOwner.java"
        interface_runner = BASE / "boundary/InterfaceBoundaryRunner.java"
        interface_result = compare_class(
            cli,
            work,
            interface_class,
            "InterfaceBoundaryProbe",
            [interface_probe, interface_owner],
            interface_runner,
            "interface",
        )

        owner_class = BASE / "boundary/OwnerBoundaryProbe.class"
        owner_probe = BASE / "boundary/OwnerBoundaryProbe.java"
        owner_helpers = [BASE / "boundary/Base.java", BASE / "boundary/Child.java"]
        owner_runner = BASE / "boundary/OwnerBoundaryRunner.java"
        owner_result = compare_class(
            cli,
            work,
            owner_class,
            "OwnerBoundaryProbe",
            [owner_probe, *owner_helpers],
            owner_runner,
            "owner",
        )

        mismatch_class = BASE / "non-invoke-owner/NonInvokeQualifierProbe.owner-other.class"
        mismatch_source = class_source(
            cli, mismatch_class, "NonInvokeQualifierProbe", "mismatch-jarde-cli"
        )
        assert "@bytecode" in mismatch_source.read_text(), "owner mismatch must stay a mapped refusal"
        assert ".ping()" not in mismatch_source.read_text(), "owner mismatch must not rebind to Child.ping"
        mismatch_helpers = [
            BASE / "non-invoke-owner/Base.java",
            BASE / "non-invoke-owner/Child.java",
            BASE / "non-invoke-owner/Other.java",
        ]
        mismatch_dir = work / "mismatch-sources"
        mismatch_dir.mkdir()
        mismatch_java = mismatch_dir / "NonInvokeQualifierProbe.java"
        shutil.copy2(mismatch_source, mismatch_java)
        copied_helpers = []
        for path in mismatch_helpers:
            target = mismatch_dir / path.name
            shutil.copy2(path, target)
            copied_helpers.append(target)
        mismatch_runner = mismatch_dir / "NonInvokeQualifierRunner.java"
        shutil.copy2(BASE / "non-invoke-owner/NonInvokeQualifierRunner.java", mismatch_runner)
        mismatch_classes = work / "mismatch-classes"
        mismatch_classes.mkdir()
        mismatch_compile = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(mismatch_classes),
                str(mismatch_java),
                *map(str, copied_helpers),
                str(mismatch_runner),
            ],
            "mismatch-jarde-javac",
        )
        mismatch_result = {
            "class_bytes": mismatch_class.stat().st_size,
            "class_sha256": sha256(mismatch_class),
            "jarde_compile_status": mismatch_compile.returncode,
            "refusal_source_contains_bytecode_quote": True,
            "rebound_to_child_ping": False,
        }

        versions = {}
        for name, command in [
            ("javac", ["javac", "-version"]),
            ("java", ["java", "-version"]),
            ("jadx", ["jadx", "--version"]),
        ]:
            result = subprocess.run(command, capture_output=True, text=True, timeout=30)
            versions[name] = (result.stdout + result.stderr).strip().splitlines()

        summary = {
            "cli_path": str(cli),
            "cli_sha256": sha256(cli),
            "versions": versions,
            "fixture": main_result,
            "boundary": boundary_result,
            "interface": interface_result,
            "owner": owner_result,
            "non_invoke_owner_mismatch": mismatch_result,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
