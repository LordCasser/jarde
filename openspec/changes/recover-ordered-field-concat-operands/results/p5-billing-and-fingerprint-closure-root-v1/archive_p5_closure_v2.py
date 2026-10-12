#!/usr/bin/env python3
"""Archive and byte-verify the local P5 billing/fingerprint closure evidence."""

import argparse
import hashlib
import json
from pathlib import Path
import sys
import tarfile
import tempfile

REPO = Path("/Users/lordcasser/workspace/projects/jarde")
DEST = Path("/private/tmp/jarde-p5-closure-archive-prep-luna-v1")
FIXED_DIRS = [
    Path("/private/tmp/jarde-p5-billing-delta-audit-root-v1"),
    Path("/private/tmp/jarde-p5-billing-recorder-root-v1"),
    Path("/private/tmp/jarde-loop-field-fingerprint-root-v1"),
    Path("/private/tmp/jarde-fingerprint-interruption-recovery-root-v1"),
    Path("/private/tmp/jarde-loop-field-fingerprint-input-audit-root-v1"),
    Path("/private/tmp/jarde-p5-billing-accept-luna-v1"),
    Path("/private/tmp/jarde-typed-p5-billing-record-invocation-root-v1"),
    Path("/private/tmp/jarde-typed-ci-p5-billing-record-root-v1"),
]
DEFAULT_SNAPSHOTS = [
    REPO / "tests/p5_bulk_corpus.rs",
    REPO / "tests/fixtures/corpus-fingerprint.json",
]
EXPECTED_STATUS = "commands_passed_pending_independent_acceptance"


def fail(message: str) -> None:
    raise SystemExit(message)


def root_summary_check(final_gate: Path) -> Path:
    summaries = sorted(p for p in final_gate.rglob("*.json")
                       if "root-summary" in p.name.lower().replace("_", "-")
                       and "__pycache__" not in p.parts)
    if len(summaries) != 1:
        fail(f"expected exactly one root-summary JSON under {final_gate}; found {len(summaries)}")
    summary_path = summaries[0]
    try:
        data = json.loads(summary_path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        fail(f"cannot read root summary {summary_path}: {e}")
    leaves = {}

    def walk(value):
        if isinstance(value, dict):
            for key, child in value.items():
                normalized = "".join(c.lower() for c in key if c.isalnum())
                if not isinstance(child, (dict, list)):
                    leaves.setdefault(normalized, []).append(child)
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)

    walk(data)
    if EXPECTED_STATUS not in leaves.get("status", []):
        fail(f"root summary status must be {EXPECTED_STATUS}")
    if True not in leaves.get("targetabsent", []):
        fail("root summary must record target_absent: true")
    before, after = leaves.get("sourcebefore", []), leaves.get("sourceafter", [])
    if len(before) != 1 or len(after) != 1 or before[0] != after[0]:
        fail("root summary must record one equal source_before and source_after value")
    return summary_path


def excluded(path: Path, final_gate: Path) -> bool:
    if "__pycache__" in path.parts:
        return True
    try:
        under_gate = path.is_relative_to(final_gate)
    except AttributeError:  # Python < 3.9
        under_gate = final_gate == path or final_gate in path.parents
    return under_gate and path.name == "candidate-cli"


def collect(source: Path, arc_root: str, final_gate: Path):
    if source.is_symlink():
        fail(f"input root is a symlink: {source}")
    if source.is_file():
        return [(source, arc_root)]
    files = []
    for path in sorted(source.rglob("*")):
        if excluded(path, final_gate):
            continue
        if path.is_symlink():
            fail(f"refusing to omit symlink input: {path}")
        if path.is_file():
            files.append((path, f"{arc_root}/{path.relative_to(source)}"))
    return files


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--collector-dir", required=True, type=Path,
                        help="private collector source/pins/README v2 directory")
    parser.add_argument("--final-gate-dir", required=True, type=Path,
                        help="completed /private/tmp/jarde-ordered-field-final-check-root-v6 directory")
    parser.add_argument("--snapshot", action="append", type=Path, default=[],
                        help="additional current source snapshot; may be repeated")
    parser.add_argument("--output", type=Path,
                        default=DEST / "p5-billing-fingerprint-closure-v2.tar.gz")
    args = parser.parse_args()

    inputs = [(p, p.name) for p in FIXED_DIRS]
    inputs += [(args.collector_dir, "collector-v2"), (args.final_gate_dir, "final-gate")]
    snapshots = DEFAULT_SNAPSHOTS + args.snapshot
    missing = [str(p) for p, _ in inputs if not p.is_dir()]
    missing += [str(p) for p in snapshots if not p.is_file()]
    if missing:
        parser.error("required input missing: " + ", ".join(missing))

    final_gate = args.final_gate_dir.resolve()
    summary = root_summary_check(final_gate)
    output = args.output.resolve()
    manifest_path = output.with_suffix(output.suffix + ".manifest.json")
    if output.exists() or manifest_path.exists():
        fail(f"refusing to overwrite existing output or manifest: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)

    members = []
    for source, arc_root in inputs:
        members.extend(collect(source, arc_root, final_gate))
    members.extend((p, f"repo-snapshots/{p.name}") for p in snapshots)
    # The root summary is naturally included from final-gate. Verify the exact selected file was among inputs.
    if not any(path.resolve() == summary.resolve() for path, _ in members):
        fail(f"root summary was excluded from archive inputs: {summary}")

    seen = set()
    records = []
    temp_name = None
    try:
        with tempfile.NamedTemporaryFile(prefix="p5-closure-", suffix=".tar.gz",
                                         dir=output.parent, delete=False) as temp:
            temp_name = Path(temp.name)
        with tarfile.open(temp_name, "w:gz") as tar:
            for source, arcname in members:
                if arcname in seen:
                    fail(f"duplicate archive member path: {arcname}")
                seen.add(arcname)
                data = source.read_bytes()
                info = tarfile.TarInfo(arcname)
                info.size = len(data)
                info.mode = source.stat().st_mode & 0o777
                with __import__("io").BytesIO(data) as stream:
                    tar.addfile(info, stream)
                records.append({"member": arcname, "original_path": str(source.resolve()),
                                "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})

        with tarfile.open(temp_name, "r:gz") as tar:
            actual = tar.getmembers()
            if len(actual) != len(records):
                fail("tar member count differs from manifest records")
            for info, record in zip(actual, records):
                if info.name != record["member"]:
                    fail(f"tar member order/name mismatch: {info.name}")
                data = tar.extractfile(info).read()
                if len(data) != record["bytes"] or hashlib.sha256(data).hexdigest() != record["sha256"]:
                    fail(f"archived bytes failed verification: {info.name}")
                if data != Path(record["original_path"]).read_bytes():
                    fail(f"archived bytes differ from source: {info.name}")

        manifest = {"archive": str(output), "archive_sha256": hashlib.sha256(temp_name.read_bytes()).hexdigest(),
                    "root_summary": str(summary.resolve()), "members": records}
        with manifest_path.open("x") as f:
            json.dump(manifest, f, indent=2, ensure_ascii=False)
            f.write("\n")
        # Exclusive hard-link install prevents replacing a concurrently-created archive.
        temp_name.link_to(output)
        temp_name.unlink()
        temp_name = None
    finally:
        if temp_name is not None:
            temp_name.unlink(missing_ok=True)
    print(f"archive={output}")
    print(f"archive_sha256={manifest['archive_sha256']}")
    print(f"manifest={manifest_path}")
    print(f"members={len(records)}")
    print("gzip_reopened_and_every_member_byte_verified=true")
    return 0


if __name__ == "__main__":
    sys.exit(main())
