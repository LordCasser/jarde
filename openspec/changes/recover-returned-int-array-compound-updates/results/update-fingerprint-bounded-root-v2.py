#!/usr/bin/env python3
"""Update only the intentional returned-update inputs while Cargo is disk-stopped.
Requires temporary Python blake3; the unchanged official Rust verifier remains the CI gate.
"""
import hashlib, json
from pathlib import Path
import blake3
ROOT = Path(__file__).resolve().parents[4]
DEST = Path(__file__).resolve().parent / 'fingerprint-bounded-root-v2.json'
assert not DEST.exists()
path = ROOT / 'tests/fixtures/corpus-fingerprint.json'
old_bytes = path.read_bytes()
m = json.loads(old_bytes)
old = {row['path']: row for row in m['files']}
scope = m['scope']
actual = {}
for root_name in scope['roots']:
 for file in (ROOT / root_name).rglob('*'):
  if not file.is_file() or file.is_symlink(): continue
  relative = file.relative_to(ROOT).as_posix()
  if set(file.relative_to(ROOT / root_name).parts[:-1]) & set(scope['excluded_directories']): continue
  if file.name in scope['excluded_file_names'] or file.suffix[1:] in scope['excluded_extensions']: continue
  b = file.read_bytes()
  actual[relative] = {'blake3': blake3.blake3(b).hexdigest(), 'bytes': len(b), 'path': relative}
assert set(old) <= set(actual), 'old input disappeared'
assert all(actual[p] == row for p,row in old.items()), 'old input changed'
added = sorted(set(actual) - set(old))
assert len(added) == 21 and all(p.startswith('tests/fixtures/returned-int-array-compound-updates/') for p in added), added
# corpus_files uses global lexicographic sorting. No classification or existing row changes.
m['files'] = [actual[p] for p in sorted(actual)]
new_bytes = (json.dumps(m,ensure_ascii=False,indent=2,sort_keys=True)+'\n').encode()
path.write_bytes(new_bytes)
DEST.write_text(json.dumps({'scope':'bounded Python fingerprint-only update; not a Rust test pass', 'python_blake3_version':'1.0.8', 'manifest_sha256_before':hashlib.sha256(old_bytes).hexdigest(), 'manifest_sha256_after':hashlib.sha256(new_bytes).hexdigest(), 'old_file_count':len(old), 'new_file_count':len(actual), 'added':added, 'old_rows_unchanged':True, 'classification_unchanged':True, 'official_rust_verification':'pending; cargo stopped below 20 GiB'},indent=2)+'\n')
print(DEST.read_text())
