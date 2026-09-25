#!/usr/bin/env python3
"""Rebuild task 1.2 Java 8 fixtures from their sources and bounded descriptor patches."""
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
STACK_EVIDENCE = (
    HERE.parents[2]
    / "openspec/evidence/java-syntax-2026-09-22/numeric-conversions/"
    / "narrow-switch-returns/actual-stack-join"
)


def run(args: list[str]) -> None:
    subprocess.run(args, check=True)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {Path(sys.argv[0]).name} OUTPUT_DIR")
    output = Path(sys.argv[1]).resolve()
    compiled = output / "compiled"
    compiled.mkdir(parents=True, exist_ok=True)
    run(
        [
            "javac", "--release", "8", "-g:none", "-d", str(compiled),
            str(HERE / "actual-stack-join/ActualStackJoin.java"),
        ]
    )
    base = output / "stack-base.class"
    run(
        [
            "python3", str(STACK_EVIDENCE / "patch_stack_join.py"),
            str(compiled / "ActualStackJoin.class"), str(base), str(output / "stack-patch.json"),
        ]
    )
    for kind, folder in [("B", "byte"), ("C", "char"), ("S", "short")]:
        destination = output / folder
        destination.mkdir(parents=True, exist_ok=True)
        run(
            [
                "python3", str(STACK_EVIDENCE / "patch_descriptors.py"), str(base),
                str(destination / "ActualStackJoin.class"),
                str(destination / "descriptor-patch.json"), kind,
            ]
        )
    boolean_compiled = output / "boolean-compiled"
    boolean_compiled.mkdir(parents=True, exist_ok=True)
    run(
        [
            "javac", "--release", "8", "-g:none", "-d", str(boolean_compiled),
            str(HERE / "BooleanReturnBoundaries.java"),
        ]
    )
    boolean_output = output / "boolean-boundaries"
    boolean_output.mkdir(parents=True, exist_ok=True)
    run(
        [
            "python3", str(HERE / "patch_return_boundaries.py"),
            str(boolean_compiled), str(boolean_output),
        ]
    )


if __name__ == "__main__":
    main()
