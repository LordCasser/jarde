"""Audit the static-initializer boundaries that are not part of the permanent fixture."""

from pathlib import Path
import json
import re
import shutil
import struct
import subprocess


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-09-22/static-initializers"
BOUNDARIES = EVIDENCE / "boundaries"
TMP = Path("/tmp/jarde-static-initializers-20260923-boundaries")
CLI = ROOT / "target/debug/jarde-cli"


def run(command, output, cwd=None):
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=20,
        )
        output.write_text(completed.stdout)
        return completed.returncode
    except subprocess.TimeoutExpired as error:
        output.write_text(f"timeout after 20s\n{error.stdout or ''}")
        return 124


def replace_once(data, old, new):
    positions = [index for index in range(len(data)) if data.startswith(old, index)]
    if len(positions) != 1:
        raise AssertionError(f"expected one {old.hex()} match, got {positions}")
    index = positions[0]
    return data[:index] + new + data[index + len(old) :]


def patch_empty(source, target):
    data = bytearray(source.read_bytes())
    pattern = re.compile(b"\x10\x07\xb3..\xb1", re.DOTALL)
    matches = list(pattern.finditer(data))
    positions = [match.start() for match in matches]
    if len(positions) != 1:
        raise AssertionError(f"expected one straight clinit, got {positions}")
    old = matches[0].group(0)
    assert len(old) == 6
    code_start = positions[0]
    code_length_offset = code_start - 4
    attribute_length_offset = code_start - 12
    assert struct.unpack_from(">I", data, code_length_offset)[0] == 6
    attribute_length = struct.unpack_from(">I", data, attribute_length_offset)[0]
    struct.pack_into(">I", data, code_length_offset, 1)
    struct.pack_into(">I", data, attribute_length_offset, attribute_length - 5)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data[:code_start] + b"\xb1" + data[code_start + 6 :])


def patch_early(source, target):
    data = bytearray(source.read_bytes())
    old = bytes.fromhex("b8001599000704b3000f1007b3000fb1")
    positions = [index for index in range(len(data)) if data.startswith(old, index)]
    if len(positions) != 1:
        raise AssertionError(f"expected one early-return clinit, got {positions}")
    code_start = positions[0]
    code_length_offset = code_start - 4
    attribute_length_offset = code_start - 12
    assert struct.unpack_from(">I", data, code_length_offset)[0] == len(old)
    # The source has an ifeq over the normal assignment. Insert a return after the true-arm write,
    # move the false target by one byte, and move the one same-frame entry with it.
    new = old[:3] + b"\x99\x00\x08" + old[6:10] + b"\xb1" + old[10:]
    assert len(new) == len(old) + 1
    tail = bytes(data[code_start + len(old) :])
    tail = replace_once(tail, b"\x00\x01\x0a", b"\x00\x01\x0b")
    attribute_length = struct.unpack_from(">I", data, attribute_length_offset)[0]
    struct.pack_into(">I", data, code_length_offset, len(new))
    struct.pack_into(">I", data, attribute_length_offset, attribute_length + 1)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data[:code_start] + new + tail)


def compile_runner(case, output_dir, runner_name, helper=False):
    inputs = [
        EVIDENCE / "runners" / f"{runner_name}.java",
    ]
    if helper:
        inputs.append(EVIDENCE / "sources" / "ThrowingHelper.java")
    return run(
        ["javac", "--release", "8", "-g:none", "-cp", str(output_dir), "-d", str(output_dir)]
        + [str(path) for path in inputs],
        case / f"{output_dir.name}-runner-javac.log",
    )


