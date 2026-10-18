#!/usr/bin/env python3
"""Re-open every current loop-arm capture archive and verify its member manifest."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

RESULTS = Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-loop-arm-returns/results')
CF11_ROOTS = (
    Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-18/cf11-sequential-loops-baseline-root-v1'),
    Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-18/cf11-sequential-loops-baseline-matrix-root-v1'),
)
AUDIT_SOURCE = Path('/private/tmp/jarde-prefixed-retry-all-archives-luna-v1-20261018/audit_archives.py')
AUDIT_SOURCE_BYTES = 7042
AUDIT_SOURCE_SHA256 = '238a90dc4cd1b903753f1b9dc033150ae219bf25b6333ae3e3faa4a56beb4c84'


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def need(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def pin(path: Path) -> dict:
    need(not path.is_symlink(), f'refusing symlink input: {path}')
    path = path.resolve(strict=True)
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}


def discover(root: Path) -> list[Path]:
    need(root.is_dir(), f'archive root missing: {root}')
    found = sorted({*root.rglob('*.tar.gz'), *root.rglob('*.tgz')})
    for archive in found:
        need(archive.is_file() and not archive.is_symlink(), f'invalid archive path: {archive}')
    return found


def inventory(archives: list[Path]) -> dict[str, dict]:
    result = {}
    for archive in archives:
        key = str(archive.resolve())
        need(key not in result, f'duplicate archive path: {key}')
        result[key] = {'archive': pin(archive), 'review': pin(archive.parent / 'capture-review.json')}
    return result


def load_audit_one():
    raw = AUDIT_SOURCE.read_bytes()
    need(len(raw) == AUDIT_SOURCE_BYTES and sha(raw) == AUDIT_SOURCE_SHA256,
         'accepted audit_one source pin mismatch')
    spec = importlib.util.spec_from_file_location('accepted_archive_audit_one', AUDIT_SOURCE)
    need(spec is not None and spec.loader is not None, 'cannot load accepted audit_one source')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    need(callable(getattr(module, 'audit_one', None)), 'accepted source has no audit_one function')
    return module.audit_one, pin(AUDIT_SOURCE)


def audit_group(archives: list[Path], audit_one) -> dict:
    entries, errors = [], []
    for archive in archives:
        try:
            entries.append(audit_one(archive))
        except Exception as exc:
            errors.append({'archive': str(archive), 'error': f'{type(exc).__name__}: {exc}'})
    return {'archive_count_discovered': len(archives), 'archive_count_verified': len(entries),
            'error_count': len(errors), 'archives': entries, 'errors': errors}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    out = args.out.resolve()
    need(not out.exists(), f'refusing existing output: {out}')
    script_pin = pin(Path(__file__))
    audit_one, source_pin = load_audit_one()

    leaf_before_archives = discover(RESULTS)
    cf11_before = {str(root): discover(root) for root in CF11_ROOTS}
    leaf_before = inventory(leaf_before_archives)
    cf11_before_inventory = {root: inventory(archives) for root, archives in cf11_before.items()}

    leaf = audit_group(leaf_before_archives, audit_one)
    cf11 = {root: audit_group(archives, audit_one) for root, archives in cf11_before.items()}

    leaf_after_archives = discover(RESULTS)
    cf11_after = {str(root): discover(root) for root in CF11_ROOTS}
    need([str(p.resolve()) for p in leaf_after_archives] == [str(p.resolve()) for p in leaf_before_archives],
         'loop-arm archive set changed during audit; rerun against stable inputs')
    need({root: [str(p.resolve()) for p in paths] for root, paths in cf11_after.items()}
         == {root: [str(p.resolve()) for p in paths] for root, paths in cf11_before.items()},
         'CF11 archive set changed during audit; rerun against stable inputs')
    need(inventory(leaf_after_archives) == leaf_before, 'loop-arm archive/review bytes changed during audit')
    cf11_after_inventory = {root: inventory(paths) for root, paths in cf11_after.items()}
    need(cf11_after_inventory == cf11_before_inventory, 'CF11 archive/review bytes changed during audit')
    need(pin(AUDIT_SOURCE) == source_pin, 'accepted audit_one source changed during audit')
    need(pin(Path(__file__)) == script_pin, 'audit script changed during audit')

    leaf_ok = leaf['archive_count_discovered'] > 0 and leaf['error_count'] == 0
    cf11_ok = all(group['archive_count_discovered'] == 1 and group['error_count'] == 0 for group in cf11.values())
    report = {
        'schema': 'loop-arm-all-archives-readonly-audit-v3',
        'status': 'passed' if leaf_ok and cf11_ok else 'failed',
        'audit_script': script_pin,
        'audit_one_source': source_pin,
        'loop_arm_results': {'root': str(RESULTS), **leaf},
        'cf11_evidence_separate': {
            'root_count': len(CF11_ROOTS),
            'roots': {root: {'root': root, **group} for root, group in cf11.items()},
            'aggregate_archive_count': sum(g['archive_count_discovered'] for g in cf11.values()),
            'aggregate_verified_count': sum(g['archive_count_verified'] for g in cf11.values()),
            'aggregate_error_count': sum(g['error_count'] for g in cf11.values()),
        },
        'input_stability': {'archive_sets_unchanged': True, 'archive_and_review_pins_unchanged': True,
                            'audit_source_unchanged': True},
        'limitations': ['Audit reopens members directly from tar streams; no archive is extracted to disk.',
                        'The count includes only archives present at this run; future own-product-ci or closing-strict archives require rerunning this script.',
                        'CF11 evidence is audited and counted separately from this OpenSpec leaf.'],
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(json.dumps({'status': report['status'], 'loop_arm_archives': leaf['archive_count_discovered'],
                      'loop_arm_verified': leaf['archive_count_verified'], 'loop_arm_errors': leaf['error_count'],
                      'cf11_archives': report['cf11_evidence_separate']['aggregate_archive_count'],
                      'cf11_errors': report['cf11_evidence_separate']['aggregate_error_count'], 'out': str(out)},
                     ensure_ascii=False, sort_keys=True))
    return 0 if report['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
