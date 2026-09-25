"""Rebuild and compare the Java 8 postfix-lvalue fixture with JADX and Jarde."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
CLI = Path('/tmp/jarde-cli-bitwise-root-after')


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def capture(args, output_dir, stem, timeout=120):
    result = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / (stem + '.stdout')).write_text(result.stdout)
    (output_dir / (stem + '.stderr')).write_text(result.stderr)
    (output_dir / (stem + '.status')).write_text(str(result.returncode) + '\n')
    return result


def compile_and_run(work, variant, generated):
    output_dir = HERE / variant
    target = work / variant / 'PostfixProbe.java'
    target.parent.mkdir(parents=True)
    shutil.copy2(generated, target)
    generated_text = generated.read_text()
    package = re.search(r'^package\s+([^;]+);', generated_text, re.MULTILINE)
    package_prefix = 'package ' + package.group(1) + ';\n\n' if package else ''
    support = []
    for name in ('PostfixBox.java', 'PostfixRunner.java'):
        source = (HERE / name).read_text()
        support_path = target.parent / name
        support_path.write_text(package_prefix + source)
        support.append(support_path)
    classes = work / variant / 'classes'
    javac = capture(
        ['javac', '--release', '8', '-g:none', '-d', str(classes),
         str(target), *(str(path) for path in support)],
        output_dir, variant + '-javac')
    result = {'javac': javac.returncode}
    if javac.returncode == 0:
        executed = capture(
            ['java', '-Xverify:all', '-cp', str(classes),
             (package.group(1) + '.PostfixRunner') if package else 'PostfixRunner'],
            output_dir, variant + '-runtime')
        result.update(runtime=executed.returncode,
                      output=executed.stdout.splitlines())
    return result


def main():
    jadx = Path(shutil.which('jadx') or '')
    if not CLI.is_file() or not jadx.is_file():
        raise SystemExit('Expected frozen Jarde CLI and jadx on PATH')
    tools = {
        'javac': shutil.which('javac'),
        'java': shutil.which('java'),
        'javap': shutil.which('javap'),
        'jadx': str(jadx.resolve()),
        'jarde_cli': str(CLI),
    }
    tool_hashes = {name: sha256(Path(path)) for name, path in tools.items()}
    with tempfile.TemporaryDirectory(prefix='jarde-postfix-audit-') as raw:
        work = Path(raw)
        original_classes = HERE / 'original' / 'classes'
        if original_classes.exists():
            shutil.rmtree(original_classes)
        original_classes.mkdir(parents=True)
        sources = [HERE / name for name in
                   ('PostfixProbe.java', 'PostfixBox.java', 'PostfixRunner.java')]
        original_compile = capture(
            ['javac', '--release', '8', '-g:none', '-d', str(original_classes),
             *(str(path) for path in sources)], HERE / 'original', 'original-javac')
        if original_compile.returncode != 0:
            raise SystemExit('Original fixture did not compile; see original/original-javac.stderr')
        class_file = original_classes / 'PostfixProbe.class'
        original_runtime = capture(
            ['java', '-Xverify:all', '-cp', str(original_classes), 'PostfixRunner'],
            HERE / 'original', 'original-runtime')
        javap = capture(
            ['javap', '-c', '-p', str(class_file)], HERE / 'original', 'original-javap')

        decompiled = subprocess.run(
            [str(CLI), 'class-source', '--input', str(class_file), '--class',
             'PostfixProbe', '--policy', 'single-class', '--release', '8',
             '--format', 'text'], capture_output=True, text=True, timeout=120)
        (HERE / 'jarde' / 'jarde.java.txt').parent.mkdir(parents=True, exist_ok=True)
        (HERE / 'jarde' / 'jarde.java.txt').write_text(decompiled.stdout)
        (HERE / 'jarde' / 'jarde-report.txt').write_text(decompiled.stderr)
        (HERE / 'jarde' / 'jarde.status').write_text(str(decompiled.returncode) + '\n')
        jarde_result = (compile_and_run(work, 'jarde', HERE / 'jarde' / 'jarde.java.txt')
                        if decompiled.returncode == 0 else {'javac': 'skipped'})

        jadx_dir = work / 'jadx-output'
        jadx_result = capture(
            [str(jadx), '--no-res', '-d', str(jadx_dir), str(class_file)],
            HERE / 'jadx', 'jadx-decompile')
        jadx_source = next(jadx_dir.rglob('PostfixProbe.java'), None) if jadx_result.returncode == 0 else None
        if jadx_source is not None:
            shutil.copy2(jadx_source, HERE / 'jadx' / 'PostfixProbe.java.txt')
            jadx_compile_result = compile_and_run(work, 'jadx', jadx_source)
        else:
            jadx_compile_result = {'javac': 'skipped'}

        summary = {
            'tools': tools,
            'tool_sha256_before': tool_hashes,
            'tool_sha256_after': {name: sha256(Path(path)) for name, path in tools.items()},
            'source_sha256': {path.name: sha256(path) for path in sources},
            'class': {
                'path': str(class_file),
                'bytes': class_file.stat().st_size,
                'sha256': sha256(class_file),
                'code_methods': javap.stdout.count('    Code:'),
            },
            'original': {
                'javac': original_compile.returncode,
                'runtime': original_runtime.returncode,
                'output': original_runtime.stdout.splitlines(),
            },
            'jadx_decompile': jadx_result.returncode,
            'jadx': jadx_compile_result,
            'jarde_decompile': decompiled.returncode,
            'jarde': jarde_result,
            'jadx_output_matches_original':
                jadx_compile_result.get('output') == original_runtime.stdout.splitlines()
                if jadx_compile_result.get('runtime') == 0 else None,
            'jarde_output_matches_original':
                jarde_result.get('output') == original_runtime.stdout.splitlines()
                if jarde_result.get('runtime') == 0 else None,
        }
        (HERE / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