def cli(case, selected, class_name):
    directory = case / "jarde"
    directory.mkdir(parents=True, exist_ok=True)
    source = directory / f"{class_name}.java"
    report = directory / "report.txt"
    with source.open("w") as stdout, report.open("w") as stderr:
        result = subprocess.run(
            [
                str(CLI),
                "class-source",
                "--input",
                str(selected),
                "--class",
                class_name,
                "--policy",
                "single-class",
                "--release",
                "8",
                "--format",
                "text",
            ],
            stdout=stdout,
            stderr=stderr,
            timeout=20,
        )
    (directory / "status.txt").write_text(f"cli_rc={result.returncode}\n")
    return result.returncode


def recovered(case, source, class_name, runner_name, helper, args):
    directory = case / "jarde"
    classes = directory / "classes"
    classes.mkdir(parents=True, exist_ok=True)
    inputs = [source, EVIDENCE / "runners" / f"{runner_name}.java"]
    if helper:
        inputs.append(EVIDENCE / "sources" / "ThrowingHelper.java")
    rc = run(
        ["javac", "--release", "8", "-g:none", "-d", str(classes)]
        + [str(path) for path in inputs],
        directory / "javac.log",
    )
    with (directory / "status.txt").open("a") as status:
        status.write(f"javac_rc={rc}\n")
    if rc != 0:
        (directory / "run.log").write_text("not run: javac failed\n")
        return rc
    for arg in args or [""]:
        suffix = arg
        run_rc = run(
            ["java", "-Xverify:all", "-cp", str(classes), runner_name] + ([arg] if arg else []),
            directory / f"run{suffix}.log",
        )
        with (directory / "status.txt").open("a") as status:
            status.write(f"run{suffix}_rc={run_rc}\n")
    return rc


def jadx(case, selected, class_name, runner_name, helper, args):
    directory = case / "jadx"
    rc = run(["jadx", "--no-res", "-d", str(directory), str(selected)], case / "jadx.log")
    found = list((directory / "sources").rglob(f"{class_name}.java"))
    if rc != 0 or len(found) != 1:
        (case / "jadx-status.txt").write_text(f"jadx_rc={rc}\nsources={len(found)}\n")
        return rc
    raw = found[0].read_text()
    (case / "jadx-raw.java").write_text(raw)
    source = case / "jadx-source" / f"{class_name}.java"
    source.parent.mkdir(parents=True, exist_ok=True)
    source.write_text("\n".join(line for line in raw.splitlines() if line != "package defpackage;") + "\n")
    classes = case / "jadx-classes"
    classes.mkdir(parents=True, exist_ok=True)
    inputs = [source, EVIDENCE / "runners" / f"{runner_name}.java"]
    if helper:
        inputs.append(EVIDENCE / "sources" / "ThrowingHelper.java")
    javac_rc = run(
        ["javac", "--release", "8", "-g:none", "-d", str(classes)]
        + [str(path) for path in inputs],
        case / "jadx-javac.log",
    )
    (case / "jadx-status.txt").write_text(f"jadx_rc=0\njavac_rc={javac_rc}\n")
    if javac_rc == 0:
        for arg in args or [""]:
            suffix = arg
            run_rc = run(
                ["java", "-Xverify:all", "-cp", str(classes), runner_name] + ([arg] if arg else []),
                case / f"jadx-run{suffix}.log",
            )
            with (case / "jadx-status.txt").open("a") as status:
                status.write(f"run{suffix}_rc={run_rc}\n")
    return javac_rc


def copy_text_outputs(case, name):
    destination = BOUNDARIES / name
    destination.mkdir(parents=True, exist_ok=True)
    for path in case.rglob("*"):
        if path.is_file() and path.suffix in {".java", ".txt", ".log", ".json"}:
            relative = path.relative_to(case)
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)


