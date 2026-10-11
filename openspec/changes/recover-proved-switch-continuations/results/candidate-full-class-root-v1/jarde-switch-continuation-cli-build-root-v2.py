import hashlib, importlib.util, json, os, shutil, stat, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(sys.argv[1]); OUT.mkdir(exist_ok=False)
CLI = Path('/private/tmp/jarde-switch-continuation-candidate-cli-root-v1')
ORACLE = Path('/private/tmp/jarde-switch-continuation-class-digest-tool-root-v1')
ORACLE_SOURCE = Path('/private/tmp/jarde-switch-continuation-class-digest-root-v1.rs')
PLAN = Path('/private/tmp/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
pin = lambda p: {'path': str(p.resolve()), 'bytes': p.stat().st_size, 'sha256': sha(p)}
assert sha(GUARD) == '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(SCAN) == 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
assert not CLI.exists() and not ORACLE.exists()
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module
g = load('continuation_cli_guard', GUARD); scan = load('continuation_cli_scan', SCAN)
g.OUT = OUT; g.target_bytes = lambda: scan.target_bytes(ROOT)
g.expected_test_summaries = lambda i, b: None
g.command_stream = lambda p: pin(p)
env = os.environ.copy(); env.update(g.ENV_VALUES)
for key in g.STRIPPED_ENV_KEYS: env.pop(key, None)
plan = json.loads(PLAN.read_text())
inputs = [entry for row in plan['fixtures'] for entry in row['pins']
          if '/input/' in entry['path'] and entry['path'].endswith('.class')]
assert len({entry['path'] for entry in inputs}) == len(inputs)
for entry in inputs:
    assert pin(Path(entry['path'])) == entry
source_pins = g.source_pins(g.PRODUCT_PATHS)
record = {'schema': 'switch-continuation-candidate-cli-build-root-v1', 'status': 'running',
          'runner': pin(Path(__file__)), 'guard': pin(GUARD), 'scanner': pin(SCAN),
          'product_source_pins': source_pins, 'cli_is_unaccepted_candidate': True,
          'plan': pin(PLAN), 'oracle_source': pin(ORACLE_SOURCE), 'commands': []}
def run(argv):
    index = len(record['commands'])
    row = g.run_command(index, [str(arg) for arg in argv], env)
    record['commands'].append(row)
    (OUT / 'execution.json').write_text(json.dumps(record, indent=2) + '\n')
    assert row['exit_code'] == 0 and row['guard_stop'] is None, str(argv)
    return row
try:
    run(['cargo', 'fmt', '--all', '--', '--check'])
    run(['cargo', 'build', '-p', 'jarde-cli', '--locked'])
    built = ROOT / 'target/debug/jarde-cli'
    assert built.is_file()
    shutil.copyfile(built, CLI); CLI.chmod(0o555)
    record['cli'] = pin(CLI); record['cli_mode'] = stat.S_IMODE(CLI.stat().st_mode)
    libraries = list((ROOT / 'target/debug/deps').glob('libblake3-*.rlib'))
    assert len(libraries) == 1
    record['oracle_blake3_library'] = pin(libraries[0])
    run(['rustc', '--edition=2024', ORACLE_SOURCE, '--extern', f'blake3={libraries[0]}',
         '-L', f'dependency={ROOT / "target/debug/deps"}', '-o', ORACLE])
    ORACLE.chmod(0o555); record['oracle_binary'] = pin(ORACLE)
    row = run([ORACLE, *[entry['path'] for entry in inputs]])
    raw = Path(row['stdout']['path'] if 'stdout' in row else row['streams']['stdout']['path'])
    lines = raw.read_text().splitlines()
    assert len(lines) == len(inputs)
    digests = []
    for line, entry in zip(lines, inputs):
        digest, length, path = line.split('\t', 2)
        assert path == entry['path'] and int(length) == entry['bytes'] and len(digest) == 64
        assert pin(Path(path)) == entry
        digests.append({**entry, 'blake3': digest})
    assert g.source_pins(g.PRODUCT_PATHS) == source_pins
    record['class_digests'] = digests
    record['status'] = 'candidate-cli-and-class-digests-built'
except Exception as error:
    record.update(status='failed', error=repr(error))
finally:
    index = len(record['commands'])
    row = g.run_command(index, ['cargo', 'clean'], env); record['commands'].append(row)
    record['target_absent_after_clean'] = not (ROOT / 'target').exists()
    if row['exit_code'] != 0 or not record['target_absent_after_clean']: record['status'] = 'failed'
    (OUT / 'execution.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps({key: record.get(key) for key in ['status', 'error', 'cli', 'target_absent_after_clean']}))
sys.exit(0 if record['status'] == 'candidate-cli-and-class-digests-built' else 1)
