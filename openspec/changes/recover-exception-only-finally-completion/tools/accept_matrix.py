#!/usr/bin/env python3
"""Read-only acceptance gate for compare.py execution.json; invokes no tools."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('execution', type=Path)
    ap.add_argument('--expect', choices=('baseline', 'recovered'), required=True)
    a = ap.parse_args()
    path = a.execution.resolve(strict=True)
    doc = json.loads(path.read_text(encoding='utf-8'))
    mode = 'baseline' if a.expect == 'baseline' else 'final'
    require(doc.get('schema') == 'exception-only-finally-comparison-v1', 'wrong receipt schema')
    require(doc.get('status') == 'complete_pending_review', 'runner did not complete cleanly')
    require(doc.get('mode') == mode, f'expected mode {mode}')
    require(doc.get('source_pins_unchanged') is True, 'repository source closure changed')
    require(doc.get('repository_target_absent_after') is True, 'repository target directory exists')
    require(doc.get('inputs_and_tools_unchanged') is True, 'an input/tool pin changed')

    commands = doc.get('commands', [])
    require(commands and [r.get('index') for r in commands] == list(range(1, len(commands) + 1)), 'command ledger is not contiguous')
    streams = {}
    for row in commands:
        require(row.get('guard_stop') is None, f"guard stopped at command {row.get('index')}")
        require(isinstance(row.get('exit_code'), int), f"command {row.get('index')} lacks exit code")
        for kind in ('stdout', 'stderr'):
            rec = row.get('streams', {}).get(kind)
            require(isinstance(rec, dict), f"command {row['index']} lacks {kind} artifact")
            raw = Path(rec['path']).resolve(strict=True)
            require(raw.is_file() and raw.stat().st_size == rec.get('bytes') and sha(raw) == rec.get('sha256'),
                    f"command {row['index']} {kind} artifact pin mismatch")
            streams[(row['index'], kind)] = raw.read_bytes()

    def command(index: int) -> dict:
        return commands[index - 1]

    require(len(doc.get('original_oracles', [])) == 4, 'expected four original JVM oracle runs')
    require(all(x.get('nine_cases_valid') is True and command(x['command'])['exit_code'] == 0
                for x in doc['original_oracles']), 'original oracle run failed validation')
    require(len(doc.get('origin_classes', [])) == 2 and all(x.get('exact_bytes_match') is True
                for x in doc['origin_classes']), 'fixture source did not reproduce both pinned class inputs')

    exception_cells = [x for x in doc.get('jarde_cells', []) if x.get('label', '').startswith('exception-only-')]
    require(len(exception_cells) == 4, 'expected Jarde default/all for both compiler inputs')
    total_runs = 0
    for cell in exception_cells:
        require(cell.get('exit_code') == 0 and cell.get('json_artifact') and cell.get('source_artifact'),
                f"Jarde source missing for {cell.get('label')}/{cell.get('evidence')}")
        for name in ('json_artifact', 'source_artifact'):
            rec = cell[name]; artifact = Path(rec['path']).resolve(strict=True)
            require(artifact.stat().st_size == rec['bytes'] and sha(artifact) == rec['sha256'], f"artifact pin mismatch: {artifact}")
        require(Path(cell['source_artifact']['path']).read_bytes(), 'empty whole-source artifact')
        for comp in cell.get('compiles', []):
            row = command(comp['command'])
            if row['exit_code'] != 0:
                require(a.expect == 'baseline', f"recovered source compile failed: {cell['label']}/{comp['compiler']}")
                require(comp.get('runs') == [], 'runtime exists despite failed source compilation')
                continue
            require(comp.get('class') is not None, 'successful javac emitted no target class')
            require(len(comp.get('runs', [])) == 2, 'successful rebuild needs both JVM runs')
            for run in comp['runs']:
                total_runs += 1
                require(run.get('nine_cases_valid') is True, f"nine-case oracle mismatch: {cell['label']}/{comp['compiler']}/{run['runtime']}")
                require(command(run['command'])['exit_code'] == 0, 'rebuilt runtime exited nonzero')
                if a.expect == 'recovered':
                    require(run.get('matches_original_oracle') is True, 'recovered output differs from original oracle')
    if a.expect == 'recovered':
        require(total_runs == 16, f'expected sixteen rebuilt JVM runs, got {total_runs}')
    else:
        require(total_runs <= 16, 'unexpected rebuilt runtime count')

    sealed = doc.get('sealed_controls', [])
    require(len(sealed) == 8, 'expected eight sealed full-class source controls')
    expected_missing = 3 if a.expect == 'baseline' else 2
    for item in sealed:
        require(item.get('missing_return_count') == expected_missing,
                f"sealed diagnostic count mismatch: {item.get('input_compiler')}/{item.get('evidence')}/{item.get('build_compiler')}")
        require(len(item.get('method_order', [])) == 7, 'sealed control lost an original method')
        require(item.get('source') and item.get('json'), 'sealed full source/JSON artifact missing')

    jadx_rows = doc.get('jadx_runs', [])
    require(len(jadx_rows) == 2, 'expected JADX 1.5.6 on both compiler inputs')
    for item in jadx_rows:
        # JADX is an observation: compiler/runtime failures are retained, never promoted to a gate failure.
        for comp in item.get('compiles', []):
            require(command(comp['command'])['exit_code'] == comp['exit_code'], 'JADX compile outcome differs from command ledger')
    report = {'status': 'accepted', 'expect': a.expect, 'execution': str(path),
              'commands': len(commands), 'rebuilt_jvm_runs': total_runs,
              'original_oracles': 4, 'sealed_controls': len(sealed), 'jadx_inputs': len(jadx_rows)}
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f'acceptance failed: {type(exc).__name__}: {exc}', file=sys.stderr)
        raise SystemExit(1)
