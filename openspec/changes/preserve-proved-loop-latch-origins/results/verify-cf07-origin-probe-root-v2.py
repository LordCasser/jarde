#!/usr/bin/env python3
"""Verify the captured method-scoped diagnostic; no product acceptance is implied."""
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
OUT = HERE / 'cf07-origin-probe-root-v1'
sha = lambda data: hashlib.sha256(data).hexdigest()

def differences(a, b, path=''):
    assert type(a) is type(b), path
    if isinstance(a, dict):
        assert a.keys() == b.keys(), path
        return sum((differences(a[k], b[k], path+'/'+k) for k in a), [])
    if isinstance(a, list):
        assert len(a) == len(b), path
        return sum((differences(x, y, path+'/'+str(i)) for i, (x, y) in enumerate(zip(a,b))), [])
    return [] if a == b else [{'path': path, 'default': a, 'probe': b}]

def main():
    result = OUT / 'diagnostic-acceptance-root-v2.json'
    assert not result.exists()
    execution_raw = (OUT/'execution.json').read_bytes()
    execution = json.loads(execution_raw)
    # The first capture incorrectly required equality of wall-clock usage. Preserve that failure.
    assert execution['status'] == 'failed' and execution['stdout_equal'] is False
    assert execution['error'] == 'AssertionError()'
    assert len(execution['commands']) == 2
    for command in execution['commands']:
        assert command['exit_code'] == 0 and command['guard_stop'] is None
        for kind in ('stdout', 'stderr'):
            record = command['streams'][kind]
            raw = (ROOT/record['path']).read_bytes()
            assert len(raw) == record['bytes'] and sha(raw) == record['sha256']
    cli = execution['cli']
    assert sha(Path(cli['path']).read_bytes()) == cli['sha256']
    assert cli['sha256'] == '5ef3fd07b554448f39f6339e516f302aac9c997b65d09759ed0045af2b888cd5'
    cls = execution['class']
    assert sha((ROOT/cls['path']).read_bytes()) == cls['sha256']
    assert len((ROOT/cls['path']).read_bytes()) == cls['bytes']
    assert sha((ROOT/'openspec/changes/recover-prefixed-one-arm-loops/results/arm-loop-diagnostic-luna-v1/execution.json').read_bytes()) == execution['diagnostic_build_execution_sha256']
    for relative, expected in execution['product_pins_before'].items():
        assert sha((ROOT/relative).read_bytes()) == expected, relative
    a = json.loads((OUT/'0.stdout.raw').read_bytes())
    b = json.loads((OUT/'1.stdout.raw').read_bytes())
    delta = differences(a, b)
    allowed = re.compile(r'^/(?:usage|execution/usage|methods/[0-9]+/outcome/(?:analysis|report)/execution/usage)/elapsed_millis$')
    assert delta and all(allowed.fullmatch(d['path']) and type(d['default']) is int and type(d['probe']) is int for d in delta)
    assert not (OUT/'0.stderr.raw').read_bytes()
    lines = (OUT/'1.stderr.raw').read_text().splitlines()
    assert lines and all(line.startswith('JRE_ARM_LOOP_PROBE ') for line in lines)
    facts = {}
    for name, desc, header, tree_snippets in (
        ('counted', '(II)I', 6, ['branch_bci: 14', 'bci: 17, path: []', 'bci: 23, path: []', 'join: Some(CanonicalBlockId { bci: 27, path: [] })', 'gateway_origins: [30]']),
        ('lastIndexOf', '([IIII)I', 5, ['branch_bci: 16', 'bci: 19, path: []', 'bci: 22, path: []', 'join: None', 'gateway_origins: []']),
    ):
        identity = f'cf07/LoopCases/{name}{desc}'
        begin = f'JRE_ARM_LOOP_PROBE method={identity} stage=recover-begin'
        end = f'JRE_ARM_LOOP_PROBE method={identity} stage=recover-end outcome=ok'
        assert lines.count(begin) == lines.count(end) == 1
        lo, hi = lines.index(begin), lines.index(end)
        assert lo < hi
        candidates = [line for line in lines[lo+1:hi] if f'stage=loop-region-return header={header} ' in line]
        assert len(candidates) == 1 and all(s in candidates[0] for s in tree_snippets)
        facts[identity] = {'loop_header': header, 'raw_loop_return': candidates[0]}
    accepted = {'schema':'cf07-remaining-origin-diagnostic-acceptance-root-v2', 'status':'accepted-diagnostic-only', 'execution_sha256':sha(execution_raw), 'capture_status_preserved':'failed', 'raw_stdout_equal':False, 'only_elapsed_usage_differs':True, 'elapsed_usage_differences':delta, 'method_facts':facts, 'product_pins_current_equal':True, 'scope':'Recorded Region trees and unchanged non-timing class-source JSON; no new syntax or complete source-map acceptance'}
    result.write_text(json.dumps(accepted,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':accepted['status'],'methods':list(facts),'only_elapsed_usage_differs':True}))

if __name__ == '__main__':
    main()