def main():
    shutil.rmtree(TMP, ignore_errors=True)
    shutil.rmtree(BOUNDARIES, ignore_errors=True)
    TMP.mkdir(parents=True)
    BOUNDARIES.mkdir(parents=True)
    sources = EVIDENCE / "sources"
    runners = EVIDENCE / "runners"
    summary = {}

    for name, runner, helper in [
        ("StaticEmpty", "StaticStraightRunner", False),
        ("StaticEarlyReturn", "StaticEarlyReturnRunner", False),
        ("StaticExternalException", "StaticExternalExceptionRunner", True),
    ]:
        case = TMP / name
        case.mkdir()
        original = case / "original"
        original.mkdir()
        source = sources / ("StaticStraight.java" if name == "StaticEmpty" else f"{name}.java")
        inputs = [source, runners / f"{runner}.java"]
        if helper:
            inputs.append(sources / "ThrowingHelper.java")
        rc = run(["javac", "--release", "8", "-g:none", "-d", str(original)] + [str(path) for path in inputs], case / "original-javac.log")
        if rc != 0:
            raise RuntimeError(f"javac failed: {name}")
        class_name = "StaticStraight" if name == "StaticEmpty" else name
        class_file = original / f"{class_name}.class"
        run(["javap", "-classpath", str(original), "-c", "-p", "-v", class_name], case / "original-javap.txt")
        original_rc = run(["java", "-Xverify:all", "-cp", str(original), runner], case / "original-run.log")
        (case / "original-status.txt").write_text(f"javac_rc={rc}\nrun_rc={original_rc}\n")
        if name == "StaticEmpty":
            selected = case / "patched" / f"{class_name}.class"
            patch_empty(class_file, selected)
        elif name == "StaticEarlyReturn":
            selected = case / "patched" / f"{class_name}.class"
            patch_early(class_file, selected)
        else:
            selected = class_file
        if name in {"StaticEmpty", "StaticEarlyReturn"}:
            runner_rc = compile_runner(case, selected.parent, runner, helper)
            run(["javap", "-classpath", str(selected.parent), "-c", "-p", "-v", class_name], case / "patched-javap.txt")
            args = ["true", "false"] if name == "StaticEarlyReturn" else []
            patched_status = []
            patched_status.append(f"runner_javac_rc={runner_rc}")
            for arg in args or [""]:
                output = case / f"patched-run{arg}.log"
                if runner_rc == 0:
                    run_rc = run(
                        ["java", "-Xverify:all", "-cp", str(selected.parent), runner] + ([arg] if arg else []),
                        output,
                    )
                else:
                    output.write_text("not run: runner javac failed\n")
                    run_rc = 2
                patched_status.append(f"run{arg}_rc={run_rc}")
            (case / "patched-status.txt").write_text("\n".join(patched_status) + "\n")
        cli_rc = cli(case, selected, class_name)
        recovered_source = case / "jarde" / f"{class_name}.java"
        recovered(case, recovered_source, class_name, runner, helper, ["true", "false"] if name == "StaticEarlyReturn" else [])
        jadx(case, selected, class_name, runner, helper, ["true", "false"] if name == "StaticEarlyReturn" else [])
        summary[name] = {
            "cli_rc": cli_rc,
            "original": (case / "original-run.log").read_text().strip() if (case / "original-run.log").exists() else None,
            "original_status": (case / "original-status.txt").read_text().splitlines(),
            "patched": {
                arg: (case / f"patched-run{arg}.log").read_text().strip()
                for arg in (["true", "false"] if name == "StaticEarlyReturn" else [""])
            } if name in {"StaticEmpty", "StaticEarlyReturn"} else None,
            "patched_status": (case / "patched-status.txt").read_text().splitlines()
            if (case / "patched-status.txt").exists()
            else None,
            "jarde": {
                "status": (case / "jarde" / "status.txt").read_text().splitlines(),
                "runs": {
                    arg: (case / "jarde" / f"run{arg}.log").read_text().strip()
                    for arg in (["true", "false"] if name == "StaticEarlyReturn" else [""])
                    if (case / "jarde" / f"run{arg}.log").exists()
                },
            },
            "jadx": (case / "jadx-status.txt").read_text().splitlines(),
        }
        copy_text_outputs(case, name)

    (BOUNDARIES / "summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(summary, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
