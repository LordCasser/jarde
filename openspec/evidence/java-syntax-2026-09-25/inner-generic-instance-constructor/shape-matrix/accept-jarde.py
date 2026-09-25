#!/usr/bin/env python3
"""Rebuild the four Jarde callers against the original Java 8 Outer family."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZIP_DEFLATED, ZipFile


HERE = Path(__file__).resolve().parent
CALLERS = ("UsePlain", "UseGenericObject", "UseGenericTyped", "UsePlainRaw")
SOURCES = ("Outer", *CALLERS, "Runner")
PLAIN_BCIS = {0, 3, 4, 5, 6, 9, 10, 11, 14, 17}
EXPECTED_BCIS = {
    "UsePlain": PLAIN_BCIS,
    "UsePlainRaw": PLAIN_BCIS,
    "UseGenericObject": PLAIN_BCIS | {20},
    "UseGenericTyped": PLAIN_BCIS | {20},
}


def run(*args: object, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(arg) for arg in args], cwd=cwd, text=True, capture_output=True
    )
    if result.returncode:
        raise RuntimeError(
            f"exit={result.returncode}: {' '.join(map(str, args))}\n"
            f"stdout:\n{result.stdout}stderr:\n{result.stderr}"
        )
    return result


def archive(classes: Path, output: Path, *, outer_only: bool) -> None:
    with ZipFile(output, "w", ZIP_DEFLATED) as jar:
        for path in sorted(classes.rglob("*.class")):
            if outer_only and not path.name.startswith("Outer"):
                continue
            jar.write(path, path.relative_to(classes))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        parser.error(f"Jarde CLI does not exist: {cli}")

    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        with TemporaryDirectory(prefix=f"jarde-generic-enclosing-{label}-") as temporary:
            work = Path(temporary)
            classes = work / "original"
            classes.mkdir()
            run(
                "javac", "--release", "8", "-Xlint:-options", debug,
                "-d", classes,
                *(HERE / "fixture" / f"{name}.java" for name in SOURCES),
            )
            full_jar = work / "original.jar"
            target_jar = work / "original-target.jar"
            archive(classes, full_jar, outer_only=False)
            archive(classes, target_jar, outer_only=True)
            original = run("java", "-Xverify:all", "-cp", full_jar, "matrix.Runner").stdout

            source = work / "source" / "matrix"
            source.mkdir(parents=True)
            for name in CALLERS:
                texts: list[str] = []
                all_report: dict | None = None
                for selection in ("all", "essential"):
                    document = work / f"{name}-{selection}.json"
                    run(
                        cli, "class-source", "--input", full_jar,
                        "--class", f"matrix/{name}", "--format", "json",
                        "--evidence", selection, "--output", document,
                    )
                    report = json.loads(document.read_text())
                    if report["execution"]["status"] != "complete":
                        raise RuntimeError(f"{label}/{name}/{selection}: class-source did not complete")
                    texts.append(report["text"])
                    if selection == "all":
                        all_report = report
                if texts[0] != texts[1]:
                    raise RuntimeError(f"{label}/{name}: evidence selection changed Java text")
                if name == "UsePlain":
                    range_result = subprocess.run(
                        [str(cli), "class-source", "--input", str(full_jar),
                         "--class", "matrix/UsePlain", "--format", "json",
                         "--evidence", "source_map", "--evidence-bci", "0..2"],
                        text=True, capture_output=True,
                    )
                    range_report = json.loads(range_result.stdout)
                    if (range_result.returncode != 4
                            or range_report["execution"]["status"] != "partial"
                            or range_report["execution"]["reason"]["code"]
                            != "jre_evidence_range_invalid"):
                        raise RuntimeError(
                            f"{label}/{name}: unsupported class-source BCI range was not an explicit stop"
                        )
                assert all_report is not None
                method = next(
                    item for item in all_report["methods"]
                    if item["item"]["name"]["escaped"] == "make"
                )
                outcome = method["outcome"]
                if outcome["kind"] != "recovered" or outcome["analysis"]["execution"]["status"] != "complete":
                    raise RuntimeError(f"{label}/{name}: make method was not recovered completely")
                recovery = outcome["report"]
                if recovery["fallbacks"]:
                    raise RuntimeError(f"{label}/{name}: make method still has fallback gaps")
                observed_bcis: set[int] = set()
                for segment in recovery["source_map"]["segments"]:
                    origin = segment["origin"]
                    observed_bcis.add(origin["primary"]["bci"])
                    observed_bcis.update(derived["bci"] for derived in origin["derived"])
                if observed_bcis != EXPECTED_BCIS[name]:
                    raise RuntimeError(f"{label}/{name}: source map BCIs {sorted(observed_bcis)}")
                (source / f"{name}.java").write_text(texts[0])

            rebuilt = work / "rebuilt"
            rebuilt.mkdir()
            run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", target_jar,
                "-d", rebuilt, *(source / f"{name}.java" for name in CALLERS),
            )
            rendered = run(
                "java", "-Xverify:all", "-cp",
                os.pathsep.join((str(rebuilt), str(full_jar))), "matrix.Runner",
            ).stdout
            if rendered != original:
                raise RuntimeError(
                    f"{label}: rebuilt Jarde callers changed behavior\n"
                    f"original:\n{original}rebuilt:\n{rendered}"
                )
            print(f"{label}: four Jarde callers recompiled; JVM trace matches original")


if __name__ == "__main__":
    main()
