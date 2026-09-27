#!/usr/bin/env python3
"""Recompile Java 8 CF-14 positive and refusal-boundary samples with all three tools."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path('/Users/lordcasser/workspace/testzone/jadx')
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
CASES = {
    'StringSwitchAudit': 'StringSwitchAuditRunner',
    'NestedStringSwitchAudit': 'NestedStringSwitchAuditRunner',
    'ExtraHashUse': 'ExtraHashUseRunner',
}
JADX_FILES = [
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchOverStrings.java',
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchOverStrings3.java',
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchOverStrings5.java',
    'jadx-core/src/test/smali/switches/TestSwitchOverStrings5.smali',
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/SwitchOverStringVisitor.java',
]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, cwd=None, env=None, allow_fail=False):
    result = subprocess.run([str(x) for x in command], cwd=cwd, env=env,
                            text=True, capture_output=True, timeout=240)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(f'exit={result.returncode}\ncommand={" ".join(map(str, command))}\n'
                   f'stdout:\n{result.stdout}\nstderr:\n{result.stderr}')
    if result.returncode and not allow_fail:
        raise RuntimeError(f'command failed; see {log}')
    return result


def compile_run(source, runner, classes, logbase, main):
    classes.mkdir(parents=True, exist_ok=True)
    comp = run(['javac', '--release', '8', '-g:none', '-d', classes, source, runner],
               logbase.with_suffix('.javac.log'), allow_fail=True)
    logbase.with_suffix('.javac.status').write_text(f'{comp.returncode}\n')
    if comp.returncode:
        return None
    exec_result = run(['java', '-Xverify:all', '-cp', classes, main],
                      logbase.with_suffix('.runtime.log'))
    logbase.with_suffix('.runtime.txt').write_text(exec_result.stdout)
    return exec_result.stdout


def main():
    checkout_rev = subprocess.check_output(['git', '-C', JADX_ROOT, 'rev-parse', 'HEAD'], text=True).strip()
    if checkout_rev != JADX_REV:
        raise RuntimeError(f'JADX revision changed: {checkout_rev}')
    pins = {rel: sha(JADX_ROOT / rel) for rel in JADX_FILES}
    base = HERE / 'baseline'
    (base / 'jadx-revision.log').write_text(checkout_rev + '\n')
    (base / 'jadx-version.log').write_text(subprocess.check_output(['jadx', '--version'], text=True))
    (base / 'jadx-source-sha256.json').write_text(json.dumps(pins, indent=2) + '\n')
    summary = {'jadx_revision': checkout_rev, 'jadx_source_sha256': pins, 'cases': {}}
    with tempfile.TemporaryDirectory(prefix='jarde-cf14-') as temporary:
        temp = Path(temporary)
        target = temp / 'cargo-target'
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(target)
        for name, runner_name in CASES.items():
            src = HERE / 'input' / f'{name}.java'
            runner = HERE / 'input' / f'{runner_name}.java'
            output = base / name
            output.mkdir(exist_ok=True)
            original = temp / name / 'original'
            original_out = compile_run(src, runner, original, output / 'original', runner_name)
            class_file = original / f'{name}.class'
            shutil.copy2(class_file, base / f'{name}.original.class')
            javap_result = run(['javap', '-classpath', original, '-c', '-p', '-v', name],
                               output / 'javap.command.log')
            (output / 'javap.log').write_text(javap_result.stdout)
            jar = temp / name / 'input.jar'
            run(['jar', 'cf', jar, '-C', original, f'{name}.class'], output / 'jar.log')
            jadx_dir = temp / name / 'jadx'
            run(['jadx', '-d', jadx_dir, jar], output / 'jadx.log')
            raw = jadx_dir / 'sources/defpackage' / f'{name}.java'
            shutil.copy2(raw, base / f'{name}.jadx.raw.java')
            jadx_src = base / f'{name}.jadx.java'
            text = raw.read_text()
            if text.startswith('package defpackage;\n'):
                text = text.removeprefix('package defpackage;\n')
            jadx_src.write_text(text)
            jadx_source = temp / name / 'jadx-source'
            jadx_source.mkdir()
            named_jadx = jadx_source / f'{name}.java'
            shutil.copy2(jadx_src, named_jadx)
            jadx_classes = temp / name / 'jadx-classes'
            jadx_out = compile_run(named_jadx, runner, jadx_classes, output / 'jadx', runner_name)
            if jadx_out is not None:
                shutil.copy2(jadx_classes / f'{name}.class', base / f'{name}.jadx.class')

            jarde_src = base / f'{name}.jarde.java'
            run(['cargo', 'run', '-q', '-p', 'jarde-cli', '--', 'class-source',
                 '--input', class_file, '--class', name, '--policy', 'single-class',
                 '--release', '8', '--format', 'text', '--output', jarde_src],
                output / 'jarde-cli.log', cwd=ROOT, env=env)
            jarde_source = temp / name / 'jarde-source'
            jarde_source.mkdir()
            named_jarde = jarde_source / f'{name}.java'
            shutil.copy2(jarde_src, named_jarde)
            jarde_classes = temp / name / 'jarde-classes'
            jarde_out = compile_run(named_jarde, runner, jarde_classes, output / 'jarde', runner_name)
            jarde_class_sha256 = None
            if jarde_out is not None:
                class_output = base / f'{name}.jarde.class'
                shutil.copy2(jarde_classes / f'{name}.class', class_output)
                jarde_class_sha256 = sha(class_output)
            if name == 'NestedStringSwitchAudit':
                detail_json = temp / f'{name}.region-details.json'
                run(['cargo', 'run', '-q', '-p', 'jarde-cli', '--', 'class-source',
                     '--input', class_file, '--class', name, '--policy', 'single-class',
                     '--release', '8', '--format', 'json', '--evidence', 'region_details',
                     '--output', detail_json], output / 'region-details.cli.log',
                    cwd=ROOT, env=env)
                detail = json.loads(detail_json.read_text())
                method = next(item for item in detail['methods']
                              if item['declaration'].startswith('public static int choose'))
                report = method['outcome']['report']
                compact = {key: report.get(key) for key in
                           ('quality', 'outcome', 'fallbacks', 'diagnostics', 'regions')}
                (base / f'{name}.region-details.json').write_text(
                    json.dumps(compact, indent=2) + '\n')
            summary['cases'][name] = {
                'source_sha256': sha(src), 'runner_sha256': sha(runner),
                'original_class_sha256': sha(class_file),
                'original_runtime': original_out.splitlines() if original_out else None,
                'jadx_javac_exit': int((output / 'jadx.javac.status').read_text()),
                'jadx_runtime': jadx_out.splitlines() if jadx_out else None,
                'jarde_javac_exit': int((output / 'jarde.javac.status').read_text()),
                'jarde_class_sha256': jarde_class_sha256,
                'jarde_runtime': jarde_out.splitlines() if jarde_out else None,
            }
            if jadx_out is not None and original_out != jadx_out:
                raise RuntimeError(f'JADX output differs for {name}')
            if jarde_out is not None and original_out != jarde_out:
                raise RuntimeError(f'Jarde output differs for {name}')
        shutil.rmtree(target, ignore_errors=True)
    (base / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
