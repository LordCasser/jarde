#!/usr/bin/env python3
"""Package a named evidence closure, then reopen every tar member and verify SHA-256.

This is preparation only. It does not run unless explicitly invoked with --out.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import tarfile
import tempfile

ROOT = Path('/private/tmp')
REPO = Path('/Users/lordcasser/workspace/projects/jarde')
BASE_DIRS = [
    ROOT / 'jarde-indexed-list-signatures-native-acceptance-execution-root-v3-20261019',
    ROOT / 'jarde-indexed-list-signatures-native-acceptance-execution-root-v4-20261019',
    ROOT / 'jarde-indexed-list-signatures-native-acceptance-root-v4-20261019',
    ROOT / 'jarde-indexed-list-signatures-final-acceptance-prep-luna-v8-20261019',
    ROOT / 'jarde-indexed-list-signatures-workspace-seed1-root-v3-20261019',
    ROOT / 'jarde-indexed-list-signatures-reader-and-remainder-root-v1-20261019',
    ROOT / 'jarde-indexed-list-signatures-own-ci-status-root-v8-20261019',
    ROOT / 'jarde-indexed-list-signatures-final-acceptance-prep-root-v9-20261019',
    ROOT / 'jarde-indexed-list-signatures-own-ci-execution-prep-root-v1-20261019',
    ROOT / 'jarde-indexed-list-signatures-own-ci-execution-prep-root-v2-20261019',
    ROOT / 'jarde-indexed-list-signatures-own-ci-execution-root-v3-20261019',
    ROOT / 'jarde-indexed-list-signatures-own-ci-execution-root-v4-20261019',
]
BASE_FILES = []  # Archive immutable execution evidence; the live ledger is tracked by Git.
EXCLUDED_NAMES = {'candidate-cli'}
EXCLUDED_PARTS = {'__pycache__'}


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def accepted_ci_tree(path: Path) -> bool:
    if not path.is_dir():
        return False
    receipts = [path / 'acceptance.json', path / 'review.json', path / 'ci-acceptance.json']
    for receipt in receipts:
        if not receipt.is_file():
            continue
        try:
            data = json.loads(receipt.read_text())
        except (OSError, json.JSONDecodeError):
            continue
        if data.get('status') == 'accepted' or data.get('accepted') is True:
            return True
    return False


def selected_sources(ci_dir: Path | None) -> list[tuple[Path, str]]:
    pairs: list[tuple[Path, str]] = []
    for root in BASE_DIRS:
        if not root.is_dir():
            raise SystemExit(f'missing required evidence directory: {root}')
        for path in sorted(root.rglob('*')):
            if not path.is_file() or path.is_symlink():
                continue
            if path.name in EXCLUDED_NAMES or path.suffix == '.pyc' or any(part in EXCLUDED_PARTS for part in path.parts):
                continue
            rel = PurePosixPath('evidence', root.name, *path.relative_to(root).parts).as_posix()
            pairs.append((path, rel))
    for path in BASE_FILES:
        if not path.is_file():
            raise SystemExit(f'missing ledger file: {path}')
        rel = PurePosixPath('ledger', path.name).as_posix()
        pairs.append((path, rel))
    if ci_dir is not None:
        if not accepted_ci_tree(ci_dir):
            raise SystemExit(f'CI directory is absent or lacks an explicit accepted receipt: {ci_dir}')
        for path in sorted(ci_dir.rglob('*')):
            if not path.is_file() or path.is_symlink():
                continue
            if path.name in EXCLUDED_NAMES or path.suffix == '.pyc' or any(part in EXCLUDED_PARTS for part in path.parts):
                continue
            rel = PurePosixPath('conditional-ciout-v3', ci_dir.name, *path.relative_to(ci_dir).parts).as_posix()
            pairs.append((path, rel))
    names = [name for _, name in pairs]
    if len(names) != len(set(names)):
        raise SystemExit('duplicate archive member path')
    return pairs


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True, type=Path, help='destination tar.gz path')
    parser.add_argument('--ciout-v3', type=Path, help='optional accepted CIout-v3 directory')
    args = parser.parse_args()
    if args.out.exists():
        raise SystemExit("archive output must be fresh")
    files = selected_sources(args.ciout_v3)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='jarde-indexed-evidence-') as temp:
        stage = Path(temp)
        records = []
        for source, member in files:
            target = stage / member
            target.parent.mkdir(parents=True, exist_ok=True)
            data = source.read_bytes()
            target.write_bytes(data)
            if target.read_bytes() != source.read_bytes():
                raise SystemExit(f"source changed during capture: {source}")
            records.append({'source': str(source), 'path': member, 'bytes': target.stat().st_size, 'sha256': sha256(target)})
        manifest = {
            'product': '5bd65ef2f36999c3d0a3b4477a412a0ef532c956',
            'cli_sha256': '90b30ad1132bdc904c20f1b4624347b84d37637c78f1f3e291ce2f0dea91f64c',
            'ciout_v3_included': args.ciout_v3 is not None,
            'members': records,
        }
        manifest_path = stage / 'archive-manifest.json'
        manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
        manifest_pin = {'bytes': manifest_path.stat().st_size, 'sha256': sha256(manifest_path)}
        with tarfile.open(args.out, 'w:gz') as archive:
            for path in sorted(stage.rglob('*')):
                if path.is_file():
                    archive.add(path, arcname=path.relative_to(stage).as_posix(), recursive=False)

    # Reopen the produced archive and verify exact member set, sizes and hashes.
    expected = {record['path']: record for record in records}
    expected['archive-manifest.json'] = manifest_pin
    with tarfile.open(args.out, 'r:gz') as archive:
        members = [m for m in archive.getmembers() if m.isfile()]
        actual_names = {m.name for m in members}
        if len(members) != len(expected) or actual_names != set(expected):
            raise SystemExit(f'archive member mismatch: missing={set(expected)-actual_names}, extra={actual_names-set(expected)}')
        for member in members:
            normalized = PurePosixPath(member.name)
            if normalized.is_absolute() or '..' in normalized.parts:
                raise SystemExit(f'unsafe archive member: {member.name}')
            stream = archive.extractfile(member)
            if stream is None:
                raise SystemExit(f'cannot reopen {member.name}')
            h = hashlib.sha256()
            size = 0
            for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                size += len(chunk)
                h.update(chunk)
            pin = expected[member.name]
            if size != pin['bytes'] or h.hexdigest() != pin['sha256']:
                raise SystemExit(f'member verification failed: {member.name}')
    print(json.dumps({'archive': str(args.out), 'members': len(expected), 'verified': True}, sort_keys=True))

if __name__ == '__main__':
    main()
