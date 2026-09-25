#!/usr/bin/env python3
"""Compile Java 8 functional receivers and compare whole-class decompilations.

JADX emits a synthetic `defpackage` for an unnamed-package class. Only the
source-only runner receives that package line; neither decompiler's subject is
edited before compilation.
"""

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
CASES = {
    "direct-array": (
        "DirectArrayCtorRef",
        "DirectArrayCtorRunner",
        "0c4544d1a9334c35d5a5191ba4ef2d17200a8ce9f0ca34b2bd6331538310ea98",
    ),
    "direct-method": (
        "DirectMethodRef",
        "DirectMethodRunner",
        "de0b0e15bdb199295ce055f36e46cb7dcd56854a1738832ccd8931f5a5053dc3",
    ),
    "bound-control": (
        "BoundFunctionalReceiver",
        "BoundFunctionalRunner",
        "aa517756621a289f1a5cf9967a35b6c1e9f42e58e1d26bac74d06973073fd4b7",
    ),
}
MECHANISM_ONLY = {
    "direct-array": (
        "((int p0) -> DirectArrayCtorRef.lambda$make$0(p0)).apply(n)",
        "((java.util.function.IntFunction) ((int p0) -> DirectArrayCtorRef.lambda$make$0(p0))).apply(n)",
    ),
    "direct-method": (
        "java.lang.Math::abs.applyAsInt(n)",
        "((java.util.function.IntUnaryOperator) java.lang.Math::abs).applyAsInt(n)",
    ),
}


def run(argv):
    result = subprocess.run(argv, text=True, capture_output=True, timeout=60, check=False)
    return result.returncode, result.stdout, result.stderr


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save_process(directory, stem, result, out):
    code, stdout, stderr = result
    (directory / f"{stem}.stdout.txt").write_text(stdout, encoding="utf-8")
    (directory / f"{stem}.stderr.txt").write_text(
        stderr.replace(str(out), "<OUT>"), encoding="utf-8"
    )
    return code


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True, help="frozen jarde-cli executable")
    parser.add_argument("--out", type=Path, default=HERE / "results")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    cli = args.cli.resolve()
    summary = {"cli_sha256": digest(cli), "cases": {}}

    for label, (subject, runner, expected_sha) in CASES.items():
        source_dir = HERE / label
        case = out / label
        case.mkdir(parents=True, exist_ok=True)
        original_classes = case / "original-classes"
        original_classes.mkdir(exist_ok=True)
        original_compile = run(
            [
                "javac", "--release", "8", "-g", "-Xlint:-options", "-d",
                str(original_classes), str(source_dir / f"{subject}.java"),
                str(source_dir / f"{runner}.java"),
            ]
        )
        assert save_process(case, "original-javac", original_compile, out) == 0
        class_file = original_classes / f"{subject}.class"
        assert digest(class_file) == expected_sha, (label, digest(class_file))
        original = run(["java", "-Xverify:all", "-cp", str(original_classes), runner])
        assert save_process(case, "original-java", original, out) == 0
        javap = run(["javap", "-c", "-p", str(class_file)])
        assert save_process(case, "javap", javap, out) == 0

        jarde = run(
            [str(cli), "class-source", "--input", str(class_file), "--class", subject,
             "--policy", "single-class", "--release", "8", "--format", "text"]
        )
        assert save_process(case, "jarde", jarde, out) == 0
        jadx_dir = case / "jadx-decompiled"
        jadx = run(["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)])
        assert save_process(case, "jadx", jadx, out) == 0
        found = list(jadx_dir.rglob(f"{subject}.java"))
        assert len(found) == 1, found
        jadx_source = found[0].read_text(encoding="utf-8")

        row = {
            "class_sha256": expected_sha,
            "class_bytes": class_file.stat().st_size,
            "code_count": javap[1].count("Code:"),
            "original_output": original[1],
            "jarde_references": jarde[1].count("@bytecode"),
            "jarde_source_sha256": hashlib.sha256(jarde[1].encode()).hexdigest(),
            "jadx_source_sha256": hashlib.sha256(jadx_source.encode()).hexdigest(),
        }
        for name, text in (("jarde", jarde[1]), ("jadx", jadx_source)):
            generated = case / name
            generated.mkdir(exist_ok=True)
            (generated / f"{subject}.java").write_text(text, encoding="utf-8")
            runner_text = (source_dir / f"{runner}.java").read_text(encoding="utf-8")
            if name == "jadx":
                runner_text = "package defpackage;\n" + runner_text
            (generated / f"{runner}.java").write_text(runner_text, encoding="utf-8")
            compiled = generated / "classes"
            compiled.mkdir(exist_ok=True)
            compile_result = run(
                ["javac", "--release", "8", "-Xlint:-options", "-d", str(compiled),
                 str(generated / f"{subject}.java"), str(generated / f"{runner}.java")]
            )
            row[f"{name}_javac_exit"] = save_process(
                case, f"{name}-javac", compile_result, out
            )
            if compile_result[0] == 0:
                main_class = f"defpackage.{runner}" if name == "jadx" else runner
                execution = run(["java", "-Xverify:all", "-cp", str(compiled), main_class])
                row[f"{name}_java_exit"] = save_process(case, f"{name}-java", execution, out)
                row[f"{name}_output"] = execution[1]

        # The controlled one-token cast belongs to the frozen failing baseline.
        # After the fix, Jarde already emits that cast, so there is nothing to edit.
        if label in MECHANISM_ONLY and MECHANISM_ONLY[label][0] in jarde[1]:
            before, after = MECHANISM_ONLY[label]
            assert jarde[1].count(before) == 1, (label, "mechanism text changed")
            controlled = jarde[1].replace(before, after)
            generated = case / "mechanism-only"
            generated.mkdir(exist_ok=True)
            (generated / f"{subject}.java").write_text(controlled, encoding="utf-8")
            (generated / f"{runner}.java").write_text(
                (source_dir / f"{runner}.java").read_text(encoding="utf-8"), encoding="utf-8"
            )
            compiled = generated / "classes"
            compiled.mkdir(exist_ok=True)
            compile_result = run(
                ["javac", "--release", "8", "-Xlint:-options", "-d", str(compiled),
                 str(generated / f"{subject}.java"), str(generated / f"{runner}.java")]
            )
            row["mechanism_only_javac_exit"] = save_process(
                case, "mechanism-only-javac", compile_result, out
            )
            if compile_result[0] == 0:
                execution = run(["java", "-Xverify:all", "-cp", str(compiled), runner])
                row["mechanism_only_java_exit"] = save_process(
                    case, "mechanism-only-java", execution, out
                )
                row["mechanism_only_output"] = execution[1]
        summary["cases"][label] = row

    (out / "summary.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(json.dumps(summary, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
