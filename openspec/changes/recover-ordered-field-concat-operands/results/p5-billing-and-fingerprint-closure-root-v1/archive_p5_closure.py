#!/usr/bin/env python3
"""Create a gzip tar of the local P5 billing/fingerprint closure evidence."""

import argparse
import hashlib
from pathlib import Path
import sys
import tarfile

REPO = Path("/Users/lordcasser/workspace/projects/jarde")
DEST = Path("/private/tmp/jarde-p5-closure-archive-prep-luna-v1")
FIXED_DIRS = [
    Path("/private/tmp/jarde-p5-billing-delta-audit-root-v1"),
    Path("/private/tmp/jarde-p5-billing-recorder-root-v1"),
    Path("/private/tmp/jarde-loop-field-fingerprint-root-v1"),
    Path("/private/tmp/jarde-fingerprint-interruption-recovery-root-v1"),
    Path("/private/tmp/jarde-loop-field-fingerprint-input-audit-root-v1"),
    Path("/private/tmp/jarde-p5-billing-accept-luna-v1"),
]
SNAPSHOTS = [
    REPO / "tests/p5_bulk_corpus.rs",
    REPO / "tests/fixtures/corpus-fingerprint.json",
]
MAX_FILE_BYTES = 20 * 1024 * 1024


def excluded(path: Path) -> bool:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_FILE_BYTES:
        return True
    if path.suffix.lower() in {".exe", ".dll", ".so", ".dylib", ".bin", ".class"}:
        return True
    with path.open("rb") as f:
        return b"\0" in f.read(8192)


def add_tree(tar: tarfile.TarFile, source: Path, archive_name: str) -> None:
    for path in sorted(source.rglob("*")):
        if excluded(path):
            continue
        tar.add(path, arcname=f"{archive_name}/{path.relative_to(source)}", recursive=False)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--collector-dir", required=True, type=Path,
                        help="private collector source/pins/README v2 directory")
    parser.add_argument("--final-gate-dir", required=True, type=Path,
                        help="completed /private/tmp/jarde-ordered-field-final-check-root-v6 directory")
    parser.add_argument("--output", type=Path,
                        default=DEST / "p5-billing-fingerprint-closure.tar.gz")
    args = parser.parse_args()

    inputs = [(p, p.name) for p in FIXED_DIRS]
    inputs += [(args.collector_dir, "collector-v2"), (args.final_gate_dir, "final-gate")]
    missing = [str(p) for p, _ in inputs if not p.is_dir()]
    missing += [str(p) for p in SNAPSHOTS if not p.is_file()]
    if missing:
        parser.error("required input missing: " + ", ".join(missing))

    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(output, "w:gz") as tar:
        for source, name in inputs:
            add_tree(tar, source, name)
        for source in SNAPSHOTS:
            tar.add(source, arcname=f"repo-snapshots/{source.relative_to(REPO)}", recursive=False)

    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    print(f"archive={output}")
    print(f"sha256={digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
