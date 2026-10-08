from pathlib import Path
import argparse, hashlib, json, subprocess

ROOT = Path(__file__).resolve().parent
SRC = ROOT / 'source'
parser = argparse.ArgumentParser(description='Replay raw-receiver evidence into a separate output directory')
parser.add_argument('--cli', required=True)
parser.add_argument('--out', required=True)
arguments = parser.parse_args()
CLI = Path(arguments.cli)
OUTPUT = Path(arguments.out)
JDKS = {
    'jdk8': (Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), ['-source','8','-target','8']),
    'jdk23': (Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'), ['--release','8']),
}
CASES = ['RawParam','StaticRawLocal','InstanceRawLocal','TypedReceiver','ThisReceiver','ShadowMethodT']

def run(args, stdout_path, stderr_path, expected=0):
    p = subprocess.run([str(x) for x in args], text=True, capture_output=True)
    Path(stdout_path).write_text(p.stdout)
    Path(stderr_path).write_text(p.stderr)
    if p.returncode != expected:
        raise SystemExit(f'command exit {p.returncode}, expected {expected}: {args}\n{p.stderr}')
    return p

for leg, (jdk, release_args) in JDKS.items():
    out = OUTPUT / leg
    original = out / 'original'
    candidate = out / 'candidate'
    cli_dir = out / 'cli'
    for d in [original, candidate, cli_dir]: d.mkdir(parents=True, exist_ok=True)
    javac = jdk / 'bin' / 'javac'
    java = jdk / 'bin' / 'java'
    jar = jdk / 'bin' / 'jar'
    java_version = run([java, '-version'], out / 'java-version.stdout', out / 'java-version.stderr')
    srcs = [SRC / (name + '.java') for name in CASES] + [SRC / 'ReceiverProbe.java']
    run([javac, *release_args, '-g:none', '-d', original, *srcs], out / 'original-javac.stdout', out / 'original-javac.stderr')
    for name in CASES:
        jar_path = out / (name + '.jar')
        run([jar, 'cfM', jar_path, '-C', original, name + '.class'], out / f'{name}.jar.stdout', out / f'{name}.jar.stderr')
        class_only = subprocess.run([str(jar), 'tf', str(jar_path)], text=True, capture_output=True, check=True).stdout.splitlines()
        (out / f'{name}.jar.contents').write_text('\n'.join(class_only) + '\n')
        if class_only != [name + '.class']:
            raise SystemExit(f'{name}: jar must contain only subject class: {class_only}')
        source_path = cli_dir / (name + '.java')
        p = run([CLI, 'class-source', '--input', jar_path, '--class', name, '--policy', 'plain-jar', '--release', '8', '--format', 'text'],
                cli_dir / f'{name}.stdout', cli_dir / f'{name}.stderr')
        if not p.stdout.strip() or f'class {name}' not in p.stdout:
            raise SystemExit(f'{name}: empty or wrong class-source output')
        source_path.write_text(p.stdout)
        run([javac, *release_args, '-g:none', '-d', candidate, source_path, SRC / 'ReceiverProbe.java'],
            candidate / f'{name}.javac.stdout', candidate / f'{name}.javac.stderr')
        run([java, '-Xverify:all', '-cp', original, 'ReceiverProbe', name],
            out / f'{name}.original-run.stdout', out / f'{name}.original-run.stderr')
        run([java, '-Xverify:all', '-cp', candidate, 'ReceiverProbe', name],
            out / f'{name}.candidate-run.stdout', out / f'{name}.candidate-run.stderr')
        orig_lines = (out / f'{name}.original-run.stdout').read_text().splitlines()
        cand_lines = (out / f'{name}.candidate-run.stdout').read_text().splitlines()
        if orig_lines[-1:] != cand_lines[-1:]:
            raise SystemExit(f'{name}: behavior differs: {orig_lines[-1:]} vs {cand_lines[-1:]}')
        (out / f'{name}.comparison.json').write_text(json.dumps({
            'original': orig_lines, 'candidate': cand_lines,
            'original_class_sha256': hashlib.sha256((original / (name + '.class')).read_bytes()).hexdigest(),
            'candidate_class_sha256': hashlib.sha256((candidate / (name + '.class')).read_bytes()).hexdigest(),
            'jarde_source_sha256': hashlib.sha256(source_path.read_bytes()).hexdigest(),
            'jar_subject_only': class_only,
        }, indent=2) + '\n')
    print(leg, 'ok')
