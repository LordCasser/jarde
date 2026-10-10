"""Verify original Java boundary inputs; no candidate or recovery acceptance."""
from pathlib import Path
import hashlib, json

HERE = Path(__file__).resolve().parent
OUT = HERE / 'boundary-preflight-root-v1'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
d = json.loads((OUT / 'execution.json').read_bytes())
assert d['status'] == 'observed-original-boundary-runtime'
assert len(d['commands']) == 12 and len(d['class_files']) == 8
assert sha(Path(d['runner']['path'])) == d['runner']['sha256']
for p, h in d['source_files'].items():
    assert sha(OUT / p) == h
for tools in d['jdk_tools'].values():
    for ref in tools.values():
        assert sha(Path(ref['path'])) == ref['sha256']
expected = [jdk + '-' + debug + '-' + action for jdk in ('javac8', 'javac23')
            for debug in ('debug', 'no-debug') for action in ('compile', 'runtime', 'javap')]
assert [c['label'] for c in d['commands']] == expected
for c in d['commands']:
    assert c['exit_code'] == 0
    for ref in c['streams'].values():
        p = OUT / ref['path']
        assert p.stat().st_size == ref['bytes'] and sha(p) == ref['sha256']
for p, ref in d['class_files'].items():
    path = OUT / p
    assert path.stat().st_size == ref['bytes'] and sha(path) == ref['sha256']
    data = path.read_bytes()
    assert data[:4] == b'\xca\xfe\xba\xbe' and int.from_bytes(data[6:8], 'big') == 52
cases = [jdk + '-' + debug for jdk in ('javac8', 'javac23') for debug in ('debug', 'no-debug')]
for stream in ('stdout', 'stderr'):
    raw = [(OUT / (c + '-runtime.' + stream + '.raw')).read_bytes() for c in cases]
    assert len(set(raw)) == 1
    if stream == 'stderr':
        assert raw[0] == b''
for jdk in ('javac8', 'javac23'):
    assert 'LocalVariableTable:' in (OUT / (jdk + '-debug-javap.stdout.raw')).read_text()
    assert 'LocalVariableTable:' not in (OUT / (jdk + '-no-debug-javap.stdout.raw')).read_text()
result = {'schema': 'typed-local-original-boundary-acceptance-root-v1',
          'status': 'accepted-original-inputs-only', 'commands': 12, 'class_files': 8,
          'original_four_runtime_legs_equal': True, 'debug_and_no_debug': True,
          'execution_sha256': sha(OUT / 'execution.json'),
          'verifier_sha256': sha(Path(__file__)),
          'scope': 'Only original Java inputs, actual compilers/class bytes/debug facts and original runtime. No JADX/Jarde candidate or type-proof acceptance.'}
with (OUT / 'acceptance-root-v1.json').open('x') as f:
    f.write(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
