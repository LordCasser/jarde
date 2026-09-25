from hashlib import sha256
from pathlib import Path
import json
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
OUT = Path(__file__).resolve().parent
FIXTURE = ROOT / "tests/fixtures/p3-instanceof/v8/InstanceOfProbe.class"
PROBE = ROOT / "tests/fixtures/p3-instanceof/InstanceOfProbe.java"
SUPPORT = ROOT / "tests/fixtures/p3-instanceof/InstanceOfSupport.java"
RUNNER = OUT / "RefusalRunner.java"


def replace_once(data: bytes, original: bytes, patched: bytes) -> bytes:
    assert data.count(original) == 1
    return data.replace(original, patched)


def method_info(data: bytes, old_hex: str, new_hex: str) -> bytes:
    return replace_once(data, bytes.fromhex(old_hex), bytes.fromhex(new_hex))


CALLED_OLD = (
    "00 09 00 2e 00 27 00 01 00 23 00 00 00 13 "
    "00 01 00 00 00 00 00 07 b8 00 13 c1 00 11 ac 00 00 00 00"
)
CALLED_POP = (
    "00 09 00 2e 00 06 00 01 00 23 00 00 00 14 "
    "00 01 00 00 00 00 00 08 b8 00 13 c1 00 11 57 b1 00 00 00 00"
)
LOCAL_OLD = (
    "00 09 00 2f 00 25 00 01 00 23 00 00 00 13 "
    "00 01 00 02 00 00 00 07 2a c1 00 07 3c 1b ac 00 00 00 00"
)
LOCAL_DUPLICATE = (
    "00 09 00 2f 00 25 00 01 00 23 00 00 00 15 "
    "00 02 00 03 00 00 00 09 2a c1 00 07 59 3c 3d 1b ac 00 00 00 00"
)
LOCAL_STALE = (
    "00 09 00 2f 00 25 00 01 00 23 00 00 00 15 "
    "00 01 00 02 00 00 00 09 2a c1 00 07 3c 03 3c 1b ac 00 00 00 00"
)
BRANCH_OLD = (
    "00 09 00 31 00 32 00 01 00 23 00 00 00 20 "
    "00 01 00 01 00 00 00 0b 2a c1 00 07 99 00 05 04 ac 03 ac "
    "00 00 00 01 00 33 00 00 00 03 00 01 09"
)
BRANCH_INT = (
    "00 09 00 31 00 32 00 01 00 23 00 00 00 13 "
    "00 02 00 01 00 00 00 07 2a c1 00 07 04 7e ac 00 00 00 00"
)


def variants(original: bytes) -> dict[str, bytes]:
    return {
        "pop": method_info(original, CALLED_OLD, CALLED_POP),
        "duplicate": method_info(original, LOCAL_OLD, LOCAL_DUPLICATE),
        "stale": method_info(original, LOCAL_OLD, LOCAL_STALE),
        "int": method_info(original, BRANCH_OLD, BRANCH_INT),
    }


def run(command: list[str], log: Path, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
    log.write_text(result.stdout + result.stderr)
    return result


def compile_variant(directory: Path, probe: bytes, mode: str) -> None:
    directory.mkdir()
    (directory / "InstanceOfProbe.class").write_bytes(probe)
    result = run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(directory),
            "-d",
            str(directory),
            str(SUPPORT),
            str(RUNNER),
        ],
        OUT / f"{mode}-javac.log",
    )
    assert result.returncode == 0, result.stderr


def main() -> None:
    original = FIXTURE.read_bytes()
    variants_by_name = variants(original)
    summary: dict[str, object] = {
        "original": {
            "bytes": len(original),
            "sha256": sha256(original).hexdigest(),
        },
        "variants": {},
    }

    with tempfile.TemporaryDirectory(prefix="jarde-instanceof-refusal-") as temporary:
        work = Path(temporary)
        original_dir = work / "original"
        original_dir.mkdir()
        result = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(original_dir),
                str(SUPPORT),
                str(PROBE),
                str(RUNNER),
            ],
            OUT / "original-javac.log",
        )
        assert result.returncode == 0, result.stderr
        shutil.copyfile(FIXTURE, original_dir / "InstanceOfProbe.class")
        run_original = {}
        for mode in ("pop", "duplicate", "stale", "int"):
            result = run(
                ["java", "-Xverify:all", "-cp", str(original_dir), "RefusalRunner", mode],
                OUT / f"original-{mode}-verified.txt",
            )
            assert result.returncode == 0, result.stderr
            run_original[mode] = result.stdout
        (OUT / "original-run.txt").write_text(
            "".join(f"[{mode}]\n{output}" for mode, output in run_original.items())
        )
        result = run(
            ["javap", "-classpath", str(original_dir), "-p", "-c", "-v", "InstanceOfProbe"],
            OUT / "original-javap.txt",
        )
        assert result.returncode == 0, result.stderr

        for mode, patched in variants_by_name.items():
            directory = work / mode
            compile_variant(directory, patched, mode)
            result = run(
                ["java", "-Xverify:all", "-cp", str(directory), "RefusalRunner", mode],
                OUT / f"{mode}-verified.txt",
            )
            assert result.returncode == 0, result.stderr
            result = run(
                ["javap", "-classpath", str(directory), "-p", "-c", "-v", "InstanceOfProbe"],
                OUT / f"{mode}-javap.txt",
            )
            assert result.returncode == 0, result.stderr
            summary["variants"][mode] = {
                "bytes": len(patched),
                "sha256": sha256(patched).hexdigest(),
                "output": (OUT / f"{mode}-verified.txt").read_text(),
            }

    (OUT / "class-sha256.txt").write_text(
        "original " + summary["original"]["sha256"] + "\n"
        + "".join(
            f"{name} {row['sha256']}\n" for name, row in summary["variants"].items()
        )
    )
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
