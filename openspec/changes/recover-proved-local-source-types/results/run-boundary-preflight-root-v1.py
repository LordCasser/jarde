from pathlib import Path
import datetime, hashlib, json, os, shutil, subprocess, time

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(__file__).resolve().parent / 'boundary-preflight-root-v1'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
baseline = json.loads((ROOT / 'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/cf07-candidate-root-v1/manifest.json').read_bytes())
jdks = {name: Path(next(r['java_home'] for r in baseline['commands'] if r['label'] == name + '-original-compile')) for name in ('javac8', 'javac23')}
commands = []
record = {'schema': 'typed-local-java-boundary-preflight-root-v1', 'commands': commands,
          'source_files': {p.name: sha(p) for p in OUT.glob('*.java')},
          'runner': {'path': str(Path(__file__)), 'sha256': sha(Path(__file__))},
          'jdk_tools': {}, 'status': 'running'}
assert not (OUT / 'execution.json').exists()
def save():
    (OUT / 'execution.json').write_text(json.dumps(record, indent=2) + '\n')
def run(label, argv, jdk):
    assert shutil.disk_usage(ROOT).free >= 5 * 1024**3
    env = os.environ.copy()
    for key in ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH'):
        env.pop(key, None)
    env['JAVA_HOME'] = str(jdk)
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    began = time.monotonic()
    with (OUT / (label + '.stdout.raw')).open('xb') as o, (OUT / (label + '.stderr.raw')).open('xb') as e:
        result = subprocess.run(list(map(str, argv)), cwd=ROOT, env=env, stdout=o, stderr=e, timeout=60)
    row = {'label': label, 'argv': list(map(str, argv)), 'cwd': str(ROOT), 'java_home': str(jdk),
           'started_at': started, 'duration_seconds': time.monotonic() - began, 'exit_code': result.returncode,
           'streams': {name: {'path': label + '.' + name + '.raw', 'bytes': (OUT / (label + '.' + name + '.raw')).stat().st_size,
                             'sha256': sha(OUT / (label + '.' + name + '.raw'))} for name in ('stdout', 'stderr')}}
    commands.append(row); save(); assert result.returncode == 0, label
try:
    for label, jdk in jdks.items():
        record['jdk_tools'][label] = {name: {'path': str(jdk / 'bin' / name), 'sha256': sha(jdk / 'bin' / name)} for name in ('java', 'javac', 'javap')}
        for debug in ('debug', 'no-debug'):
            case = label + '-' + debug
            directory = OUT / case
            directory.mkdir(); classes = directory / 'classes'; classes.mkdir(); empty = directory / 'empty'; empty.mkdir()
            run(case + '-compile', [jdk / 'bin/javac', '-source', '8', '-target', '8', '-g' if debug == 'debug' else '-g:none',
                                   '-Xlint:-options', '-proc:none', '-classpath', empty, '-sourcepath', empty, '-d', classes,
                                   OUT / 'LocalSourceTypesBoundaries.java', OUT / 'BoundaryRunner.java'], jdk)
            run(case + '-runtime', [jdk / 'bin/java', '-Xverify:all', '-cp', classes, 'BoundaryRunner'], jdk)
            run(case + '-javap', [jdk / 'bin/javap', '-p', '-c', '-s', '-v', classes / 'LocalSourceTypesBoundaries.class'], jdk)
    outputs = [(OUT / (label + '-' + debug + '-runtime.stdout.raw')).read_bytes() for label in jdks for debug in ('debug', 'no-debug')]
    errors = [(OUT / (label + '-' + debug + '-runtime.stderr.raw')).read_bytes() for label in jdks for debug in ('debug', 'no-debug')]
    assert len(set(outputs)) == len(set(errors)) == 1 and errors[0] == b''
    record.update(status='observed-original-boundary-runtime', original_four_legs_raw_equal=True,
                  class_files={p.relative_to(OUT).as_posix(): {'bytes': p.stat().st_size, 'sha256': sha(p)} for p in sorted(OUT.rglob('*.class'))},
                  free_bytes_after=shutil.disk_usage(ROOT).free)
except Exception as e:
    record.update(status='failed', error=f'{type(e).__name__}: {e}')
finally:
    save(); print(json.dumps({k: record.get(k) for k in ('status', 'error', 'original_four_legs_raw_equal')}))
assert record['status'] == 'observed-original-boundary-runtime'
