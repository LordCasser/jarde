#!/usr/bin/env python3
"""Read-only verification of capture.tar.gz archives against adjacent reviews/manifests."""
from __future__ import annotations
import hashlib
import json
import re
import sys
import tarfile
from pathlib import Path, PurePosixPath

sys.dont_write_bytecode = True
RESULTS = Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-prefixed-exceptional-retry-loops/results')
OUT = Path('/private/tmp/jarde-prefixed-retry-all-archives-luna-v1-20261018')
HEX = re.compile(r'^[0-9a-f]{64}$')

class AuditError(Exception):
    pass

def need(ok: bool, msg: str) -> None:
    if not ok:
        raise AuditError(msg)

def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def sha_member(tf: tarfile.TarFile, member: tarfile.TarInfo) -> tuple[int, str]:
    stream = tf.extractfile(member)
    need(stream is not None, f'{member.name}: cannot read regular member')
    digest = hashlib.sha256()
    size = 0
    with stream:
        while True:
            block = stream.read(1024 * 1024)
            if not block:
                break
            size += len(block)
            digest.update(block)
    return size, digest.hexdigest()

def safe_member_name(name: str) -> bool:
    if not name or '\x00' in name or '\\' in name or name.startswith('/'):
        return False
    if re.match(r'^[A-Za-z]:', name):
        return False
    parts = name.split('/')
    if any(part in ('', '.', '..') for part in parts):
        return False
    path = PurePosixPath(name)
    return not path.is_absolute() and path.as_posix() == name

def audit_one(archive: Path) -> dict:
    review_path = archive.parent / 'capture-review.json'
    need(review_path.is_file() and not review_path.is_symlink(), f'{archive.parent.name}: capture-review.json missing or symlink')
    review = json.loads(review_path.read_text(encoding='utf-8'))
    raw = archive.read_bytes()
    archive_bytes, archive_sha = len(raw), sha_bytes(raw)
    need(review.get('archive_bytes') == archive_bytes, f'{archive.parent.name}: archive byte count differs from capture-review')
    need(review.get('archive_sha256') == archive_sha, f'{archive.parent.name}: archive SHA-256 differs from capture-review')
    need(type(review.get('members_reopened')) is int, f'{archive.parent.name}: invalid members_reopened')

    with tarfile.open(archive, mode='r:gz') as tf:
        members = tf.getmembers()
        names = [m.name for m in members]
        need(len(names) == len(set(names)), f'{archive.parent.name}: duplicate tar member path')
        for m in members:
            need(safe_member_name(m.name), f'{archive.parent.name}: unsafe/noncanonical path {m.name!r}')
            need(m.isfile() and not m.issym() and not m.islnk(), f'{archive.parent.name}: non-regular tar member {m.name!r}')
            need(m.size >= 0, f'{archive.parent.name}: negative tar size for {m.name!r}')
        need(review['members_reopened'] == len(members), f'{archive.parent.name}: member count differs from capture-review')
        need('manifest.json' in names, f'{archive.parent.name}: embedded manifest missing')
        manifest_member = next(m for m in members if m.name == 'manifest.json')
        stream = tf.extractfile(manifest_member)
        need(stream is not None, f'{archive.parent.name}: cannot read manifest')
        with stream:
            manifest_bytes = stream.read()
        manifest = json.loads(manifest_bytes.decode('utf-8'))
        need(isinstance(manifest, dict), f'{archive.parent.name}: manifest is not a member map')
        expected_names = set(manifest)
        actual_names = set(names) - {'manifest.json'}
        need(expected_names == actual_names,
             f'{archive.parent.name}: manifest/member set mismatch (missing={sorted(expected_names-actual_names)[:3]}, unlisted={sorted(actual_names-expected_names)[:3]})')
        checked = []
        for m in members:
            if m.name == 'manifest.json':
                continue
            expected = manifest[m.name]
            need(isinstance(expected, dict) and set(expected) == {'bytes', 'sha256'},
                 f'{archive.parent.name}: malformed manifest entry for {m.name!r}')
            need(type(expected['bytes']) is int and expected['bytes'] >= 0 and isinstance(expected['sha256'], str)
                 and HEX.fullmatch(expected['sha256']) is not None,
                 f'{archive.parent.name}: invalid manifest digest record for {m.name!r}')
            actual_size, actual_sha = sha_member(tf, m)
            need(m.size == actual_size == expected['bytes'], f'{archive.parent.name}: byte count mismatch for {m.name!r}')
            need(actual_sha == expected['sha256'], f'{archive.parent.name}: SHA-256 mismatch for {m.name!r}')
            checked.append({'path': m.name, 'bytes': actual_size, 'sha256': actual_sha})
    return {
        'name': archive.parent.name,
        'archive': {'path': str(archive), 'bytes': archive_bytes, 'sha256': archive_sha,
                    'capture_review_path': str(review_path), 'capture_review_members_reopened': review['members_reopened']},
        'tar_member_count': len(names),
        'manifest_entry_count': len(checked),
        'manifest_sha256': sha_bytes(manifest_bytes),
        'checks': {'archive_review_pin': 'passed', 'unique_paths': 'passed', 'safe_canonical_paths': 'passed',
                   'regular_members_only': 'passed', 'manifest_set_exact': 'passed',
                   'member_bytes_and_sha256': 'passed'},
        'members': checked,
    }

def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    archives = sorted(RESULTS.glob('*/capture.tar.gz'))
    entries, errors = [], []
    for archive in archives:
        try:
            entries.append(audit_one(archive))
        except Exception as exc:
            errors.append({'name': archive.parent.name, 'error': f'{type(exc).__name__}: {exc}'})
    report = {
        'schema': 'prefixed-retry-capture-archive-audit-v1',
        'status': 'passed' if archives and not errors else 'failed',
        'results_root': str(RESULTS),
        'archive_count_discovered': len(archives),
        'archive_count_verified': len(entries),
        'error_count': len(errors),
        'archives': entries,
        'errors': errors,
        'limitations': ['This audit covers only capture.tar.gz files discovered at execution time.',
                       'The later final own-CI archive was not present in this audit and is not claimed as verified.',
                       'No archive was extracted to disk; member bytes were streamed from tar readers.'],
    }
    output = OUT / 'archive-audit.json'
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': report['status'], 'archives': len(archives),
                      'verified': len(entries), 'errors': len(errors), 'output': str(output)}, ensure_ascii=False))
    return 0 if report['status'] == 'passed' else 1

if __name__ == '__main__':
    raise SystemExit(main())
