#!/usr/bin/env python3
"""Independently recompile Jarde's complete shadowing classes and compare reflection."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZIP_DEFLATED, ZipFile


HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"
CLASSES = ("ShadowPlain", "ShadowBounded")
REFLECT = """package shadow;
import java.lang.reflect.Method;
import java.util.Arrays;
public final class Reflect {
    public static void main(String[] args) throws Exception {
        show(ShadowPlain.class, Object.class);
        show(ShadowBounded.class, CharSequence.class);
    }
    private static void show(Class<?> owner, Class<?> erased) throws Exception {
        Method method = owner.getMethod("echo", erased);
        System.out.println(owner.getName() + " class="
            + Arrays.toString(owner.getTypeParameters()[0].getBounds())
            + " method=" + Arrays.toString(method.getTypeParameters()[0].getBounds())
            + " arg=" + method.getGenericParameterTypes()[0].getTypeName()
            + " return=" + method.getGenericReturnType().getTypeName());
    }
}
"""


def run(*args: object) -> str:
    result = subprocess.run([str(arg) for arg in args], text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(
            f"exit={result.returncode}: {' '.join(map(str, args))}\n"
            f"stdout:\n{result.stdout}stderr:\n{result.stderr}"
        )
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        parser.error(f"Jarde CLI does not exist: {cli}")

    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        with TemporaryDirectory(prefix=f"jarde-shadowing-accept-{label}-") as temporary:
            work = Path(temporary)
            original = work / "original"
            original.mkdir()
            reflect = work / "Reflect.java"
            reflect.write_text(REFLECT)
            run(
                "javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
                *(FIXTURE / f"{name}.java" for name in (*CLASSES, "StrongCaller")),
                reflect,
            )
            expected = run("java", "-Xverify:all", "-cp", original, "shadow.StrongCaller")
            expected_reflection = run("java", "-Xverify:all", "-cp", original, "shadow.Reflect")
            jar = work / "original.jar"
            with ZipFile(jar, "w", ZIP_DEFLATED) as archive:
                for item in sorted(original.rglob("*.class")):
                    archive.write(item, item.relative_to(original))

            source = work / "source" / "shadow"
            source.mkdir(parents=True)
            for name in CLASSES:
                texts: list[str] = []
                for selection, flags in (
                    ("all", ("--evidence", "all")),
                    ("essential", ("--evidence", "essential")),
                ):
                    document = work / f"{name}-{selection}.json"
                    run(
                        cli, "class-source", "--input", jar,
                        "--class", f"shadow/{name}", "--format", "json",
                        *flags, "--output", document,
                    )
                    report = json.loads(document.read_text())
                    if report["execution"]["status"] != "complete":
                        raise RuntimeError(f"{label}/{name}/{selection}: class-source did not complete")
                    texts.append(report["text"])
                if any(text != texts[0] for text in texts[1:]):
                    raise RuntimeError(f"{label}/{name}: evidence selection changed Java text")
                (source / f"{name}.java").write_text(texts[0])
            rebuilt = work / "rebuilt"
            rebuilt.mkdir()
            run(
                "javac", "--release", "8", "-Xlint:-options", "-d", rebuilt,
                *(source / f"{name}.java" for name in CLASSES),
                FIXTURE / "StrongCaller.java", reflect,
            )
            actual = run("java", "-Xverify:all", "-cp", rebuilt, "shadow.StrongCaller")
            actual_reflection = run("java", "-Xverify:all", "-cp", rebuilt, "shadow.Reflect")
            if actual != expected or actual_reflection != expected_reflection:
                raise RuntimeError(
                    f"{label}: Jarde changed behavior or generic reflection\n"
                    f"original: {expected!r} {expected_reflection!r}\n"
                    f"rebuilt:  {actual!r} {actual_reflection!r}"
                )
            print(f"{label}: complete Jarde classes recompiled; values and reflection match")


if __name__ == "__main__":
    main()
