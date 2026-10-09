from pathlib import Path
import hashlib
import json
import re
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / 'legacy-v3-root'
OUT.mkdir()
INPUT = ROOT / 'openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/variants-vca/V3.class'
CLI = Path('/private/tmp/jarde-em18-candidate-v1-cli')
JADX = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
homes = {'javac8': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), 'javac23': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')}
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
commands = []
cases = []

def run(label, argv):
    so = OUT / 'logs' / (label + '.stdout')
    se = so.with_suffix('.stderr')
    so.parent.mkdir(exist_ok=True)
    r = subprocess.run(list(map(str, argv)), cwd=ROOT, capture_output=True)
    so.write_bytes(r.stdout)
    se.write_bytes(r.stderr)
    row = {'label': label, 'argv': list(map(str, argv)), 'cwd': str(ROOT), 'exit': r.returncode, 'stdout': str(so.relative_to(OUT)), 'stderr': str(se.relative_to(OUT)), 'stdout_sha256': h(so), 'stderr_sha256': h(se)}
    commands.append(row)
    return r, row

jar = OUT / 'input.jar'
with zipfile.ZipFile(jar, 'w') as z:
    z.write(INPUT, 'V3.class')
original = OUT / 'original-classes'
original.mkdir()
(original / 'V3.class').write_bytes(INPUT.read_bytes())
empty = OUT / 'empty-classpath-sourcepath'
empty.mkdir()
candidate = OUT / 'jarde-sources'
candidate.mkdir()
r, _ = run('render-jarde', [CLI, 'class-source', '--input', jar, '--class', 'V3', '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'])
report = json.loads(r.stdout)
(candidate / 'V3.java').write_text(report['text'])
_, jr = run('render-jadx', [JADX, '-d', OUT / 'jadx', jar])
reference = OUT / 'jadx/sources'
for name, home in homes.items():
    _, original_run = run(name + '-original', [home / 'bin/java', '-Xverify:all', '-cp', original, 'V3'])
    for side, sources in [('jarde', candidate), ('jadx', reference)]:
        srcs = sorted(sources.rglob('*.java'))
        classes = OUT / (name + '-' + side + '-classes')
        classes.mkdir()
        r, compile = run(name + '-' + side + '-compile', [home / 'bin/javac', '-source', '8', '-target', '8', '-g:none', '-classpath', empty, '-sourcepath', empty, '-d', classes, *srcs])
        runtime = None
        if r.returncode == 0:
            main = next(p for p in srcs if p.name == 'V3.java')
            match = re.search(r'(?m)^package\s+([\w.]+)\s*;', main.read_text())
            entry = (match.group(1) + '.' if match else '') + 'V3'
            _, runtime = run(name + '-' + side + '-run', [home / 'bin/java', '-Xverify:all', '-cp', classes, entry])
        matched = runtime is not None and runtime['exit'] == original_run['exit'] == 0 and all(runtime[s + '_sha256'] == original_run[s + '_sha256'] for s in ['stdout', 'stderr'])
        accepted = matched and (side != 'jarde' or ('@bytecode' not in report['text'] and 'jarde_refused_body' not in report['text']))
        cases.append({'jdk': name, 'side': side, 'source_count': len(srcs), 'compile_exit': compile['exit'], 'runtime': runtime, 'original_run': original_run, 'accepted': accepted})
meta = {'runner_sha256': h(Path(__file__)), 'input_class_sha256': h(INPUT), 'cli_sha256': h(CLI), 'jadx_launcher_sha256': h(JADX), 'commands': commands, 'cases': cases, 'files': [{'path': str(p.relative_to(OUT)), 'sha256': h(p), 'bytes': p.stat().st_size} for p in sorted(OUT.rglob('*')) if p.is_file()]}
(OUT / 'manifest.json').write_text(json.dumps(meta, indent=2) + '\n')
print(json.dumps(cases, indent=2))
if not all(c['accepted'] for c in cases):
    raise SystemExit(1)
