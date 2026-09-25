from pathlib import Path
import hashlib
import json
import shutil
import struct
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
FIXTURE = ROOT / "tests/fixtures/p3-floating-constants"
EVIDENCE = Path(__file__).resolve().parent
SOURCE_NAMES = ("FloatingConstants.java", "FloatingSupport.java", "FloatingRunner.java")
EXPECTED = """floatZero=0
floatNegativeZero=80000000
floatOne=3f800000
floatTwo=40000000
floatNegativeOne=bf800000
floatFraction=3dcccccd
floatMinimum=1
floatNormal=800000
floatMaximum=7f7fffff
floatNan=7fc00000
floatPositiveInfinity=7f800000
floatNegativeInfinity=ff800000
doubleZero=0
doubleNegativeZero=8000000000000000
doubleOne=3ff0000000000000
doubleTwo=4000000000000000
doubleNegativeOne=bff0000000000000
doubleFraction=3fb999999999999a
doubleMinimum=1
doubleNormal=10000000000000
doubleMaximum=7fefffffffffffff
doubleNan=7ff8000000000000
doublePositiveInfinity=7ff0000000000000
doubleNegativeInfinity=fff0000000000000
floatArgument=1
doubleArgument=2
nested=ffc00000
threshold=9
nested=3f800000
threshold=9
nested=0
threshold=9
nested=80000000
threshold=9
nested=bf800000
threshold=7
"""


def run(args, output, cwd=None, check=False):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)
    output.write_text(result.stdout + result.stderr)
    if check and result.returncode != 0:
        raise RuntimeError(f"{args[0]} failed with {result.returncode}")
    return result


def code_attribute(code, stack):
    return struct.pack(">IHHI", 12 + len(code), stack, 0, len(code)) + code + b"\0\0\0\0"


def replace_code(class_bytes, before, after, stack, new_stack=None):
    old = code_attribute(before, stack)
    new = code_attribute(after, stack if new_stack is None else new_stack)
    assert class_bytes.count(old) == 1, "the frozen class has one exact Code attribute"
    return class_bytes.replace(old, new)


def run_runner(class_dir, output):
    return run(
        ["java", "-Xverify:all", "-cp", str(class_dir), "FloatingRunner"],
        output,
        cwd=class_dir,
        check=True,
    )


