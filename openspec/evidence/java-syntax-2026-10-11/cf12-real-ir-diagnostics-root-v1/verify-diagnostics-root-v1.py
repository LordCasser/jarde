"""Independently verify actual diagnostic streams and restored product inputs."""
from pathlib import Path
import hashlib
import json
import re
import subprocess

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
HERE = Path(__file__).resolve().parent
PRODUCT = '0ae30a7c8d217522f2e5f5b954aa86c9b59356dd'
META = ROOT / 'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/candidate-cli-v1.json'

def sha(data):
    return hashlib.sha256(data).hexdigest()

assert sha(META.read_bytes()) == '9913bb51cfa4acc91ed10708f94acdcedb6b4eb85dd4ea31d827febbd05ad6ef'
md = json.loads(META.read_bytes())
pins = {p: h for group in ('candidate_sources', 'test_sources', 'canonical_files') for p, h in md[group].items()}
assert len(pins) == 52
for p, h in pins.items():
    assert sha((ROOT / p).read_bytes()) == h, p
    blob = subprocess.run(['git', 'show', PRODUCT + ':' + p], cwd=ROOT, capture_output=True, check=True).stdout
    assert sha(blob) == h, p

d = json.loads((HERE / 'diagnostics-v2/execution.json').read_bytes())
assert d['status'] == 'diagnostics-passed' and d['all_52_pins_restored'] is True
assert d['source_pins_before'] == d['source_pins_after'] == pins
names = [
    'cf12_local_type_diagnostic_uses_frozen_class_ir_and_real_slot_uses',
    'diagnose_cf12_switch_fallthrough_from_original_class_ir',
]
assert len(d['commands']) == len(names)
for i, (row, name) in enumerate(zip(d['commands'], names)):
    assert row['index'] == i and row['cwd'] == str(ROOT)
    assert row['argv'] == ['cargo', 'test', '-p', 'jarde-java', '--lib', name, '--locked', '--', '--nocapture']
    assert row['exit_code'] == 0 and row['guard_stop'] is None
    assert row['peak_target_bytes'] < 1024**3 and row['free_bytes_after'] >= 5 * 1024**3
    for stream in ('stdout', 'stderr'):
        ref = row['streams'][stream]
        assert ref['path'] == f'/private/tmp/jarde-cf12-real-ir-diagnostics-root-v2/{i}.{stream}.raw'
        raw = (HERE / 'diagnostics-v2' / f'{i}.{stream}.raw').read_bytes()
        assert len(raw) == ref['bytes'] and sha(raw) == ref['sha256']
    stdout = (HERE / 'diagnostics-v2' / f'{i}.stdout.raw').read_text()
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout) == [('1', '0', '0')]
    assert re.search(r'(?m)^test (?:[A-Za-z0-9_]+::)*' + re.escape(name) + r' \.\.\. ok$', stdout)
for stem, path in [('build', 'crates/jarde-java/src/build.rs'), ('region', 'crates/jarde-java/src/region.rs')]:
    assert sha((HERE / 'diagnostics-v2' / f'{stem}-product-before.rs').read_bytes()) == pins[path]
    assert (HERE / 'diagnostics-v2' / f'{stem}-diagnostic-root.patch').stat().st_size > 0

failed = json.loads((HERE / 'diagnostics-v1/execution.json').read_bytes())
assert failed['status'] == 'failed' and failed['all_52_pins_restored'] is True
assert failed['commands'] == [] and 'not in the subpath' in failed['error']
assert '1 passed; 0 failed; 0 ignored;' in (HERE / 'diagnostics-v1/0.stdout.raw').read_text()
# v1 failed after the first child ran, while recording its private stream path. Its
# absent command row supplies no exit-code proof and is not counted as acceptance.
clean = json.loads((HERE / 'cargo-clean-v1/execution.json').read_bytes())
assert clean['status'] == 'cleaned' and clean['target_absent'] is True
row = clean['command']
assert row['argv'] == ['cargo', 'clean'] and row['exit_code'] == 0 and row['guard_stop'] is None
for stream in ('stdout', 'stderr'):
    ref = row['streams'][stream]
    raw = (HERE / 'cargo-clean-v1' / f'0.{stream}.raw').read_bytes()
    assert len(raw) == ref['bytes'] and sha(raw) == ref['sha256']
assert not (ROOT / 'target').exists()

inventory = {p.relative_to(HERE).as_posix(): {'bytes': p.stat().st_size, 'sha256': sha(p.read_bytes())}
             for p in sorted(HERE.rglob('*')) if p.is_file()}
result = {
    'schema': 'cf12-real-ir-diagnostics-root-acceptance-v1',
    'status': 'accepted-diagnostic-observations', 'product_commit': PRODUCT,
    'source_pins': pins, 'actual_tests': names, 'passed': 2, 'failed': 0, 'ignored': 0,
    'target_absent': True, 'inventory': inventory,
    'scope': 'Actual whole-class reader/SSA/local-use and canonical-switch observations only; no product implementation or whole CF12 acceptance.',
}
with (HERE / 'acceptance-root-v1.json').open('x') as f:
    f.write(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ('status', 'passed', 'failed', 'ignored', 'target_absent')}))
