import hashlib, importlib.util, json, os, re, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(sys.argv[1]); OUT.mkdir(exist_ok=False)
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(GUARD) == '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(SCAN) == 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module
g = load('continuation_guard', GUARD); scan = load('continuation_scan', SCAN)
g.OUT = OUT; g.target_bytes = lambda: scan.target_bytes(ROOT)
g.expected_test_summaries = lambda i, b: None
g.command_stream = lambda p: {'path': str(p), 'bytes': p.stat().st_size, 'sha256': sha(p)}
env = os.environ.copy(); env.update(g.ENV_VALUES)
for key in g.STRIPPED_ENV_KEYS: env.pop(key, None)
region = ROOT / 'crates/jarde-java/src/region.rs'
record = {'schema': 'switch-continuation-focused-root-v1', 'runner_sha256': sha(Path(__file__)),
          'guard_sha256': sha(GUARD), 'scanner_sha256': sha(SCAN),
          'region_sha256_before': sha(region), 'commands': [], 'status': 'running'}
try:
    row = g.run_command(0, ['cargo', 'test', '-p', 'jarde-java', '--lib', '--locked',
                            'region::tests::cf12_ft2_switch_join_stays_outside_cases_and_straight_tail_is_claimed_once', '--', '--exact', '--nocapture'], env)
    record['commands'].append(row)
    raw = (OUT / '0.stdout.raw').read_text()
    summaries = [list(map(int, match)) for match in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored', raw)]
    record['actual_summaries'] = summaries
    assert row['exit_code'] == 0 and row['guard_stop'] is None
    assert summaries and all(p > 0 and f == 0 for p, f, i in summaries)
    assert sha(region) == record['region_sha256_before']
    record['status'] = 'focused-tests-passed'
except Exception as error:
    record.update(status='failed', error=str(error))
finally:
    row = g.run_command(1, ['cargo', 'clean'], env); record['commands'].append(row)
    record['target_absent_after_clean'] = not (ROOT / 'target').exists()
    record['region_sha256_after'] = sha(region)
    if row['exit_code'] != 0 or not record['target_absent_after_clean']: record['status'] = 'failed'
    (OUT / 'execution.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps({k: record.get(k) for k in ['status', 'error', 'actual_summaries', 'target_absent_after_clean']}))
sys.exit(0 if record['status'] == 'focused-tests-passed' else 1)