def write_pool_variant(base, name, float_bits, double_bits, work):
    float_pool = bytes.fromhex("04 7fc00000")
    double_pool = bytes.fromhex("06 7ff8000000000000")
    assert base.count(float_pool) == 1
    assert base.count(double_pool) == 1
    patched = base.replace(float_pool, b"\x04" + float_bits.to_bytes(4, "big"))
    patched = patched.replace(double_pool, b"\x06" + double_bits.to_bytes(8, "big"))
    directory = work / name
    directory.mkdir()
    (directory / "FloatingConstants.class").write_bytes(patched)
    for helper in ("FloatingSupport", "FloatingRunner"):
        shutil.copy2(work / "compiled" / f"{helper}.class", directory / f"{helper}.class")
    evidence = EVIDENCE / "variants" / name
    evidence.mkdir(parents=True, exist_ok=True)
    run_runner(directory, evidence / "original.txt")
    run(["javap", "-p", "-c", "-v", str(directory / "FloatingConstants.class")], evidence / "javap.txt")
    return patched, directory, evidence


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="jarde-floating-fixture-") as temp:
        work = Path(temp)
        compiled = work / "compiled"
        compiled.mkdir()
        compile_log = EVIDENCE / "original-javac.log"
        result = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(compiled),
                *(str(FIXTURE / source) for source in SOURCE_NAMES),
            ],
            compile_log,
            check=True,
        )
        base_path = compiled / "FloatingConstants.class"
        base = base_path.read_bytes()
        frozen = (FIXTURE / "v8/FloatingConstants.class").read_bytes()
        assert base == frozen, "source compilation differs from the committed frozen class"

        run_runner(compiled, EVIDENCE / "original.txt")
        assert (EVIDENCE / "original.txt").read_text() == EXPECTED
        run(["javap", "-p", "-c", "-v", str(base_path)], EVIDENCE / "original-javap.txt")
        sha = hashlib.sha256(base).hexdigest()
        (EVIDENCE / "class-sha256.txt").write_text(f"{sha}  FloatingConstants.class\n")

        variants = []
        for name, fbits, dbits in (
            ("canonical-nan-fneg", None, None),
            ("runtime-zero-div-zero", None, None),
        ):
            if name == "canonical-nan-fneg":
                patched = replace_code(base, bytes.fromhex("120fae"), bytes.fromhex("120f76ae"), 1)
                patched = replace_code(patched, bytes.fromhex("140022af"), bytes.fromhex("14002277af"), 2)
            else:
                patched = replace_code(
                    base, bytes.fromhex("120fae"), bytes.fromhex("0b0b6eae"), 1, 2
                )
                patched = replace_code(
                    patched, bytes.fromhex("140022af"), bytes.fromhex("0e0e6faf"), 2, 4
                )
            directory = work / name
            directory.mkdir()
            (directory / "FloatingConstants.class").write_bytes(patched)
            for helper in ("FloatingSupport", "FloatingRunner"):
                shutil.copy2(compiled / f"{helper}.class", directory / f"{helper}.class")
            evidence = EVIDENCE / "variants" / name
            evidence.mkdir(parents=True, exist_ok=True)
            run_runner(directory, evidence / "original.txt")
            run(["javap", "-p", "-c", "-v", str(directory / "FloatingConstants.class")], evidence / "javap.txt")
            variants.append(
                {
                    "name": name,
                    "sha256": hashlib.sha256(patched).hexdigest(),
                    "size": len(patched),
                    "floatNan": next(
                        line.split("=", 1)[1]
                        for line in (evidence / "original.txt").read_text().splitlines()
                        if line.startswith("floatNan=")
                    ),
                    "doubleNan": next(
                        line.split("=", 1)[1]
                        for line in (evidence / "original.txt").read_text().splitlines()
                        if line.startswith("doubleNan=")
                    ),
                }
            )

        for name, fbits, dbits in (
            ("positive-quiet", 0x7FC12345, 0x7FF8123456789ABC),
            ("negative-quiet", 0xFFC12345, 0xFFF8123456789ABC),
            ("positive-signaling", 0x7F812345, 0x7FF0123456789ABC),
        ):
            patched, directory, evidence = write_pool_variant(base, name, fbits, dbits, work)
            variants.append(
                {
                    "name": name,
                    "sha256": hashlib.sha256(patched).hexdigest(),
                    "size": len(patched),
                    "floatNan": f"{fbits:08x}",
                    "doubleNan": f"{dbits:016x}",
                    "observed": [
                        line
                        for line in (evidence / "original.txt").read_text().splitlines()
                        if line.startswith(("floatNan=", "doubleNan="))
                    ],
                }
            )

        (EVIDENCE / "variants-summary.json").write_text(json.dumps(variants, indent=2) + "\n")

        cli = subprocess.run(
            [
                str(ROOT / "target/debug/jarde-cli"),
                "class-source",
                "--input",
                str(base_path),
                "--class",
                "FloatingConstants",
                "--policy",
                "single-class",
                "--release",
                "8",
                "--format",
                "text",
                "--evidence",
                "all",
            ],
            capture_output=True,
            text=True,
            timeout=45,
        )
        (EVIDENCE / "jarde.java.txt").write_text(cli.stdout)
        (EVIDENCE / "jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
        (EVIDENCE / "summary.json").write_text(
            json.dumps(
                {
                    "class_sha256": sha,
                    "class_bytes": len(base),
                    "code_attributes": 29,
                    "original_lines": len(EXPECTED.splitlines()),
                    "jarde_exit": cli.returncode,
                    "jarde_quotes": (EVIDENCE / "jarde.java.txt").read_text().count("@bytecode"),
                    "variants": variants,
                },
                indent=2,
            )
            + "\n"
        )


if __name__ == "__main__":
    main()
