from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[6]
EVIDENCE = Path(__file__).resolve().parent
INPUTS = (
    EVIDENCE / "UnknownTypes.java",
    EVIDENCE / "GenerateUnknownReference.java",
    EVIDENCE / "UnknownReferenceRunner.java",
    EVIDENCE / "UnknownReferenceJavacControl.java",
)


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)
    log.write_text(result.stdout + result.stderr)
    return result


def main():
    (EVIDENCE / "input-sha256.txt").write_text(
        "".join(
            f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n"
            for path in (*INPUTS, Path(__file__))
        )
    )
    java_version = subprocess.run(
        ["java", "-version"], capture_output=True, text=True, timeout=10
    )
    javac_version = subprocess.run(
        ["javac", "-version"], capture_output=True, text=True, timeout=10
    )
    (EVIDENCE / "toolchain.txt").write_text(
        java_version.stderr + javac_version.stdout + javac_version.stderr
    )

    with tempfile.TemporaryDirectory(prefix="jarde-unknown-reference-probe-") as temp:
        work = Path(temp)
        classes = work / "classes"
        classes.mkdir()
        types = run(
            ["javac", "--release", "8", "-g:none", "-d", str(classes), str(INPUTS[0])],
            EVIDENCE / "types-javac.log",
        )
        (EVIDENCE / "types-javac-status.txt").write_text(f"exit={types.returncode}\n")
        if types.returncode:
            raise SystemExit(types.returncode)

        exports = "java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED"
        generator_compile = run(
            [
                "javac",
                "--add-exports",
                exports,
                "-cp",
                str(classes),
                "-d",
                str(classes),
                str(INPUTS[1]),
            ],
            EVIDENCE / "generator-javac.log",
        )
        (EVIDENCE / "generator-javac-status.txt").write_text(
            f"exit={generator_compile.returncode}\n"
        )
        if generator_compile.returncode:
            raise SystemExit(generator_compile.returncode)
        generated = run(
            [
                "java",
                "--add-exports",
                exports,
                "-cp",
                str(classes),
                "GenerateUnknownReference",
                str(classes / "UnknownReferenceProbe.class"),
            ],
            EVIDENCE / "generator-run.log",
        )
        (EVIDENCE / "generator-run-status.txt").write_text(f"exit={generated.returncode}\n")
        if generated.returncode:
            raise SystemExit(generated.returncode)

        target_class = classes / "UnknownReferenceProbe.class"
        shutil.copyfile(target_class, EVIDENCE / "UnknownReferenceProbe.class")
        javap = run(
            ["javap", "-p", "-c", "-v", str(target_class)],
            EVIDENCE / "original-javap.txt",
        )
        (EVIDENCE / "original-javap-status.txt").write_text(f"exit={javap.returncode}\n")

        runner_compile = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-cp",
                str(classes),
                "-d",
                str(classes),
                str(INPUTS[2]),
            ],
            EVIDENCE / "runner-javac.log",
        )
        (EVIDENCE / "runner-javac-status.txt").write_text(
            f"exit={runner_compile.returncode}\n"
        )
        if runner_compile.returncode:
            raise SystemExit(runner_compile.returncode)
        original_run = run(
            ["java", "-Xverify:all", "-cp", str(classes), "UnknownReferenceRunner"],
            EVIDENCE / "original-runtime.txt",
        )
        (EVIDENCE / "original-runtime-status.txt").write_text(
            f"exit={original_run.returncode}\n"
        )

        source_control_out = work / "source-control"
        source_control_out.mkdir()
        control = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-cp",
                str(classes),
                "-d",
                str(source_control_out),
                str(INPUTS[3]),
            ],
            EVIDENCE / "javac-source-control.log",
        )
        (EVIDENCE / "javac-source-control-status.txt").write_text(
            f"exit={control.returncode}\n"
        )

        if original_run.returncode != 0 or control.returncode == 0:
            raise SystemExit("the observed runtime/refusal boundary changed")
        summary = {
            "class_sha256": hashlib.sha256(target_class.read_bytes()).hexdigest(),
            "class_bytes": target_class.stat().st_size,
            "class_major_version": int.from_bytes(target_class.read_bytes()[6:8], "big"),
            "original_bytecode_verifier": "passed under java -Xverify:all before invokedynamic linkage",
            "original_runtime_exit": original_run.returncode,
            "original_runtime": (EVIDENCE / "original-runtime.txt").read_text().splitlines(),
            "javac_source_control_exit": control.returncode,
            "boundary_is_hand_generated_indy_control": True,
            "not_a_javac_origin_class_or_positive_recovery_case": True,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
