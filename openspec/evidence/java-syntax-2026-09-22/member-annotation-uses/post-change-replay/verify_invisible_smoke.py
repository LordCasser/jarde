#!/usr/bin/env python3
"""Compile and compare one small runtime-invisible member-annotation class."""

from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
from pathlib import Path


CLI = Path(os.environ["JARDE_CLI"]).resolve()


SOURCES = {
    "BoundaryMark.java": """import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.CLASS)
@Target({ElementType.FIELD, ElementType.METHOD, ElementType.PARAMETER})
@interface BoundaryMark {
    int value();
}
""",
    "BoundarySmoke.java": """public final class BoundarySmoke {
    @BoundaryMark(1) int field;

    @BoundaryMark(2)
    int wide(@BoundaryMark(3) long first, @BoundaryMark(4) double second,
             @BoundaryMark(5) String... rest) {
        return 7;
    }
}
""",
    "BoundarySmokeRunner.java": """public final class BoundarySmokeRunner {
    public static void main(String[] args) {
        System.out.println(new BoundarySmoke().wide(2L, 3.0d, "x", "yy"));
    }
}
""",
}


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {args}\n{result.stderr}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_java(output: Path, sources: list[Path]) -> None:
    run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-d",
            str(output),
            *(str(path) for path in sources),
        ]
    )


def verify_and_run(classpath: Path) -> str:
    return run(
        ["java", "-Xverify:all", "-cp", str(classpath), "BoundarySmokeRunner"]
    ).stdout


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="jarde-member-ann-invisible-") as temporary:
        root = Path(temporary)
        inputs = root / "input"
        original_classes = root / "original-classes"
        jarde_sources = root / "jarde-sources"
        jarde_classes = root / "jarde-classes"
        inputs.mkdir()
        jarde_sources.mkdir()
        for name, contents in SOURCES.items():
            (inputs / name).write_text(contents)

        compile_java(original_classes, [inputs / name for name in SOURCES])
        original_output = verify_and_run(original_classes)
        assert original_output == "7\n"

        for name in ("BoundaryMark", "BoundarySmoke"):
            result = run(
                [
                    str(CLI),
                    "class-source",
                    "--input",
                    str(original_classes / f"{name}.class"),
                    "--class",
                    name,
                    "--policy",
                    "single-class",
                    "--release",
                    "8",
                    "--format",
                    "text",
                ]
            )
            (jarde_sources / f"{name}.java").write_text(result.stdout)
        (jarde_sources / "BoundarySmokeRunner.java").write_text(
            (inputs / "BoundarySmokeRunner.java").read_text()
        )

        compile_java(
            jarde_classes,
            [jarde_sources / name for name in
             ("BoundaryMark.java", "BoundarySmoke.java", "BoundarySmokeRunner.java")],
        )
        jarde_output = verify_and_run(jarde_classes)
        assert jarde_output == original_output == "7\n"

        original_class = original_classes / "BoundarySmoke.class"
        jarde_class = jarde_classes / "BoundarySmoke.class"
        assert sha(original_class) == sha(jarde_class)
        dump = run(["javap", "-v", "-p", "-classpath", str(jarde_classes), "BoundarySmoke"]).stdout
        for fact in (
            "RuntimeInvisibleAnnotations:",
            "RuntimeInvisibleParameterAnnotations:",
            "parameter 0:",
            "parameter 1:",
            "parameter 2:",
            "value=1",
            "value=2",
            "value=3",
            "value=4",
            "value=5",
        ):
            assert fact in dump, fact

        print(f"original and Jarde runner output: {original_output.strip()}")
        print(f"BoundarySmoke.class SHA-256: {sha(original_class)}")
        print("javac --release 8 and java -Xverify:all: passed for both outputs")
        print("RuntimeInvisibleAnnotations and all three parameter groups: verified")


if __name__ == "__main__":
    main()
