#!/usr/bin/env python3
"""Reversibly compress old jarde frozen CLIs; retain current three decompiler slices."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import stat
import time

TMP = Path('/private/tmp')
REPO = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = Path(__file__).resolve().parent
OUT = TMP / 'jarde-historical-frozen-clis-gzip-v1'
MANIFEST = RESULTS / 'historical-cli-compression-v1.json'
KEEP = {'jarde-child-array-covariance-cli-v1', 'jarde-constructor-primitive-conversions-cli-v1',
        'jarde-em18-composition-cli-v1', 'jarde-em18-composition-cli-v2', 'jarde-em18-candidate-v1-cli'}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    if OUT.exists() or MANIFEST.exists():
        raise SystemExit('refusing existing compression evidence')
    candidates = sorted(set(TMP.glob('jarde*-cli')) | set(TMP.glob('jarde*-cli-v*')))
    selected = []
    for path in candidates:
        if path.name in KEEP or path.is_symlink() or not path.is_file():
            continue
        meta = path.stat()
        if not stat.S_ISREG(meta.st_mode) or not meta.st_mode & stat.S_IXUSR:
            continue
        with path.open('rb') as stream:
            # Native macOS executable magics only; do not compress logs, scripts or arbitrary files.
            if stream.read(4) not in (b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\xca\xfe\xba\xbe'):
                continue
        selected.append(path)
    OUT.mkdir()
    record = {'schema': 'jarde-historical-cli-reversible-compression-v1', 'free_before': shutil.disk_usage(REPO).free,
              'kept_uncompressed': sorted(KEEP), 'items': [],
              'restore': 'gzip.decompress(archive.read_bytes()) -> original_path; restore recorded mode; verify original_sha256',
              'scope': 'only old project-named Mach-O executables under /private/tmp; no source/results/other project targets'}
    MANIFEST.write_text(json.dumps(record, indent=2) + '\n')
    for path in selected:
        original = path.read_bytes()
        meta = path.stat()
        archive = OUT / (path.name + '.gz')
        compressed = gzip.compress(original, compresslevel=6, mtime=0)
        with archive.open('xb') as stream:
            stream.write(compressed)
        if gzip.decompress(archive.read_bytes()) != original or path.read_bytes() != original:
            raise RuntimeError('archive/original changed: ' + str(path))
        row = {'original_path': str(path), 'original_bytes': len(original), 'original_sha256': digest(original),
               'original_mode': stat.S_IMODE(meta.st_mode), 'original_mtime_ns': meta.st_mtime_ns,
               'archive_path': str(archive), 'archive_bytes': len(compressed), 'archive_sha256': digest(compressed),
               'decompressed_bytes_verified': True}
        path.unlink()
        row['uncompressed_removed'] = not path.exists()
        record['items'].append(row)
        record['free_after'] = shutil.disk_usage(REPO).free
        MANIFEST.write_text(json.dumps(record, indent=2) + '\n')
    record['saved_bytes'] = sum(r['original_bytes'] - r['archive_bytes'] for r in record['items'])
    record['free_after'] = shutil.disk_usage(REPO).free
    record['status'] = 'complete'
    MANIFEST.write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps({'archives': len(record['items']), 'saved_bytes': record['saved_bytes'],
                      'free_before': record['free_before'], 'free_after': record['free_after']}))


if __name__ == '__main__':
    main()
